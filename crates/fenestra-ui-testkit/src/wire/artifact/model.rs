use std::fmt;

use fenestra_ui_runtime::prototype::RuntimeCapacity;

use crate::case::{GeneratedCaseV1, GeneratorConfigV1, SeedV1};
use crate::failure::ReplayFailureV1;
use crate::fixture::ReplayConfigV1;
use crate::reducer::ReductionCompletionV1;
use crate::trace::{TraceEventV1, TraceFaultV1};

/// Declared fixture metadata retained by one decoded V1 failure artifact.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArtifactFixtureMetadataV1 {
    fixture_revision: u32,
    schema_format: u32,
    schema_namespace: u64,
    schema_revision: u32,
    construction_format: u32,
}

impl ArtifactFixtureMetadataV1 {
    pub(super) const fn new(
        fixture_revision: u32,
        schema_format: u32,
        schema_namespace: u64,
        schema_revision: u32,
        construction_format: u32,
    ) -> Self {
        Self {
            fixture_revision,
            schema_format,
            schema_namespace,
            schema_revision,
            construction_format,
        }
    }

    /// Returns the declared runtime-oracle fixture revision.
    #[must_use]
    pub const fn fixture_revision(self) -> u32 {
        self.fixture_revision
    }

    /// Returns the declared IR schema format.
    #[must_use]
    pub const fn schema_format(self) -> u32 {
        self.schema_format
    }

    /// Returns the declared IR schema namespace.
    #[must_use]
    pub const fn schema_namespace(self) -> u64 {
        self.schema_namespace
    }

    /// Returns the declared IR schema revision.
    #[must_use]
    pub const fn schema_revision(self) -> u32 {
        self.schema_revision
    }

    /// Returns the declared IR construction format.
    #[must_use]
    pub const fn construction_format(self) -> u32 {
        self.construction_format
    }
}

/// Runtime capacity fields retained in their platform-independent wire type.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArtifactReplayConfigV1 {
    fields: [u32; 6],
}

impl ArtifactReplayConfigV1 {
    pub(super) const fn new(fields: [u32; 6]) -> Self {
        Self { fields }
    }

    pub(super) const fn fields(self) -> [u32; 6] {
        self.fields
    }

    /// Returns the maximum operations staged in one runtime transaction.
    #[must_use]
    pub const fn operations(self) -> u32 {
        self.fields[0]
    }

    /// Returns the maximum structural changes staged in one transaction.
    #[must_use]
    pub const fn structural_changes(self) -> u32 {
        self.fields[1]
    }

    /// Returns the maximum live runtime nodes.
    #[must_use]
    pub const fn live_nodes(self) -> u32 {
        self.fields[2]
    }

    /// Returns the maximum live runtime fragments.
    #[must_use]
    pub const fn live_fragments(self) -> u32 {
        self.fields[3]
    }

    /// Returns the maximum live runtime property slots.
    #[must_use]
    pub const fn live_property_slots(self) -> u32 {
        self.fields[4]
    }

    /// Returns the maximum retained runtime generations.
    #[must_use]
    pub const fn retained_generations(self) -> u32 {
        self.fields[5]
    }

    fn runtime_capacity(self) -> Option<RuntimeCapacity> {
        let [
            operations,
            structural,
            nodes,
            fragments,
            properties,
            retained,
        ] = self.fields;
        Some(RuntimeCapacity::new(
            usize::try_from(operations).ok()?,
            usize::try_from(structural).ok()?,
            usize::try_from(nodes).ok()?,
            usize::try_from(fragments).ok()?,
            usize::try_from(properties).ok()?,
            usize::try_from(retained).ok()?,
        ))
    }
}

impl PartialEq<ReplayConfigV1> for ArtifactReplayConfigV1 {
    fn eq(&self, other: &ReplayConfigV1) -> bool {
        self.runtime_capacity()
            .is_some_and(|capacity| capacity == other.runtime_capacity())
    }
}

/// Bounded reduction metadata retained by one V1 failure artifact.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArtifactReductionV1 {
    max_evaluations: u32,
    used_evaluations: u32,
    completion: ReductionCompletionV1,
}

impl ArtifactReductionV1 {
    pub(super) const fn new(
        max_evaluations: u32,
        used_evaluations: u32,
        completion: ReductionCompletionV1,
    ) -> Self {
        Self {
            max_evaluations,
            used_evaluations,
            completion,
        }
    }

