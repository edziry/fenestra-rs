mod bind;
mod diagnose;
mod limits;
mod partial;
mod view;

#[cfg(test)]
mod tests;

use fenestra_ui_ir::prototype::ValidatedConstruction;
use fenestra_ui_runtime::prototype::CommittedRuntimeSnapshot;

use crate::error::HarnessError;
use crate::fingerprint::FailureFingerprintV1;
use crate::fixture::HarnessLimitsV1;
use crate::identity::IdentityIndexV1;
use crate::semantic::NormalizedStateV1;

pub(crate) enum ObservationOutcomeV1 {
    Complete(ObservedSnapshotV1),
    Mismatch(FailureFingerprintV1),
}

pub(crate) struct ObservedSnapshotV1 {
    normalized: NormalizedStateV1,
    identities: IdentityIndexV1,
}

impl ObservedSnapshotV1 {
    pub(super) fn new(normalized: NormalizedStateV1, identities: IdentityIndexV1) -> Self {
        Self {
            normalized,
            identities,
        }
    }

    pub(crate) fn normalized(&self) -> &NormalizedStateV1 {
        &self.normalized
    }

    pub(crate) fn identities(&self) -> &IdentityIndexV1 {
        &self.identities
    }

    fn into_normalized(self) -> NormalizedStateV1 {
        self.normalized
    }
}

/// Observes a committed runtime state through public queries only.
pub fn observe_snapshot_v1(
    construction: &ValidatedConstruction,
    snapshot: &CommittedRuntimeSnapshot,
    limits: HarnessLimitsV1,
) -> Result<NormalizedStateV1, HarnessError> {
    observe_snapshot_indexed_v1(construction, snapshot, limits)
        .map(ObservedSnapshotV1::into_normalized)
}

pub(crate) fn observe_snapshot_indexed_v1(
    construction: &ValidatedConstruction,
    snapshot: &CommittedRuntimeSnapshot,
    limits: HarnessLimitsV1,
) -> Result<ObservedSnapshotV1, HarnessError> {
    partial::observe_snapshot_indexed_v1(construction, snapshot, limits)
}

pub(crate) fn observe_snapshot_against_view_v1<V: view::SnapshotViewV1 + ?Sized>(
    _construction: &ValidatedConstruction,
    expected: &NormalizedStateV1,
    view: &V,
    limits: HarnessLimitsV1,
) -> Result<ObservationOutcomeV1, HarnessError> {
    let observed = bind::bind_expected_paths_v1(expected, view, limits)?;
    if let Some(fingerprint) =
        diagnose::diagnose_node_fields_v1(expected, view, observed.identities())?
    {
        return Ok(ObservationOutcomeV1::Mismatch(fingerprint));
    }
    if let Some(fingerprint) = diagnose::diagnose_fragments_v1(expected, &observed)? {
        return Ok(ObservationOutcomeV1::Mismatch(fingerprint));
    }
    if let Some(fingerprint) = diagnose::diagnose_child_order_v1(expected, &observed)? {
        return Ok(ObservationOutcomeV1::Mismatch(fingerprint));
    }
    if let Some(fingerprint) = diagnose::diagnose_counts_v1(expected, &observed)? {
        return Ok(ObservationOutcomeV1::Mismatch(fingerprint));
    }
    Ok(ObservationOutcomeV1::Complete(ObservedSnapshotV1::new(
        expected.clone(),
        observed.into_identities(),
    )))
}
