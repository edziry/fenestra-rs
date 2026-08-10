mod compare;
mod fault;
mod observe;
mod rebuild;
mod types;

use fenestra_ui_ir::prototype::ValidatedStyleProgram;
use fenestra_ui_runtime::prototype::{HeadlessProjectionSpec, HeadlessSurface};

use crate::case::SemanticOperationV1;
use crate::desired::DesiredStateV1;
use crate::error::HarnessError;
use crate::fixture::HarnessLimitsV1;

use super::fixture::HeadlessFixtureV1;

pub use compare::compare_headless_projection_v1;
pub use fault::{inject_headless_projection_fault_v1, inject_headless_surface_fault_v1};
pub use observe::observe_headless_projection_v1;
pub use types::{
    HeadlessMismatchFieldV1, HeadlessMismatchKindV1, HeadlessMismatchLocationV1,
    HeadlessMismatchV1, HeadlessProjectionFaultV1, NormalizedHeadlessComputedStyleV1,
    NormalizedHeadlessGeometryV1, NormalizedHeadlessHitRegionV1, NormalizedHeadlessProjectionV1,
    NormalizedHeadlessSceneRectangleV1, NormalizedHeadlessSemanticV1, ObservedHeadlessProjectionV1,
};

pub(super) const ORACLE_LIMITS: HarnessLimitsV1 = HarnessLimitsV1 {
    transactions: 16,
    operations_per_transaction: 8,
    operations: 128,
    live_memberships: 5,
    path_depth: 3,
    normalized_nodes: 8,
    normalized_fragments: 2,
    normalized_properties: 40,
    applicable_actions: 64,
    trace_bytes: 20_480,
};

#[derive(Clone, Copy)]
pub(super) struct RebuildInput<'a> {
    style: &'a ValidatedStyleProgram,
    spec: HeadlessProjectionSpec,
    surface: HeadlessSurface,
    desired: &'a DesiredStateV1,
}

/// Mutable desired state feeding the independent headless clean rebuild.
pub struct HeadlessOracleV1 {
    style: ValidatedStyleProgram,
    spec: HeadlessProjectionSpec,
    surface: HeadlessSurface,
    desired: DesiredStateV1,
}

impl HeadlessOracleV1 {
    /// Creates the initial desired state without observing a runtime snapshot.
    pub fn new(fixture: &HeadlessFixtureV1) -> Result<Self, HarnessError> {
        let desired =
            DesiredStateV1::from_construction(fixture.style().construction(), ORACLE_LIMITS)?;
        let oracle = Self {
            style: fixture.style().clone(),
            spec: fixture.spec(),
            surface: fixture.surface(),
            desired,
        };
        oracle.rebuild()?;
        Ok(oracle)
    }

    /// Applies one semantic operation atomically to the desired state.
    pub fn apply_operation(&mut self, operation: &SemanticOperationV1) -> Result<(), HarnessError> {
        let mut draft = self.desired.clone();
        draft.apply_operation(operation, ORACLE_LIMITS)?;
        rebuild::rebuild(
            RebuildInput {
                style: &self.style,
                spec: self.spec,
                surface: self.surface,
                desired: &draft,
            },
            ORACLE_LIMITS,
        )?;
        self.desired = draft;
        Ok(())
    }

    /// Replaces the desired logical surface after a successful clean rebuild.
    pub fn resize(&mut self, surface: HeadlessSurface) -> Result<(), HarnessError> {
        rebuild::rebuild(
            RebuildInput {
                style: &self.style,
                spec: self.spec,
                surface,
                desired: &self.desired,
            },
            ORACLE_LIMITS,
        )?;
        self.surface = surface;
        Ok(())
    }

    /// Independently rebuilds the complete normalized projection.
    pub fn rebuild(&self) -> Result<NormalizedHeadlessProjectionV1, HarnessError> {
        rebuild::rebuild(
            RebuildInput {
                style: &self.style,
                spec: self.spec,
                surface: self.surface,
                desired: &self.desired,
            },
            ORACLE_LIMITS,
        )
    }
}
