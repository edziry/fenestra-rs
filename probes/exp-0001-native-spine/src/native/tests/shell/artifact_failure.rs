use fenestra_ui_runtime::prototype::SchedulerTick;

use super::super::super::artifact::{
    NativeArtifactCapabilitiesV1, NativeArtifactManifestV1, NativeArtifactTerminalV1,
    NativeOsFamilyV1, NativeProbeResultV1, NativeTargetV1, NativeWindowSystemV1,
    encode_native_artifact_v1,
};
use super::super::super::driver::{NativeDriverActionV1, NativeDriverV1, PresenterPortV1};
use super::super::super::raster::CpuFrameV1;
use super::super::super::shell::script::classify_native_failure_v1;
use super::super::super::trace::{NativeFailureCauseV1, NativeInputSourceV1};
use super::super::super::types::NativePhysicalExtentV1;
use super::support::AcceptingPresenter;

#[test]
fn environment_scale_failure_serializes_the_rejected_observation_and_adapt_result() {
    let mut driver = NativeDriverV1::new(AcceptingPresenter).expect("driver should build");
    driver
        .observe_surface(
            NativePhysicalExtentV1::new(640, 480),
            2.0,
            SchedulerTick::new(0),
        )
        .expect("initial surface should stage");
    driver
        .drain_scheduler(SchedulerTick::new(1))
        .expect("initial surface should publish");
    let cause = driver
        .observe_surface(
            NativePhysicalExtentV1::new(720, 520),
            2.01,
            SchedulerTick::new(2),
        )
        .expect_err("effective scale change should adapt");
    assert_eq!(cause, NativeFailureCauseV1::EnvironmentScaleChanged);
    assert_eq!(
        classify_native_failure_v1(cause),
        NativeProbeResultV1::Adapt
    );
    driver
        .close_requested(NativeInputSourceV1::Scripted, SchedulerTick::new(3))
        .expect("adapt path should still shut down");
    assert!(matches!(
        driver
            .drain_scheduler(SchedulerTick::new(4))
            .expect("adapt shutdown should stop"),
        NativeDriverActionV1::StopRenderer { .. }
    ));
    let terminal = NativeArtifactTerminalV1::try_from_driver(NativeProbeResultV1::Adapt, &driver)
        .expect("clean adapted driver should form an artifact");
    let capabilities = NativeArtifactCapabilitiesV1::new(true, true, true, true, true);
    let manifest = NativeArtifactManifestV1::new(
        NativeOsFamilyV1::Linux,
        NativeTargetV1::X86_64UnknownLinuxGnu,
        NativeWindowSystemV1::Wayland,
        driver
            .accepted_surface()
            .expect("accepted tuple remains old"),
        capabilities,
        capabilities,
        capabilities,
    );
    let encoded = encode_native_artifact_v1(&manifest, driver.trace(), &terminal)
        .expect("adapt artifact should encode");
    let failure = encoded
        .lines()
        .find(|line| line.contains("outcome=failed:environment-scale-changed"))
        .expect("scale failure should be serialized");
    assert!(failure.contains("native_generation=-"));
    assert!(failure.contains("physical=720x520"));
    assert!(failure.contains("logical=359x259"));
    assert!(failure.contains("scale_micros=2010000"));
    assert!(encoded.ends_with("terminal|result=adapt|generation=1|scheduler=stopped|deferred=0:0|controls=0:0|visual=0:0|in_flight=0:0|redraw=0|pending=0:0:0\n"));
}

#[test]
fn closed_failure_classification_is_adapt_only_for_environment_boundaries() {
    for cause in NativeFailureCauseV1::ALL {
        let expected = if matches!(
            cause,
            NativeFailureCauseV1::EnvironmentScaleChanged
                | NativeFailureCauseV1::EnvironmentSurfaceChanged
                | NativeFailureCauseV1::SurfaceRepaintUnavailable
        ) {
            NativeProbeResultV1::Adapt
        } else {
            NativeProbeResultV1::Stop
        };
        assert_eq!(classify_native_failure_v1(cause), expected);
    }
}

#[test]
fn postaccept_presenter_stop_retires_before_forming_a_stop_artifact() {
    let mut driver = NativeDriverV1::new(FailingPresenter).expect("driver should build");
    driver
        .observe_surface(
            NativePhysicalExtentV1::new(640, 480),
            2.0,
            SchedulerTick::new(0),
        )
        .expect("initial surface should stage");
    driver
        .drain_scheduler(SchedulerTick::new(1))
        .expect("initial surface should publish");
    let cause = driver
        .redraw_requested(SchedulerTick::new(2))
        .expect_err("postaccept failure should be typed");
    assert_eq!(classify_native_failure_v1(cause), NativeProbeResultV1::Stop);
    driver
        .close_requested(NativeInputSourceV1::Scripted, SchedulerTick::new(2))
        .expect("stop path should queue shutdown");
    assert_eq!(
        driver
            .drain_scheduler(SchedulerTick::new(3))
            .expect("loss control should process"),
        NativeDriverActionV1::Idle
    );
    assert!(matches!(
        driver
            .drain_scheduler(SchedulerTick::new(4))
            .expect("renderer stop should drain"),
        NativeDriverActionV1::StopRenderer { .. }
    ));
    driver
        .renderer_stopped(SchedulerTick::new(5))
        .expect("failed submission should retire");
    NativeArtifactTerminalV1::try_from_driver(NativeProbeResultV1::Stop, &driver)
        .expect("clean failed driver should form a stop artifact");
}

struct FailingPresenter;

impl PresenterPortV1 for FailingPresenter {
    fn present_offer<A>(
        &mut self,
        _frame: CpuFrameV1,
        accept_once: A,
    ) -> Result<(), NativeFailureCauseV1>
    where
        A: FnOnce() -> Result<fenestra_ui_runtime::prototype::SubmissionId, NativeFailureCauseV1>,
    {
        accept_once()?;
        Err(NativeFailureCauseV1::Presenter)
    }
}
