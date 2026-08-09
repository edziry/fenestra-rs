use std::fmt::Write as _;

use fenestra_ui_ir::prototype::{InvalidationClass, InvalidationSet};

use super::{TraceComparisonV1, TraceEventV1, TraceOutcomeV1, TraceTerminationV1};
use crate::case::GeneratedCaseV1;
use crate::error::{HarnessError, HarnessErrorKind, HarnessLimitKind};
use crate::failure::ReplayFailureV1;
use crate::fingerprint::{
    FailureFingerprintKindV1, FingerprintFieldV1, FingerprintLocationV1, FingerprintSummaryV1,
};
use crate::identity::IdentitySummaryV1;
use crate::replay::ReplayReportV1;

pub(super) fn derive_report_v1(
    case: &GeneratedCaseV1,
    identity: IdentitySummaryV1,
    events: &[TraceEventV1],
) -> Result<ReplayReportV1, HarnessError> {
    let transactions = case.transactions();
    if transactions.is_empty() || events.is_empty() || events.len() > transactions.len() {
        return Err(trace_error());
    }

    let transaction_count = u32::try_from(transactions.len()).map_err(|_| arithmetic_error())?;
    let operation_count = transactions
        .iter()
        .try_fold(0_usize, |total, transaction| {
            if transaction.operations().is_empty() {
                return Err(trace_error());
            }
            total
                .checked_add(transaction.operations().len())
                .ok_or_else(arithmetic_error)
        })?;

    let mut verified = 0_u32;
    let mut publications = 0_u32;
    let mut noops = 0_u32;
    let mut previous_after = 0_u64;
    for (index, (event, transaction)) in events.iter().zip(transactions).enumerate() {
        validate_reference(index, event, transaction)?;
        if event.before_generation() != previous_after {
            return Err(trace_error());
        }
        validate_shape(event)?;
        if event.comparison() == TraceComparisonV1::Mismatch && index + 1 != events.len() {
            return Err(trace_error());
        }

        match event.comparison() {
            TraceComparisonV1::Match => increment(&mut verified)?,
            TraceComparisonV1::Mismatch => {}
        }
        match event.outcome() {
            TraceOutcomeV1::Commit => increment(&mut publications)?,
            TraceOutcomeV1::Noop => increment(&mut noops)?,
            TraceOutcomeV1::Reject(_) => {}
        }
        previous_after = event.after_generation();
    }

    let terminal = events.last().ok_or_else(trace_error)?;
    if terminal.comparison() == TraceComparisonV1::Match && events.len() != transactions.len() {
        return Err(trace_error());
    }
    Ok(ReplayReportV1::new(
        transaction_count,
        operation_count,
        verified,
        publications,
        noops,
        terminal.after_generation(),
        identity,
    ))
}

pub(super) fn validate_failure_v1(
    events: &[TraceEventV1],
    failure: Option<&ReplayFailureV1>,
) -> Result<(), HarnessError> {
    let terminal = events.last().ok_or_else(trace_error)?;
    match termination_v1(events) {
        TraceTerminationV1::Success => {
            if failure.is_some() {
                return Err(trace_error());
            }
        }
        TraceTerminationV1::Rejected(rejection) => {
            let failure = failure.ok_or_else(trace_error)?;
            if failure.transaction() != terminal.transaction()
                || failure
                    .operation()
                    .is_some_and(|operation| !terminal.operations().contains(&operation))
            {
                return Err(trace_error());
            }
            let fingerprint = failure.fingerprint();
            if fingerprint.kind() != FailureFingerprintKindV1::CandidateRejected
                || fingerprint.location() != &FingerprintLocationV1::Global
                || fingerprint.field() != FingerprintFieldV1::CandidateOutcome
                || fingerprint.expected() != &FingerprintSummaryV1::CandidateAccepted
                || fingerprint.observed() != &FingerprintSummaryV1::CandidateRejected(rejection)
            {
                return Err(trace_error());
            }
        }
        TraceTerminationV1::Mismatch => {
            let failure = failure.ok_or_else(trace_error)?;
            if failure.transaction() != terminal.transaction() || failure.operation().is_some() {
                return Err(trace_error());
            }
            if !matches!(
                failure.fingerprint().kind(),
                FailureFingerprintKindV1::StateMismatch
                    | FailureFingerprintKindV1::IdentityMismatch
            ) {
                return Err(trace_error());
            }
        }
    }
    Ok(())
}

fn validate_reference(
    index: usize,
    event: &TraceEventV1,
    transaction: &crate::case::TransactionV1,
) -> Result<(), HarnessError> {
    let sequence = u32::try_from(index).map_err(|_| arithmetic_error())?;
    let expected_operations = transaction.operations();
    if event.sequence() != sequence
        || event.transaction() != transaction.id()
        || event.operations().len() != expected_operations.len()
        || event
            .operations()
            .iter()
            .zip(expected_operations)
            .any(|(actual, expected)| *actual != expected.id())
    {
        return Err(trace_error());
    }
    Ok(())
}

