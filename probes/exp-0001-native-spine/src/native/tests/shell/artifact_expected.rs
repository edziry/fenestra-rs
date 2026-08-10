use super::super::super::trace::{
    NativeFailureCauseV1, NativeInputSourceV1, NativeObservationV1, NativeOutcomeV1,
    NativeTraceEventV1, NativeTraceLaneStatsV1, NativeTraceStageV1,
};

const EVENT_KEYS: [&str; 27] = [
    "sequence",
    "tick",
    "scheduler_turn",
    "stage",
    "observation",
    "outcome",
    "source",
    "state",
    "generation",
    "captured",
    "published",
    "native_generation",
    "physical",
    "logical",
    "scale_micros",
    "target",
    "frame",
    "submission",
    "control",
    "digest",
    "redraw",
    "pending",
    "deferred",
    "controls",
    "visual",
    "in_flight",
    "accounted_bytes",
];

pub(super) fn assert_complete_event_line(event: NativeTraceEventV1, line: &str) {
    let keys = line
        .split('|')
        .skip(1)
        .map(|part| {
            part.split_once('=')
                .expect("event field must use key=value")
                .0
        })
        .collect::<Vec<_>>();
    assert_eq!(keys, EVENT_KEYS);
    assert_eq!(field(line, "sequence"), event.sequence().to_string());
    assert_eq!(field(line, "tick"), event.tick().get().to_string());
    assert_eq!(
        field(line, "scheduler_turn"),
        optional(event.scheduler_turn())
    );
    assert_eq!(field(line, "stage"), stage(event.stage()));
    assert_eq!(field(line, "observation"), observation(event.observation()));
    assert_eq!(field(line, "outcome"), outcome(event.outcome()));
    assert_eq!(
        field(line, "source"),
        event.input_source().map_or("-", source)
    );
    assert_eq!(
        field(line, "state"),
        scheduler_state(event.scheduler_state())
    );
    assert_eq!(
        field(line, "generation"),
        event.current_generation().get().to_string()
    );
    assert_eq!(
        field(line, "captured"),
        optional(event.captured_generation().map(|value| value.get()))
    );
    assert_eq!(
        field(line, "published"),
        optional(event.published_generation().map(|value| value.get()))
    );
    assert_surface(event, line);
    assert_eq!(
        field(line, "target"),
        event.target().map_or_else(|| "-".to_owned(), target)
    );
    assert_eq!(field(line, "frame"), optional(event.frame()));
    assert_eq!(
        field(line, "submission"),
        event.submission().map_or_else(
            || "-".to_owned(),
            |value| format!("{}:{}", value.epoch(), value.token()),
        )
    );
    assert_eq!(field(line, "control"), optional(event.control()));
    assert_eq!(
        field(line, "digest"),
        event
            .staging_digest()
            .map_or_else(|| "-".to_owned(), |digest| format!("{digest:016x}"),)
    );
    assert_eq!(
        field(line, "redraw"),
        u8::from(event.redraw_armed()).to_string()
    );
    assert_eq!(
        field(line, "pending"),
        format!(
            "{}:{}:{}",
            event.pending().surface(),
            event.pending().pointer(),
            event.pending().presenter()
        )
    );
    assert_lane(line, "deferred", event.deferred());
    assert_lane(line, "controls", event.controls());
    assert_lane(line, "visual", event.visual());
    assert_lane(line, "in_flight", event.in_flight());
    assert_eq!(field(line, "accounted_bytes"), "192");
}

fn assert_surface(event: NativeTraceEventV1, line: &str) {
    if let Some(surface) = event.surface() {
        assert_eq!(
            field(line, "native_generation"),
            surface.generation().get().to_string()
        );
        assert_eq!(
            field(line, "physical"),
            format!(
                "{}x{}",
                surface.physical().width(),
                surface.physical().height()
            )
        );
        assert_eq!(
            field(line, "logical"),
            format!(
                "{}x{}",
                surface.logical_surface().width(),
                surface.logical_surface().height()
            )
        );
        assert_eq!(
            field(line, "scale_micros"),
            surface.scale().micros().to_string()
        );
    } else if let Some(observed) = event.surface_observation() {
        assert_eq!(field(line, "native_generation"), "-");
        assert_eq!(
            field(line, "physical"),
            format!(
                "{}x{}",
                observed.physical().width(),
                observed.physical().height()
            )
        );
        assert_eq!(
            field(line, "logical"),
            format!(
                "{}x{}",
                observed.logical_surface().width(),
                observed.logical_surface().height()
            )
        );
        assert_eq!(
            field(line, "scale_micros"),
            observed.scale().micros().to_string()
        );
    } else {
        for key in ["native_generation", "physical", "logical", "scale_micros"] {
            assert_eq!(field(line, key), "-");
        }
    }
}

fn field<'a>(line: &'a str, key: &str) -> &'a str {
    line.split('|')
        .skip(1)
        .find_map(|part| {
            let (candidate, value) = part.split_once('=')?;
            (candidate == key).then_some(value)
        })
        .unwrap_or_else(|| panic!("missing field {key} in {line}"))
}

