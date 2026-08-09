mod alias;

use std::collections::BTreeMap;

use fenestra_ui_runtime::prototype::{CommittedRuntimeSnapshot, FragmentId, NodeId};

use crate::desired::DesiredStateV1;
use crate::error::{HarnessError, HarnessErrorKind};
use crate::fingerprint::{FailureFingerprintV1, FingerprintLocationV1, FingerprintSummaryV1};
use crate::semantic::{FragmentPathV1, NodePathV1};

#[derive(Default)]
pub(crate) struct IdentityIndexV1 {
    nodes: BTreeMap<NodePathV1, NodeId>,
    fragments: BTreeMap<FragmentPathV1, FragmentId>,
    authored_nodes: Vec<(NodePathV1, NodeId)>,
    authored_fragments: Vec<(FragmentPathV1, FragmentId)>,
}

impl IdentityIndexV1 {
    pub(crate) fn record_node(&mut self, path: NodePathV1, node: NodeId) -> bool {
        if self.nodes.contains_key(&path) {
            return false;
        }
        self.nodes.insert(path.clone(), node);
        self.authored_nodes.push((path, node));
        true
    }

    pub(crate) fn record_fragment(&mut self, path: FragmentPathV1, fragment: FragmentId) -> bool {
        if self.fragments.contains_key(&path) {
            return false;
        }
        self.fragments.insert(path.clone(), fragment);
        self.authored_fragments.push((path, fragment));
        true
    }

    pub(crate) fn node(&self, path: &NodePathV1) -> Option<NodeId> {
        self.nodes.get(path).copied()
    }

    pub(crate) fn node_path(&self, node: NodeId) -> Option<&NodePathV1> {
        self.authored_nodes
            .iter()
            .find_map(|(path, candidate)| (*candidate == node).then_some(path))
    }

    pub(crate) fn fragment(&self, path: &FragmentPathV1) -> Option<FragmentId> {
        self.fragments.get(path).copied()
    }

    pub(crate) fn nodes_in_authored_order(&self) -> impl Iterator<Item = (&NodePathV1, NodeId)> {
        self.authored_nodes.iter().map(|(path, node)| (path, *node))
    }

    pub(crate) fn fragments_in_authored_order(
        &self,
    ) -> impl Iterator<Item = (&FragmentPathV1, FragmentId)> {
        self.authored_fragments
            .iter()
            .map(|(path, fragment)| (path, *fragment))
    }
}

/// Aggregated physical-identity checks from one deterministic replay.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct IdentitySummaryV1 {
    preserved: u32,
    retired: u32,
    fresh: u32,
    alias_free_snapshots: u32,
}

impl IdentitySummaryV1 {
    /// Returns surviving node and fragment identity checks.
    #[must_use]
    pub const fn preserved(self) -> u32 {
        self.preserved
    }

    /// Returns node and fragment handles verified stale after retirement.
    #[must_use]
    pub const fn retired(self) -> u32 {
        self.retired
    }

    /// Returns semantic paths verified fresh after a prior retirement.
    #[must_use]
    pub const fn fresh(self) -> u32 {
        self.fresh
    }

    /// Returns committed snapshots whose live handles were alias-free.
    #[must_use]
    pub const fn alias_free_snapshots(self) -> u32 {
        self.alias_free_snapshots
    }
}

#[derive(Clone)]
pub(crate) struct IdentityLedgerV1 {
    retired_nodes: BTreeMap<NodePathV1, NodeId>,
    retired_fragments: BTreeMap<FragmentPathV1, FragmentId>,
    summary: IdentitySummaryV1,
}

impl IdentityLedgerV1 {
    pub(crate) fn new() -> Self {
        Self {
            retired_nodes: BTreeMap::new(),
            retired_fragments: BTreeMap::new(),
            summary: IdentitySummaryV1::default(),
        }
    }

    fn record_alias_free_snapshot(&mut self) -> Result<(), HarnessError> {
        increment(&mut self.summary.alias_free_snapshots)
    }

    pub(crate) fn first_alias(
        identities: &IdentityIndexV1,
    ) -> Result<Option<FailureFingerprintV1>, HarnessError> {
        alias::first_alias_v1(identities)
    }

