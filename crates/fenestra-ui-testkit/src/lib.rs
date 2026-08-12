#![forbid(unsafe_code)]
#![allow(
    clippy::result_large_err,
    reason = "the frozen testkit APIs retain typed runtime errors by value"
)]

//! Deterministic test support for the experimental Fenestra runtime.
//!
//! Product crates must not depend on this package.

mod case;
mod desired;
mod error;
mod failure;
mod fingerprint;
mod fixture;
mod generate;
mod headless;
mod identity;
mod model;
mod observe;
mod reducer;
mod replay;
mod resolve;
mod scheduler;
mod semantic;
mod trace;
mod wire;

/// Unstable cross-crate surface used only by unpublished feasibility probes.
#[doc(hidden)]
pub mod prototype {
    pub use crate::case::{
        GeneratedCaseV1, GeneratorConfigV1, OperationIdV1, OperationV1, SeedV1,
        SemanticOperationV1, TransactionIdV1, TransactionV1,
    };
    pub use crate::desired::DesiredStateV1;
    pub use crate::error::{
        GeneratorError, GeneratorErrorKind, HarnessError, HarnessErrorKind, HarnessLimitKind,
    };
    pub use crate::failure::ReplayFailureV1;
    pub use crate::fingerprint::{
        FailureFingerprintKindV1, FailureFingerprintV1, FingerprintFieldV1, FingerprintLocationV1,
        FingerprintSummaryV1,
    };
    pub use crate::fixture::{HarnessLimitsV1, ReplayConfigV1, RuntimeOracleFixtureV1};
    pub use crate::generate::generate_case_v1;
    pub use crate::headless::{
        HeadlessArtifactCapacityKindV1, HeadlessArtifactCountKindV1,
        HeadlessArtifactDecodeErrorKindV1, HeadlessArtifactDecodeErrorV1,
        HeadlessArtifactEncodeErrorKindV1, HeadlessArtifactEncodeErrorV1,
        HeadlessArtifactLimitKindV1, HeadlessArtifactSectionKindV1, HeadlessArtifactV1,
        HeadlessArtifactVerificationErrorKindV1, HeadlessArtifactVerificationErrorV1,
        HeadlessArtifactVersionKindV1, HeadlessCallbackReportV1, HeadlessFailureCauseV1,
        HeadlessFixtureV1, HeadlessInputKindV1, HeadlessMismatchFieldV1, HeadlessMismatchKindV1,
        HeadlessMismatchLocationV1, HeadlessMismatchV1, HeadlessOracleV1, HeadlessOutcomeV1,
        HeadlessPlatformErrorKindV1, HeadlessPlatformErrorV1, HeadlessPointerCaptureV1,
        HeadlessPointerMutationV1, HeadlessPointerScriptV1, HeadlessPointerTargetV1,
        HeadlessProjectionFaultV1, HeadlessRendererErrorKindV1, HeadlessRendererErrorV1,
        HeadlessResultV1, HeadlessRunErrorV1, HeadlessRunV1, HeadlessTraceCapacityV1,
        HeadlessTraceEventV1, HeadlessTraceProjectionCountsV1, HeadlessTraceQueueStatsV1,
        HeadlessTraceRendererStatsV1, HeadlessTraceStageV1, HeadlessTraceV1,
        NormalizedHeadlessComputedStyleV1, NormalizedHeadlessGeometryV1,
        NormalizedHeadlessHitRegionV1, NormalizedHeadlessProjectionV1,
        NormalizedHeadlessSceneRectangleV1, NormalizedHeadlessSemanticV1,
        ObservedHeadlessProjectionV1, build_headless_artifact_v1, compare_headless_projection_v1,
        decode_headless_artifact_v1, encode_headless_artifact_v1, headless_frame_resource_v1,
        inject_headless_projection_fault_v1, inject_headless_surface_fault_v1,
        observe_headless_projection_v1, run_headless_spine_v1, verify_headless_artifact_v1,
    };
    pub use crate::identity::IdentitySummaryV1;
    pub use crate::model::clean_rebuild_v1;
    pub use crate::observe::observe_snapshot_v1;
    pub use crate::reducer::{
        ReducerConfigV1, ReducerError, ReducerErrorKind, ReductionCompletionV1, ReductionResultV1,
        reduce_failure_case_v1,
    };
    pub use crate::replay::{
        ReplayReportV1, replay_case_v1, replay_case_with_fault_v1, replay_case_with_trace_v1,
    };
    pub use crate::scheduler::{
        FakeCallbackDepthV1, FakeCallbackMutationV1, FakeCallbackReportV1, FakeCallbackScriptV1,
        FakeClockDomainV1, FakeClockErrorKindV1, FakeClockErrorV1, FakeClockV1,
        FakeControlDeliveryV1, FakeFrameReadyDeliveryV1, FakePlatformErrorKindV1,
        FakePlatformErrorV1, FakePlatformV1, FakeRendererCapacityV1, FakeRendererErrorKindV1,
        FakeRendererErrorV1, FakeRendererModeV1, FakeRendererOfferOutcomeV1, FakeRendererStatsV1,
        FakeRendererV1, SchedulerTraceActionV1, SchedulerTraceCallbackOutcomeV1,
        SchedulerTraceCapacityV1, SchedulerTraceCommitOutcomeV1, SchedulerTraceErrorKindV1,
        SchedulerTraceErrorV1, SchedulerTraceEventV1, SchedulerTraceInputOutcomeV1,
        SchedulerTraceLaneStatsV1, SchedulerTraceLimitV1, SchedulerTraceRendererStatsV1,
        SchedulerTraceStageV1, SchedulerTraceStepV1, SchedulerTraceV1, SyntheticResourceIdV1,
        SyntheticResourceUseV1,
    };
    pub use crate::semantic::{
        FragmentPathV1, NodePathV1, NormalizedChildGroupV1, NormalizedFragmentV1,
        NormalizedMemberV1, NormalizedNodeV1, NormalizedPropertyV1, NormalizedStateFaultV1,
        NormalizedStateV1, PathSegmentV1, inject_normalized_state_fault_v1,
    };
    pub use crate::trace::{
        CandidateRejectionV1, LogicalTraceV1, TraceComparisonV1, TraceEventV1, TraceFaultV1,
        TraceOutcomeV1, TraceProvenanceV1, TraceTerminationV1,
    };
    pub use crate::wire::{
        ArtifactDecodeError, ArtifactDecodeErrorKind, ArtifactEncodeError,
        ArtifactFixtureMetadataV1, ArtifactLimitKind, ArtifactReductionV1, ArtifactReplayConfigV1,
        ArtifactVerificationError, ArtifactVerificationErrorKind, CaseDecodeContextV1, CountKind,
        FailureArtifactV1, SectionKind, VersionKind, decode_case_v1, decode_failure_artifact_v1,
        encode_case_v1, encode_failure_artifact_v1, verify_failure_artifact_v1,
    };
}
