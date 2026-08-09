use fenestra_ui_ir::prototype::PropertyValue;

use super::ReducerError;
use crate::case::{GeneratedCaseV1, OperationV1, SemanticOperationV1, TransactionV1};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CandidateDispositionV1 {
    Continue,
    Accept,
    Exhausted { accepted: bool },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum SearchOutcomeV1 {
    FixedPoint,
    Accepted(GeneratedCaseV1),
    Exhausted(Option<GeneratedCaseV1>),
}

pub(super) fn search_candidates_v1(
    current: &GeneratedCaseV1,
    visit: &mut impl FnMut(GeneratedCaseV1) -> Result<CandidateDispositionV1, ReducerError>,
) -> Result<SearchOutcomeV1, ReducerError> {
    if let Some(outcome) = remove_transaction_blocks_v1(current, visit)? {
        return Ok(outcome);
    }
    if let Some(outcome) = remove_operations_v1(current, visit)? {
        return Ok(outcome);
    }
    if let Some(outcome) = reduce_final_indices_v1(current, visit)? {
        return Ok(outcome);
    }
    if let Some(outcome) = reduce_explicit_keys_v1(current, visit)? {
        return Ok(outcome);
    }
    if let Some(outcome) = reduce_scalar_values_v1(current, visit)? {
        return Ok(outcome);
    }
    Ok(SearchOutcomeV1::FixedPoint)
}

fn remove_transaction_blocks_v1(
    current: &GeneratedCaseV1,
    visit: &mut impl FnMut(GeneratedCaseV1) -> Result<CandidateDispositionV1, ReducerError>,
) -> Result<Option<SearchOutcomeV1>, ReducerError> {
    let transaction_count = current.transactions().len();
    for block_len in (1..=transaction_count).rev() {
        for start in 0..=transaction_count - block_len {
            let mut transactions = Vec::with_capacity(transaction_count - block_len);
            transactions.extend_from_slice(&current.transactions()[..start]);
            transactions.extend_from_slice(&current.transactions()[start + block_len..]);
            let candidate = case_with_transactions_v1(current, transactions);
            if let Some(outcome) = visit_candidate_v1(candidate, visit)? {
                return Ok(Some(outcome));
            }
        }
    }
    Ok(None)
}

fn remove_operations_v1(
    current: &GeneratedCaseV1,
    visit: &mut impl FnMut(GeneratedCaseV1) -> Result<CandidateDispositionV1, ReducerError>,
) -> Result<Option<SearchOutcomeV1>, ReducerError> {
    for (transaction_position, transaction) in current.transactions().iter().enumerate() {
        for operation_position in 0..transaction.operations().len() {
            let mut transactions = current.transactions().to_vec();
            let mut operations = transaction.operations().to_vec();
            operations.remove(operation_position);
            if operations.is_empty() {
                transactions.remove(transaction_position);
            } else {
                transactions[transaction_position] =
                    TransactionV1::new(transaction.id(), operations);
            }
            let candidate = case_with_transactions_v1(current, transactions);
            if let Some(outcome) = visit_candidate_v1(candidate, visit)? {
                return Ok(Some(outcome));
            }
        }
    }
    Ok(None)
}

fn reduce_final_indices_v1(
    current: &GeneratedCaseV1,
    visit: &mut impl FnMut(GeneratedCaseV1) -> Result<CandidateDispositionV1, ReducerError>,
) -> Result<Option<SearchOutcomeV1>, ReducerError> {
    for (transaction_position, transaction) in current.transactions().iter().enumerate() {
        for (operation_position, operation) in transaction.operations().iter().enumerate() {
            let (fragment, key, current_index, is_insert) = match operation.operation() {
                SemanticOperationV1::InsertKeyed {
                    fragment,
                    key,
                    final_index,
                } => (fragment, *key, *final_index, true),
                SemanticOperationV1::MoveKeyed {
                    fragment,
                    key,
                    final_index,
                } => (fragment, *key, *final_index, false),
                _ => continue,
            };
            for final_index in 0..current_index {
                let replacement = if is_insert {
                    SemanticOperationV1::InsertKeyed {
                        fragment: fragment.clone(),
                        key,
                        final_index,
                    }
                } else {
                    SemanticOperationV1::MoveKeyed {
                        fragment: fragment.clone(),
                        key,
                        final_index,
                    }
                };
                let candidate = replace_operation_v1(
                    current,
                    transaction_position,
                    operation_position,
                    replacement,
                );
                if let Some(outcome) = visit_candidate_v1(candidate, visit)? {
                    return Ok(Some(outcome));
                }
            }
        }
    }
    Ok(None)
}

fn reduce_explicit_keys_v1(
    current: &GeneratedCaseV1,
    visit: &mut impl FnMut(GeneratedCaseV1) -> Result<CandidateDispositionV1, ReducerError>,
) -> Result<Option<SearchOutcomeV1>, ReducerError> {
    for (transaction_position, transaction) in current.transactions().iter().enumerate() {
        for (operation_position, operation) in transaction.operations().iter().enumerate() {
            let current_key = match operation.operation() {
                SemanticOperationV1::InsertKeyed { key, .. }
                | SemanticOperationV1::MoveKeyed { key, .. }
                | SemanticOperationV1::UpdateKeyed { key, .. }
                | SemanticOperationV1::RemoveKeyed { key, .. } => *key,
                SemanticOperationV1::SetProperty { .. } => continue,
            };
            for key in 0..current_key {
                let replacement = operation_with_key_v1(operation.operation(), key);
                let candidate = replace_operation_v1(
                    current,
                    transaction_position,
                    operation_position,
                    replacement,
                );
                if let Some(outcome) = visit_candidate_v1(candidate, visit)? {
                    return Ok(Some(outcome));
                }
            }
        }
    }
    Ok(None)
}

fn reduce_scalar_values_v1(
    current: &GeneratedCaseV1,
    visit: &mut impl FnMut(GeneratedCaseV1) -> Result<CandidateDispositionV1, ReducerError>,
) -> Result<Option<SearchOutcomeV1>, ReducerError> {
    for (transaction_position, transaction) in current.transactions().iter().enumerate() {
        for (operation_position, operation) in transaction.operations().iter().enumerate() {
            let current_value = match operation.operation() {
                SemanticOperationV1::SetProperty {
                    value: PropertyValue::ScalarI32(value),
                    ..
                }
                | SemanticOperationV1::UpdateKeyed {
                    value: PropertyValue::ScalarI32(value),
                    ..
                } => *value,
                _ => continue,
            };
            for value in [0, 1, -1] {
                if value == current_value {
                    continue;
                }
                let replacement = operation_with_scalar_v1(operation.operation(), value);
                let candidate = replace_operation_v1(
                    current,
                    transaction_position,
                    operation_position,
                    replacement,
                );
                if let Some(outcome) = visit_candidate_v1(candidate, visit)? {
                    return Ok(Some(outcome));
                }
            }
        }
    }
    Ok(None)
}

fn operation_with_key_v1(operation: &SemanticOperationV1, key: u64) -> SemanticOperationV1 {
    match operation {
        SemanticOperationV1::InsertKeyed {
            fragment,
            final_index,
            ..
        } => SemanticOperationV1::InsertKeyed {
            fragment: fragment.clone(),
            key,
            final_index: *final_index,
        },
        SemanticOperationV1::MoveKeyed {
            fragment,
            final_index,
            ..
        } => SemanticOperationV1::MoveKeyed {
            fragment: fragment.clone(),
            key,
            final_index: *final_index,
        },
        SemanticOperationV1::UpdateKeyed {
            fragment,
            property,
            value,
            ..
        } => SemanticOperationV1::UpdateKeyed {
            fragment: fragment.clone(),
            key,
            property: *property,
            value: value.clone(),
        },
        SemanticOperationV1::RemoveKeyed { fragment, .. } => SemanticOperationV1::RemoveKeyed {
            fragment: fragment.clone(),
            key,
        },
        SemanticOperationV1::SetProperty { .. } => unreachable!(),
    }
}

fn operation_with_scalar_v1(operation: &SemanticOperationV1, value: i32) -> SemanticOperationV1 {
    match operation {
        SemanticOperationV1::SetProperty { node, property, .. } => {
            SemanticOperationV1::SetProperty {
                node: node.clone(),
                property: *property,
                value: PropertyValue::ScalarI32(value),
            }
        }
        SemanticOperationV1::UpdateKeyed {
            fragment,
            key,
            property,
            ..
        } => SemanticOperationV1::UpdateKeyed {
            fragment: fragment.clone(),
            key: *key,
            property: *property,
            value: PropertyValue::ScalarI32(value),
        },
        _ => unreachable!(),
    }
}

fn replace_operation_v1(
    current: &GeneratedCaseV1,
    transaction_position: usize,
    operation_position: usize,
    replacement: SemanticOperationV1,
) -> GeneratedCaseV1 {
    let transaction = &current.transactions()[transaction_position];
    let mut operations = transaction.operations().to_vec();
    operations[operation_position] =
        OperationV1::new(operations[operation_position].id(), replacement);
    let mut transactions = current.transactions().to_vec();
    transactions[transaction_position] = TransactionV1::new(transaction.id(), operations);
    case_with_transactions_v1(current, transactions)
}

fn case_with_transactions_v1(
    current: &GeneratedCaseV1,
    transactions: Vec<TransactionV1>,
) -> GeneratedCaseV1 {
    GeneratedCaseV1::new(
        current.fixture_revision(),
        current.config(),
        current.seed(),
        transactions,
    )
}

fn visit_candidate_v1(
    candidate: GeneratedCaseV1,
    visit: &mut impl FnMut(GeneratedCaseV1) -> Result<CandidateDispositionV1, ReducerError>,
) -> Result<Option<SearchOutcomeV1>, ReducerError> {
    let disposition = visit(candidate.clone())?;
    Ok(match disposition {
        CandidateDispositionV1::Continue => None,
        CandidateDispositionV1::Accept => Some(SearchOutcomeV1::Accepted(candidate)),
        CandidateDispositionV1::Exhausted { accepted } => {
            Some(SearchOutcomeV1::Exhausted(accepted.then_some(candidate)))
        }
    })
}
