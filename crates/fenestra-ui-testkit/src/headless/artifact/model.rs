use std::fmt;

use fenestra_ui_runtime::prototype::HeadlessSurface;

use super::record::{ProjectionRecordV1, SchedulerEventRecordV1, TraceEventRecordV1};
use crate::headless::runner::{HeadlessResultV1, HeadlessRunV1};
use crate::headless::trace::{HeadlessTraceEventV1, HeadlessTraceProjectionCountsV1};
use crate::scheduler::SchedulerTraceEventV1;

/// Owned physical-identity-free evidence for the fixed headless V1 run.
#[derive(Clone, Eq, PartialEq)]
pub struct HeadlessArtifactV1 {
    pub(super) metadata: ArtifactMetadataV1,
    pub(super) capacities: ArtifactCapacitiesV1,
    pub(super) headless_events: Vec<TraceEventRecordV1>,
    pub(super) scheduler_events: Vec<SchedulerEventRecordV1>,
    pub(super) final_generation: u64,
    pub(super) projection: ProjectionRecordV1,
    pub(super) result: HeadlessResultV1,
}

impl HeadlessArtifactV1 {
    /// Returns the closed synthetic result.
    #[must_use]
    pub const fn result(&self) -> HeadlessResultV1 {
        self.result
    }

    /// Returns the final numeric committed generation.
    #[must_use]
    pub const fn final_generation(&self) -> u64 {
        self.final_generation
    }

    /// Returns the final logical surface.
    #[must_use]
    pub const fn final_surface(&self) -> HeadlessSurface {
        self.projection.surface
    }

    /// Returns counts for the five final projection families.
    #[must_use]
    pub const fn final_projection_counts(&self) -> HeadlessTraceProjectionCountsV1 {
        self.projection.counts
    }

    /// Returns the complete headless event count.
    #[must_use]
    pub const fn headless_event_count(&self) -> usize {
        self.headless_events.len()
    }

    /// Returns the complete scheduler event count.
    #[must_use]
    pub const fn scheduler_event_count(&self) -> usize {
        self.scheduler_events.len()
    }
}

impl fmt::Debug for HeadlessArtifactV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HeadlessArtifactV1")
            .field("headless_event_count", &self.headless_events.len())
            .field("scheduler_event_count", &self.scheduler_events.len())
            .field("computed_style_count", &self.projection.computed.len())
            .field("geometry_count", &self.projection.geometry.len())
            .field("semantic_count", &self.projection.semantics.len())
            .field("hit_region_count", &self.projection.hits.len())
            .field("scene_rectangle_count", &self.projection.scene.len())
            .finish()
    }
}

/// Projects one trusted fixed runner result into an owned wire model.
#[must_use]
pub fn build_headless_artifact_v1(run: &HeadlessRunV1) -> HeadlessArtifactV1 {
    HeadlessArtifactV1 {
        metadata: ArtifactMetadataV1::REGISTERED,
        capacities: ArtifactCapacitiesV1::from_run(run),
        headless_events: run
            .headless_trace()
            .events()
            .iter()
            .copied()
            .map(TraceEventRecordV1::from_event)
            .collect(),
        scheduler_events: run
            .scheduler_trace()
            .events()
            .iter()
            .copied()
            .map(SchedulerEventRecordV1::from_event)
            .collect(),
        final_generation: run.final_generation().get(),
        projection: ProjectionRecordV1::from_projection(run.final_projection()),
        result: run.result(),
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) struct ArtifactMetadataV1 {
    pub(super) fixture_revision: u32,
    pub(super) schema_format: u32,
    pub(super) schema_namespace: u64,
    pub(super) schema_revision: u32,
    pub(super) construction_format: u32,
    pub(super) style_format: u32,
}

impl ArtifactMetadataV1 {
    pub(in crate::headless::artifact) const REGISTERED: Self = Self {
        fixture_revision: 1,
        schema_format: 1,
        schema_namespace: 8_001,
        schema_revision: 1,
        construction_format: 1,
        style_format: 1,
    };
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) struct ArtifactCapacitiesV1 {
    pub(super) ir: [usize; 9],
    pub(super) style: usize,
    pub(super) runtime: [usize; 6],
    pub(super) projection: [usize; 5],
    pub(super) scheduler: [usize; 12],
    pub(super) renderer: [usize; 3],
    pub(super) scheduler_trace: [usize; 3],
    pub(super) headless_trace: [usize; 3],
    pub(super) artifact: [usize; 3],
}

impl ArtifactCapacitiesV1 {
    pub(in crate::headless::artifact) const REGISTERED: Self = Self {
        ir: [1, 5, 4, 1, 3, 12, 2, 3, 5],
        style: 2,
        runtime: [8, 8, 8, 2, 40, 3],
        projection: [8, 8, 1, 8, 8],
        scheduler: [1, 80, 8, 4, 128, 8, 1, 40, 8, 2, 80, 8],
        renderer: [2, 192, 8],
        scheduler_trace: [256, 24_576, SchedulerTraceEventV1::ACCOUNTED_BYTES],
        headless_trace: [128, 20_480, HeadlessTraceEventV1::ACCOUNTED_BYTES],
        artifact: [65_536, 1_024, 512],
    };

    fn from_run(run: &HeadlessRunV1) -> Self {
        let scheduler_trace = run.scheduler_trace().capacity();
        let headless_trace = run.headless_trace().capacity();
        Self {
            ir: Self::REGISTERED.ir,
            style: Self::REGISTERED.style,
            runtime: Self::REGISTERED.runtime,
            projection: Self::REGISTERED.projection,
            scheduler: Self::REGISTERED.scheduler,
            renderer: Self::REGISTERED.renderer,
            scheduler_trace: [
                scheduler_trace.max_events(),
                scheduler_trace.max_bytes(),
                SchedulerTraceEventV1::ACCOUNTED_BYTES,
            ],
            headless_trace: [
                headless_trace.max_events(),
                headless_trace.max_bytes(),
                HeadlessTraceEventV1::ACCOUNTED_BYTES,
            ],
            artifact: Self::REGISTERED.artifact,
        }
    }
}
