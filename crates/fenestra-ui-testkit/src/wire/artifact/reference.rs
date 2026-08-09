use crate::case::{
    GeneratedCaseV1, OperationIdV1, SemanticOperationV1, TransactionIdV1, TransactionV1,
};
use crate::trace::{TraceEventV1, TraceFaultV1};

use super::super::error::{ArtifactDecodeError, ArtifactDecodeErrorKind};
use super::boundary::SectionBoundaryV1;
use super::fingerprint::ParsedFailurePartsV1;

pub(super) struct ReferenceInputV1<'value, 'scan, 'source> {
    pub(super) original_case: &'value GeneratedCaseV1,
    pub(super) original_boundary: SectionBoundaryV1<'scan, 'source>,
    pub(super) minimized_case: &'value GeneratedCaseV1,
    pub(super) minimized_boundary: SectionBoundaryV1<'scan, 'source>,
    pub(super) fault: TraceFaultV1,
    pub(super) original_failure: &'value ParsedFailurePartsV1,
    pub(super) minimized_failure: &'value ParsedFailurePartsV1,
    pub(super) trace_events: &'value [TraceEventV1],
    pub(super) trace_boundary: SectionBoundaryV1<'scan, 'source>,
}

pub(super) fn validate_references_v1(
    input: ReferenceInputV1<'_, '_, '_>,
) -> Result<(), ArtifactDecodeError> {
    validate_trace_sequences(input.trace_events, input.trace_boundary)?;
    validate_minimized_subsequence(
        input.original_case,
        input.minimized_case,
        input.minimized_boundary,
    )?;

    let fault_line = input.original_boundary.end().number + 1;
    let original_failure_line = fault_line + 1;
    let minimized_failure_line = input.minimized_boundary.end().number + 1;
    validate_failures(
        input.original_case,
        input.original_failure,
        original_failure_line,
        input.minimized_case,
        input.minimized_failure,
        minimized_failure_line,
    )?;

    validate_fault_target(
        input.original_case,
        input.minimized_case,
        input.fault,
        fault_line,
    )?;

    validate_trace_references(
        input.minimized_case,
        input.minimized_failure.transaction(),
        input.trace_events,
        input.trace_boundary,
    )
}

fn validate_minimized_subsequence(
    original: &GeneratedCaseV1,
    minimized: &GeneratedCaseV1,
    minimized_boundary: SectionBoundaryV1<'_, '_>,
) -> Result<(), ArtifactDecodeError> {
    let mut original_transactions = original.transactions().iter();
    let mut minimized_lines = minimized_boundary.records().iter();

    for minimized_transaction in minimized.transactions() {
        let transaction_line = next_case_line(&mut minimized_lines, minimized_boundary);
        let original_transaction = find_transaction(
            &mut original_transactions,
            minimized_transaction.id(),
            transaction_line,
        )?;
        validate_operation_subsequence(
            original_transaction,
            minimized_transaction,
            &mut minimized_lines,
            minimized_boundary,
        )?;
    }
    Ok(())
}

fn find_transaction<'case>(
    original: &mut impl Iterator<Item = &'case TransactionV1>,
    target: TransactionIdV1,
    line: u32,
) -> Result<&'case TransactionV1, ArtifactDecodeError> {
    for transaction in original {
        match transaction.id().get().cmp(&target.get()) {
            std::cmp::Ordering::Less => {}
            std::cmp::Ordering::Equal => return Ok(transaction),
            std::cmp::Ordering::Greater => return Err(invalid_reference(line)),
        }
    }
    Err(invalid_reference(line))
}

fn validate_operation_subsequence<'scan, 'source>(
    original: &TransactionV1,
    minimized: &TransactionV1,
    minimized_lines: &mut std::slice::Iter<'scan, super::super::scan::ScannedLine<'source>>,
    minimized_boundary: SectionBoundaryV1<'_, '_>,
) -> Result<(), ArtifactDecodeError> {
    let mut original_operations = original.operations().iter();
    for minimized_operation in minimized.operations() {
        let line = next_case_line(minimized_lines, minimized_boundary);
        let target = minimized_operation.id();
        let found = original_operations
            .by_ref()
            .find(|operation| operation.id().get() >= target.get());
        if found.is_none_or(|operation| operation.id() != target) {
            return Err(invalid_reference(line));
        }
    }
    Ok(())
}