    pub(crate) fn verify_initial_aliases(
        &mut self,
        identities: &IdentityIndexV1,
    ) -> Result<Option<FailureFingerprintV1>, HarnessError> {
        if let Some(fingerprint) = Self::first_alias(identities)? {
            return Ok(Some(fingerprint));
        }

        let mut draft = self.clone();
        draft.record_alias_free_snapshot()?;
        *self = draft;
        Ok(None)
    }

    pub(crate) fn verify_transition(
        &mut self,
        before: &IdentityIndexV1,
        before_desired: &DesiredStateV1,
        after: &IdentityIndexV1,
        after_desired: &DesiredStateV1,
        after_snapshot: &CommittedRuntimeSnapshot,
    ) -> Result<Option<FailureFingerprintV1>, HarnessError> {
        if let Some(fingerprint) = Self::first_alias(after)? {
            return Ok(Some(fingerprint));
        }

        let mut draft = self.clone();
        if let Some(fingerprint) =
            draft.verify_nodes(before, before_desired, after, after_desired, after_snapshot)?
        {
            return Ok(Some(fingerprint));
        }
        if let Some(fingerprint) =
            draft.verify_fragments(before, before_desired, after, after_desired, after_snapshot)?
        {
            return Ok(Some(fingerprint));
        }
        draft.record_alias_free_snapshot()?;
        *self = draft;
        Ok(None)
    }

    pub(crate) const fn summary(&self) -> IdentitySummaryV1 {
        self.summary
    }

    fn verify_nodes(
        &mut self,
        before: &IdentityIndexV1,
        before_desired: &DesiredStateV1,
        after: &IdentityIndexV1,
        after_desired: &DesiredStateV1,
        snapshot: &CommittedRuntimeSnapshot,
    ) -> Result<Option<FailureFingerprintV1>, HarnessError> {
        for (path, old) in before.nodes_in_authored_order() {
            let old_token = before_desired
                .incarnation_token(path)
                .ok_or_else(identity_error)?;
            match after.node(path) {
                Some(new) => {
                    let new_token = after_desired
                        .incarnation_token(path)
                        .ok_or_else(identity_error)?;
                    if old_token == new_token {
                        if old != new {
                            return identity_mismatch(
                                FingerprintLocationV1::Node(path.clone()),
                                FingerprintSummaryV1::LifecyclePreserved,
                                FingerprintSummaryV1::LifecycleFresh,
                            );
                        }
                        increment(&mut self.summary.preserved)?;
                    } else {
                        if old == new {
                            return identity_mismatch(
                                FingerprintLocationV1::Node(path.clone()),
                                FingerprintSummaryV1::LifecycleFresh,
                                FingerprintSummaryV1::LifecyclePreserved,
                            );
                        }
                        if node_is_live(snapshot, old) {
                            return identity_mismatch(
                                FingerprintLocationV1::Node(path.clone()),
                                FingerprintSummaryV1::LifecycleRetired,
                                FingerprintSummaryV1::LifecyclePreserved,
                            );
                        }
                        increment(&mut self.summary.retired)?;
                        increment(&mut self.summary.fresh)?;
                        self.retired_nodes.remove(path);
                    }
                }
                None => {
                    if node_is_live(snapshot, old) {
                        return identity_mismatch(
                            FingerprintLocationV1::Node(path.clone()),
                            FingerprintSummaryV1::LifecycleRetired,
                            FingerprintSummaryV1::LifecyclePreserved,
                        );
                    }
                    increment(&mut self.summary.retired)?;
                    self.retired_nodes.insert(path.clone(), old);
                }
            }
        }
        for (path, new) in after.nodes_in_authored_order() {
            if before.node(path).is_some() {
                continue;
            }
            if let Some(old) = self.retired_nodes.remove(path) {
                if old == new {
                    return identity_mismatch(
                        FingerprintLocationV1::Node(path.clone()),
                        FingerprintSummaryV1::LifecycleFresh,
                        FingerprintSummaryV1::LifecyclePreserved,
                    );
                }
                if node_is_live(snapshot, old) {
                    return identity_mismatch(
                        FingerprintLocationV1::Node(path.clone()),
                        FingerprintSummaryV1::LifecycleRetired,
                        FingerprintSummaryV1::LifecyclePreserved,
                    );
                }
                increment(&mut self.summary.fresh)?;
            }
        }
        Ok(None)
    }

