mod step;

use std::fmt::Write;

use fenestra_ui_runtime::prototype::SchedulerState;

use super::super::record::{QueueRecordV1, SchedulerEventRecordV1, TraceEventRecordV1};
use crate::headless::platform::HeadlessPointerTargetV1;
use crate::headless::trace::{
    HeadlessFailureCauseV1, HeadlessInputKindV1, HeadlessOutcomeV1, HeadlessTraceStageV1,
};

pub(super) fn headless_line(event: &TraceEventRecordV1) -> String {
    let mut line = format!(
        "h-event|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
        event.schema,
        event.sequence,
        event.domain,
        event.tick,
        stage(event.stage),
        input(event.input),
        outcome(event.outcome),
        option(event.captured),
        option(event.published),
        target(event.target),
        option(event.frame),
        option(event.control),
        event.surface.width(),
        event.surface.height(),
    );
    let counts = event.counts;
    let _ = write!(
        line,
        "|{}|{}|{}|{}|{}",
        counts.computed_styles(),
        counts.geometries(),
        counts.semantics(),
        counts.hit_regions(),
        counts.scene_rectangles(),
    );
    for queue in [
        event.deferred,
        event.controls,
        event.visual,
        event.in_flight,
        event.renderer,
    ] {
        write_queue(&mut line, queue);
    }
    line
}

pub(super) fn scheduler_line(event: &SchedulerEventRecordV1) -> String {
    let mut line = format!(
        "s-event|{}|{}|{}|{}",
        event.schema, event.sequence, event.domain, event.tick
    );
    step::write_step(&mut line, event.step);
    let _ = write!(
        line,
        "|{}|{}|{}|{}",
        lifecycle(event.lifecycle),
        event.generation,
        option(event.frame),
        option(event.control),
    );
    for queue in [
        event.deferred,
        event.controls,
        event.visual,
        event.in_flight,
    ] {
        let _ = write!(
            line,
            "|{}|{}|{}",
            queue.items,
            queue.bytes,
            option(queue.residence)
        );
    }
    let renderer = event.renderer;
    let _ = write!(
        line,
        "|{}|{}|{}|{}|{}|{}",
        renderer.items,
        renderer.bytes,
        option(renderer.residence),
        submission(renderer.last),
        submission(renderer.completed),
        bool_word(renderer.pending),
    );
    line
}

fn write_queue(line: &mut String, queue: QueueRecordV1) {
    let _ = write!(line, "|{}|{}", queue.items, queue.bytes);
}

const fn stage(value: HeadlessTraceStageV1) -> &'static str {
    match value {
        HeadlessTraceStageV1::Build => "build",
        HeadlessTraceStageV1::Input => "input",
        HeadlessTraceStageV1::Callback => "callback",
        HeadlessTraceStageV1::Transaction => "transaction",
        HeadlessTraceStageV1::Projection => "projection",
        HeadlessTraceStageV1::Scheduler => "scheduler",
        HeadlessTraceStageV1::Renderer => "renderer",
    }
}

const fn input(value: HeadlessInputKindV1) -> &'static str {
    match value {
        HeadlessInputKindV1::None => "none",
        HeadlessInputKindV1::Pointer => "pointer",
        HeadlessInputKindV1::Direct => "direct",
        HeadlessInputKindV1::Insert => "insert",
        HeadlessInputKindV1::Move => "move",
        HeadlessInputKindV1::Update => "update",
        HeadlessInputKindV1::Remove => "remove",
        HeadlessInputKindV1::Resize => "resize",
        HeadlessInputKindV1::FrameReady => "frame-ready",
        HeadlessInputKindV1::Completion => "completion",
        HeadlessInputKindV1::Loss => "loss",
        HeadlessInputKindV1::Shutdown => "shutdown",
    }
}

fn outcome(value: HeadlessOutcomeV1) -> &'static str {
    match value {
        HeadlessOutcomeV1::Observed => "observed",
        HeadlessOutcomeV1::Deferred => "deferred",
        HeadlessOutcomeV1::Published => "published",
        HeadlessOutcomeV1::NoChange => "no-change",
        HeadlessOutcomeV1::Matched => "matched",
        HeadlessOutcomeV1::Action => "action",
        HeadlessOutcomeV1::Accepted => "accepted",
        HeadlessOutcomeV1::Rejected => "rejected",
        HeadlessOutcomeV1::Completed => "completed",
        HeadlessOutcomeV1::Lost => "lost",
        HeadlessOutcomeV1::Stopped => "stopped",
        HeadlessOutcomeV1::Failed(cause) => match cause {
            HeadlessFailureCauseV1::Runtime => "failed:runtime",
            HeadlessFailureCauseV1::Projection => "failed:projection",
            HeadlessFailureCauseV1::Oracle => "failed:oracle",
            HeadlessFailureCauseV1::Scheduler => "failed:scheduler",
            HeadlessFailureCauseV1::Renderer => "failed:renderer",
            HeadlessFailureCauseV1::Trace => "failed:trace",
        },
    }
}

fn target(value: HeadlessPointerTargetV1) -> String {
    match value {
        HeadlessPointerTargetV1::None => "none".to_owned(),
        HeadlessPointerTargetV1::StaticControl => "static-control".to_owned(),
        HeadlessPointerTargetV1::Key(key) => format!("key:{key}"),
    }
}

const fn lifecycle(value: SchedulerState) -> &'static str {
    match value {
        SchedulerState::Running => "running",
        SchedulerState::ShutdownQueued => "shutdown-queued",
        SchedulerState::Draining => "draining",
        SchedulerState::Stopped => "stopped",
        SchedulerState::Faulted => "faulted",
    }
}

fn submission(value: Option<super::super::record::scheduler::SubmissionRecordV1>) -> String {
    value.map_or_else(
        || "-".to_owned(),
        |value| format!("{}:{}", value.epoch, value.token),
    )
}

fn option(value: Option<u64>) -> String {
    value.map_or_else(|| "-".to_owned(), |value| value.to_string())
}

const fn bool_word(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}
