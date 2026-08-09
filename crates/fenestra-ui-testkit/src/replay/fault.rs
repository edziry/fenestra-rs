use crate::case::{GeneratedCaseV1, OperationV1, SemanticOperationV1};
use crate::error::{HarnessError, HarnessErrorKind};
use crate::trace::TraceFaultV1;

pub(super) struct FaultAdapterV1 {
    fault: Option<TraceFaultV1>,
}

impl FaultAdapterV1 {
    pub(super) fn validate(
        case: &GeneratedCaseV1,
        fault: Option<TraceFaultV1>,
    ) -> Result<Self, HarnessError> {
        let Some(fault) = fault else {
            return Ok(Self { fault: None });
        };
        let target = fault.target();
        let mut found = None;
        for transaction in case.transactions() {
            for operation in transaction.operations() {
                if operation.id() != target {
                    continue;
                }
                if found.is_some()
                    || !matches!(operation.operation(), SemanticOperationV1::MoveKeyed { .. })
                {
                    return Err(invalid_fault().at_operation(transaction.id(), operation.id()));
                }
                found = Some((transaction.id(), operation.id()));
            }
        }
        if found.is_none() {
            return Err(invalid_fault());
        }
        Ok(Self { fault: Some(fault) })
    }

    pub(super) fn omits(&self, operation: &OperationV1) -> bool {
        self.fault
            .is_some_and(|fault| fault.target() == operation.id())
    }
}

fn invalid_fault() -> HarnessError {
    HarnessError::new(HarnessErrorKind::InvalidOperation)
}
