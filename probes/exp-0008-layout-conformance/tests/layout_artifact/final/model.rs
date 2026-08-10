mod faults;

use fenestra_ui_layout::prototype::{LayoutLimitKindV1, REGISTERED_LAYOUT_LIMITS_V1};

use crate::direct::DirectArtifactV1;
use crate::encode::{LayoutArtifactLimitKindV1, REGISTERED_LAYOUT_ARTIFACT_LIMITS_V1};
use crate::runtime::RuntimeArtifactV1;

pub(super) use self::faults::{
    fault_row_v1, inject_metadata_fault_v1, registered_metadata_faults_v1,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct FinalArtifactV1 {
    pub(super) metadata: FinalMetadataV1,
    pub(super) direct: DirectArtifactV1,
    pub(super) runtime: RuntimeArtifactV1,
    pub(super) classification: FinalClassificationV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct FinalMetadataV1 {
    pub(super) versions: FormatVersionsV1,
    pub(super) packages: PackageVersionsV1,
    pub(super) candidate: CandidateMetadataV1,
    pub(super) limits: RegisteredLimitsV1,
    pub(super) dependencies: DependencyFactsV1,
    pub(super) counts: ArtifactCountsV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct FormatVersionsV1 {
    pub(super) artifact: u16,
    pub(super) layout_contract: u16,
    pub(super) layout_corpus: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct SemanticVersionV1 {
    pub(super) major: u16,
    pub(super) minor: u16,
    pub(super) patch: u16,
}

impl SemanticVersionV1 {
    pub(super) const fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    pub(super) fn bump_patch(&mut self) {
        self.patch += 1;
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct PackageVersionsV1 {
    pub(super) probe: SemanticVersionV1,
    pub(super) layout: SemanticVersionV1,
    pub(super) ir: SemanticVersionV1,
    pub(super) runtime: SemanticVersionV1,
    pub(super) testkit: SemanticVersionV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct CandidateMetadataV1 {
    pub(super) version: SemanticVersionV1,
    pub(super) features: CandidateFeaturesV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct CandidateFeaturesV1 {
    pub(super) default: bool,
    pub(super) std: bool,
    pub(super) taffy_tree: bool,
    pub(super) flexbox: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct RegisteredLimitsV1 {
    pub(super) nodes: usize,
    pub(super) depth: usize,
    pub(super) children_per_node: usize,
    pub(super) candidate_input_scalar: usize,
    pub(super) candidate_output_edge: usize,
    pub(super) artifact_records: usize,
    pub(super) artifact_line_bytes: usize,
    pub(super) artifact_bytes: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct DependencyFactsV1 {
    pub(super) scope: CandidateScopeV1,
    pub(super) requirement: CandidateRequirementV1,
    pub(super) arrayvec: SemanticVersionV1,
    pub(super) slotmap: SemanticVersionV1,
    pub(super) version_check: SemanticVersionV1,
    pub(super) taffy_serde_lock_edge: bool,
    pub(super) serde_derive_lock_edge: bool,
    pub(super) runtime_dev_ir: bool,
    pub(super) runtime_dev_runtime: bool,
    pub(super) runtime_dev_testkit: bool,
    pub(super) other_taffy_manifests: usize,
    pub(super) native: bool,
    pub(super) ffi: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CandidateScopeV1 {
    ProbeNormalPrivate,
    WorkspaceNormal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CandidateRequirementV1 {
    Exact,
    Compatible,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ArtifactCountsV1 {
    pub(super) metadata: usize,
    pub(super) direct: usize,
    pub(super) runtime: usize,
    pub(super) closing: usize,
    pub(super) records: usize,
    pub(super) cases: usize,
    pub(super) inputs: usize,
    pub(super) outputs: usize,
    pub(super) steps: usize,
    pub(super) generations: usize,
    pub(super) geometry: usize,
    pub(super) hit: usize,
    pub(super) scene: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum FinalClassificationV1 {
    Pass,
    Adapt,
    Stop,
}

impl FinalClassificationV1 {
    pub(super) const ALL: [Self; 3] = [Self::Pass, Self::Adapt, Self::Stop];
}

pub(super) fn registered_final_artifact_v1(
    direct: DirectArtifactV1,
    runtime: RuntimeArtifactV1,
) -> FinalArtifactV1 {
    let package = SemanticVersionV1::new(0, 1, 1);
    FinalArtifactV1 {
        metadata: FinalMetadataV1 {
            versions: FormatVersionsV1 {
                artifact: 1,
                layout_contract: 1,
                layout_corpus: 1,
            },
            packages: PackageVersionsV1 {
                probe: package,
                layout: package,
                ir: package,
                runtime: package,
                testkit: package,
            },
            candidate: CandidateMetadataV1 {
                version: SemanticVersionV1::new(0, 13, 0),
                features: CandidateFeaturesV1 {
                    default: false,
                    std: true,
                    taffy_tree: true,
                    flexbox: true,
                },
            },
            limits: registered_limits(),
            dependencies: DependencyFactsV1 {
                scope: CandidateScopeV1::ProbeNormalPrivate,
                requirement: CandidateRequirementV1::Exact,
                arrayvec: SemanticVersionV1::new(0, 7, 8),
                slotmap: SemanticVersionV1::new(1, 1, 1),
                version_check: SemanticVersionV1::new(0, 9, 5),
                taffy_serde_lock_edge: true,
                serde_derive_lock_edge: true,
                runtime_dev_ir: true,
                runtime_dev_runtime: true,
                runtime_dev_testkit: true,
                other_taffy_manifests: 0,
                native: false,
                ffi: false,
            },
            counts: ArtifactCountsV1 {
                metadata: 15,
                direct: 286,
                runtime: 109,
                closing: 2,
                records: 412,
                cases: 23,
                inputs: 120,
                outputs: 120,
                steps: 7,
                generations: 7,
                geometry: 38,
                hit: 24,
                scene: 38,
            },
        },
        direct,
        runtime,
        classification: FinalClassificationV1::Pass,
    }
}

fn registered_limits() -> RegisteredLimitsV1 {
    RegisteredLimitsV1 {
        nodes: REGISTERED_LAYOUT_LIMITS_V1.limit(LayoutLimitKindV1::Nodes),
        depth: REGISTERED_LAYOUT_LIMITS_V1.limit(LayoutLimitKindV1::Depth),
        children_per_node: REGISTERED_LAYOUT_LIMITS_V1.limit(LayoutLimitKindV1::ChildrenPerNode),
        candidate_input_scalar: 4096,
        candidate_output_edge: 524_288,
        artifact_records: REGISTERED_LAYOUT_ARTIFACT_LIMITS_V1
            .limit(LayoutArtifactLimitKindV1::Records),
        artifact_line_bytes: REGISTERED_LAYOUT_ARTIFACT_LIMITS_V1
            .limit(LayoutArtifactLimitKindV1::LineBytes),
        artifact_bytes: REGISTERED_LAYOUT_ARTIFACT_LIMITS_V1
            .limit(LayoutArtifactLimitKindV1::ArtifactBytes),
    }
}
