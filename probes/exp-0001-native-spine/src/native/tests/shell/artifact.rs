use fenestra_ui_runtime::prototype::SchedulerTick;

use super::super::super::artifact::{
    NATIVE_ARTIFACT_MAX_BYTES_V1, NATIVE_ARTIFACT_MAX_EVENTS_V1, NATIVE_ARTIFACT_MAX_LINES_V1,
    NATIVE_ARTIFACT_SCHEMA_REVISION_V1, NativeArtifactCapabilitiesV1, NativeArtifactManifestV1,
    NativeArtifactTerminalV1, NativeOsFamilyV1, NativeProbeResultV1, NativeTargetV1,
    NativeWindowSystemV1, encode_native_artifact_v1,
};
use super::super::super::driver::NativeDriverV1;
use super::super::super::trace::{
    NativeFailureCauseV1, NativeInputSourceV1, NativeObservationV1, NativeOutcomeV1,
    NativeTraceStageV1,
};
use super::artifact_expected::assert_complete_event_line;
use super::support::{AcceptingPresenter, completed_reference_driver};

#[test]
fn artifact_constants_and_reference_manifest_are_exact() {
    assert_eq!(NATIVE_ARTIFACT_SCHEMA_REVISION_V1, 1);
    assert_eq!(NATIVE_ARTIFACT_MAX_EVENTS_V1, 128);
    assert_eq!(NATIVE_ARTIFACT_MAX_LINES_V1, 131);
    assert_eq!(NATIVE_ARTIFACT_MAX_BYTES_V1, 65_536);

    let driver = completed_reference_driver();
    let terminal = NativeArtifactTerminalV1::try_from_driver(NativeProbeResultV1::Pass, &driver)
        .expect("stopped reference run should form a terminal snapshot");
    let encoded = encode_native_artifact_v1(&manifest(&driver), driver.trace(), &terminal)
        .expect("bounded reference artifact should encode");
    let lines = encoded.lines().collect::<Vec<_>>();

    assert_eq!(lines[0], "fenestra-native-artifact|1");
    assert_eq!(
        lines[1],
        concat!(
            "manifest|os=linux|target=x86_64-unknown-linux-gnu|window=wayland",
            "|winit=0.30.13|winit_features=rwh_06,wayland,wayland-dlopen",
            "|softbuffer=0.4.8|softbuffer_features=wayland,wayland-dlopen",
            "|physical=720x520|logical=360x260|scale_micros=2000000",
            "|requested=31|detected=31|effective=31"
        )
    );
}

#[test]
fn artifact_is_ascii_lf_bounded_and_serializes_every_complete_event() {
    let driver = completed_reference_driver();
    let terminal = NativeArtifactTerminalV1::try_from_driver(NativeProbeResultV1::Pass, &driver)
        .expect("stopped reference run should form a terminal snapshot");
    let manifest = manifest(&driver);
    let encoded = encode_native_artifact_v1(&manifest, driver.trace(), &terminal)
        .expect("bounded reference artifact should encode");
    assert_eq!(
        encoded,
        encode_native_artifact_v1(&manifest, driver.trace(), &terminal)
            .expect("same typed evidence should encode deterministically")
    );
    assert!(encoded.ends_with('\n'));
    assert!(!encoded.contains('\r'));
    assert!(
        encoded
            .bytes()
            .all(|byte| byte == b'\n' || (b' '..=b'~').contains(&byte))
    );
    assert!(encoded.len() <= NATIVE_ARTIFACT_MAX_BYTES_V1);

    let lines = encoded.lines().collect::<Vec<_>>();
    assert_eq!(lines.len(), driver.trace().len() + 3);
    assert!(lines.len() <= NATIVE_ARTIFACT_MAX_LINES_V1);
    let event_lines = &lines[2..2 + driver.trace().len()];
    for (event, line) in driver.trace().events().iter().zip(event_lines) {
        assert_complete_event_line(*event, line);
    }
    assert_eq!(
        lines.last().copied(),
        Some(concat!(
            "terminal|result=pass|generation=2|scheduler=stopped",
            "|deferred=0:0|controls=0:0|visual=0:0|in_flight=0:0",
            "|redraw=0|pending=0:0:0"
        ))
    );
    for forbidden in ["eddndev", "/home/", "WAYLAND_DISPLAY", "wayland-0"] {
        assert!(!encoded.contains(forbidden));
    }
}

