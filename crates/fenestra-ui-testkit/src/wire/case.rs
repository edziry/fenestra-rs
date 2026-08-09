use crate::case::{
    GeneratedCaseV1, GeneratorConfigV1, OperationIdV1, OperationV1, SeedV1, SemanticOperationV1,
    TransactionIdV1, TransactionV1,
};

use super::error::{ArtifactDecodeError, ArtifactDecodeErrorKind, ArtifactLimitKind, CountKind};
use super::primitive::{parse_u32, property_value_shape};
use super::scan::{ARTIFACT_BYTES_LIMIT, ScannedLine, scan_lines};

mod encode;
mod inspect;
mod operation;
#[cfg(test)]
mod tests;

pub use encode::encode_case_v1;
pub(super) use inspect::{CaseInspectionV1, inspect_case_records_v1};
use operation::parse_operation;

pub(super) const CASE_BYTES_LIMIT: usize = 131_072;
pub(super) const TRANSACTIONS_LIMIT: usize = 64;
pub(super) const OPERATIONS_PER_TRANSACTION_LIMIT: usize = 4;
pub(super) const OPERATIONS_LIMIT: usize = 256;
pub(super) const PATH_DEPTH_LIMIT: usize = 8;

/// Provenance supplied outside standalone canonical case records.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CaseDecodeContextV1 {
    fixture_revision: u32,
    config: GeneratorConfigV1,
    seed: SeedV1,
}

impl CaseDecodeContextV1 {
    /// Creates the metadata attached to one decoded sequence of case records.
    #[must_use]
    pub const fn new(fixture_revision: u32, config: GeneratorConfigV1, seed: SeedV1) -> Self {
        Self {
            fixture_revision,
            config,
            seed,
        }
    }
}

/// Decodes standalone canonical `tx` and `op` records with supplied provenance.
pub fn decode_case_v1(
    bytes: &[u8],
    context: CaseDecodeContextV1,
) -> Result<GeneratedCaseV1, ArtifactDecodeError> {
    if bytes.len() > ARTIFACT_BYTES_LIMIT {
        return Err(ArtifactDecodeError::new(
            ArtifactDecodeErrorKind::LimitExceeded(ArtifactLimitKind::ArtifactBytes),
            None,
        ));
    }
    let lines = scan_lines(bytes)?;
    decode_case_records_v1(&lines, bytes.len(), context)
}

pub(super) fn decode_case_records_v1(
    lines: &[ScannedLine<'_>],
    byte_count: usize,
    context: CaseDecodeContextV1,
) -> Result<GeneratedCaseV1, ArtifactDecodeError> {
    let raw = classify_records(lines)?;
    let parsed = parse_records(&raw)?;
    validate_decoded_limits(byte_count, &parsed)?;
    validate_counts(&parsed)?;
    let transactions = build_case(parsed)?;
    Ok(GeneratedCaseV1::new(
        context.fixture_revision,
        context.config,
        context.seed,
        transactions,
    ))
}

enum RawRecord<'a> {
    Transaction {
        line: u32,
        id: &'a str,
        operation_count: &'a str,
    },
    Operation {
        line: u32,
        id: &'a str,
        operation: RawOperation<'a>,
    },
}

