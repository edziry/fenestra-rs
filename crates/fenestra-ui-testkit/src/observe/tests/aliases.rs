use super::support::{ObservationCaseV1, ViewDefectV1};
use crate::fingerprint::{
    FailureFingerprintKindV1, FingerprintFieldV1, FingerprintLocationV1, FingerprintSummaryV1,
};
use crate::identity::IdentityLedgerV1;
use crate::observe::{ObservationOutcomeV1, ObservedSnapshotV1};
use crate::semantic::{FragmentPathV1, NodePathV1};

#[test]
fn coherent_node_alias_preserves_state_and_reports_the_target_path() {
    let case = ObservationCaseV1::initial();
    let source = NodePathV1::root().member(1, 7);
    let target = NodePathV1::root().member(1, 8);

    let observed = expect_complete(
        &case,
        vec![ViewDefectV1::NodeAlias {
            source,
            target: target.clone(),
        }],
    );
    assert_alias(
        IdentityLedgerV1::first_alias(observed.identities()),
        FingerprintLocationV1::Node(target),
    );
}

#[test]
fn coherent_empty_fragment_alias_preserves_state_and_reports_the_target_path() {
    let case = ObservationCaseV1::with_empty_root_fragments();
    let source = FragmentPathV1::new(NodePathV1::root(), 1);
    let target = FragmentPathV1::new(NodePathV1::root(), 2);

    let observed = expect_complete(
        &case,
        vec![ViewDefectV1::FragmentAlias {
            source,
            target: target.clone(),
        }],
    );
    assert_alias(
        IdentityLedgerV1::first_alias(observed.identities()),
        FingerprintLocationV1::Fragment(target),
    );
}

fn expect_complete(case: &ObservationCaseV1, defects: Vec<ViewDefectV1>) -> ObservedSnapshotV1 {
    let outcome = case
        .observe_defects(defects)
        .expect("coherent alias observation should remain operational");
    let ObservationOutcomeV1::Complete(observed) = outcome else {
        panic!("coherent alias should preserve normalized state");
    };
    assert_eq!(observed.normalized(), case.expected());
    observed
}

fn assert_alias(
    result: Result<Option<crate::fingerprint::FailureFingerprintV1>, crate::error::HarnessError>,
    location: FingerprintLocationV1,
) {
    let fingerprint = result
        .expect("alias inspection should remain operational")
        .expect("alias inspection should find a duplicate identity");
    assert_eq!(
        fingerprint.kind(),
        FailureFingerprintKindV1::IdentityMismatch
    );
    assert_eq!(fingerprint.location(), &location);
    assert_eq!(fingerprint.field(), FingerprintFieldV1::IdentityLifecycle);
    assert_eq!(
        fingerprint.expected(),
        &FingerprintSummaryV1::LifecycleDistinct
    );
    assert_eq!(
        fingerprint.observed(),
        &FingerprintSummaryV1::LifecycleAliased
    );
}
