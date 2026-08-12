use std::sync::Arc;

use fenestra_ui_ir::prototype::{
    ValidatedConstruction, ValidatedSpatialProgramV2, ValidatedStyleProgram,
};
use fenestra_ui_layout::prototype::{LayoutEngineV1, ReferenceStackEngineV1};
use fenestra_ui_spatial::prototype::{SpatialLimitsV2, SpatialViewportV2};

use super::UiRuntime;
use crate::runtime::capacity::RuntimeCapacity;
use crate::runtime::error::{RuntimeInitializationError, RuntimeInitializationErrorKind};
use crate::runtime::headless::{HeadlessProjectionSpec, HeadlessRuntimeConfig, HeadlessSurface};
use crate::runtime::spatial::{RuntimeSpatialProgramV2, SpatialRuntimeConfig};
use crate::runtime::state::RuntimeState;

impl UiRuntime {
    /// Materializes generation zero from one exact validated construction.
    pub fn new(
        construction: ValidatedConstruction,
        capacity: RuntimeCapacity,
    ) -> Result<Self, RuntimeInitializationError> {
        let state = RuntimeState::initialize(&construction, capacity)?;
        Ok(Self {
            construction,
            capacity,
            headless: None,
            spatial: None,
            state: Arc::new(state),
            retired: Vec::new(),
        })
    }

    /// Materializes generation zero with a provisional headless projection.
    pub fn new_headless(
        style: ValidatedStyleProgram,
        spec: HeadlessProjectionSpec,
        surface: HeadlessSurface,
        capacity: RuntimeCapacity,
    ) -> Result<Self, RuntimeInitializationError> {
        Self::new_headless_with_layout_engine(
            style,
            spec,
            surface,
            capacity,
            Box::new(ReferenceStackEngineV1::new()),
        )
    }

    /// Materializes generation zero with an injected provisional layout engine.
    #[doc(hidden)]
    pub fn new_headless_with_layout_engine(
        style: ValidatedStyleProgram,
        spec: HeadlessProjectionSpec,
        surface: HeadlessSurface,
        capacity: RuntimeCapacity,
        layout_engine: Box<dyn LayoutEngineV1>,
    ) -> Result<Self, RuntimeInitializationError> {
        let construction = style.construction().clone();
        let headless =
            HeadlessRuntimeConfig::new(style, spec, surface, layout_engine).map_err(|kind| {
                RuntimeInitializationError::new(RuntimeInitializationErrorKind::Headless(kind))
            })?;
        let state = RuntimeState::initialize_headless(&construction, capacity, &headless, surface)?;
        Ok(Self {
            construction,
            capacity,
            headless: Some(headless),
            spatial: None,
            state: Arc::new(state),
            retired: Vec::new(),
        })
    }

    /// Materializes generation zero with a spatial publication.
    pub fn new_spatial(
        style: ValidatedStyleProgram,
        program: Box<dyn RuntimeSpatialProgramV2>,
        viewport: SpatialViewportV2,
        limits: SpatialLimitsV2,
        capacity: RuntimeCapacity,
    ) -> Result<Self, RuntimeInitializationError> {
        Self::new_spatial_with_layout_engine(
            style,
            program,
            viewport,
            limits,
            capacity,
            Box::new(ReferenceStackEngineV1::new()),
        )
    }

    /// Materializes generation zero with an injected spatial layout engine.
    #[doc(hidden)]
    pub fn new_spatial_with_layout_engine(
        style: ValidatedStyleProgram,
        program: Box<dyn RuntimeSpatialProgramV2>,
        viewport: SpatialViewportV2,
        limits: SpatialLimitsV2,
        capacity: RuntimeCapacity,
        layout_engine: Box<dyn LayoutEngineV1>,
    ) -> Result<Self, RuntimeInitializationError> {
        let construction = style.construction().clone();
        let mut state = RuntimeState::initialize_styled(&construction, capacity, &style)?;
        let spatial = SpatialRuntimeConfig::new(style, program, limits, layout_engine);
        let publication = spatial.build(&state, viewport).map_err(|kind| {
            RuntimeInitializationError::new(RuntimeInitializationErrorKind::Spatial(kind))
        })?;
        state.spatial = Some(publication);
        Ok(Self {
            construction,
            capacity,
            headless: None,
            spatial: Some(spatial),
            state: Arc::new(state),
            retired: Vec::new(),
        })
    }

    /// Materializes generation zero from a validated symbolic spatial program.
    pub fn new_spatial_ir(
        program: ValidatedSpatialProgramV2,
        viewport: SpatialViewportV2,
        limits: SpatialLimitsV2,
        capacity: RuntimeCapacity,
    ) -> Result<Self, RuntimeInitializationError> {
        Self::new_spatial_ir_with_layout_engine(
            program,
            viewport,
            limits,
            capacity,
            Box::new(ReferenceStackEngineV1::new()),
        )
    }

    /// Materializes generation zero from symbolic spatial input with an injected layout engine.
    #[doc(hidden)]
    pub fn new_spatial_ir_with_layout_engine(
        program: ValidatedSpatialProgramV2,
        viewport: SpatialViewportV2,
        limits: SpatialLimitsV2,
        capacity: RuntimeCapacity,
        layout_engine: Box<dyn LayoutEngineV1>,
    ) -> Result<Self, RuntimeInitializationError> {
        let style = program.style().clone();
        let construction = style.construction().clone();
        let mut state = RuntimeState::initialize_styled(&construction, capacity, &style)?;
        let spatial = SpatialRuntimeConfig::new_ir(program, limits, layout_engine);
        let publication = spatial.build(&state, viewport).map_err(|kind| {
            RuntimeInitializationError::new(RuntimeInitializationErrorKind::Spatial(kind))
        })?;
        state.spatial = Some(publication);
        Ok(Self {
            construction,
            capacity,
            headless: None,
            spatial: Some(spatial),
            state: Arc::new(state),
            retired: Vec::new(),
        })
    }
}
