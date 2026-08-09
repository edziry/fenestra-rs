use super::super::super::error::{
    HeadlessArtifactDecodeErrorKindV1 as ErrorKind, HeadlessArtifactDecodeErrorV1,
    HeadlessArtifactLimitKindV1 as LimitKind, HeadlessArtifactVersionKindV1 as VersionKind,
};
use super::super::event::parse_headless_event_v1;
use super::super::grammar::split_fields_v1;
use super::super::scan::{ScannedArtifactV1, ScannedLineV1};
use super::super::scheduler::parse_scheduler_event_v1;
use super::super::state::{LayoutV1, SectionRangeV1};
use super::super::value::{
    inspect_node_path, parse_bool, parse_i32, parse_rgba, parse_u32, parse_u64, parse_usize,
};
use crate::headless::artifact::model::{ArtifactCapacitiesV1, ArtifactMetadataV1};

pub(super) struct InspectedV1 {
    pub(super) metadata: ArtifactMetadataV1,
    pub(super) capacities: ArtifactCapacitiesV1,
    pub(super) declared_headless: [usize; 2],
    pub(super) declared_scheduler: [usize; 2],
    pub(super) declared_projection: [usize; 5],
    pub(super) actual: UsageV1,
}

pub(super) struct UsageV1 {
    pub(super) headless_events: usize,
    pub(super) headless_bytes: usize,
    pub(super) scheduler_events: usize,
    pub(super) scheduler_bytes: usize,
    pub(super) projection: [usize; 5],
    pub(super) max_path_depth: usize,
    pub(super) max_path_line: Option<u32>,
}

pub(super) fn validate_versions_v1(
    scanned: &ScannedArtifactV1<'_>,
    layout: LayoutV1,
) -> Result<(), HeadlessArtifactDecodeErrorV1> {
    let lines = scanned.lines();
    check_version(&lines[0], 1, VersionKind::Envelope)?;
    for (field, kind) in [
        (2, VersionKind::Fixture),
        (4, VersionKind::Schema),
        (6, VersionKind::Construction),
        (8, VersionKind::Style),
        (10, VersionKind::Trace),
        (12, VersionKind::Projection),
    ] {
        check_version(&lines[1], field, kind)?;
    }
    for line in section_records(lines, layout.headless)
        .iter()
        .chain(section_records(lines, layout.scheduler))
    {
        check_version(line, 1, VersionKind::Trace)?;
    }
    Ok(())
}

pub(super) fn inspect_canonical_v1(
    scanned: &ScannedArtifactV1<'_>,
    layout: LayoutV1,
) -> Result<InspectedV1, HeadlessArtifactDecodeErrorV1> {
    let lines = scanned.lines();
    parse_version_fields(lines, layout)?;
    let metadata = parse_metadata(&lines[2])?;
    let _ = parse_u32(field(&lines[3], 6)?, lines[3].number)?;
    let capacities = parse_capacities(lines)?;
    let declared_headless = parse_declaration(&lines[layout.headless.declaration])?;
    for line in section_records(lines, layout.headless) {
        let _ = parse_headless_event_v1(line)?;
    }
    let declared_scheduler = parse_declaration(&lines[layout.scheduler.declaration])?;
    for line in section_records(lines, layout.scheduler) {
        let _ = parse_scheduler_event_v1(line)?;
    }
    let declared_projection = parse_projection_declaration(&lines[layout.projection_begin])?;
    let (max_path_depth, max_path_line) = inspect_projection(lines, layout)?;
    let projection = [
        range_len(layout.computed),
        range_len(layout.geometry),
        range_len(layout.semantics),
        range_len(layout.hits),
        range_len(layout.scene),
    ];
    let headless_events = range_len(layout.headless);
    let scheduler_events = range_len(layout.scheduler);
    let headless_bytes = headless_events.checked_mul(160).ok_or_else(|| {
        limit_error(
            LimitKind::HeadlessTraceBytes,
            lines[layout.headless.declaration].number,
        )
    })?;
    let scheduler_bytes = scheduler_events.checked_mul(96).ok_or_else(|| {
        limit_error(
            LimitKind::SchedulerTraceBytes,
            lines[layout.scheduler.declaration].number,
        )
    })?;
    Ok(InspectedV1 {
        metadata,
        capacities,
        declared_headless,
        declared_scheduler,
        declared_projection,
        actual: UsageV1 {
            headless_events,
            headless_bytes,
            scheduler_events,
            scheduler_bytes,
            projection,
            max_path_depth,
            max_path_line,
        },
    })
}

