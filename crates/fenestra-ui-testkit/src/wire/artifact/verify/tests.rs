use fenestra_ui_ir::prototype::{PropertyId, PropertyValue};

use super::verify_fault_free_replay_v1;
use crate::case::{
    GeneratedCaseV1, GeneratorConfigV1, OperationIdV1, OperationV1, REGISTERED_FIXTURE_REVISION_V1,
    SeedV1, SemanticOperationV1, TransactionIdV1, TransactionV1,
};
use crate::fixture::RuntimeOracleFixtureV1;
use crate::semantic::NodePathV1;
use crate::wire::error::ArtifactVerificationErrorKind;

#[test]
fn fault_free_replay_guard_preserves_the_closed_error_and_location() {
    let transaction = TransactionIdV1::new(2);
    let operation = OperationIdV1::new(4);
    let invalid_case = GeneratedCaseV1::new(
        REGISTERED_FIXTURE_REVISION_V1,
        GeneratorConfigV1::new(8, 2, 8),
        SeedV1::new(0),
        vec![TransactionV1::new(
            transaction,
            vec![OperationV1::new(
                operation,
                SemanticOperationV1::SetProperty {
                    node: NodePathV1::root().static_child(u16::MAX),
                    property: PropertyId::new(0),
                    value: PropertyValue::ScalarI32(320),
                },
            )],
        )],
    );
    let fixture = RuntimeOracleFixtureV1::build().expect("fixture should validate");

    let error = verify_fault_free_replay_v1(&fixture, &invalid_case)
        .expect_err("a fault-free replay regression should fail verification");
    assert_eq!(
        error.kind(),
        ArtifactVerificationErrorKind::FaultFreeReplayFailed
    );
    assert_eq!(error.transaction(), Some(transaction));
    assert_eq!(error.operation(), Some(operation));
}
