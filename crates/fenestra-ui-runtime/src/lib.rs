#![forbid(unsafe_code)]

//! Experimental headless runtime kernel for Fenestra.
//!
//! Runtime behavior remains private until its owning feasibility work provides
//! executable evidence.

mod arena;
mod logical_tree;
mod runtime;

/// Unstable cross-crate surface used only by unpublished feasibility probes.
#[doc(hidden)]
pub mod prototype {
    pub use crate::logical_tree::{LogicalTree, NodeId, TreeError, TreeInvariantError};
    pub use crate::runtime::{
        CallbackFinish, CallbackScope, CapacityKind, CommitReceipt, CommittedRuntimeSnapshot,
        CompletionWatermark, ComputedStyleView, ControlAdmission, ControlSequence, FragmentId,
        FrameId, FrameWork, HeadlessGeometryView, HeadlessHitRegionView, HeadlessPoint,
        HeadlessProjectionCapacity, HeadlessProjectionErrorKind, HeadlessProjectionLimitKind,
        HeadlessProjectionSpec, HeadlessProjectionView, HeadlessRect, HeadlessSceneRectangleView,
        HeadlessSemanticAction, HeadlessSemanticRole, HeadlessSemanticView, HeadlessSurface,
        HeadlessSurfaceChangeView, KeyInsertView, KeyMoveView, KeyRemoveView, KeyedMemberIter,
        ManifestEntry, ManifestIter, MutationIter, MutationRecordView, NestedCallbackScope,
        PropertyChangeView, QueueCapacity, QueueStats, RendererEpoch, RuntimeCapacity,
        RuntimeGeneration, RuntimeInitializationError, RuntimeInitializationErrorKind,
        RuntimePaintFrameV2, RuntimeSpatialBuildViewV2, RuntimeSpatialErrorV2,
        RuntimeSpatialInputV2, RuntimeSpatialIrErrorKindV2, RuntimeSpatialIrErrorV2,
        RuntimeSpatialProgramV2, RuntimeSpatialViewV2, ScheduledCommit, SchedulerAction,
        SchedulerCapacity, SchedulerError, SchedulerErrorKind, SchedulerInput,
        SchedulerInputResult, SchedulerLane, SchedulerState, SchedulerStats, SchedulerTick,
        SpatialViewportChangeViewV2, SubmissionId, TransactionError, TransactionErrorKind,
        UiRuntime, UiScheduler, UiTransaction, VisualCancelResult, VisualRequestResult,
    };
}
