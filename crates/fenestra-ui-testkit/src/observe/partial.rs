use std::collections::HashSet;

use fenestra_ui_ir::prototype::{
    ChildFactory, StructuralRegionId, TemplateNodeId, ValidatedConstruction,
};
use fenestra_ui_runtime::prototype::{FragmentId, NodeId};

use super::ObservedSnapshotV1;
use super::limits::{
    arithmetic_error, checked_increment, ensure_next_count, ensure_path_depth, state_mismatch,
};
use super::view::SnapshotViewV1;
use crate::error::{HarnessError, HarnessLimitKind};
use crate::fixture::HarnessLimitsV1;
use crate::identity::IdentityIndexV1;
use crate::semantic::{
    FragmentPathV1, NodePathV1, NormalizedChildGroupV1, NormalizedFragmentV1, NormalizedMemberV1,
    NormalizedNodeV1, NormalizedPropertyV1, NormalizedStateV1,
};

struct ObservedMember {
    key: u64,
    node: NodeId,
    path: NodePathV1,
}

enum ObserveWork {
    Node {
        path: NodePathV1,
        parent_path: Option<NodePathV1>,
        node: NodeId,
        parent: Option<NodeId>,
        template: TemplateNodeId,
    },
    Fragment {
        path: FragmentPathV1,
        fragment: FragmentId,
        owner: NodeId,
        descriptor: StructuralRegionId,
        repeat_body: TemplateNodeId,
        members: Vec<ObservedMember>,
    },
}

pub(super) fn observe_snapshot_indexed_v1<V: SnapshotViewV1 + ?Sized>(
    construction: &ValidatedConstruction,
    snapshot: &V,
    limits: HarnessLimitsV1,
) -> Result<ObservedSnapshotV1, HarnessError> {
    let mut work = vec![ObserveWork::Node {
        path: NodePathV1::root(),
        parent_path: None,
        node: snapshot.root(),
        parent: None,
        template: construction.root_factory().id(),
    }];
    let mut nodes = Vec::new();
    let mut fragments = Vec::new();
    let mut visited_nodes = HashSet::new();
    let mut visited_fragments = HashSet::new();
    let mut identities = IdentityIndexV1::default();
    let mut property_slots = 0_usize;
    let mut memberships = 0_usize;

    while let Some(next) = work.pop() {
        match next {
            ObserveWork::Node {
                path,
                parent_path,
                node,
                parent,
                template,
            } => observe_node(
                construction,
                snapshot,
                limits,
                &mut work,
                &mut nodes,
                &mut visited_nodes,
                &mut identities,
                &mut property_slots,
                &mut memberships,
                path,
                parent_path,
                node,
                parent,
                template,
            )?,
            ObserveWork::Fragment {
                path,
                fragment,
                owner,
                descriptor,
                repeat_body,
                members,
            } => observe_fragment(
                limits,
                &mut work,
                &mut fragments,
                &mut visited_fragments,
                &mut identities,
                path,
                fragment,
                owner,
                descriptor,
                repeat_body,
                members,
            )?,
        }
    }

    if nodes.len() != snapshot.node_count()
        || fragments.len() != snapshot.fragment_count()
        || property_slots != snapshot.property_slot_count()
    {
        return Err(state_mismatch());
    }

    Ok(ObservedSnapshotV1::new(
        NormalizedStateV1::new(nodes, fragments),
        identities,
    ))
}

