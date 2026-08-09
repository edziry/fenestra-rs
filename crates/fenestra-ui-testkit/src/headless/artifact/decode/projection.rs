use fenestra_ui_ir::prototype::InputPolicy;
use fenestra_ui_runtime::prototype::{
    HeadlessRect, HeadlessSemanticAction, HeadlessSemanticRole, HeadlessSurface,
};

use super::super::error::{
    HeadlessArtifactDecodeErrorKindV1 as ErrorKind, HeadlessArtifactDecodeErrorV1,
};
use super::super::record::{
    ComputedRecordV1, GeometryRecordV1, ProjectionRecordV1, RectangleRecordV1, SceneRecordV1,
    SemanticRecordV1,
};
use super::scan::{ScannedArtifactV1, ScannedLineV1};
use super::state::{LayoutV1, SectionRangeV1};
use super::value::{
    parse_bool, parse_i32, parse_node_path, parse_rgba, parse_u32, parse_u64, parse_usize,
};
use crate::headless::trace::HeadlessTraceProjectionCountsV1;

pub(super) fn parse_projection_v1(
    scanned: &ScannedArtifactV1<'_>,
    layout: LayoutV1,
) -> Result<(u64, ProjectionRecordV1), HeadlessArtifactDecodeErrorV1> {
    let lines = scanned.lines();
    let declaration = &lines[layout.projection_begin];
    let fields = declaration.text.split('|').collect::<Vec<_>>();
    let generation = parse_u64(fields[1], declaration.number)?;
    let surface = HeadlessSurface::new(
        parse_i32(fields[2], declaration.number)?,
        parse_i32(fields[3], declaration.number)?,
    );
    let counts = HeadlessTraceProjectionCountsV1::new(
        parse_usize(fields[4], declaration.number)?,
        parse_usize(fields[5], declaration.number)?,
        parse_usize(fields[6], declaration.number)?,
        parse_usize(fields[7], declaration.number)?,
        parse_usize(fields[8], declaration.number)?,
    );
    Ok((
        generation,
        ProjectionRecordV1 {
            surface,
            counts,
            computed: parse_records(lines, layout.computed, parse_computed)?,
            geometry: parse_records(lines, layout.geometry, parse_geometry)?,
            semantics: parse_records(lines, layout.semantics, parse_semantic)?,
            hits: parse_records(lines, layout.hits, parse_hit)?,
            scene: parse_records(lines, layout.scene, parse_scene)?,
        },
    ))
}

fn parse_records<T>(
    lines: &[ScannedLineV1<'_>],
    range: SectionRangeV1,
    parse: fn(&ScannedLineV1<'_>) -> Result<T, HeadlessArtifactDecodeErrorV1>,
) -> Result<Vec<T>, HeadlessArtifactDecodeErrorV1> {
    let count = range.records_end - range.records_start;
    let mut records = Vec::with_capacity(count);
    for line in &lines[range.records_start..range.records_end] {
        records.push(parse(line)?);
    }
    Ok(records)
}

fn parse_computed(
    line: &ScannedLineV1<'_>,
) -> Result<ComputedRecordV1, HeadlessArtifactDecodeErrorV1> {
    let fields = line.text.split('|').collect::<Vec<_>>();
    Ok(ComputedRecordV1 {
        path: parse_node_path(fields[1], line.number)?,
        width: parse_i32(fields[2], line.number)?,
        height: parse_i32(fields[3], line.number)?,
        color: parse_rgba(fields[4], line.number)?,
        visible: parse_bool(fields[5], line.number)?,
        input: parse_input(fields[6], line.number)?,
    })
}

fn parse_geometry(
    line: &ScannedLineV1<'_>,
) -> Result<GeometryRecordV1, HeadlessArtifactDecodeErrorV1> {
    let fields = line.text.split('|').collect::<Vec<_>>();
    Ok(GeometryRecordV1 {
        path: parse_node_path(fields[1], line.number)?,
        bounds: parse_rect(&fields[2..6], line.number)?,
        clip: parse_rect(&fields[6..10], line.number)?,
    })
}

fn parse_semantic(
    line: &ScannedLineV1<'_>,
) -> Result<SemanticRecordV1, HeadlessArtifactDecodeErrorV1> {
    let fields = line.text.split('|').collect::<Vec<_>>();
    Ok(SemanticRecordV1 {
        path: parse_node_path(fields[1], line.number)?,
        role: parse_role(fields[2], line.number)?,
        label: parse_u32(fields[3], line.number)?,
        action: parse_action(fields[4], line.number)?,
    })
}

fn parse_hit(line: &ScannedLineV1<'_>) -> Result<RectangleRecordV1, HeadlessArtifactDecodeErrorV1> {
    let fields = line.text.split('|').collect::<Vec<_>>();
    Ok(RectangleRecordV1 {
        path: parse_node_path(fields[1], line.number)?,
        rectangle: parse_rect(&fields[2..6], line.number)?,
    })
}

fn parse_scene(line: &ScannedLineV1<'_>) -> Result<SceneRecordV1, HeadlessArtifactDecodeErrorV1> {
    let fields = line.text.split('|').collect::<Vec<_>>();
    Ok(SceneRecordV1 {
        path: parse_node_path(fields[1], line.number)?,
        rectangle: parse_rect(&fields[2..6], line.number)?,
        color: parse_rgba(fields[6], line.number)?,
    })
}

fn parse_rect(fields: &[&str], line: u32) -> Result<HeadlessRect, HeadlessArtifactDecodeErrorV1> {
    Ok(HeadlessRect::new(
        parse_i32(fields[0], line)?,
        parse_i32(fields[1], line)?,
        parse_i32(fields[2], line)?,
        parse_i32(fields[3], line)?,
    ))
}

fn parse_input(value: &str, line: u32) -> Result<InputPolicy, HeadlessArtifactDecodeErrorV1> {
    match value {
        "accept" => Ok(InputPolicy::Accept),
        "ignore" => Ok(InputPolicy::Ignore),
        _ => Err(malformed(line)),
    }
}

fn parse_role(
    value: &str,
    line: u32,
) -> Result<HeadlessSemanticRole, HeadlessArtifactDecodeErrorV1> {
    match value {
        "control" => Ok(HeadlessSemanticRole::Control),
        _ => Err(malformed(line)),
    }
}

fn parse_action(
    value: &str,
    line: u32,
) -> Result<HeadlessSemanticAction, HeadlessArtifactDecodeErrorV1> {
    match value {
        "activate" => Ok(HeadlessSemanticAction::Activate),
        _ => Err(malformed(line)),
    }
}

fn malformed(line: u32) -> HeadlessArtifactDecodeErrorV1 {
    HeadlessArtifactDecodeErrorV1::at(ErrorKind::MalformedRecord, line)
}