    /// Returns the configured maximum number of candidate evaluations.
    #[must_use]
    pub const fn max_evaluations(self) -> u32 {
        self.max_evaluations
    }

    /// Returns the number of candidate evaluations consumed.
    #[must_use]
    pub const fn used_evaluations(self) -> u32 {
        self.used_evaluations
    }

    /// Returns how the bounded reduction stopped.
    #[must_use]
    pub const fn completion(self) -> ReductionCompletionV1 {
        self.completion
    }
}

/// Structurally decoded, bounded, physical-identity-free V1 failure artifact.
///
/// Construction is private because decoding alone does not establish semantic
/// provenance or replay reproducibility.
#[derive(Clone, Eq, PartialEq)]
pub struct FailureArtifactV1 {
    fixture: ArtifactFixtureMetadataV1,
    replay_config: ArtifactReplayConfigV1,
    generator_config: GeneratorConfigV1,
    seed: SeedV1,
    original_case: GeneratedCaseV1,
    fault: TraceFaultV1,
    original_failure: ReplayFailureV1,
    reduction: ArtifactReductionV1,
    minimized_case: GeneratedCaseV1,
    minimized_failure: ReplayFailureV1,
    events: Vec<TraceEventV1>,
}

impl fmt::Debug for FailureArtifactV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("FailureArtifactV1")
            .field("fixture_revision", &self.fixture.fixture_revision())
            .field(
                "original_transaction_count",
                &self.original_case.transactions().len(),
            )
            .field(
                "original_operation_count",
                &self.original_case.operation_count(),
            )
            .field(
                "minimized_transaction_count",
                &self.minimized_case.transactions().len(),
            )
            .field(
                "minimized_operation_count",
                &self.minimized_case.operation_count(),
            )
            .field("trace_event_count", &self.events.len())
            .finish()
    }
}

impl FailureArtifactV1 {
    #[allow(clippy::too_many_arguments)]
    pub(super) const fn new(
        fixture: ArtifactFixtureMetadataV1,
        replay_config: ArtifactReplayConfigV1,
        generator_config: GeneratorConfigV1,
        seed: SeedV1,
        original_case: GeneratedCaseV1,
        fault: TraceFaultV1,
        original_failure: ReplayFailureV1,
        reduction: ArtifactReductionV1,
        minimized_case: GeneratedCaseV1,
        minimized_failure: ReplayFailureV1,
        events: Vec<TraceEventV1>,
    ) -> Self {
        Self {
            fixture,
            replay_config,
            generator_config,
            seed,
            original_case,
            fault,
            original_failure,
            reduction,
            minimized_case,
            minimized_failure,
            events,
        }
    }

    /// Returns the retained declared fixture metadata.
    #[must_use]
    pub const fn fixture(&self) -> ArtifactFixtureMetadataV1 {
        self.fixture
    }

    /// Returns the retained runtime replay capacity fields.
    #[must_use]
    pub const fn replay_config(&self) -> ArtifactReplayConfigV1 {
        self.replay_config
    }

    /// Returns the retained deterministic generator configuration.
    #[must_use]
    pub const fn generator_config(&self) -> GeneratorConfigV1 {
        self.generator_config
    }

    /// Returns the retained deterministic generator seed.
    #[must_use]
    pub const fn seed(&self) -> SeedV1 {
        self.seed
    }

    /// Returns the exact decoded original case.
    #[must_use]
    pub const fn original_case(&self) -> &GeneratedCaseV1 {
        &self.original_case
    }

    /// Returns the closed injected-fault provenance.
    #[must_use]
    pub const fn fault(&self) -> TraceFaultV1 {
        self.fault
    }

    /// Returns the first failure retained for the original case.
    #[must_use]
    pub const fn original_failure(&self) -> &ReplayFailureV1 {
        &self.original_failure
    }

    /// Returns the retained bounded-reduction metadata.
    #[must_use]
    pub const fn reduction(&self) -> ArtifactReductionV1 {
        self.reduction
    }

    /// Returns the exact decoded minimized case.
    #[must_use]
    pub const fn minimized_case(&self) -> &GeneratedCaseV1 {
        &self.minimized_case
    }

    /// Returns the first failure retained for the minimized case.
    #[must_use]
    pub const fn minimized_failure(&self) -> &ReplayFailureV1 {
        &self.minimized_failure
    }

    /// Returns the bounded minimized logical trace events.
    #[must_use]
    pub fn events(&self) -> &[TraceEventV1] {
        &self.events
    }
}
