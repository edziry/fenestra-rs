mod preflight;

use std::collections::BTreeMap;

use fenestra_ui_runtime::prototype::{FragmentId, NodeId};

use super::limits::{arithmetic_error, ensure_path_depth, state_mismatch};
use super::view::SnapshotViewV1;
use crate::error::{HarnessError, HarnessLimitKind};
use crate::fixture::HarnessLimitsV1;
use crate::identity::IdentityIndexV1;
use crate::semantic::{FragmentPathV1, NodePathV1, NormalizedStateV1};

pub(super) enum BoundFragmentV1 {
    Absent,
    Present(Vec<(u64, NodeId)>),
}

#[derive(Clone, Copy)]
pub(super) struct ReportedCountsV1 {
    nodes: usize,
    fragments: usize,
    properties: usize,
}

impl ReportedCountsV1 {
    pub(super) const fn new(nodes: usize, fragments: usize, properties: usize) -> Self {
        Self {
            nodes,
            fragments,
            properties,
        }
    }

    pub(super) const fn nodes(self) -> usize {
        self.nodes
    }

    pub(super) const fn fragments(self) -> usize {
        self.fragments
    }

    pub(super) const fn properties(self) -> usize {
        self.properties
    }
}

pub(super) struct BoundSnapshotV1 {
    identities: IdentityIndexV1,
    children: BTreeMap<NodePathV1, Vec<NodeId>>,
    fragments: BTreeMap<FragmentPathV1, BoundFragmentV1>,
    reported_counts: ReportedCountsV1,
}

impl BoundSnapshotV1 {
    pub(super) fn identities(&self) -> &IdentityIndexV1 {
        &self.identities
    }

    pub(super) fn children(&self, path: &NodePathV1) -> Option<&[NodeId]> {
        self.children.get(path).map(Vec::as_slice)
    }

    pub(super) fn fragment(&self, path: &FragmentPathV1) -> Option<&BoundFragmentV1> {
        self.fragments.get(path)
    }

    pub(super) const fn reported_counts(&self) -> ReportedCountsV1 {
        self.reported_counts
    }

    pub(super) fn into_identities(self) -> IdentityIndexV1 {
        self.identities
    }
}

pub(super) fn bind_expected_paths_v1<V: SnapshotViewV1 + ?Sized>(
    expected: &NormalizedStateV1,
    view: &V,
    limits: HarnessLimitsV1,
) -> Result<BoundSnapshotV1, HarnessError> {
    let reported_counts = preflight::preflight_snapshot_v1(expected, view, limits)?;
    let root = NodePathV1::root();
    if expected.node(&root).is_none() {
        return Err(state_mismatch());
    }

    let mut nodes = BTreeMap::new();
    let mut fragment_handles = BTreeMap::new();
    let mut children = BTreeMap::new();
    let mut fragments = BTreeMap::new();
    nodes.insert(root, view.root());
    let mut observed_nodes = 1_usize;
    let mut observed_memberships = 0_usize;

    for expected_node in expected.nodes() {
        let Some(owner) = nodes.get(expected_node.path()).copied() else {
            continue;
        };
        let remaining_nodes = limits
            .normalized_nodes()
            .checked_sub(observed_nodes)
            .ok_or_else(arithmetic_error)?;
        let observed_children = read_children(view, owner, remaining_nodes)?;
        observed_nodes = checked_add(observed_nodes, observed_children.len())?;
        let mut child_offset = Some(0_usize);
        for group in expected_node.child_groups() {
            match group {
                crate::semantic::NormalizedChildGroupV1::Static(path) => {
                    ensure_path_depth(path, limits)?;
                    if let Some(offset) = child_offset {
                        if let Some(child) = observed_children.get(offset) {
                            insert_node(&mut nodes, path, *child)?;
                        }
                        child_offset = Some(checked_add(offset, 1)?);
                    }
                }
                crate::semantic::NormalizedChildGroupV1::Region(path) => {
                    let expected_fragment = expected.fragment(path).ok_or_else(state_mismatch)?;
                    let Some(fragment) = view.fragment(owner, expected_fragment.descriptor())
                    else {
                        insert_fragment_observation(&mut fragments, path, BoundFragmentV1::Absent)?;
                        child_offset = None;
                        continue;
                    };
                    insert_fragment_handle(&mut fragment_handles, path, fragment)?;
                    let remaining_memberships = limits
                        .live_memberships()
                        .checked_sub(observed_memberships)
                        .ok_or_else(arithmetic_error)?;
                    let members = read_keyed_members(view, fragment, remaining_memberships)?;
                    observed_memberships = checked_add(observed_memberships, members.len())?;
                    bind_members(&mut nodes, expected_fragment.members(), &members, limits)?;
                    if let Some(offset) = child_offset {
                        child_offset = Some(checked_add(offset, members.len())?);
                    }
                    insert_fragment_observation(
                        &mut fragments,
                        path,
                        BoundFragmentV1::Present(members),
                    )?;
                }
            }
        }
        if children
            .insert(expected_node.path().clone(), observed_children)
            .is_some()
        {
            return Err(state_mismatch());
        }
    }

    let identities = build_identity_index(expected, &nodes, &fragment_handles)?;
    Ok(BoundSnapshotV1 {
        identities,
        children,
        fragments,
        reported_counts,
    })
}

