use super::mutations::Controls;
use super::types::{FailureKind, FailureLocation, Span};

pub(super) fn failure_controls(controls: &mut Controls<'_>) {
    controls.check("failure kind", |log| {
        log.failure.kind = FailureKind::ScalarOutOfDomain;
    });
    controls.check("failure location variant", |log| {
        log.failure.location = FailureLocation::Input;
    });
    controls.check("failure node index", |log| {
        let FailureLocation::Node { index } = &mut log.failure.location else {
            panic!("expected node failure location");
        };
        *index += 1;
    });
    controls.check("failure span source", |log| {
        *span_parts(&mut log.failure.ir_span).0 += 1;
    });
    controls.check("failure span start", |log| {
        *span_parts(&mut log.failure.ir_span).1 += 1;
    });
    controls.check("failure span end", |log| {
        *span_parts(&mut log.failure.ir_span).2 += 1;
    });
    controls.check("failure span variant", |log| {
        log.failure.ir_span = Span::Synthetic;
    });
    controls.check("failure operation index", |log| {
        log.failure.operation_index = Some(0);
    });
    controls.check("failure outer preservation", |log| {
        log.failure.outer_state_preserved = false;
    });
    controls.check("failure spatial preservation", |log| {
        log.failure.spatial_snapshot_preserved = false;
    });
    controls.check("failure observation preservation", |log| {
        log.failure.complete_observation_preserved = false;
    });
    controls.check("factor span source", |log| {
        *span_parts(&mut log.failure.authored_factor_span).0 += 1;
    });
    controls.check("factor span start", |log| {
        *span_parts(&mut log.failure.authored_factor_span).1 += 1;
    });
    controls.check("factor span end", |log| {
        *span_parts(&mut log.failure.authored_factor_span).2 += 1;
    });
    controls.check("factor span variant", |log| {
        log.failure.authored_factor_span = Span::Synthetic;
    });
}

fn span_parts(value: &mut Span) -> (&mut u32, &mut u32, &mut u32) {
    let Span::Bytes { source, start, end } = value else {
        panic!("expected byte span");
    };
    (source, start, end)
}
