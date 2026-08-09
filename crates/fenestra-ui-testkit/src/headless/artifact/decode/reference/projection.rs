use super::super::super::error::{
    HeadlessArtifactDecodeErrorKindV1, HeadlessArtifactDecodeErrorV1,
};
use super::super::scan::{ScannedArtifactV1, ScannedLineV1};
use super::super::state::LayoutV1;
use crate::headless::artifact::model::HeadlessArtifactV1;
use crate::headless::artifact::record::ProjectionRecordV1;
use crate::semantic::NodePathV1;

pub(super) fn validate_projection_v1(
    artifact: &HeadlessArtifactV1,
    scanned: &ScannedArtifactV1<'_>,
    layout: LayoutV1,
) -> Result<(), HeadlessArtifactDecodeErrorV1> {
    let lines = scanned.lines();
    let Some(last) = artifact.headless_events.last() else {
        return Err(invalid(lines[layout.projection_begin].number));
    };
    let final_generation = artifact
        .headless_events
        .iter()
        .filter_map(|event| event.published)
        .next_back()
        .unwrap_or_default();
    if artifact.final_generation != final_generation || artifact.projection.surface != last.surface
    {
        return Err(invalid(lines[layout.projection_begin].number));
    }
    validate_computed_paths(&artifact.projection, lines, layout)?;
    validate_derived_paths(&artifact.projection, lines, layout)
}

fn validate_computed_paths(
    projection: &ProjectionRecordV1,
    lines: &[ScannedLineV1<'_>],
    layout: LayoutV1,
) -> Result<(), HeadlessArtifactDecodeErrorV1> {
    for (index, record) in projection.computed.iter().enumerate() {
        let line = &lines[layout.computed.records_start + index];
        if !known_fixture_path(&record.path)
            || projection.computed[..index]
                .iter()
                .any(|prior| prior.path == record.path)
        {
            return Err(invalid(line.number));
        }
    }
    Ok(())
}

fn validate_derived_paths(
    projection: &ProjectionRecordV1,
    lines: &[ScannedLineV1<'_>],
    layout: LayoutV1,
) -> Result<(), HeadlessArtifactDecodeErrorV1> {
    for (index, record) in projection.geometry.iter().enumerate() {
        require_unique_computed_path(
            projection,
            &record.path,
            projection.geometry[..index]
                .iter()
                .any(|prior| prior.path == record.path),
            lines[layout.geometry.records_start + index].number,
        )?;
    }
    for (index, record) in projection.semantics.iter().enumerate() {
        require_unique_computed_path(
            projection,
            &record.path,
            projection.semantics[..index]
                .iter()
                .any(|prior| prior.path == record.path),
            lines[layout.semantics.records_start + index].number,
        )?;
    }
    for (index, record) in projection.hits.iter().enumerate() {
        require_unique_computed_path(
            projection,
            &record.path,
            projection.hits[..index]
                .iter()
                .any(|prior| prior.path == record.path),
            lines[layout.hits.records_start + index].number,
        )?;
    }
    for (index, record) in projection.scene.iter().enumerate() {
        require_unique_computed_path(
            projection,
            &record.path,
            projection.scene[..index]
                .iter()
                .any(|prior| prior.path == record.path),
            lines[layout.scene.records_start + index].number,
        )?;
    }
    Ok(())
}

fn require_unique_computed_path(
    projection: &ProjectionRecordV1,
    path: &NodePathV1,
    duplicate: bool,
    line: u32,
) -> Result<(), HeadlessArtifactDecodeErrorV1> {
    if !duplicate
        && projection
            .computed
            .iter()
            .any(|record| &record.path == path)
    {
        Ok(())
    } else {
        Err(invalid(line))
    }
}

fn known_fixture_path(path: &NodePathV1) -> bool {
    let root = NodePathV1::root();
    let container = root.clone().static_child(0);
    path == &root
        || path == &container
        || path == &container.clone().static_child(0)
        || path == &container.clone().member(1, 10)
        || path == &container.clone().member(1, 20)
        || path == &container.member(1, 30)
}

fn invalid(line: u32) -> HeadlessArtifactDecodeErrorV1 {
    HeadlessArtifactDecodeErrorV1::at(HeadlessArtifactDecodeErrorKindV1::InvalidReference, line)
}