fn validate_shape(event: &TraceEventV1) -> Result<(), HarnessError> {
    match event.outcome() {
        TraceOutcomeV1::Commit => {
            let after = event
                .before_generation()
                .checked_add(1)
                .ok_or_else(trace_error)?;
            if event.after_generation() != after || event.mutation_count() == 0 {
                return Err(trace_error());
            }
        }
        TraceOutcomeV1::Noop => {
            if event.after_generation() != event.before_generation()
                || event.mutation_count() != 0
                || !event.invalidation().is_empty()
            {
                return Err(trace_error());
            }
        }
        TraceOutcomeV1::Reject(_) => {
            if event.after_generation() != event.before_generation()
                || event.mutation_count() != 0
                || !event.invalidation().is_empty()
                || event.comparison() != TraceComparisonV1::Mismatch
            {
                return Err(trace_error());
            }
        }
    }
    Ok(())
}

pub(super) fn termination_v1(events: &[TraceEventV1]) -> TraceTerminationV1 {
    let Some(last) = events.last() else {
        return TraceTerminationV1::Mismatch;
    };
    match (last.outcome(), last.comparison()) {
        (TraceOutcomeV1::Reject(rejection), _) => TraceTerminationV1::Rejected(rejection),
        (_, TraceComparisonV1::Mismatch) => TraceTerminationV1::Mismatch,
        (_, TraceComparisonV1::Match) => TraceTerminationV1::Success,
    }
}

pub(super) fn encode_events_v1(
    events: &[TraceEventV1],
    max_bytes: usize,
) -> Result<Vec<u8>, HarnessError> {
    let mut bytes = Vec::new();
    for event in events {
        let line = encode_event(event)?;
        let next = bytes
            .len()
            .checked_add(line.len())
            .ok_or_else(arithmetic_error)?;
        if next > max_bytes {
            return Err(HarnessError::limit(HarnessLimitKind::TraceBytes));
        }
        bytes.extend_from_slice(line.as_bytes());
    }
    Ok(bytes)
}

fn encode_event(event: &TraceEventV1) -> Result<String, HarnessError> {
    let mut line = String::new();
    write!(
        line,
        "event|{}|{}|",
        event.sequence(),
        event.transaction().get()
    )
    .map_err(|_| arithmetic_error())?;
    for (index, operation) in event.operations().iter().enumerate() {
        if index != 0 {
            line.push(',');
        }
        write!(line, "{}", operation.get()).map_err(|_| arithmetic_error())?;
    }
    write!(
        line,
        "|{}|{}|",
        event.before_generation(),
        event.after_generation()
    )
    .map_err(|_| arithmetic_error())?;
    write_outcome(&mut line, event.outcome())?;
    write!(line, "|{}|", event.mutation_count()).map_err(|_| arithmetic_error())?;
    write_invalidation(&mut line, event.invalidation());
    writeln!(line, "|{}", comparison_word(event.comparison())).map_err(|_| arithmetic_error())?;
    Ok(line)
}

fn write_outcome(output: &mut String, outcome: TraceOutcomeV1) -> Result<(), HarnessError> {
    match outcome {
        TraceOutcomeV1::Commit => output.push_str("commit"),
        TraceOutcomeV1::Noop => output.push_str("noop"),
        TraceOutcomeV1::Reject(rejection) => {
            write!(output, "reject:{}", rejection.code()).map_err(|_| arithmetic_error())?;
        }
    }
    Ok(())
}

fn write_invalidation(output: &mut String, invalidation: InvalidationSet) {
    if invalidation.is_empty() {
        output.push('-');
        return;
    }
    for (index, class) in invalidation.iter().enumerate() {
        if index != 0 {
            output.push(',');
        }
        output.push_str(invalidation_word(class));
    }
}

const fn invalidation_word(class: InvalidationClass) -> &'static str {
    match class {
        InvalidationClass::Structure => "structure",
        InvalidationClass::StyleMatch => "style-match",
        InvalidationClass::Intrinsic => "intrinsic",
        InvalidationClass::Layout => "layout",
        InvalidationClass::Semantics => "semantics",
        InvalidationClass::HitTest => "hit-test",
        InvalidationClass::Paint => "paint",
        InvalidationClass::Composition => "composition",
        InvalidationClass::Surface => "surface",
    }
}

const fn comparison_word(comparison: TraceComparisonV1) -> &'static str {
    match comparison {
        TraceComparisonV1::Match => "match",
        TraceComparisonV1::Mismatch => "mismatch",
    }
}

fn increment(value: &mut u32) -> Result<(), HarnessError> {
    *value = value.checked_add(1).ok_or_else(arithmetic_error)?;
    Ok(())
}

fn trace_error() -> HarnessError {
    HarnessError::new(HarnessErrorKind::TraceMismatch)
}

fn arithmetic_error() -> HarnessError {
    HarnessError::new(HarnessErrorKind::ArithmeticExhausted)
}
