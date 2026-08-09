use fenestra_ui_ir::prototype::StructuralRegionId;
use fenestra_ui_runtime::prototype::{CommittedRuntimeSnapshot, UiRuntime};

use super::super::super::{
    ObservationOutcomeV1, ObservedSnapshotV1, observe_snapshot_against_view_v1,
    observe_snapshot_indexed_v1,
};
use super::view::{DefectViewV1, NodeViewDefectV1, ViewDefectV1};
use crate::case::SemanticOperationV1;
use crate::desired::DesiredStateV1;
use crate::error::HarnessError;
use crate::fingerprint::{
    FailureFingerprintKindV1, FingerprintFieldV1, FingerprintLocationV1, FingerprintSummaryV1,
};
use crate::fixture::RuntimeOracleFixtureV1;
use crate::identity::IdentityIndexV1;
use crate::model::clean_rebuild_v1;
use crate::semantic::{FragmentPathV1, NodePathV1, NormalizedStateV1};

pub(in crate::observe::tests) struct ObservationCaseV1 {
    fixture: RuntimeOracleFixtureV1,
    snapshot: CommittedRuntimeSnapshot,
    expected: NormalizedStateV1,
    identities: IdentityIndexV1,
}

impl ObservationCaseV1 {
    pub(in crate::observe::tests) fn initial() -> Self {
        let fixture = RuntimeOracleFixtureV1::build().expect("fixture should validate");
        let desired =
            DesiredStateV1::from_construction(fixture.construction(), fixture.harness_limits())
                .expect("desired state should initialize");
        let runtime = UiRuntime::new(
            fixture.construction().clone(),
            fixture.replay_config().runtime_capacity(),
        )
        .expect("runtime should initialize");
        Self::from_committed(fixture, desired, runtime.committed())
    }

    pub(in crate::observe::tests) fn with_empty_root_fragments() -> Self {
        let fixture = RuntimeOracleFixtureV1::build().expect("fixture should validate");
        let mut desired =
            DesiredStateV1::from_construction(fixture.construction(), fixture.harness_limits())
                .expect("desired state should initialize");
        let mut runtime = UiRuntime::new(
            fixture.construction().clone(),
            fixture.replay_config().runtime_capacity(),
        )
        .expect("runtime should initialize");
        let base = runtime.committed();
        let root = base.root();
        let primary = base
            .fragment(root, StructuralRegionId::new(0))
            .expect("primary root fragment should exist");
        let secondary = base
            .fragment(root, StructuralRegionId::new(2))
            .expect("secondary root fragment should exist");
        let mut transaction = runtime.begin_transaction();
        transaction
            .remove_keyed(primary, 7)
            .expect("primary key seven removal should stage");
        transaction
            .remove_keyed(primary, 8)
            .expect("primary key eight removal should stage");
        transaction
            .remove_keyed(secondary, 7)
            .expect("secondary key seven removal should stage");
        let receipt = runtime
            .commit(transaction)
            .expect("empty-fragment setup should commit");
        assert_eq!(receipt.mutations().count(), 3);
        drop(receipt);
        drop(base);

        let primary_path = FragmentPathV1::new(NodePathV1::root(), 1);
        let secondary_path = FragmentPathV1::new(NodePathV1::root(), 2);
        for operation in [
            SemanticOperationV1::RemoveKeyed {
                fragment: primary_path.clone(),
                key: 7,
            },
            SemanticOperationV1::RemoveKeyed {
                fragment: primary_path,
                key: 8,
            },
            SemanticOperationV1::RemoveKeyed {
                fragment: secondary_path,
                key: 7,
            },
        ] {
            desired
                .apply_operation(&operation, fixture.harness_limits())
                .expect("desired removal should apply");
        }
        Self::from_committed(fixture, desired, runtime.committed())
    }

    fn from_committed(
        fixture: RuntimeOracleFixtureV1,
        desired: DesiredStateV1,
        snapshot: CommittedRuntimeSnapshot,
    ) -> Self {
        let expected = clean_rebuild_v1(fixture.construction(), &desired, fixture.harness_limits())
            .expect("clean state should rebuild");
        let ObservedSnapshotV1 { identities, .. } = observe_snapshot_indexed_v1(
            fixture.construction(),
            &snapshot,
            fixture.harness_limits(),
        )
        .expect("clean snapshot should observe");
        Self {
            fixture,
            snapshot,
            expected,
            identities,
        }
    }

    pub(in crate::observe::tests) fn expected(&self) -> &NormalizedStateV1 {
        &self.expected
    }

    pub(in crate::observe::tests) fn observe(
        &self,
        defect: Option<NodeViewDefectV1>,
    ) -> Result<ObservationOutcomeV1, HarnessError> {
        self.observe_defects(defect.into_iter().map(ViewDefectV1::from).collect())
    }

    pub(in crate::observe::tests) fn observe_defects(
        &self,
        defects: Vec<ViewDefectV1>,
    ) -> Result<ObservationOutcomeV1, HarnessError> {
        let view = DefectViewV1::new(&self.snapshot, &self.identities, &self.expected, defects);
        observe_snapshot_against_view_v1(
            self.fixture.construction(),
            &self.expected,
            &view,
            self.fixture.harness_limits(),
        )
    }
}

pub(in crate::observe::tests) fn assert_state_mismatch(
    result: Result<ObservationOutcomeV1, HarnessError>,
    location: FingerprintLocationV1,
    field: FingerprintFieldV1,
    expected: FingerprintSummaryV1,
    observed: FingerprintSummaryV1,
) {
    let outcome = result.expect("diagnostic observation should remain operational");
    let ObservationOutcomeV1::Mismatch(fingerprint) = outcome else {
        panic!("observation should report a mismatch");
    };
    assert_eq!(fingerprint.kind(), FailureFingerprintKindV1::StateMismatch);
    assert_eq!(fingerprint.location(), &location);
    assert_eq!(fingerprint.field(), field);
    assert_eq!(fingerprint.expected(), &expected);
    assert_eq!(fingerprint.observed(), &observed);
}
