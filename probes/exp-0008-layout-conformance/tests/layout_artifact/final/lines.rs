use crate::direct::direct_lines_v1;
use crate::runtime::runtime_lines_v1;

use super::model::{
    CandidateRequirementV1, CandidateScopeV1, DependencyFactsV1, FinalArtifactV1,
    FinalClassificationV1, SemanticVersionV1,
};

pub(super) fn final_artifact_lines_v1(model: &FinalArtifactV1) -> Vec<String> {
    let mut lines = metadata_lines_v1(model);
    lines.extend(direct_lines_v1(&model.direct));
    lines.extend(runtime_lines_v1(&model.runtime));
    lines.push(format!(
        "artifact-result|classification={}",
        classification_label(model.classification)
    ));
    lines.push("end".to_owned());
    lines
}

fn metadata_lines_v1(model: &FinalArtifactV1) -> Vec<String> {
    let metadata = &model.metadata;
    let versions = metadata.versions;
    let packages = metadata.packages;
    let candidate = metadata.candidate;
    let limits = metadata.limits;
    let counts = metadata.counts;
    vec![
        format!("fenestra-layout-conformance|{}", versions.artifact),
        "work-unit|WU-0011|experiment=EXP-0008".to_owned(),
        format!(
            "formats|layout-contract={}|layout-corpus={}",
            versions.layout_contract, versions.layout_corpus
        ),
        package_line("fenestra-ui-exp-0008-layout-conformance", packages.probe),
        package_line("fenestra-ui-layout", packages.layout),
        package_line("fenestra-ui-ir", packages.ir),
        package_line("fenestra-ui-runtime", packages.runtime),
        package_line("fenestra-ui-testkit", packages.testkit),
        format!(
            "candidate|name=taffy|version={}|role=disposable-probe|product-selected=false",
            version(candidate.version)
        ),
        format!(
            "layout-limits|nodes={}|depth={}|children-per-node={}",
            limits.nodes, limits.depth, limits.children_per_node
        ),
        format!(
            "candidate-limits|input-scalar={}|output-edge={}",
            limits.candidate_input_scalar, limits.candidate_output_edge
        ),
        format!(
            "artifact-limits|records={}|line-bytes={}|artifact-bytes={}|priority=records,line-bytes,artifact-bytes",
            limits.artifact_records, limits.artifact_line_bytes, limits.artifact_bytes
        ),
        format!(
            "candidate-features|default={}|std={}|taffy_tree={}|flexbox={}",
            boolean(candidate.features.default),
            boolean(candidate.features.std),
            boolean(candidate.features.taffy_tree),
            boolean(candidate.features.flexbox)
        ),
        dependency_line(metadata.dependencies),
        format!(
            "artifact-counts|metadata={}|direct={}|runtime={}|closing={}|records={}|cases={}|inputs={}|outputs={}|steps={}|generations={}|geometry={}|hit={}|scene={}",
            counts.metadata,
            counts.direct,
            counts.runtime,
            counts.closing,
            counts.records,
            counts.cases,
            counts.inputs,
            counts.outputs,
            counts.steps,
            counts.generations,
            counts.geometry,
            counts.hit,
            counts.scene
        ),
    ]
}

fn package_line(name: &str, package: SemanticVersionV1) -> String {
    format!(
        "package|name={name}|version={}|edition=2024|publish=false",
        version(package)
    )
}

fn dependency_line(facts: DependencyFactsV1) -> String {
    format!(
        "dependency-facts|candidate-scope={}|candidate-requirement={}|active-transitives=arrayvec@{},slotmap@{},version_check@{}|lock-only={}|runtime-acceptance={}|runtime-acceptance-scope=dev-only|other-taffy-manifests={}|native={}|ffi={}",
        scope_label(facts.scope),
        requirement_label(facts.requirement),
        version(facts.arrayvec),
        version(facts.slotmap),
        version(facts.version_check),
        lock_only(facts),
        runtime_acceptance(facts),
        facts.other_taffy_manifests,
        boolean(facts.native),
        boolean(facts.ffi)
    )
}

fn lock_only(facts: DependencyFactsV1) -> String {
    enabled_values([
        (facts.taffy_serde_lock_edge, "taffy>serde"),
        (facts.serde_derive_lock_edge, "serde>serde_derive"),
    ])
}

fn runtime_acceptance(facts: DependencyFactsV1) -> String {
    enabled_values([
        (facts.runtime_dev_ir, "fenestra-ui-ir"),
        (facts.runtime_dev_runtime, "fenestra-ui-runtime"),
        (facts.runtime_dev_testkit, "fenestra-ui-testkit"),
    ])
}

fn enabled_values<const N: usize>(values: [(bool, &'static str); N]) -> String {
    let mut rendered = String::new();
    for (enabled, value) in values {
        if enabled {
            if !rendered.is_empty() {
                rendered.push(',');
            }
            rendered.push_str(value);
        }
    }
    if rendered.is_empty() {
        rendered.push_str("none");
    }
    rendered
}

fn version(value: SemanticVersionV1) -> String {
    format!("{}.{}.{}", value.major, value.minor, value.patch)
}

const fn scope_label(scope: CandidateScopeV1) -> &'static str {
    match scope {
        CandidateScopeV1::ProbeNormalPrivate => "probe-normal-private",
        CandidateScopeV1::WorkspaceNormal => "workspace-normal",
    }
}

const fn requirement_label(requirement: CandidateRequirementV1) -> &'static str {
    match requirement {
        CandidateRequirementV1::Exact => "exact",
        CandidateRequirementV1::Compatible => "compatible",
    }
}

const fn boolean(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}

const fn classification_label(classification: FinalClassificationV1) -> &'static str {
    match classification {
        FinalClassificationV1::Pass => "pass",
        FinalClassificationV1::Adapt => "adapt",
        FinalClassificationV1::Stop => "stop",
    }
}
