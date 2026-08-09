mod artifact;
mod fixture;
mod oracle;
mod platform;
mod renderer;
mod runner;
mod trace;

pub use artifact::{
    HeadlessArtifactCapacityKindV1, HeadlessArtifactCountKindV1, HeadlessArtifactDecodeErrorKindV1,
    HeadlessArtifactDecodeErrorV1, HeadlessArtifactEncodeErrorKindV1,
    HeadlessArtifactEncodeErrorV1, HeadlessArtifactLimitKindV1, HeadlessArtifactSectionKindV1,
    HeadlessArtifactV1, HeadlessArtifactVerificationErrorKindV1,
    HeadlessArtifactVerificationErrorV1, HeadlessArtifactVersionKindV1, build_headless_artifact_v1,
    decode_headless_artifact_v1, encode_headless_artifact_v1, verify_headless_artifact_v1,
};
pub use fixture::HeadlessFixtureV1;
pub use oracle::{
    HeadlessMismatchFieldV1, HeadlessMismatchKindV1, HeadlessMismatchLocationV1,
    HeadlessMismatchV1, HeadlessOracleV1, HeadlessProjectionFaultV1,
    NormalizedHeadlessComputedStyleV1, NormalizedHeadlessGeometryV1, NormalizedHeadlessHitRegionV1,
    NormalizedHeadlessProjectionV1, NormalizedHeadlessSceneRectangleV1,
    NormalizedHeadlessSemanticV1, ObservedHeadlessProjectionV1, compare_headless_projection_v1,
    inject_headless_projection_fault_v1, observe_headless_projection_v1,
};
pub use platform::{
    HeadlessCallbackReportV1, HeadlessPlatformErrorKindV1, HeadlessPlatformErrorV1,
    HeadlessPointerCaptureV1, HeadlessPointerMutationV1, HeadlessPointerScriptV1,
    HeadlessPointerTargetV1,
};
pub use renderer::{
    HeadlessRendererErrorKindV1, HeadlessRendererErrorV1, headless_frame_resource_v1,
};
pub use runner::{HeadlessResultV1, HeadlessRunErrorV1, HeadlessRunV1, run_headless_spine_v1};
pub use trace::{
    HeadlessFailureCauseV1, HeadlessInputKindV1, HeadlessOutcomeV1, HeadlessTraceCapacityV1,
    HeadlessTraceEventV1, HeadlessTraceProjectionCountsV1, HeadlessTraceQueueStatsV1,
    HeadlessTraceRendererStatsV1, HeadlessTraceStageV1, HeadlessTraceV1,
};
