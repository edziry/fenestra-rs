use fenestra_ui_ir::prototype::PropertyValue;

use super::{ReducerError, ReducerErrorKind};
use crate::case::{GeneratedCaseV1, SemanticOperationV1};
use crate::wire::encode_case_v1;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct ReductionMetricV1 {
    operations: u32,
    canonical_bytes: u32,
    magnitude: u128,
}

impl ReductionMetricV1 {
    #[cfg(test)]
    pub(super) const fn operations(self) -> u32 {
        self.operations
    }

    #[cfg(test)]
    pub(super) const fn canonical_bytes(self) -> u32 {
        self.canonical_bytes
    }

    #[cfg(test)]
    pub(super) const fn magnitude(self) -> u128 {
        self.magnitude
    }
}

pub(super) fn measure_case_v1(case: &GeneratedCaseV1) -> Result<ReductionMetricV1, ReducerError> {
    let encoded = encode_case_v1(case)
        .map_err(|_| ReducerError::new(ReducerErrorKind::MetricLimitExceeded))?;
    let operations = u32::try_from(case.operation_count()).map_err(|_| arithmetic_error())?;
    let canonical_bytes = u32::try_from(encoded.len()).map_err(|_| arithmetic_error())?;
    let magnitude = operand_magnitude(case)?;

    Ok(ReductionMetricV1 {
        operations,
        canonical_bytes,
        magnitude,
    })
}

fn operand_magnitude(case: &GeneratedCaseV1) -> Result<u128, ReducerError> {
    let mut magnitude = 0_u128;
    for operation in case
        .transactions()
        .iter()
        .flat_map(|transaction| transaction.operations())
    {
        match operation.operation() {
            SemanticOperationV1::SetProperty { value, .. } => {
                add_magnitude(&mut magnitude, scalar_magnitude(value))?;
            }
            SemanticOperationV1::InsertKeyed {
                key, final_index, ..
            }
            | SemanticOperationV1::MoveKeyed {
                key, final_index, ..
            } => {
                add_magnitude(&mut magnitude, u128::from(*key))?;
                add_magnitude(&mut magnitude, u128::from(*final_index))?;
            }
            SemanticOperationV1::UpdateKeyed { key, value, .. } => {
                add_magnitude(&mut magnitude, u128::from(*key))?;
                add_magnitude(&mut magnitude, scalar_magnitude(value))?;
            }
            SemanticOperationV1::RemoveKeyed { key, .. } => {
                add_magnitude(&mut magnitude, u128::from(*key))?;
            }
        }
    }
    Ok(magnitude)
}

fn scalar_magnitude(value: &PropertyValue) -> u128 {
    let PropertyValue::ScalarI32(value) = value else {
        return 0;
    };
    u128::from(i64::from(*value).unsigned_abs())
}

fn add_magnitude(total: &mut u128, operand: u128) -> Result<(), ReducerError> {
    *total = total.checked_add(operand).ok_or_else(arithmetic_error)?;
    Ok(())
}

fn arithmetic_error() -> ReducerError {
    ReducerError::new(ReducerErrorKind::ArithmeticExhausted)
}

#[cfg(test)]
mod tests {
    use fenestra_ui_ir::prototype::{PropertyId, PropertyValue};

    use super::measure_case_v1;
    use crate::case::{
        GeneratedCaseV1, GeneratorConfigV1, OperationIdV1, OperationV1, SeedV1,
        SemanticOperationV1, TransactionIdV1, TransactionV1,
    };
    use crate::reducer::ReducerErrorKind;
    use crate::semantic::{FragmentPathV1, NodePathV1};
    use crate::wire::encode_case_v1;

    #[test]
    fn metric_counts_only_explicit_reducible_operands() {
        let owner = NodePathV1::root().member(7, u64::MAX);
        let fragment = FragmentPathV1::new(owner.clone(), 9);
        let case = test_case(vec![
            SemanticOperationV1::SetProperty {
                node: owner,
                property: PropertyId::new(u32::MAX),
                value: PropertyValue::ScalarI32(i32::MIN),
            },
            SemanticOperationV1::InsertKeyed {
                fragment: fragment.clone(),
                key: 7,
                final_index: 3,
            },
            SemanticOperationV1::UpdateKeyed {
                fragment: fragment.clone(),
                key: 11,
                property: PropertyId::new(u32::MAX),
                value: PropertyValue::ScalarI32(-5),
            },
            SemanticOperationV1::RemoveKeyed { fragment, key: 13 },
        ]);

        let metric = measure_case_v1(&case).expect("bounded case should have a metric");

        assert_eq!(metric.operations(), 4);
        assert_eq!(
            metric.canonical_bytes(),
            u32::try_from(
                encode_case_v1(&case)
                    .expect("bounded case should encode")
                    .len()
            )
            .expect("bounded case byte count should fit")
        );
        assert_eq!(metric.magnitude(), 2_147_483_687);
    }

    #[test]
    fn case_encoding_limits_map_to_the_metric_error() {
        let mut node = NodePathV1::root();
        for slot in 0..33 {
            node = node.static_child(slot);
        }
        let case = test_case(vec![SemanticOperationV1::SetProperty {
            node,
            property: PropertyId::new(0),
            value: PropertyValue::Bool(false),
        }]);

        let error = measure_case_v1(&case).expect_err("deep path should exceed case limits");

        assert_eq!(error.kind(), ReducerErrorKind::MetricLimitExceeded);
    }

    fn test_case(operations: Vec<SemanticOperationV1>) -> GeneratedCaseV1 {
        let operations = operations
            .into_iter()
            .enumerate()
            .map(|(index, operation)| {
                OperationV1::new(
                    OperationIdV1::new(u32::try_from(index).expect("test index should fit")),
                    operation,
                )
            })
            .collect();
        GeneratedCaseV1::new(
            1,
            GeneratorConfigV1::new(1, 8, 8),
            SeedV1::new(0),
            vec![TransactionV1::new(TransactionIdV1::new(0), operations)],
        )
    }
}
