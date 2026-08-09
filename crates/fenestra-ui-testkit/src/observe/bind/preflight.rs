use std::collections::BTreeMap;

use fenestra_ui_runtime::prototype::{FragmentId, NodeId};

use super::super::limits::{
    ObservationLimitCrossingsV1, first_observation_limit_v1, state_mismatch,
};
use super::super::view::SnapshotViewV1;
use super::ReportedCountsV1;
use crate::error::{HarnessError, HarnessLimitKind};
use crate::fixture::HarnessLimitsV1;
use crate::semantic::{
    NodePathV1, NormalizedChildGroupV1, NormalizedFragmentV1, NormalizedNodeV1, NormalizedStateV1,
};

pub(super) fn preflight_snapshot_v1<V: SnapshotViewV1 + ?Sized>(
    expected: &NormalizedStateV1,
    view: &V,
    limits: HarnessLimitsV1,
) -> Result<ReportedCountsV1, HarnessError> {
    let root = NodePathV1::root();
    let expected_root = expected.node(&root).ok_or_else(state_mismatch)?;

    let reported = ReportedCountsV1::new(
        view.node_count(),
        view.fragment_count(),
        view.property_slot_count(),
    );
    let crossings = ObservationLimitCrossingsV1::new(
        false,
        false,
        reported.nodes() > limits.normalized_nodes(),
        reported.fragments() > limits.normalized_fragments(),
        reported.properties() > limits.normalized_properties(),
    );
    let mut preflight = PreflightV1 {
        expected,
        view,
        limits,
        nodes: BTreeMap::from([(expected_root.path(), view.root())]),
        observed_nodes: 1,
        observed_fragments: 0,
        observed_properties: 0,
        observed_memberships: 0,
        crossings,
    };
    preflight.observe()?;
    if let Some(kind) = first_observation_limit_v1(preflight.crossings) {
        return Err(HarnessError::limit(kind));
    }
    Ok(reported)
}

struct PreflightV1<'a, V: ?Sized> {
    expected: &'a NormalizedStateV1,
    view: &'a V,
    limits: HarnessLimitsV1,
    nodes: BTreeMap<&'a NodePathV1, NodeId>,
    observed_nodes: usize,
    observed_fragments: usize,
    observed_properties: usize,
    observed_memberships: usize,
    crossings: ObservationLimitCrossingsV1,
}

impl<'a, V: SnapshotViewV1 + ?Sized> PreflightV1<'a, V> {
    fn observe(&mut self) -> Result<(), HarnessError> {
        let expected = self.expected;
        for node in expected.nodes() {
            self.record_path_depth(node.path().depth());
        }
        for fragment in expected.fragments() {
            self.record_path_depth(fragment.path().owner().depth());
        }
        for node in expected.nodes() {
            self.observe_node(node)?;
        }
        Ok(())
    }

    fn observe_node(&mut self, expected_node: &'a NormalizedNodeV1) -> Result<(), HarnessError> {
        let Some(owner) = self.nodes.get(expected_node.path()).copied() else {
            return Ok(());
        };
        record_total(
            &mut self.observed_properties,
            expected_node.properties().len(),
            HarnessLimitKind::NormalizedProperties,
            self.limits.normalized_properties(),
            &mut self.crossings,
        );

        let child_count = self.view.child_count(owner);
        if let Some(count) = child_count {
            record_total(
                &mut self.observed_nodes,
                count,
                HarnessLimitKind::NormalizedNodes,
                self.limits.normalized_nodes(),
                &mut self.crossings,
            );
        }
        let mut child_offset = child_count.map(|_| 0_usize);
        let expected = self.expected;
        for group in expected_node.child_groups() {
            match group {
                NormalizedChildGroupV1::Static(path) => {
                    self.observe_static(owner, child_count, &mut child_offset, path)?;
                }
                NormalizedChildGroupV1::Region(path) => {
                    let fragment = expected.fragment(path).ok_or_else(state_mismatch)?;
                    self.observe_region(owner, &mut child_offset, fragment)?;
                }
            }
        }
        Ok(())
    }

    fn observe_static(
        &mut self,
        owner: NodeId,
        child_count: Option<usize>,
        child_offset: &mut Option<usize>,
        path: &'a NodePathV1,
    ) -> Result<(), HarnessError> {
        self.record_path_depth(path.depth());
        let Some(offset) = *child_offset else {
            return Ok(());
        };
        if child_count.is_some_and(|count| offset < count)
            && let Some(child) = self.view.child_at(owner, offset)
        {
            insert_node(&mut self.nodes, path, child)?;
        }
        *child_offset = offset.checked_add(1);
        Ok(())
    }

    fn observe_region(
        &mut self,
        owner: NodeId,
        child_offset: &mut Option<usize>,
        expected: &'a NormalizedFragmentV1,
    ) -> Result<(), HarnessError> {
        let Some(fragment) = self.view.fragment(owner, expected.descriptor()) else {
            *child_offset = None;
            return Ok(());
        };
        record_total(
            &mut self.observed_fragments,
            1,
            HarnessLimitKind::NormalizedFragments,
            self.limits.normalized_fragments(),
            &mut self.crossings,
        );

        let Some(count) = self.view.keyed_count(fragment) else {
            *child_offset = None;
            return Ok(());
        };
        let membership_crossed = record_total(
            &mut self.observed_memberships,
            count,
            HarnessLimitKind::LiveMemberships,
            self.limits.live_memberships(),
            &mut self.crossings,
        );
        if count > 0 {
            let member_depth = expected.path().owner().depth().saturating_add(1);
            self.record_path_depth(member_depth);
        }
        if membership_crossed {
            *child_offset = None;
            return Ok(());
        }
        if let Some(offset) = *child_offset {
            *child_offset = offset.checked_add(count);
        }

        self.bind_expected_members(fragment, count, expected)?;
        Ok(())
    }

    fn bind_expected_members(
        &mut self,
        fragment: FragmentId,
        count: usize,
        expected: &'a NormalizedFragmentV1,
    ) -> Result<(), HarnessError> {
        for member in expected.members() {
            let mut found = None;
            let mut duplicate = false;
            for index in 0..count {
                let Some((key, node)) = self.view.keyed_at(fragment, index) else {
                    found = None;
                    duplicate = true;
                    break;
                };
                if key == member.key() {
                    duplicate = found.replace(node).is_some();
                }
            }
            if let Some(node) = found
                && !duplicate
            {
                insert_node(&mut self.nodes, member.node(), node)?;
            }
        }
        Ok(())
    }

    fn record_path_depth(&mut self, depth: usize) {
        self.record_if_over(HarnessLimitKind::PathDepth, depth, self.limits.path_depth());
    }

    fn record_if_over(&mut self, kind: HarnessLimitKind, value: usize, limit: usize) {
        if value > limit {
            self.crossings.record(kind);
        }
    }
}

fn insert_node<'a>(
    nodes: &mut BTreeMap<&'a NodePathV1, NodeId>,
    path: &'a NodePathV1,
    node: NodeId,
) -> Result<(), HarnessError> {
    if nodes.insert(path, node).is_some() {
        Err(state_mismatch())
    } else {
        Ok(())
    }
}

fn record_total(
    current: &mut usize,
    amount: usize,
    kind: HarnessLimitKind,
    limit: usize,
    crossings: &mut ObservationLimitCrossingsV1,
) -> bool {
    let Some(next) = current.checked_add(amount) else {
        *current = usize::MAX;
        crossings.record(kind);
        return true;
    };
    *current = next;
    if next > limit {
        crossings.record(kind);
        true
    } else {
        false
    }
}
