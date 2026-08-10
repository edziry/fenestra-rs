use super::{
    CandidateFeaturesV1, CandidateRequirementV1, CandidateScopeV1, FinalArtifactV1,
    FinalClassificationV1, FinalMetadataV1, RegisteredLimitsV1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FinalVersionFieldV1 {
    ArtifactFormat,
    LayoutContract,
    LayoutCorpus,
    ProbePackage,
    LayoutPackage,
    IrPackage,
    RuntimePackage,
    TestkitPackage,
    Candidate,
}

impl FinalVersionFieldV1 {
    const ALL: [Self; 9] = [
        Self::ArtifactFormat,
        Self::LayoutContract,
        Self::LayoutCorpus,
        Self::ProbePackage,
        Self::LayoutPackage,
        Self::IrPackage,
        Self::RuntimePackage,
        Self::TestkitPackage,
        Self::Candidate,
    ];
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FinalLimitFieldV1 {
    Nodes,
    Depth,
    ChildrenPerNode,
    CandidateInputScalar,
    CandidateOutputEdge,
    ArtifactRecords,
    ArtifactLineBytes,
    ArtifactBytes,
}

impl FinalLimitFieldV1 {
    const ALL: [Self; 8] = [
        Self::Nodes,
        Self::Depth,
        Self::ChildrenPerNode,
        Self::CandidateInputScalar,
        Self::CandidateOutputEdge,
        Self::ArtifactRecords,
        Self::ArtifactLineBytes,
        Self::ArtifactBytes,
    ];
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CandidateFeatureV1 {
    Default,
    Std,
    TaffyTree,
    Flexbox,
}

impl CandidateFeatureV1 {
    const ALL: [Self; 4] = [Self::Default, Self::Std, Self::TaffyTree, Self::Flexbox];
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DependencyFieldV1 {
    Scope,
    ExactRequirement,
    Arrayvec,
    Slotmap,
    VersionCheck,
    TaffySerdeLockEdge,
    SerdeDeriveLockEdge,
    RuntimeDevIr,
    RuntimeDevRuntime,
    RuntimeDevTestkit,
    OtherManifestLeak,
    Native,
    Ffi,
}

impl DependencyFieldV1 {
    const ALL: [Self; 13] = [
        Self::Scope,
        Self::ExactRequirement,
        Self::Arrayvec,
        Self::Slotmap,
        Self::VersionCheck,
        Self::TaffySerdeLockEdge,
        Self::SerdeDeriveLockEdge,
        Self::RuntimeDevIr,
        Self::RuntimeDevRuntime,
        Self::RuntimeDevTestkit,
        Self::OtherManifestLeak,
        Self::Native,
        Self::Ffi,
    ];
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FinalMetadataFaultV1 {
    Version(FinalVersionFieldV1),
    Limit(FinalLimitFieldV1),
    CandidateFeature(CandidateFeatureV1),
    Dependency(DependencyFieldV1),
    Classification,
}

pub(crate) fn registered_metadata_faults_v1() -> Vec<FinalMetadataFaultV1> {
    let mut faults = Vec::with_capacity(35);
    faults.extend(FinalVersionFieldV1::ALL.map(FinalMetadataFaultV1::Version));
    faults.extend(FinalLimitFieldV1::ALL.map(FinalMetadataFaultV1::Limit));
    faults.extend(CandidateFeatureV1::ALL.map(FinalMetadataFaultV1::CandidateFeature));
    faults.extend(DependencyFieldV1::ALL.map(FinalMetadataFaultV1::Dependency));
    faults.push(FinalMetadataFaultV1::Classification);
    faults
}

pub(crate) fn inject_metadata_fault_v1(
    model: &FinalArtifactV1,
    fault: FinalMetadataFaultV1,
) -> FinalArtifactV1 {
    let mut faulted = model.clone();
    match fault {
        FinalMetadataFaultV1::Version(field) => mutate_version(&mut faulted.metadata, field),
        FinalMetadataFaultV1::Limit(field) => mutate_limit(&mut faulted.metadata.limits, field),
        FinalMetadataFaultV1::CandidateFeature(feature) => {
            mutate_feature(&mut faulted.metadata.candidate.features, feature);
        }
        FinalMetadataFaultV1::Dependency(field) => mutate_dependency(&mut faulted, field),
        FinalMetadataFaultV1::Classification => {
            faulted.classification = FinalClassificationV1::Adapt;
        }
    }
    faulted
}

fn mutate_version(metadata: &mut FinalMetadataV1, field: FinalVersionFieldV1) {
    match field {
        FinalVersionFieldV1::ArtifactFormat => metadata.versions.artifact += 1,
        FinalVersionFieldV1::LayoutContract => metadata.versions.layout_contract += 1,
        FinalVersionFieldV1::LayoutCorpus => metadata.versions.layout_corpus += 1,
        FinalVersionFieldV1::ProbePackage => metadata.packages.probe.bump_patch(),
        FinalVersionFieldV1::LayoutPackage => metadata.packages.layout.bump_patch(),
        FinalVersionFieldV1::IrPackage => metadata.packages.ir.bump_patch(),
        FinalVersionFieldV1::RuntimePackage => metadata.packages.runtime.bump_patch(),
        FinalVersionFieldV1::TestkitPackage => metadata.packages.testkit.bump_patch(),
        FinalVersionFieldV1::Candidate => metadata.candidate.version.bump_patch(),
    }
}

fn mutate_limit(limits: &mut RegisteredLimitsV1, field: FinalLimitFieldV1) {
    match field {
        FinalLimitFieldV1::Nodes => limits.nodes += 1,
        FinalLimitFieldV1::Depth => limits.depth += 1,
        FinalLimitFieldV1::ChildrenPerNode => limits.children_per_node += 1,
        FinalLimitFieldV1::CandidateInputScalar => limits.candidate_input_scalar += 1,
        FinalLimitFieldV1::CandidateOutputEdge => limits.candidate_output_edge += 1,
        FinalLimitFieldV1::ArtifactRecords => limits.artifact_records += 1,
        FinalLimitFieldV1::ArtifactLineBytes => limits.artifact_line_bytes += 1,
        FinalLimitFieldV1::ArtifactBytes => limits.artifact_bytes += 1,
    }
}

fn mutate_feature(features: &mut CandidateFeaturesV1, feature: CandidateFeatureV1) {
    match feature {
        CandidateFeatureV1::Default => features.default = !features.default,
        CandidateFeatureV1::Std => features.std = !features.std,
        CandidateFeatureV1::TaffyTree => features.taffy_tree = !features.taffy_tree,
        CandidateFeatureV1::Flexbox => features.flexbox = !features.flexbox,
    }
}

fn mutate_dependency(model: &mut FinalArtifactV1, field: DependencyFieldV1) {
    let dependencies = &mut model.metadata.dependencies;
    match field {
        DependencyFieldV1::Scope => dependencies.scope = CandidateScopeV1::WorkspaceNormal,
        DependencyFieldV1::ExactRequirement => {
            dependencies.requirement = CandidateRequirementV1::Compatible;
        }
        DependencyFieldV1::Arrayvec => dependencies.arrayvec.bump_patch(),
        DependencyFieldV1::Slotmap => dependencies.slotmap.bump_patch(),
        DependencyFieldV1::VersionCheck => dependencies.version_check.bump_patch(),
        DependencyFieldV1::TaffySerdeLockEdge => {
            dependencies.taffy_serde_lock_edge = !dependencies.taffy_serde_lock_edge;
        }
        DependencyFieldV1::SerdeDeriveLockEdge => {
            dependencies.serde_derive_lock_edge = !dependencies.serde_derive_lock_edge;
        }
        DependencyFieldV1::RuntimeDevIr => {
            dependencies.runtime_dev_ir = !dependencies.runtime_dev_ir;
        }
        DependencyFieldV1::RuntimeDevRuntime => {
            dependencies.runtime_dev_runtime = !dependencies.runtime_dev_runtime;
        }
        DependencyFieldV1::RuntimeDevTestkit => {
            dependencies.runtime_dev_testkit = !dependencies.runtime_dev_testkit;
        }
        DependencyFieldV1::OtherManifestLeak => dependencies.other_taffy_manifests += 1,
        DependencyFieldV1::Native => dependencies.native = !dependencies.native,
        DependencyFieldV1::Ffi => dependencies.ffi = !dependencies.ffi,
    }
}

pub(crate) const fn fault_row_v1(fault: FinalMetadataFaultV1) -> usize {
    match fault {
        FinalMetadataFaultV1::Version(field) => match field {
            FinalVersionFieldV1::ArtifactFormat => 0,
            FinalVersionFieldV1::LayoutContract | FinalVersionFieldV1::LayoutCorpus => 2,
            FinalVersionFieldV1::ProbePackage => 3,
            FinalVersionFieldV1::LayoutPackage => 4,
            FinalVersionFieldV1::IrPackage => 5,
            FinalVersionFieldV1::RuntimePackage => 6,
            FinalVersionFieldV1::TestkitPackage => 7,
            FinalVersionFieldV1::Candidate => 8,
        },
        FinalMetadataFaultV1::Limit(field) => match field {
            FinalLimitFieldV1::Nodes
            | FinalLimitFieldV1::Depth
            | FinalLimitFieldV1::ChildrenPerNode => 9,
            FinalLimitFieldV1::CandidateInputScalar | FinalLimitFieldV1::CandidateOutputEdge => 10,
            FinalLimitFieldV1::ArtifactRecords
            | FinalLimitFieldV1::ArtifactLineBytes
            | FinalLimitFieldV1::ArtifactBytes => 11,
        },
        FinalMetadataFaultV1::CandidateFeature(_) => 12,
        FinalMetadataFaultV1::Dependency(_) => 13,
        FinalMetadataFaultV1::Classification => 410,
    }
}
