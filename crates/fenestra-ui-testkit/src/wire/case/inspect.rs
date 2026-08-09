use super::{
    OPERATIONS_LIMIT, OPERATIONS_PER_TRANSACTION_LIMIT, PATH_DEPTH_LIMIT, ParsedRecord,
    TRANSACTIONS_LIMIT, classify_record, parse_record,
};
use crate::wire::error::{ArtifactDecodeError, ArtifactDecodeErrorKind, ArtifactLimitKind};
use crate::wire::scan::ScannedLine;

#[derive(Clone, Copy)]
pub(in crate::wire) struct CaseInspectionV1 {
    byte_count: usize,
    transaction_count: usize,
    operation_count: usize,
    transaction_count_mismatch_line: Option<u32>,
    transactions_limit_line: Option<u32>,
    operations_per_transaction_limit_line: Option<u32>,
    operations_limit_line: Option<u32>,
    path_depth_limit_line: Option<u32>,
}

impl CaseInspectionV1 {
    pub(in crate::wire) const fn byte_count(self) -> usize {
        self.byte_count
    }

    pub(in crate::wire) const fn transaction_count(self) -> usize {
        self.transaction_count
    }

    pub(in crate::wire) const fn operation_count(self) -> usize {
        self.operation_count
    }

    pub(in crate::wire) const fn transaction_count_mismatch_line(self) -> Option<u32> {
        self.transaction_count_mismatch_line
    }

    pub(in crate::wire) const fn transactions_limit_line(self) -> Option<u32> {
        self.transactions_limit_line
    }

    pub(in crate::wire) const fn operations_per_transaction_limit_line(self) -> Option<u32> {
        self.operations_per_transaction_limit_line
    }

    pub(in crate::wire) const fn operations_limit_line(self) -> Option<u32> {
        self.operations_limit_line
    }

    pub(in crate::wire) const fn path_depth_limit_line(self) -> Option<u32> {
        self.path_depth_limit_line
    }
}

pub(in crate::wire) fn inspect_case_records_v1(
    lines: &[ScannedLine<'_>],
    byte_count: usize,
) -> Result<CaseInspectionV1, ArtifactDecodeError> {
    let mut inspection = CaseInspectionV1 {
        byte_count,
        transaction_count: 0,
        operation_count: 0,
        transaction_count_mismatch_line: None,
        transactions_limit_line: None,
        operations_per_transaction_limit_line: None,
        operations_limit_line: None,
        path_depth_limit_line: None,
    };
    let mut current_transaction: Option<(u32, u32, usize)> = None;
    let mut transaction_count = 0_usize;
    let mut operation_count = 0_usize;

    for line in lines {
        let raw = classify_record(line)?;
        match parse_record(&raw)? {
            ParsedRecord::Transaction {
                line,
                operation_count,
                ..
            } => {
                record_count_mismatch(&mut inspection, current_transaction.take());
                transaction_count = transaction_count
                    .checked_add(1)
                    .ok_or_else(|| limit(ArtifactLimitKind::Transactions, line))?;
                if transaction_count > TRANSACTIONS_LIMIT
                    && inspection.transactions_limit_line.is_none()
                {
                    inspection.transactions_limit_line = Some(line);
                }
                if usize::try_from(operation_count)
                    .map_or(true, |count| count > OPERATIONS_PER_TRANSACTION_LIMIT)
                    && inspection.operations_per_transaction_limit_line.is_none()
                {
                    inspection.operations_per_transaction_limit_line = Some(line);
                }
                current_transaction = Some((line, operation_count, 0));
            }
            ParsedRecord::Operation { line, depth, .. } => {
                operation_count = operation_count
                    .checked_add(1)
                    .ok_or_else(|| limit(ArtifactLimitKind::Operations, line))?;
                if operation_count > OPERATIONS_LIMIT && inspection.operations_limit_line.is_none()
                {
                    inspection.operations_limit_line = Some(line);
                }
                if depth > PATH_DEPTH_LIMIT && inspection.path_depth_limit_line.is_none() {
                    inspection.path_depth_limit_line = Some(line);
                }
                if let Some((transaction_line, _, count)) = current_transaction.as_mut() {
                    *count = count.checked_add(1).ok_or_else(|| {
                        limit(
                            ArtifactLimitKind::OperationsPerTransaction,
                            *transaction_line,
                        )
                    })?;
                    if *count > OPERATIONS_PER_TRANSACTION_LIMIT
                        && inspection.operations_per_transaction_limit_line.is_none()
                    {
                        inspection.operations_per_transaction_limit_line = Some(*transaction_line);
                    }
                }
            }
        }
    }
    record_count_mismatch(&mut inspection, current_transaction);
    inspection.transaction_count = transaction_count;
    inspection.operation_count = operation_count;
    Ok(inspection)
}

fn record_count_mismatch(
    inspection: &mut CaseInspectionV1,
    transaction: Option<(u32, u32, usize)>,
) {
    let Some((line, declared, actual)) = transaction else {
        return;
    };
    if inspection.transaction_count_mismatch_line.is_none()
        && usize::try_from(declared).ok() != Some(actual)
    {
        inspection.transaction_count_mismatch_line = Some(line);
    }
}

fn limit(kind: ArtifactLimitKind, line: u32) -> ArtifactDecodeError {
    ArtifactDecodeError::at(ArtifactDecodeErrorKind::LimitExceeded(kind), line)
}