fn next_case_line<'scan, 'source>(
    lines: &mut std::slice::Iter<'scan, super::super::scan::ScannedLine<'source>>,
    boundary: SectionBoundaryV1<'_, '_>,
) -> u32 {
    lines
        .next()
        .map_or(boundary.begin().number, |line| line.number)
}

fn validate_fault_target(
    original: &GeneratedCaseV1,
    minimized: &GeneratedCaseV1,
    fault: TraceFaultV1,
    line: u32,
) -> Result<(), ArtifactDecodeError> {
    let target = fault.target();
    if !is_move_target(original, target) || !is_move_target(minimized, target) {
        return Err(invalid_reference(line));
    }
    Ok(())
}

fn is_move_target(case: &GeneratedCaseV1, target: OperationIdV1) -> bool {
    case.transactions()
        .iter()
        .flat_map(TransactionV1::operations)
        .any(|operation| {
            operation.id() == target
                && matches!(operation.operation(), SemanticOperationV1::MoveKeyed { .. })
        })
}

fn validate_failures(
    original_case: &GeneratedCaseV1,
    original: &ParsedFailurePartsV1,
    original_line: u32,
    minimized_case: &GeneratedCaseV1,
    minimized: &ParsedFailurePartsV1,
    minimized_line: u32,
) -> Result<(), ArtifactDecodeError> {
    if !failure_resolves(original_case, original) {
        return Err(invalid_reference(original_line));
    }
    if !failure_resolves(minimized_case, minimized)
        || original.transaction() != minimized.transaction()
        || original.operation() != minimized.operation()
    {
        return Err(invalid_reference(minimized_line));
    }
    Ok(())
}

fn failure_resolves(case: &GeneratedCaseV1, failure: &ParsedFailurePartsV1) -> bool {
    let Some(transaction) = case
        .transactions()
        .iter()
        .find(|transaction| transaction.id() == failure.transaction())
    else {
        return false;
    };
    failure.operation().is_none_or(|target| {
        transaction
            .operations()
            .iter()
            .any(|operation| operation.id() == target)
    })
}

fn validate_trace_sequences(
    events: &[TraceEventV1],
    boundary: SectionBoundaryV1<'_, '_>,
) -> Result<(), ArtifactDecodeError> {
    for (index, event) in events.iter().enumerate() {
        if usize::try_from(event.sequence()).ok() != Some(index) {
            return Err(ArtifactDecodeError::at(
                ArtifactDecodeErrorKind::OrderingViolation,
                event_line(boundary, index),
            ));
        }
    }
    Ok(())
}

fn validate_trace_references(
    minimized: &GeneratedCaseV1,
    failure_transaction: TransactionIdV1,
    events: &[TraceEventV1],
    boundary: SectionBoundaryV1<'_, '_>,
) -> Result<(), ArtifactDecodeError> {
    for (index, event) in events.iter().enumerate() {
        let Some(transaction) = minimized.transactions().get(index) else {
            return Err(invalid_reference(event_line(boundary, index)));
        };
        if event.transaction() != transaction.id()
            || event.operations().len() != transaction.operations().len()
            || event
                .operations()
                .iter()
                .zip(transaction.operations())
                .any(|(actual, expected)| *actual != expected.id())
        {
            return Err(invalid_reference(event_line(boundary, index)));
        }
    }

    let Some(failure_index) = minimized
        .transactions()
        .iter()
        .position(|transaction| transaction.id() == failure_transaction)
    else {
        return Err(invalid_reference(boundary.begin().number));
    };
    let required_events = failure_index + 1;
    if events.len() < required_events {
        return Err(invalid_reference(boundary.begin().number));
    }
    if events.len() > required_events {
        return Err(invalid_reference(event_line(boundary, required_events)));
    }
    Ok(())
}

fn event_line(boundary: SectionBoundaryV1<'_, '_>, index: usize) -> u32 {
    boundary
        .records()
        .get(index)
        .map_or(boundary.begin().number, |line| line.number)
}

fn invalid_reference(line: u32) -> ArtifactDecodeError {
    ArtifactDecodeError::at(ArtifactDecodeErrorKind::InvalidReference, line)
}