#[test]
fn only_accepted_frames_carry_digest_before_their_present_event() {
    let driver = completed_reference_driver();
    let terminal = NativeArtifactTerminalV1::try_from_driver(NativeProbeResultV1::Pass, &driver)
        .expect("stopped reference run should form a terminal snapshot");
    let encoded = encode_native_artifact_v1(&manifest(&driver), driver.trace(), &terminal)
        .expect("reference artifact should encode");
    let lines = encoded.lines().skip(2).collect::<Vec<_>>();
    let events = driver.trace().events();
    let accepted = events
        .iter()
        .filter(|event| {
            event.stage() == NativeTraceStageV1::Scheduler
                && event.observation() == NativeObservationV1::Frame
                && event.outcome() == NativeOutcomeV1::Accepted
        })
        .collect::<Vec<_>>();
    assert_eq!(accepted.len(), 2);

    for (identity, event) in accepted.into_iter().enumerate() {
        let digest = event
            .staging_digest()
            .expect("accepted frame must record its staging digest");
        let line = lines[event.sequence() as usize];
        assert_eq!(field(line, "frame"), identity.to_string());
        assert_eq!(field(line, "submission"), format!("0:{identity}"));
        assert_eq!(field(line, "digest"), format!("{digest:016x}"));
        let presented = events.iter().find(|candidate| {
            candidate.sequence() > event.sequence()
                && candidate.stage() == NativeTraceStageV1::Renderer
                && candidate.observation() == NativeObservationV1::Present
                && candidate.outcome() == NativeOutcomeV1::Completed
                && candidate.frame() == event.frame()
                && candidate.submission() == event.submission()
        });
        assert!(presented.is_some(), "accepted frame must present later");
    }
    assert!(events.iter().all(|event| {
        let is_accepted = event.stage() == NativeTraceStageV1::Scheduler
            && event.observation() == NativeObservationV1::Frame
            && event.outcome() == NativeOutcomeV1::Accepted;
        is_accepted == event.staging_digest().is_some()
    }));
}

#[test]
fn terminal_snapshot_rejects_running_and_nonempty_drivers() {
    let running = NativeDriverV1::new(AcceptingPresenter).expect("running driver should build");
    assert_eq!(
        NativeArtifactTerminalV1::try_from_driver(NativeProbeResultV1::Stop, &running)
            .expect_err("running state is not terminal"),
        NativeFailureCauseV1::Invariant
    );

    let mut queued = NativeDriverV1::new(AcceptingPresenter).expect("queued driver should build");
    queued
        .close_requested(NativeInputSourceV1::Scripted, SchedulerTick::new(0))
        .expect("shutdown should queue");
    assert_eq!(queued.scheduler_stats().controls().items(), 1);
    assert_eq!(
        NativeArtifactTerminalV1::try_from_driver(NativeProbeResultV1::Stop, &queued)
            .expect_err("nonempty shutdown state is not terminal"),
        NativeFailureCauseV1::Invariant
    );

    let stopped = completed_reference_driver();
    let terminal = NativeArtifactTerminalV1::try_from_driver(NativeProbeResultV1::Pass, &stopped)
        .expect("stopped empty driver should be terminal");
    assert_eq!(terminal.result(), NativeProbeResultV1::Pass);
    assert_eq!(terminal.runtime_generation(), 2);
    assert!(terminal.is_stopped_and_empty());
}

fn manifest(driver: &NativeDriverV1<AcceptingPresenter>) -> NativeArtifactManifestV1 {
    let capabilities = NativeArtifactCapabilitiesV1::new(true, true, true, true, true);
    NativeArtifactManifestV1::new(
        NativeOsFamilyV1::Linux,
        NativeTargetV1::X86_64UnknownLinuxGnu,
        NativeWindowSystemV1::Wayland,
        driver
            .accepted_surface()
            .expect("reference run should retain its final surface"),
        capabilities,
        capabilities,
        capabilities,
    )
}

fn field(line: &str, key: &str) -> String {
    line.split('|')
        .skip(1)
        .find_map(|part| {
            let (candidate, value) = part.split_once('=')?;
            (candidate == key).then(|| value.to_owned())
        })
        .unwrap_or_else(|| panic!("missing field {key} in {line}"))
}
