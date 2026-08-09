use std::collections::HashSet;

use fenestra_ui_ir::prototype::ValidatedConstruction;

use crate::logical_tree::NodeId;

use super::capacity::RuntimeCapacity;
use super::change::{StateEditError, StructuralTracker};
use super::fragment::{FragmentId, KeyedMember};
use super::mutation::ManifestItem;
use super::state::{ChildGroup, RuntimeState};

enum RetireTask {
    EnterNode(NodeId),
    ExitNode(NodeId),
    ExitFragment(FragmentId),
}

impl RuntimeState {
    pub(crate) fn insert_member(
        &mut self,
        construction: &ValidatedConstruction,
        capacity: RuntimeCapacity,
        structural: &mut StructuralTracker,
        fragment: FragmentId,
        key: u64,
        final_index: usize,
    ) -> Result<(NodeId, Vec<ManifestItem>), StateEditError> {
        let stored = self
            .fragments
            .get(fragment)
            .ok_or(StateEditError::Invariant)?;
        let owner = stored.owner;
        let repeat_body = construction
            .region(stored.descriptor)
            .ok_or(StateEditError::Invariant)?
            .repeat_body();
        let footprint = Self::factory_footprint(repeat_body);
        self.preflight_created(footprint, capacity, structural)?;
        let mut manifest = Vec::with_capacity(footprint.structural()?);
        let value = self.build_expanded_node(repeat_body)?;
        let root = self
            .tree
            .append_child(owner, value)
            .map_err(|_| StateEditError::Invariant)?;
        manifest.push(ManifestItem::Node(root));
        self.fragments
            .get_mut(fragment)
            .ok_or(StateEditError::Invariant)?
            .members
            .insert(final_index, KeyedMember { key, root });
        let flat_index = self.member_flat_index(owner, fragment, final_index)?;
        self.tree
            .reorder_direct_child(owner, root, flat_index)
            .map_err(|()| StateEditError::Invariant)?;
        self.populate_expansion(root, repeat_body, &mut Some(&mut manifest))?;
        Ok((root, manifest))
    }

    pub(crate) fn move_member(
        &mut self,
        fragment: FragmentId,
        old_index: usize,
        final_index: usize,
    ) -> Result<NodeId, StateEditError> {
        let stored = self
            .fragments
            .get_mut(fragment)
            .ok_or(StateEditError::Invariant)?;
        let member = stored.members.remove(old_index);
        stored.members.insert(final_index, member);
        let owner = stored.owner;
        let flat_index = self.member_flat_index(owner, fragment, final_index)?;
        self.tree
            .reorder_direct_child(owner, member.root, flat_index)
            .map_err(|()| StateEditError::Invariant)?;
        Ok(member.root)
    }

    pub(crate) fn remove_member(
        &mut self,
        structural: &mut StructuralTracker,
        fragment: FragmentId,
        member_index: usize,
    ) -> Result<(NodeId, Vec<ManifestItem>), StateEditError> {
        let root = self
            .fragments
            .get(fragment)
            .and_then(|stored| stored.members.get(member_index))
            .map(|member| member.root)
            .ok_or(StateEditError::Invariant)?;
        let manifest = self.retirement_manifest(root, structural.remaining())?;
        structural.reserve(manifest.len())?;
        let retired_slots = manifest.iter().try_fold(0usize, |count, item| {
            let ManifestItem::Node(node) = item else {
                return Ok(count);
            };
            count
                .checked_add(
                    self.tree
                        .value(*node)
                        .ok_or(StateEditError::Invariant)?
                        .properties
                        .len(),
                )
                .ok_or(StateEditError::Invariant)
        })?;
        for item in &manifest {
            if let ManifestItem::Fragment(nested) = item {
                self.fragments
                    .remove(*nested)
                    .ok_or(StateEditError::Invariant)?;
            }
        }
        self.tree
            .remove_subtree(root)
            .map_err(|_| StateEditError::Invariant)?;
        self.fragments
            .get_mut(fragment)
            .ok_or(StateEditError::Invariant)?
            .members
            .remove(member_index);
        self.property_slot_count = self
            .property_slot_count
            .checked_sub(retired_slots)
            .ok_or(StateEditError::Invariant)?;
        Ok((root, manifest))
    }

    fn retirement_manifest(
        &self,
        root: NodeId,
        remaining: usize,
    ) -> Result<Vec<ManifestItem>, StateEditError> {
        let mut manifest = Vec::new();
        let mut pending = vec![RetireTask::EnterNode(root)];
        let mut nodes = HashSet::new();
        let mut fragments = HashSet::new();
        while let Some(task) = pending.pop() {
            match task {
                RetireTask::EnterNode(node) => {
                    if !nodes.insert(node) {
                        return Err(StateEditError::Invariant);
                    }
                    let stored = self.tree.value(node).ok_or(StateEditError::Invariant)?;
                    pending.push(RetireTask::ExitNode(node));
                    for group in stored.child_groups.iter().rev() {
                        match *group {
                            ChildGroup::Static(child) => {
                                if self.tree.parent(child) != Some(node) {
                                    return Err(StateEditError::Invariant);
                                }
                                pending.push(RetireTask::EnterNode(child));
                            }
                            ChildGroup::Region(fragment) => {
                                let nested = self
                                    .fragments
                                    .get(fragment)
                                    .ok_or(StateEditError::Invariant)?;
                                if nested.owner != node || !fragments.insert(fragment) {
                                    return Err(StateEditError::Invariant);
                                }
                                pending.push(RetireTask::ExitFragment(fragment));
                                for member in nested.members.iter().rev() {
                                    pending.push(RetireTask::EnterNode(member.root));
                                }
                            }
                        }
                    }
                }
                RetireTask::ExitNode(node) => {
                    Self::push_retired(&mut manifest, ManifestItem::Node(node), remaining)?;
                }
                RetireTask::ExitFragment(fragment) => {
                    Self::push_retired(&mut manifest, ManifestItem::Fragment(fragment), remaining)?;
                }
            }
        }
        Ok(manifest)
    }

    fn push_retired(
        manifest: &mut Vec<ManifestItem>,
        item: ManifestItem,
        remaining: usize,
    ) -> Result<(), StateEditError> {
        if manifest.len() >= remaining {
            return Err(StateEditError::Capacity(
                super::error::CapacityKind::StructuralChanges,
            ));
        }
        manifest.push(item);
        Ok(())
    }

    fn member_flat_index(
        &self,
        owner: NodeId,
        fragment: FragmentId,
        member_index: usize,
    ) -> Result<usize, StateEditError> {
        let node = self.tree.value(owner).ok_or(StateEditError::Invariant)?;
        let mut offset = 0usize;
        for group in &node.child_groups {
            match *group {
                ChildGroup::Static(_) => {
                    offset = offset.checked_add(1).ok_or(StateEditError::Invariant)?;
                }
                ChildGroup::Region(candidate) if candidate == fragment => {
                    return offset
                        .checked_add(member_index)
                        .ok_or(StateEditError::Invariant);
                }
                ChildGroup::Region(candidate) => {
                    offset = offset
                        .checked_add(
                            self.fragments
                                .get(candidate)
                                .ok_or(StateEditError::Invariant)?
                                .members
                                .len(),
                        )
                        .ok_or(StateEditError::Invariant)?;
                }
            }
        }
        Err(StateEditError::Invariant)
    }
}
