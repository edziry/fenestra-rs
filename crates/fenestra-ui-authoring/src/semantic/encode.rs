use fenestra_ui_ir::prototype::{
    SUPPORTED_CONSTRUCTION_FORMAT, SUPPORTED_SCHEMA_FORMAT, SUPPORTED_STYLE_FORMAT,
};

use crate::resolved::ResolvedDocumentV1;

use super::logical::{collect_logical_records_v1, logical_record_count_v1};
use super::record::sort_and_validate;
use super::writer::{BoundedAsciiWriter, BoundedAsciiWriterError};
use super::{
    SemanticArtifactErrorKindV1, SemanticArtifactErrorV1, SemanticArtifactLimitKindV1,
    SemanticArtifactLimitsV1,
};

pub(super) fn encode_resolved_v1(
    resolved: &ResolvedDocumentV1,
    limits: SemanticArtifactLimitsV1,
) -> Result<Box<str>, SemanticArtifactErrorV1> {
    let count = logical_record_count_v1(resolved).ok_or_else(invalid_document)?;
    if count > limits.limit(SemanticArtifactLimitKindV1::Records) {
        return Err(limit_exceeded(SemanticArtifactLimitKindV1::Records));
    }

    let mut records = Vec::with_capacity(count);
    collect_logical_records_v1(resolved, &mut records).map_err(|_| invalid_document())?;
    sort_and_validate(&mut records, count).map_err(|_| invalid_document())?;

    let mut writer = BoundedAsciiWriter::new(
        limits.limit(SemanticArtifactLimitKindV1::LineBytes),
        limits.limit(SemanticArtifactLimitKindV1::ArtifactBytes),
    );
    push_line(
        &mut writer,
        &format!(
            concat!(
                "fenestra-authoring-semantics|1|authoring-format={}",
                "|schema-format={}|construction-format={}|style-format={}|records={}"
            ),
            resolved.format,
            SUPPORTED_SCHEMA_FORMAT.get(),
            SUPPORTED_CONSTRUCTION_FORMAT.get(),
            SUPPORTED_STYLE_FORMAT.get(),
            count,
        ),
    )?;
    for record in records {
        push_line(&mut writer, record.line())?;
    }
    Ok(writer.finish().into_boxed_str())
}

fn push_line(writer: &mut BoundedAsciiWriter, line: &str) -> Result<(), SemanticArtifactErrorV1> {
    writer.push_line(line).map_err(|error| match error {
        BoundedAsciiWriterError::LineBytes => {
            limit_exceeded(SemanticArtifactLimitKindV1::LineBytes)
        }
        BoundedAsciiWriterError::ArtifactBytes => {
            limit_exceeded(SemanticArtifactLimitKindV1::ArtifactBytes)
        }
        BoundedAsciiWriterError::InvalidOutput => invalid_document(),
    })
}

fn limit_exceeded(limit: SemanticArtifactLimitKindV1) -> SemanticArtifactErrorV1 {
    SemanticArtifactErrorV1::new(SemanticArtifactErrorKindV1::LimitExceeded(limit))
}

fn invalid_document() -> SemanticArtifactErrorV1 {
    SemanticArtifactErrorV1::new(SemanticArtifactErrorKindV1::InvalidCompiledDocument)
}