enum RawOperation<'a> {
    Set(&'a str, &'a str, &'a str),
    Insert(&'a str, &'a str, &'a str),
    Move(&'a str, &'a str, &'a str),
    Update(&'a str, &'a str, &'a str, &'a str),
    Remove(&'a str, &'a str),
}

enum ParsedRecord {
    Transaction {
        line: u32,
        id: u32,
        operation_count: u32,
    },
    Operation {
        line: u32,
        id: u32,
        depth: usize,
        operation: SemanticOperationV1,
    },
}

fn classify_records<'source>(
    lines: &[ScannedLine<'source>],
) -> Result<Vec<RawRecord<'source>>, ArtifactDecodeError> {
    let mut records = Vec::with_capacity(lines.len());
    for line in lines {
        records.push(classify_record(line)?);
    }
    Ok(records)
}

fn classify_record<'source>(
    line: &ScannedLine<'source>,
) -> Result<RawRecord<'source>, ArtifactDecodeError> {
    let fields: Vec<_> = line.text.split('|').collect();
    let record = match fields.as_slice() {
        ["tx", id, operation_count] => RawRecord::Transaction {
            line: line.number,
            id,
            operation_count,
        },
        ["op", id, "set", node, property, value] => {
            property_value_shape(value, line.number)?;
            RawRecord::Operation {
                line: line.number,
                id,
                operation: RawOperation::Set(node, property, value),
            }
        }
        ["op", id, "insert", fragment, key, index] => RawRecord::Operation {
            line: line.number,
            id,
            operation: RawOperation::Insert(fragment, key, index),
        },
        ["op", id, "move", fragment, key, index] => RawRecord::Operation {
            line: line.number,
            id,
            operation: RawOperation::Move(fragment, key, index),
        },
        ["op", id, "update", fragment, key, property, value] => {
            property_value_shape(value, line.number)?;
            RawRecord::Operation {
                line: line.number,
                id,
                operation: RawOperation::Update(fragment, key, property, value),
            }
        }
        ["op", id, "remove", fragment, key] => RawRecord::Operation {
            line: line.number,
            id,
            operation: RawOperation::Remove(fragment, key),
        },
        _ => return Err(at(ArtifactDecodeErrorKind::MalformedRecord, line.number)),
    };
    Ok(record)
}