#[allow(clippy::too_many_arguments)]
fn observe_node<V: SnapshotViewV1 + ?Sized>(
    construction: &ValidatedConstruction,
    snapshot: &V,
    limits: HarnessLimitsV1,
    work: &mut Vec<ObserveWork>,
    nodes: &mut Vec<NormalizedNodeV1>,
    visited_nodes: &mut HashSet<NodeId>,
    identities: &mut IdentityIndexV1,
    property_slots: &mut usize,
    memberships: &mut usize,
    path: NodePathV1,
    parent_path: Option<NodePathV1>,
    node: NodeId,
    parent: Option<NodeId>,
    template: TemplateNodeId,
) -> Result<(), HarnessError> {
    ensure_path_depth(&path, limits)?;
    if !visited_nodes.insert(node) || !identities.record_node(path.clone(), node) {
        return Err(state_mismatch());
    }
    ensure_next_count(
        nodes.len(),
        limits.normalized_nodes(),
        HarnessLimitKind::NormalizedNodes,
    )?;
    if snapshot.parent(node) != parent || snapshot.template(node) != Some(template) {
        return Err(state_mismatch());
    }

    let factory = construction.template(template).ok_or_else(state_mismatch)?;
    let component = factory.component();
    if snapshot.component(node) != Some(component.id()) {
        return Err(state_mismatch());
    }

    let mut properties = Vec::new();
    for property in component.properties() {
        *property_slots = checked_increment(
            *property_slots,
            limits.normalized_properties(),
            HarnessLimitKind::NormalizedProperties,
        )?;
        let value = snapshot
            .property(node, property.id())
            .ok_or_else(state_mismatch)?;
        if value.value_type() != property.value_type() {
            return Err(state_mismatch());
        }
        properties.push(NormalizedPropertyV1::new(property.id(), value));
    }

    let child_count = snapshot.child_count(node).ok_or_else(state_mismatch)?;
    let mut offset = 0_usize;
    let mut child_groups = Vec::new();
    let mut child_work = Vec::new();
    for (slot, child) in factory.children().enumerate() {
        let slot = u16::try_from(slot).map_err(|_| arithmetic_error())?;
        match child {
            ChildFactory::Static { template, .. } => {
                let child_node = child_at(snapshot, node, offset)?;
                offset = checked_offset(offset)?;
                if snapshot.parent(child_node) != Some(node) {
                    return Err(state_mismatch());
                }
                let child_path = path.clone().static_child(slot);
                ensure_path_depth(&child_path, limits)?;
                child_groups.push(NormalizedChildGroupV1::Static(child_path.clone()));
                child_work.push(ObserveWork::Node {
                    path: child_path,
                    parent_path: Some(path.clone()),
                    node: child_node,
                    parent: Some(node),
                    template: template.id(),
                });
            }
            ChildFactory::Region { region, .. } => {
                let fragment_path = FragmentPathV1::new(path.clone(), slot);
                let fragment = snapshot
                    .fragment(node, region.id())
                    .ok_or_else(state_mismatch)?;
                let keyed_count = snapshot.keyed_count(fragment).ok_or_else(state_mismatch)?;
                let mut unique_keys = HashSet::new();
                let mut members = Vec::new();
                for index in 0..keyed_count {
                    let (key, member) = snapshot
                        .keyed_at(fragment, index)
                        .ok_or_else(state_mismatch)?;
                    *memberships = checked_increment(
                        *memberships,
                        limits.live_memberships(),
                        HarnessLimitKind::LiveMemberships,
                    )?;
                    if !unique_keys.insert(key)
                        || child_at(snapshot, node, offset)? != member
                        || snapshot.parent(member) != Some(node)
                    {
                        return Err(state_mismatch());
                    }
                    offset = checked_offset(offset)?;
                    let member_path = path.clone().member(slot, key);
                    ensure_path_depth(&member_path, limits)?;
                    members.push(ObservedMember {
                        key,
                        node: member,
                        path: member_path,
                    });
                }
                child_groups.push(NormalizedChildGroupV1::Region(fragment_path.clone()));
                child_work.push(ObserveWork::Fragment {
                    path: fragment_path,
                    fragment,
                    owner: node,
                    descriptor: region.id(),
                    repeat_body: region.repeat_body().id(),
                    members,
                });
            }
        }
    }
    if offset != child_count {
        return Err(state_mismatch());
    }

    work.extend(child_work.into_iter().rev());
    nodes.push(NormalizedNodeV1::new(
        path,
        parent_path,
        template,
        component.id(),
        properties,
        child_groups,
    ));
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn observe_fragment(
    limits: HarnessLimitsV1,
    work: &mut Vec<ObserveWork>,
    fragments: &mut Vec<NormalizedFragmentV1>,
    visited_fragments: &mut HashSet<FragmentId>,
    identities: &mut IdentityIndexV1,
    path: FragmentPathV1,
    fragment: FragmentId,
    owner: NodeId,
    descriptor: StructuralRegionId,
    repeat_body: TemplateNodeId,
    members: Vec<ObservedMember>,
) -> Result<(), HarnessError> {
    ensure_path_depth(path.owner(), limits)?;
    if !visited_fragments.insert(fragment) || !identities.record_fragment(path.clone(), fragment) {
        return Err(state_mismatch());
    }
    ensure_next_count(
        fragments.len(),
        limits.normalized_fragments(),
        HarnessLimitKind::NormalizedFragments,
    )?;

    let normalized_members = members
        .iter()
        .map(|member| NormalizedMemberV1::new(member.key, member.path.clone()))
        .collect();
    let parent_path = path.owner().clone();
    work.extend(members.into_iter().rev().map(|member| ObserveWork::Node {
        path: member.path,
        parent_path: Some(parent_path.clone()),
        node: member.node,
        parent: Some(owner),
        template: repeat_body,
    }));
    fragments.push(NormalizedFragmentV1::new(
        path,
        descriptor,
        normalized_members,
    ));
    Ok(())
}

fn child_at<V: SnapshotViewV1 + ?Sized>(
    snapshot: &V,
    node: NodeId,
    offset: usize,
) -> Result<NodeId, HarnessError> {
    snapshot.child_at(node, offset).ok_or_else(state_mismatch)
}

fn checked_offset(current: usize) -> Result<usize, HarnessError> {
    current.checked_add(1).ok_or_else(arithmetic_error)
}
