use std::panic::{RefUnwindSafe, UnwindSafe};
use std::sync::Arc;

use fenestra_ui_ir::prototype::ValidatedStyleProgram;
use fenestra_ui_layout::prototype::LayoutEngineV1;
use fenestra_ui_spatial::prototype::{
    SpatialLimitsV2, SpatialOwnedInputV2, SpatialResolvedSnapshotV2, SpatialViewportV2,
};

use crate::logical_tree::NodeId;

use super::build::build_publication;
use super::error::RuntimeSpatialErrorV2;
use super::view::RuntimeSpatialBuildViewV2;
use crate::runtime::state::RuntimeState;

/// Runtime-owned translation from immutable logical state to raw spatial input.
pub trait RuntimeSpatialProgramV2:
    Send + Sync + Unpin + UnwindSafe + RefUnwindSafe + 'static
{
    /// Builds candidate-neutral spatial input for one exact logical state.
    #[must_use]
    fn build(
        &self,
        runtime: RuntimeSpatialBuildViewV2<'_>,
        viewport: SpatialViewportV2,
    ) -> RuntimeSpatialInputV2;
}

/// Owned spatial source and its accepted logical-node mapping.
pub struct RuntimeSpatialInputV2 {
    pub(super) source: Arc<SpatialOwnedInputV2>,
    pub(super) logical_nodes: Box<[NodeId]>,
}

impl RuntimeSpatialInputV2 {
    /// Creates one unvalidated runtime spatial input wrapper.
    #[must_use]
    pub fn new(source: Arc<SpatialOwnedInputV2>, logical_nodes: Box<[NodeId]>) -> Self {
        Self {
            source,
            logical_nodes,
        }
    }
}

pub(crate) struct SpatialRuntimeConfig {
    style: ValidatedStyleProgram,
    program: Box<dyn RuntimeSpatialProgramV2>,
    limits: SpatialLimitsV2,
    layout_engine: Box<dyn LayoutEngineV1>,
}

impl SpatialRuntimeConfig {
    pub(crate) fn new(
        style: ValidatedStyleProgram,
        program: Box<dyn RuntimeSpatialProgramV2>,
        limits: SpatialLimitsV2,
        layout_engine: Box<dyn LayoutEngineV1>,
    ) -> Self {
        Self {
            style,
            program,
            limits,
            layout_engine,
        }
    }

    pub(crate) const fn style(&self) -> &ValidatedStyleProgram {
        &self.style
    }

    pub(crate) fn build(
        &self,
        state: &RuntimeState,
        viewport: SpatialViewportV2,
    ) -> Result<Arc<SpatialPublication>, RuntimeSpatialErrorV2> {
        let input = self
            .program
            .build(RuntimeSpatialBuildViewV2::new(state), viewport);
        build_publication(
            state,
            input,
            viewport,
            self.limits,
            self.layout_engine.as_ref(),
        )
        .map(Arc::new)
    }
}

pub(crate) struct SpatialPublication {
    pub(super) snapshot: Arc<SpatialResolvedSnapshotV2>,
    pub(super) logical_nodes: Box<[NodeId]>,
}

impl SpatialPublication {
    pub(crate) fn viewport(&self) -> SpatialViewportV2 {
        self.snapshot.viewport()
    }
}
