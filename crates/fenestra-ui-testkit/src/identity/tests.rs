use fenestra_ui_runtime::prototype::{CommittedRuntimeSnapshot, FragmentId, NodeId, UiRuntime};

use super::{IdentityIndexV1, IdentityLedgerV1, IdentitySummaryV1};
use crate::case::SemanticOperationV1;
use crate::desired::DesiredStateV1;
use crate::fingerprint::{
    FailureFingerprintKindV1, FailureFingerprintV1, FingerprintFieldV1, FingerprintLocationV1,
    FingerprintSummaryV1,
};
use crate::fixture::RuntimeOracleFixtureV1;
use crate::observe::observe_snapshot_indexed_v1;
use crate::semantic::{FragmentPathV1, NodePathV1};

mod atomicity;
mod initial;
mod lifecycle;

struct RemovalTransition {
    fixture: RuntimeOracleFixtureV1,
    before_desired: DesiredStateV1,
    after_desired: DesiredStateV1,
    before_snapshot: CommittedRuntimeSnapshot,
    after_snapshot: CommittedRuntimeSnapshot,
    before: IdentityIndexV1,
    after: IdentityIndexV1,
}

fn initial_state() -> (
    RuntimeOracleFixtureV1,
    DesiredStateV1,
    CommittedRuntimeSnapshot,
    IdentityIndexV1,
) {
    let fixture = RuntimeOracleFixtureV1::build().expect("fixture should validate");
    let desired =
        DesiredStateV1::from_construction(fixture.construction(), fixture.harness_limits())
            .expect("desired state should initialize");
    let runtime = UiRuntime::new(
        fixture.construction().clone(),
        fixture
            .replay_config()
            .runtime_capacity()
            .with_retained_generations(4),
    )
    .expect("runtime should initialize");
    let snapshot = runtime.committed();
    let identities = observe_index(&fixture, &snapshot);
    (fixture, desired, snapshot, identities)
}

fn removal_transition() -> RemovalTransition {
    let fixture = RuntimeOracleFixtureV1::build().expect("fixture should validate");
    let before_desired =
        DesiredStateV1::from_construction(fixture.construction(), fixture.harness_limits())
            .expect("desired state should initialize");
    let mut runtime = UiRuntime::new(
        fixture.construction().clone(),
        fixture
            .replay_config()
            .runtime_capacity()
            .with_retained_generations(4),
    )
    .expect("runtime should initialize");
    let before_snapshot = runtime.committed();
    let before = observe_index(&fixture, &before_snapshot);
    let primary_path = primary_fragment();
    let primary = before
        .fragment(&primary_path)
        .expect("primary fragment should exist");
    let mut transaction = runtime.begin_transaction();
    transaction
        .remove_keyed(primary, 7)
        .expect("removal should stage");
    let receipt = runtime.commit(transaction).expect("removal should commit");
    drop(receipt);
    let after_snapshot = runtime.committed();
    let after = observe_index(&fixture, &after_snapshot);
    let mut after_desired = before_desired.clone();
    after_desired
        .apply_operation(
            &SemanticOperationV1::RemoveKeyed {
                fragment: primary_path,
                key: 7,
            },
            fixture.harness_limits(),
        )
        .expect("desired removal should apply");

    RemovalTransition {
        fixture,
        before_desired,
        after_desired,
        before_snapshot,
        after_snapshot,
        before,
        after,
    }
}

fn observe_index(
    fixture: &RuntimeOracleFixtureV1,
    snapshot: &CommittedRuntimeSnapshot,
) -> IdentityIndexV1 {
    let observed =
        observe_snapshot_indexed_v1(fixture.construction(), snapshot, fixture.harness_limits())
            .expect("snapshot should observe");
    copy_index(observed.identities())
}

fn copy_index(source: &IdentityIndexV1) -> IdentityIndexV1 {
    build_index(
        source
            .nodes_in_authored_order()
            .map(|(path, id)| (path.clone(), id)),
        source
            .fragments_in_authored_order()
            .map(|(path, id)| (path.clone(), id)),
    )
}

fn build_index(
    nodes: impl IntoIterator<Item = (NodePathV1, NodeId)>,
    fragments: impl IntoIterator<Item = (FragmentPathV1, FragmentId)>,
) -> IdentityIndexV1 {
    let mut index = IdentityIndexV1::default();
    for (path, id) in nodes {
        assert!(index.record_node(path, id));
    }
    for (path, id) in fragments {
        assert!(index.record_fragment(path, id));
    }
    index
}

fn replace_node(
    source: &IdentityIndexV1,
    target: &NodePathV1,
    replacement: NodeId,
) -> IdentityIndexV1 {
    build_index(
        source
            .nodes_in_authored_order()
            .map(|(path, id)| (path.clone(), if path == target { replacement } else { id })),
        source
            .fragments_in_authored_order()
            .map(|(path, id)| (path.clone(), id)),
    )
}

fn replace_fragment(
    source: &IdentityIndexV1,
    target: &FragmentPathV1,
    replacement: FragmentId,
) -> IdentityIndexV1 {
    build_index(
        source
            .nodes_in_authored_order()
            .map(|(path, id)| (path.clone(), id)),
        source
            .fragments_in_authored_order()
            .map(|(path, id)| (path.clone(), if path == target { replacement } else { id })),
    )
}

fn without_node_subtree(source: &IdentityIndexV1, retired: &NodePathV1) -> IdentityIndexV1 {
    build_index(
        source
            .nodes_in_authored_order()
            .filter(|(path, _)| !is_within(path, retired))
            .map(|(path, id)| (path.clone(), id)),
        source
            .fragments_in_authored_order()
            .filter(|(path, _)| !is_within(path.owner(), retired))
            .map(|(path, id)| (path.clone(), id)),
    )
}

fn fragments_only(source: &IdentityIndexV1) -> IdentityIndexV1 {
    build_index(
        [],
        source
            .fragments_in_authored_order()
            .map(|(path, id)| (path.clone(), id)),
    )
}

fn without_fragment(source: &IdentityIndexV1, retired: &FragmentPathV1) -> IdentityIndexV1 {
    build_index(
        source
            .nodes_in_authored_order()
            .map(|(path, id)| (path.clone(), id)),
        source
            .fragments_in_authored_order()
            .filter(|(path, _)| *path != retired)
            .map(|(path, id)| (path.clone(), id)),
    )
}

fn is_within(candidate: &NodePathV1, ancestor: &NodePathV1) -> bool {
    candidate.segments().starts_with(ancestor.segments())
}

fn primary_fragment() -> FragmentPathV1 {
    FragmentPathV1::new(NodePathV1::root(), 1)
}

fn retired_member() -> NodePathV1 {
    NodePathV1::root().member(1, 7)
}

fn retired_nested_fragment() -> FragmentPathV1 {
    FragmentPathV1::new(retired_member(), 1)
}

fn expect_mismatch(
    result: Result<Option<FailureFingerprintV1>, crate::error::HarnessError>,
    location: FingerprintLocationV1,
    expected: FingerprintSummaryV1,
    observed: FingerprintSummaryV1,
) {
    let fingerprint = result
        .expect("identity verification should remain operational")
        .expect("identity mismatch should be fingerprinted");
    assert_eq!(
        fingerprint.kind(),
        FailureFingerprintKindV1::IdentityMismatch
    );
    assert_eq!(fingerprint.location(), &location);
    assert_eq!(fingerprint.field(), FingerprintFieldV1::IdentityLifecycle);
    assert_eq!(fingerprint.expected(), &expected);
    assert_eq!(fingerprint.observed(), &observed);
}
