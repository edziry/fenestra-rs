mod clock;
mod platform;
mod renderer;
mod trace;

pub use clock::{FakeClockDomainV1, FakeClockErrorKindV1, FakeClockErrorV1, FakeClockV1};
pub use platform::{
    FakeCallbackDepthV1, FakeCallbackMutationV1, FakeCallbackReportV1, FakeCallbackScriptV1,
    FakeFrameReadyDeliveryV1, FakePlatformErrorKindV1, FakePlatformErrorV1, FakePlatformV1,
};
pub use renderer::{
    FakeControlDeliveryV1, FakeRendererCapacityV1, FakeRendererErrorKindV1, FakeRendererErrorV1,
    FakeRendererModeV1, FakeRendererOfferOutcomeV1, FakeRendererStatsV1, FakeRendererV1,
    SyntheticResourceIdV1, SyntheticResourceUseV1,
};
pub use trace::{
    SchedulerTraceActionV1, SchedulerTraceCallbackOutcomeV1, SchedulerTraceCapacityV1,
    SchedulerTraceCommitOutcomeV1, SchedulerTraceErrorKindV1, SchedulerTraceErrorV1,
    SchedulerTraceEventV1, SchedulerTraceInputOutcomeV1, SchedulerTraceLaneStatsV1,
    SchedulerTraceLimitV1, SchedulerTraceRendererStatsV1, SchedulerTraceStageV1,
    SchedulerTraceStepV1, SchedulerTraceV1,
};