fn read_children<V: SnapshotViewV1 + ?Sized>(
    view: &V,
    node: NodeId,
    remaining: usize,
) -> Result<Vec<NodeId>, HarnessError> {
    let count = view.child_count(node).ok_or_else(state_mismatch)?;
    if count > remaining {
        return Err(HarnessError::limit(HarnessLimitKind::NormalizedNodes));
    }
    (0..count)
        .map(|index| view.child_at(node, index).ok_or_else(state_mismatch))
        .collect()
}

fn read_keyed_members<V: SnapshotViewV1 + ?Sized>(
    view: &V,
    fragment: FragmentId,
    remaining: usize,
) -> Result<Vec<(u64, NodeId)>, HarnessError> {
    let count = view.keyed_count(fragment).ok_or_else(state_mismatch)?;
    if count > remaining {
        return Err(HarnessError::limit(HarnessLimitKind::LiveMemberships));
    }
    (0..count)
        .map(|index| view.keyed_at(fragment, index).ok_or_else(state_mismatch))
        .collect()
}

fn bind_members(
    nodes: &mut BTreeMap<NodePathV1, NodeId>,
    expected: &[crate::semantic::NormalizedMemberV1],
    observed: &[(u64, NodeId)],
    limits: HarnessLimitsV1,
) -> Result<(), HarnessError> {
    for member in expected {
        ensure_path_depth(member.node(), limits)?;
        let mut matches = observed
            .iter()
            .filter(|(key, _)| *key == member.key())
            .map(|(_, node)| *node);
        let Some(node) = matches.next() else {
            continue;
        };
        if matches.next().is_none() {
            insert_node(nodes, member.node(), node)?;
        }
    }
    Ok(())
}

fn insert_node(
    nodes: &mut BTreeMap<NodePathV1, NodeId>,
    path: &NodePathV1,
    node: NodeId,
) -> Result<(), HarnessError> {
    if nodes.insert(path.clone(), node).is_some() {
        Err(state_mismatch())
    } else {
        Ok(())
    }
}

fn insert_fragment_handle(
    fragments: &mut BTreeMap<FragmentPathV1, FragmentId>,
    path: &FragmentPathV1,
    fragment: FragmentId,
) -> Result<(), HarnessError> {
    if fragments.insert(path.clone(), fragment).is_some() {
        Err(state_mismatch())
    } else {
        Ok(())
    }
}

fn insert_fragment_observation(
    fragments: &mut BTreeMap<FragmentPathV1, BoundFragmentV1>,
    path: &FragmentPathV1,
    fragment: BoundFragmentV1,
) -> Result<(), HarnessError> {
    if fragments.insert(path.clone(), fragment).is_some() {
        Err(state_mismatch())
    } else {
        Ok(())
    }
}

fn build_identity_index(
    expected: &NormalizedStateV1,
    nodes: &BTreeMap<NodePathV1, NodeId>,
    fragments: &BTreeMap<FragmentPathV1, FragmentId>,
) -> Result<IdentityIndexV1, HarnessError> {
    let mut identities = IdentityIndexV1::default();
    for expected_node in expected.nodes() {
        if let Some(node) = nodes.get(expected_node.path())
            && !identities.record_node(expected_node.path().clone(), *node)
        {
            return Err(state_mismatch());
        }
    }
    for expected_fragment in expected.fragments() {
        if let Some(fragment) = fragments.get(expected_fragment.path())
            && !identities.record_fragment(expected_fragment.path().clone(), *fragment)
        {
            return Err(state_mismatch());
        }
    }
    Ok(identities)
}

fn checked_add(left: usize, right: usize) -> Result<usize, HarnessError> {
    left.checked_add(right).ok_or_else(arithmetic_error)
}
