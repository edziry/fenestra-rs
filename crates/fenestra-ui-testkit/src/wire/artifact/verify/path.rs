use fenestra_ui_ir::prototype::{ChildFactory, TemplateFactory, ValidatedConstruction};

use super::{ArtifactVerificationError, ArtifactVerificationErrorKind};
use crate::case::{GeneratedCaseV1, SemanticOperationV1};
use crate::failure::ReplayFailureV1;
use crate::fingerprint::{FingerprintLocationV1, FingerprintSummaryV1};
use crate::semantic::{FragmentPathV1, NodePathV1, NormalizedChildGroupV1, PathSegmentV1};
use crate::wire::artifact::FailureArtifactV1;

pub(super) fn validate_artifact_paths_v1(
    construction: &ValidatedConstruction,
    artifact: &FailureArtifactV1,
) -> Result<(), ArtifactVerificationError> {
    validate_case_paths_v1(construction, artifact.original_case())?;
    validate_failure_paths_v1(construction, artifact.original_failure())?;
    validate_case_paths_v1(construction, artifact.minimized_case())?;
    validate_failure_paths_v1(construction, artifact.minimized_failure())
}

fn validate_case_paths_v1(
    construction: &ValidatedConstruction,
    case: &GeneratedCaseV1,
) -> Result<(), ArtifactVerificationError> {
    for transaction in case.transactions() {
        for operation in transaction.operations() {
            let valid = match operation.operation() {
                SemanticOperationV1::SetProperty { node, .. } => {
                    resolve_node_factory_v1(construction, node).is_some()
                }
                SemanticOperationV1::InsertKeyed { fragment, .. }
                | SemanticOperationV1::MoveKeyed { fragment, .. }
                | SemanticOperationV1::UpdateKeyed { fragment, .. }
                | SemanticOperationV1::RemoveKeyed { fragment, .. } => {
                    resolve_fragment_v1(construction, fragment)
                }
            };
            if !valid {
                return Err(invalid_path().at_operation(transaction.id(), operation.id()));
            }
        }
    }
    Ok(())
}

fn validate_failure_paths_v1(
    construction: &ValidatedConstruction,
    failure: &ReplayFailureV1,
) -> Result<(), ArtifactVerificationError> {
    let fingerprint = failure.fingerprint();
    if !valid_location_v1(construction, fingerprint.location())
        || !valid_summary_v1(construction, fingerprint.expected())
        || !valid_summary_v1(construction, fingerprint.observed())
    {
        return Err(at_failure(invalid_path(), failure));
    }
    Ok(())
}

fn valid_location_v1(
    construction: &ValidatedConstruction,
    location: &FingerprintLocationV1,
) -> bool {
    match location {
        FingerprintLocationV1::Global => true,
        FingerprintLocationV1::Node(path) => resolve_node_factory_v1(construction, path).is_some(),
        FingerprintLocationV1::Fragment(path) => resolve_fragment_v1(construction, path),
    }
}

fn valid_summary_v1(construction: &ValidatedConstruction, summary: &FingerprintSummaryV1) -> bool {
    match summary {
        FingerprintSummaryV1::Node(path) => resolve_node_factory_v1(construction, path).is_some(),
        FingerprintSummaryV1::Nodes(paths) => paths
            .iter()
            .all(|path| resolve_node_factory_v1(construction, path).is_some()),
        FingerprintSummaryV1::Children(groups) => groups.iter().all(|group| match group {
            NormalizedChildGroupV1::Static(path) => {
                resolve_node_factory_v1(construction, path).is_some()
            }
            NormalizedChildGroupV1::Region(path) => resolve_fragment_v1(construction, path),
        }),
        _ => true,
    }
}

fn resolve_node_factory_v1<'a>(
    construction: &'a ValidatedConstruction,
    path: &NodePathV1,
) -> Option<TemplateFactory<'a>> {
    let mut factory = construction.root_factory();
    for segment in path.segments() {
        let child = factory
            .children()
            .nth(usize::from(segment.authored_slot()))?;
        factory = match (segment, child) {
            (PathSegmentV1::Static { .. }, ChildFactory::Static { template, .. }) => template,
            (PathSegmentV1::Member { .. }, ChildFactory::Region { region, .. }) => {
                region.repeat_body()
            }
            _ => return None,
        };
    }
    Some(factory)
}

fn resolve_fragment_v1(construction: &ValidatedConstruction, path: &FragmentPathV1) -> bool {
    resolve_node_factory_v1(construction, path.owner()).is_some_and(|owner| {
        matches!(
            owner.children().nth(usize::from(path.region_slot())),
            Some(ChildFactory::Region { .. })
        )
    })
}

fn at_failure(
    error: ArtifactVerificationError,
    failure: &ReplayFailureV1,
) -> ArtifactVerificationError {
    failure.operation().map_or_else(
        || error.at_transaction(failure.transaction()),
        |operation| error.at_operation(failure.transaction(), operation),
    )
}

fn invalid_path() -> ArtifactVerificationError {
    ArtifactVerificationError::new(ArtifactVerificationErrorKind::InvalidSemanticPath)
}