fn optional(value: Option<u64>) -> String {
    value.map_or_else(|| "-".to_owned(), |value| value.to_string())
}

fn assert_lane(line: &str, key: &str, lane: NativeTraceLaneStatsV1) {
    assert_eq!(
        field(line, key),
        format!("{}:{}", lane.items(), lane.accounted_bytes())
    );
}

const fn stage(value: NativeTraceStageV1) -> &'static str {
    match value {
        NativeTraceStageV1::Manifest => "manifest",
        NativeTraceStageV1::Shell => "shell",
        NativeTraceStageV1::Platform => "platform",
        NativeTraceStageV1::Scheduler => "scheduler",
        NativeTraceStageV1::Renderer => "renderer",
        NativeTraceStageV1::Oracle => "oracle",
    }
}

const fn observation(value: NativeObservationV1) -> &'static str {
    match value {
        NativeObservationV1::Build => "build",
        NativeObservationV1::Resumed => "resumed",
        NativeObservationV1::Surface => "surface",
        NativeObservationV1::Scale => "scale",
        NativeObservationV1::Pointer => "pointer",
        NativeObservationV1::Redraw => "redraw",
        NativeObservationV1::Frame => "frame",
        NativeObservationV1::Present => "present",
        NativeObservationV1::Close => "close",
        NativeObservationV1::Completion => "completion",
        NativeObservationV1::Shutdown => "shutdown",
        NativeObservationV1::Timeout => "timeout",
    }
}

fn outcome(value: NativeOutcomeV1) -> String {
    match value {
        NativeOutcomeV1::Observed => "observed".into(),
        NativeOutcomeV1::Coalesced => "coalesced".into(),
        NativeOutcomeV1::Ignored => "ignored".into(),
        NativeOutcomeV1::Deferred => "deferred".into(),
        NativeOutcomeV1::Published => "published".into(),
        NativeOutcomeV1::Armed => "armed".into(),
        NativeOutcomeV1::Offered => "offered".into(),
        NativeOutcomeV1::Accepted => "accepted".into(),
        NativeOutcomeV1::Rejected => "rejected".into(),
        NativeOutcomeV1::Completed => "completed".into(),
        NativeOutcomeV1::Matched => "matched".into(),
        NativeOutcomeV1::Stopped => "stopped".into(),
        NativeOutcomeV1::Failed(cause) => format!("failed:{}", failure(cause)),
    }
}

const fn source(value: NativeInputSourceV1) -> &'static str {
    match value {
        NativeInputSourceV1::Native => "native",
        NativeInputSourceV1::Scripted => "scripted",
    }
}

fn scheduler_state(value: fenestra_ui_runtime::prototype::SchedulerState) -> &'static str {
    match value {
        fenestra_ui_runtime::prototype::SchedulerState::Running => "running",
        fenestra_ui_runtime::prototype::SchedulerState::ShutdownQueued => "shutdown-queued",
        fenestra_ui_runtime::prototype::SchedulerState::Faulted => "faulted",
        fenestra_ui_runtime::prototype::SchedulerState::Draining => "draining",
        fenestra_ui_runtime::prototype::SchedulerState::Stopped => "stopped",
    }
}

fn failure(value: NativeFailureCauseV1) -> &'static str {
    match value {
        NativeFailureCauseV1::InvalidScale => "invalid-scale",
        NativeFailureCauseV1::InvalidPoint => "invalid-point",
        NativeFailureCauseV1::Arithmetic => "arithmetic",
        NativeFailureCauseV1::WidthLimit => "width-limit",
        NativeFailureCauseV1::HeightLimit => "height-limit",
        NativeFailureCauseV1::PixelLimit => "pixel-limit",
        NativeFailureCauseV1::ByteLimit => "byte-limit",
        NativeFailureCauseV1::UnsupportedAlpha => "unsupported-alpha",
        NativeFailureCauseV1::Storage => "storage",
        NativeFailureCauseV1::EnvironmentScaleChanged => "environment-scale-changed",
        NativeFailureCauseV1::SurfaceRepaintUnavailable => "surface-repaint-unavailable",
        NativeFailureCauseV1::Runtime => "runtime",
        NativeFailureCauseV1::Oracle => "oracle",
        NativeFailureCauseV1::Scheduler => "scheduler",
        NativeFailureCauseV1::PrePresent => "pre-present",
        NativeFailureCauseV1::Presenter => "presenter",
        NativeFailureCauseV1::Trace => "trace",
        NativeFailureCauseV1::Timeout => "timeout",
        NativeFailureCauseV1::Invariant => "invariant",
    }
}

fn target(value: fenestra_ui_testkit::prototype::HeadlessPointerTargetV1) -> String {
    match value {
        fenestra_ui_testkit::prototype::HeadlessPointerTargetV1::None => "none".to_owned(),
        fenestra_ui_testkit::prototype::HeadlessPointerTargetV1::StaticControl => {
            "static-control".to_owned()
        }
        fenestra_ui_testkit::prototype::HeadlessPointerTargetV1::Key(key) => format!("key:{key}"),
    }
}