fn parse_version_fields(
    lines: &[ScannedLineV1<'_>],
    layout: LayoutV1,
) -> Result<(), HeadlessArtifactDecodeErrorV1> {
    let _ = parse_u32(field(&lines[0], 1)?, lines[0].number)?;
    for field_index in [2, 4, 6, 8, 10, 12] {
        let _ = parse_u32(field(&lines[1], field_index)?, lines[1].number)?;
    }
    for line in section_records(lines, layout.headless)
        .iter()
        .chain(section_records(lines, layout.scheduler))
    {
        let _ = parse_u32(field(line, 1)?, line.number)?;
    }
    Ok(())
}

fn parse_metadata(
    line: &ScannedLineV1<'_>,
) -> Result<ArtifactMetadataV1, HeadlessArtifactDecodeErrorV1> {
    Ok(ArtifactMetadataV1 {
        fixture_revision: parse_u32(field(line, 2)?, line.number)?,
        schema_format: parse_u32(field(line, 3)?, line.number)?,
        schema_namespace: parse_u64(field(line, 4)?, line.number)?,
        schema_revision: parse_u32(field(line, 5)?, line.number)?,
        construction_format: parse_u32(field(line, 6)?, line.number)?,
        style_format: parse_u32(field(line, 7)?, line.number)?,
    })
}

fn parse_capacities(
    lines: &[ScannedLineV1<'_>],
) -> Result<ArtifactCapacitiesV1, HeadlessArtifactDecodeErrorV1> {
    Ok(ArtifactCapacitiesV1 {
        ir: parse_capacity(&lines[5])?,
        style: parse_usize(field(&lines[6], 1)?, lines[6].number)?,
        runtime: parse_capacity(&lines[7])?,
        projection: parse_capacity(&lines[8])?,
        scheduler: parse_capacity(&lines[9])?,
        renderer: parse_capacity(&lines[10])?,
        scheduler_trace: parse_capacity(&lines[11])?,
        headless_trace: parse_capacity(&lines[12])?,
        artifact: parse_capacity(&lines[13])?,
    })
}

fn parse_capacity<const N: usize>(
    line: &ScannedLineV1<'_>,
) -> Result<[usize; N], HeadlessArtifactDecodeErrorV1> {
    let fields = split_fields_v1(line)?;
    let values = fields.as_slice();
    let mut output = [0_usize; N];
    for (slot, value) in output.iter_mut().zip(&values[1..]) {
        *slot = parse_usize(value, line.number)?;
    }
    Ok(output)
}

fn parse_declaration(
    line: &ScannedLineV1<'_>,
) -> Result<[usize; 2], HeadlessArtifactDecodeErrorV1> {
    Ok([
        parse_usize(field(line, 1)?, line.number)?,
        parse_usize(field(line, 2)?, line.number)?,
    ])
}

fn parse_projection_declaration(
    line: &ScannedLineV1<'_>,
) -> Result<[usize; 5], HeadlessArtifactDecodeErrorV1> {
    let _ = parse_u64(field(line, 1)?, line.number)?;
    let _ = parse_i32(field(line, 2)?, line.number)?;
    let _ = parse_i32(field(line, 3)?, line.number)?;
    Ok([
        parse_usize(field(line, 4)?, line.number)?,
        parse_usize(field(line, 5)?, line.number)?,
        parse_usize(field(line, 6)?, line.number)?,
        parse_usize(field(line, 7)?, line.number)?,
        parse_usize(field(line, 8)?, line.number)?,
    ])
}

fn inspect_projection(
    lines: &[ScannedLineV1<'_>],
    layout: LayoutV1,
) -> Result<(usize, Option<u32>), HeadlessArtifactDecodeErrorV1> {
    let mut maximum = (0_usize, None);
    for range in [
        layout.computed,
        layout.geometry,
        layout.semantics,
        layout.hits,
        layout.scene,
    ] {
        for line in section_records(lines, range) {
            inspect_projection_line(line, &mut maximum)?;
        }
    }
    Ok(maximum)
}

fn inspect_projection_line(
    line: &ScannedLineV1<'_>,
    maximum: &mut (usize, Option<u32>),
) -> Result<(), HeadlessArtifactDecodeErrorV1> {
    let fields = split_fields_v1(line)?;
    let values = fields.as_slice();
    let depth = inspect_node_path(values[1], line.number)?;
    if depth > maximum.0 {
        maximum.0 = depth;
    }
    if depth > 3 && maximum.1.is_none() {
        maximum.1 = Some(line.number);
    }
    match values[0] {
        "computed" => {
            let _ = parse_i32(values[2], line.number)?;
            let _ = parse_i32(values[3], line.number)?;
            let _ = parse_rgba(values[4], line.number)?;
            let _ = parse_bool(values[5], line.number)?;
        }
        "geometry" => parse_i32_fields(&values[2..], line.number)?,
        "semantic" => {
            let _ = parse_u32(values[3], line.number)?;
        }
        "hit" => parse_i32_fields(&values[2..], line.number)?,
        "scene" => {
            parse_i32_fields(&values[2..6], line.number)?;
            let _ = parse_rgba(values[6], line.number)?;
        }
        _ => return Err(malformed(line.number)),
    }
    Ok(())
}

fn parse_i32_fields(values: &[&str], line: u32) -> Result<(), HeadlessArtifactDecodeErrorV1> {
    for value in values {
        let _ = parse_i32(value, line)?;
    }
    Ok(())
}

fn check_version(
    line: &ScannedLineV1<'_>,
    field_index: usize,
    kind: VersionKind,
) -> Result<(), HeadlessArtifactDecodeErrorV1> {
    let value = field(line, field_index)?;
    if version_candidate(value).is_some_and(|version| version != 1) {
        return Err(HeadlessArtifactDecodeErrorV1::at(
            ErrorKind::UnsupportedVersion(kind),
            line.number,
        ));
    }
    Ok(())
}

fn version_candidate(value: &str) -> Option<u64> {
    if value.is_empty()
        || !value.bytes().all(|byte| byte.is_ascii_digit())
        || (value.len() > 1 && value.starts_with('0'))
    {
        return None;
    }
    value.parse().ok()
}

fn field<'line>(
    line: &'line ScannedLineV1<'_>,
    index: usize,
) -> Result<&'line str, HeadlessArtifactDecodeErrorV1> {
    split_fields_v1(line)?
        .as_slice()
        .get(index)
        .copied()
        .ok_or_else(|| malformed(line.number))
}

fn section_records<'line, 'source>(
    lines: &'line [ScannedLineV1<'source>],
    range: SectionRangeV1,
) -> &'line [ScannedLineV1<'source>] {
    &lines[range.records_start..range.records_end]
}

const fn range_len(range: SectionRangeV1) -> usize {
    range.records_end - range.records_start
}

fn limit_error(limit: LimitKind, line: u32) -> HeadlessArtifactDecodeErrorV1 {
    HeadlessArtifactDecodeErrorV1::at(ErrorKind::LimitExceeded(limit), line)
}

fn malformed(line: u32) -> HeadlessArtifactDecodeErrorV1 {
    HeadlessArtifactDecodeErrorV1::at(ErrorKind::MalformedRecord, line)
}