    fn verify_fragments(
        &mut self,
        before: &IdentityIndexV1,
        before_desired: &DesiredStateV1,
        after: &IdentityIndexV1,
        after_desired: &DesiredStateV1,
        snapshot: &CommittedRuntimeSnapshot,
    ) -> Result<Option<FailureFingerprintV1>, HarnessError> {
        for (path, old) in before.fragments_in_authored_order() {
            let old_token = before_desired
                .incarnation_token(path.owner())
                .ok_or_else(identity_error)?;
            match after.fragment(path) {
                Some(new) => {
                    let new_token = after_desired
                        .incarnation_token(path.owner())
                        .ok_or_else(identity_error)?;
                    if old_token == new_token {
                        if old != new {
                            return identity_mismatch(
                                FingerprintLocationV1::Fragment(path.clone()),
                                FingerprintSummaryV1::LifecyclePreserved,
                                FingerprintSummaryV1::LifecycleFresh,
                            );
                        }
                        increment(&mut self.summary.preserved)?;
                    } else {
                        if old == new {
                            return identity_mismatch(
                                FingerprintLocationV1::Fragment(path.clone()),
                                FingerprintSummaryV1::LifecycleFresh,
                                FingerprintSummaryV1::LifecyclePreserved,
                            );
                        }
                        if fragment_is_live(snapshot, old) {
                            return identity_mismatch(
                                FingerprintLocationV1::Fragment(path.clone()),
                                FingerprintSummaryV1::LifecycleRetired,
                                FingerprintSummaryV1::LifecyclePreserved,
                            );
                        }
                        increment(&mut self.summary.retired)?;
                        increment(&mut self.summary.fresh)?;
                        self.retired_fragments.remove(path);
                    }
                }
                None => {
                    if fragment_is_live(snapshot, old) {
                        return identity_mismatch(
                            FingerprintLocationV1::Fragment(path.clone()),
                            FingerprintSummaryV1::LifecycleRetired,
                            FingerprintSummaryV1::LifecyclePreserved,
                        );
                    }
                    increment(&mut self.summary.retired)?;
                    self.retired_fragments.insert(path.clone(), old);
                }
            }
        }
        for (path, new) in after.fragments_in_authored_order() {
            if before.fragment(path).is_some() {
                continue;
            }
            if let Some(old) = self.retired_fragments.remove(path) {
                if old == new {
                    return identity_mismatch(
                        FingerprintLocationV1::Fragment(path.clone()),
                        FingerprintSummaryV1::LifecycleFresh,
                        FingerprintSummaryV1::LifecyclePreserved,
                    );
                }
                if fragment_is_live(snapshot, old) {
                    return identity_mismatch(
                        FingerprintLocationV1::Fragment(path.clone()),
                        FingerprintSummaryV1::LifecycleRetired,
                        FingerprintSummaryV1::LifecyclePreserved,
                    );
                }
                increment(&mut self.summary.fresh)?;
            }
        }
        Ok(None)
    }
}

fn node_is_live(snapshot: &CommittedRuntimeSnapshot, node: NodeId) -> bool {
    snapshot.template(node).is_some()
}

fn fragment_is_live(snapshot: &CommittedRuntimeSnapshot, fragment: FragmentId) -> bool {
    snapshot.keyed_members(fragment).is_some()
}

fn identity_mismatch(
    location: FingerprintLocationV1,
    expected: FingerprintSummaryV1,
    observed: FingerprintSummaryV1,
) -> Result<Option<FailureFingerprintV1>, HarnessError> {
    FailureFingerprintV1::identity_mismatch(location, expected, observed)
        .map(Some)
        .ok_or_else(identity_error)
}

fn increment(value: &mut u32) -> Result<(), HarnessError> {
    *value = value.checked_add(1).ok_or_else(arithmetic_error)?;
    Ok(())
}

fn identity_error() -> HarnessError {
    HarnessError::new(HarnessErrorKind::IdentityMismatch)
}

fn arithmetic_error() -> HarnessError {
    HarnessError::new(HarnessErrorKind::ArithmeticExhausted)
}

#[cfg(test)]
mod tests;
