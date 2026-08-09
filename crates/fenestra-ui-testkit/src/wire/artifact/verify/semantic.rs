use crate::case::{GeneratedCaseV1, SemanticOperationV1};
use crate::desired::DesiredStateV1;
use crate::fixture::RuntimeOracleFixtureV1;
use crate::model::clean_rebuild_v1;
use crate::semantic::NormalizedStateV1;
use crate::wire::error::{ArtifactVerificationError, ArtifactVerificationErrorKind};

pub(super) fn validate_artifact_operations_v1(
    fixture: &RuntimeOracleFixtureV1,
    original: &GeneratedCaseV1,
    minimized: &GeneratedCaseV1,
) -> Result<(), ArtifactVerificationError> {
    validate_case_operations_v1(fixture, original)?;
    validate_case_operations_v1(fixture, minimized)
}

fn validate_case_operations_v1(
    fixture: &RuntimeOracleFixtureV1,
    case: &GeneratedCaseV1,
) -> Result<(), ArtifactVerificationError> {
    let limits = fixture.harness_limits();
    let mut desired = DesiredStateV1::from_construction(fixture.construction(), limits)
        .map_err(|_| fixture_mismatch())?;

    for transaction in case.transactions() {
        let base = clean_rebuild_v1(fixture.construction(), &desired, limits)
            .map_err(|_| invalid_operation().at_transaction(transaction.id()))?;
        let mut draft = desired.clone();
        for operation in transaction.operations() {
            if !target_is_from_base_v1(&base, &desired, &draft, operation.operation()) {
                return Err(invalid_operation().at_operation(transaction.id(), operation.id()));
            }
            draft
                .apply_operation(operation.operation(), limits)
                .map_err(|_| invalid_operation().at_operation(transaction.id(), operation.id()))?;
        }
        desired = draft;
    }
    Ok(())
}

fn target_is_from_base_v1(
    base: &NormalizedStateV1,
    desired: &DesiredStateV1,
    draft: &DesiredStateV1,
    operation: &SemanticOperationV1,
) -> bool {
    match operation {
        SemanticOperationV1::SetProperty { node, .. } => {
            base.contains_node(node) && desired.preserves_incarnation(draft, node)
        }
        SemanticOperationV1::InsertKeyed { fragment, .. }
        | SemanticOperationV1::MoveKeyed { fragment, .. }
        | SemanticOperationV1::UpdateKeyed { fragment, .. }
        | SemanticOperationV1::RemoveKeyed { fragment, .. } => {
            base.contains_fragment(fragment)
                && desired.preserves_incarnation(draft, fragment.owner())
        }
    }
}

fn fixture_mismatch() -> ArtifactVerificationError {
    ArtifactVerificationError::new(ArtifactVerificationErrorKind::FixtureMismatch)
}

fn invalid_operation() -> ArtifactVerificationError {
    ArtifactVerificationError::new(ArtifactVerificationErrorKind::InvalidSemanticOperation)
}