fn parse_records(raw: &[RawRecord<'_>]) -> Result<Vec<ParsedRecord>, ArtifactDecodeError> {
    let mut records = Vec::with_capacity(raw.len());
    for record in raw {
        records.push(parse_record(record)?);
    }
    Ok(records)
}

fn parse_record(record: &RawRecord<'_>) -> Result<ParsedRecord, ArtifactDecodeError> {
    Ok(match record {
        RawRecord::Transaction {
            line,
            id,
            operation_count,
        } => ParsedRecord::Transaction {
            line: *line,
            id: parse_u32(id, *line)?,
            operation_count: parse_u32(operation_count, *line)?,
        },
        RawRecord::Operation {
            line,
            id,
            operation,
        } => {
            let id = parse_u32(id, *line)?;
            let (operation, depth) = parse_operation(operation, *line)?;
            ParsedRecord::Operation {
                line: *line,
                id,
                depth,
                operation,
            }
        }
    })
}

fn validate_decoded_limits(
    byte_count: usize,
    records: &[ParsedRecord],
) -> Result<(), ArtifactDecodeError> {
    if byte_count > CASE_BYTES_LIMIT {
        return Err(ArtifactDecodeError::new(
            ArtifactDecodeErrorKind::LimitExceeded(ArtifactLimitKind::CaseBytes),
            None,
        ));
    }
    let transaction_lines: Vec<_> = records
        .iter()
        .filter_map(|record| match record {
            ParsedRecord::Transaction { line, .. } => Some(*line),
            ParsedRecord::Operation { .. } => None,
        })
        .collect();
    if let Some(line) = transaction_lines.get(TRANSACTIONS_LIMIT) {
        return Err(limit(ArtifactLimitKind::Transactions, *line));
    }
    if let Some(line) = operations_per_transaction_limit(records)? {
        return Err(limit(ArtifactLimitKind::OperationsPerTransaction, line));
    }
    let operation_lines: Vec<_> = records
        .iter()
        .filter_map(|record| match record {
            ParsedRecord::Operation { line, .. } => Some(*line),
            ParsedRecord::Transaction { .. } => None,
        })
        .collect();
    if let Some(line) = operation_lines.get(OPERATIONS_LIMIT) {
        return Err(limit(ArtifactLimitKind::Operations, *line));
    }
    if let Some(line) = records.iter().find_map(|record| match record {
        ParsedRecord::Operation { line, depth, .. } if *depth > PATH_DEPTH_LIMIT => Some(*line),
        _ => None,
    }) {
        return Err(limit(ArtifactLimitKind::PathDepth, line));
    }
    Ok(())
}

fn operations_per_transaction_limit(
    records: &[ParsedRecord],
) -> Result<Option<u32>, ArtifactDecodeError> {
    let mut current: Option<(u32, usize)> = None;
    for record in records {
        match record {
            ParsedRecord::Transaction {
                line,
                operation_count,
                ..
            } => {
                if let Some((line, actual)) = current
                    && actual > OPERATIONS_PER_TRANSACTION_LIMIT
                {
                    return Ok(Some(line));
                }
                if usize::try_from(*operation_count)
                    .map_or(true, |count| count > OPERATIONS_PER_TRANSACTION_LIMIT)
                {
                    return Ok(Some(*line));
                }
                current = Some((*line, 0));
            }
            ParsedRecord::Operation { .. } => {
                if let Some((line, actual)) = current.as_mut() {
                    *actual = actual
                        .checked_add(1)
                        .ok_or_else(|| limit(ArtifactLimitKind::OperationsPerTransaction, *line))?;
                }
            }
        }
    }
    Ok(current
        .and_then(|(line, actual)| (actual > OPERATIONS_PER_TRANSACTION_LIMIT).then_some(line)))
}

fn validate_counts(records: &[ParsedRecord]) -> Result<(), ArtifactDecodeError> {
    let mut current: Option<(u32, u32, usize)> = None;
    for record in records {
        match record {
            ParsedRecord::Transaction {
                line,
                operation_count,
                ..
            } => {
                validate_transaction_count(current)?;
                current = Some((*line, *operation_count, 0));
            }
            ParsedRecord::Operation { .. } => {
                if let Some((line, _, actual)) = current.as_mut() {
                    *actual = actual
                        .checked_add(1)
                        .ok_or_else(|| limit(ArtifactLimitKind::OperationsPerTransaction, *line))?;
                }
            }
        }
    }
    validate_transaction_count(current)
}

fn validate_transaction_count(
    current: Option<(u32, u32, usize)>,
) -> Result<(), ArtifactDecodeError> {
    let Some((line, declared, actual)) = current else {
        return Ok(());
    };
    if declared == 0 && actual == 0 {
        return Err(at(ArtifactDecodeErrorKind::MalformedRecord, line));
    }
    if usize::try_from(declared).ok() != Some(actual) {
        return Err(at(
            ArtifactDecodeErrorKind::CountMismatch(CountKind::OperationsPerTransaction),
            line,
        ));
    }
    Ok(())
}

fn build_case(records: Vec<ParsedRecord>) -> Result<Vec<TransactionV1>, ArtifactDecodeError> {
    let mut transactions = Vec::new();
    let mut current: Option<(u32, TransactionIdV1, Vec<OperationV1>)> = None;
    let mut previous_transaction = None;
    let mut previous_operation = None;
    for record in records {
        match record {
            ParsedRecord::Transaction { line, id, .. } => {
                if previous_transaction.is_some_and(|previous| id <= previous) {
                    return Err(at(ArtifactDecodeErrorKind::OrderingViolation, line));
                }
                if let Some((_, id, operations)) = current.take() {
                    transactions.push(TransactionV1::new(id, operations));
                }
                previous_transaction = Some(id);
                current = Some((line, TransactionIdV1::new(id), Vec::new()));
            }
            ParsedRecord::Operation {
                line,
                id,
                operation,
                ..
            } => {
                let Some((_, _, operations)) = current.as_mut() else {
                    return Err(at(ArtifactDecodeErrorKind::OrderingViolation, line));
                };
                if previous_operation.is_some_and(|previous| id <= previous) {
                    return Err(at(ArtifactDecodeErrorKind::OrderingViolation, line));
                }
                previous_operation = Some(id);
                operations.push(OperationV1::new(OperationIdV1::new(id), operation));
            }
        }
    }
    if let Some((_, id, operations)) = current {
        transactions.push(TransactionV1::new(id, operations));
    }
    Ok(transactions)
}

fn limit(kind: ArtifactLimitKind, line: u32) -> ArtifactDecodeError {
    ArtifactDecodeError::at(ArtifactDecodeErrorKind::LimitExceeded(kind), line)
}

fn at(kind: ArtifactDecodeErrorKind, line: u32) -> ArtifactDecodeError {
    ArtifactDecodeError::at(kind, line)
}
