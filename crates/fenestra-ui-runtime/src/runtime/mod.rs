#![allow(
    clippy::result_large_err,
    reason = "the frozen runtime API retains typed spatial resolver evidence by value"
)]

mod apply;
mod capacity;
mod change;
mod commit_control;
mod edit;
mod error;
mod expand;
mod fragment;
mod headless;
mod instantiate;
mod mutation;
mod scheduler;
mod spatial;
mod state;
mod transaction;
mod view;

pub use capacity::RuntimeCapacity;
pub use error::{
    CapacityKind, RuntimeInitializationError, RuntimeInitializationErrorKind, TransactionError,
    TransactionErrorKind,
};
pub use fragment::FragmentId;
pub use headless::{
    ComputedStyleView, HeadlessGeometryView, HeadlessHitRegionView, HeadlessPoint,
    HeadlessProjectionCapacity, HeadlessProjectionErrorKind, HeadlessProjectionLimitKind,
    HeadlessProjectionSpec, HeadlessProjectionView, HeadlessRect, HeadlessSceneRectangleView,
    HeadlessSemanticAction, HeadlessSemanticRole, HeadlessSemanticView, HeadlessSurface,
};
pub use mutation::{
    HeadlessSurfaceChangeView, KeyInsertView, KeyMoveView, KeyRemoveView, ManifestEntry,
    ManifestIter, MutationIter, MutationRecordView, PropertyChangeView,
    SpatialViewportChangeViewV2,
};
pub use scheduler::{
    CallbackFinish, CallbackScope, CompletionWatermark, ControlAdmission, ControlSequence, FrameId,
    FrameWork, NestedCallbackScope, QueueCapacity, QueueStats, RendererEpoch, RuntimePaintFrameV2,
    ScheduledCommit, SchedulerAction, SchedulerCapacity, SchedulerError, SchedulerErrorKind,
    SchedulerInput, SchedulerInputResult, SchedulerLane, SchedulerState, SchedulerStats,
    SchedulerTick, SubmissionId, UiScheduler, VisualCancelResult,
};
pub use spatial::{
    RuntimeSpatialBuildViewV2, RuntimeSpatialErrorV2, RuntimeSpatialInputV2,
    RuntimeSpatialIrErrorKindV2, RuntimeSpatialIrErrorV2, RuntimeSpatialProgramV2,
    RuntimeSpatialViewV2,
};
pub use state::RuntimeGeneration;
pub use transaction::{CommitReceipt, UiRuntime, UiTransaction};
pub use view::{CommittedRuntimeSnapshot, KeyedMemberIter};

#[cfg(test)]
mod tests;
