use fenestra_ui_ir::prototype::{
    SUPPORTED_CONSTRUCTION_FORMAT, SUPPORTED_SCHEMA_FORMAT, SUPPORTED_SPATIAL_FORMAT,
    SUPPORTED_STYLE_FORMAT,
};

use crate::compiled_v2::CompiledAuthoringV2;
use crate::semantic::{
    BoundedAsciiWriter, BoundedAsciiWriterError, collect_logical_records_v1, sort_and_validate,
};

use super::catalog::SourceCatalog;
use super::count::record_count;
use super::{
    SemanticArtifactErrorKindV2, SemanticArtifactErrorV2, SemanticArtifactLimitKindV2,
    SemanticArtifactLimitsV2, spatial,
};

pub(super) fn encode(
    compiled: &CompiledAuthoringV2,
    limits: SemanticArtifactLimitsV2,
) -> Result<Box<str>, SemanticArtifactErrorV2> {
    let resolved = compiled.resolved();
    let count = record_count(resolved, compiled.spatial()).ok_or_else(invalid_document)?;
    if count > limits.limit(SemanticArtifactLimitKindV2::Records) {
        return Err(limit_exceeded(SemanticArtifactLimitKindV2::Records));
    }
    let catalog = SourceCatalog::new(compiled, count).map_err(|_| invalid_document())?;
    let mut records = Vec::with_capacity(count);
    collect_logical_records_v1(&resolved.core, &mut records).map_err(|_| invalid_document())?;
    spatial::collect(resolved, compiled.spatial(), &mut records, &catalog)
        .map_err(|_| invalid_document())?;
    sort_and_validate(&mut records, count).map_err(|_| invalid_document())?;

    let mut writer = BoundedAsciiWriter::new(
        limits.limit(SemanticArtifactLimitKindV2::LineBytes),
        limits.limit(SemanticArtifactLimitKindV2::ArtifactBytes),
    );
    push_line(
        &mut writer,
        &format!(
            concat!(
                "fenestra-authoring-semantics|2|authoring-format={}",
                "|schema-format={}|construction-format={}|style-format={}",
                "|spatial-format={}|records={}"
            ),
            resolved.authoring_format(),
            SUPPORTED_SCHEMA_FORMAT.get(),
            SUPPORTED_CONSTRUCTION_FORMAT.get(),
            SUPPORTED_STYLE_FORMAT.get(),
            SUPPORTED_SPATIAL_FORMAT.get(),
            count,
        ),
    )?;
    for record in records {
        push_line(&mut writer, record.line())?;
    }
    Ok(writer.finish().into_boxed_str())
}

fn push_line(writer: &mut BoundedAsciiWriter, line: &str) -> Result<(), SemanticArtifactErrorV2> {
    writer.push_line(line).map_err(|error| match error {
        BoundedAsciiWriterError::LineBytes => {
            limit_exceeded(SemanticArtifactLimitKindV2::LineBytes)
        }
        BoundedAsciiWriterError::ArtifactBytes => {
            limit_exceeded(SemanticArtifactLimitKindV2::ArtifactBytes)
        }
        BoundedAsciiWriterError::InvalidOutput => invalid_document(),
    })
}

fn limit_exceeded(limit: SemanticArtifactLimitKindV2) -> SemanticArtifactErrorV2 {
    SemanticArtifactErrorV2::new(SemanticArtifactErrorKindV2::LimitExceeded(limit))
}

fn invalid_document() -> SemanticArtifactErrorV2 {
    SemanticArtifactErrorV2::new(SemanticArtifactErrorKindV2::InvalidCompiledDocument)
}
