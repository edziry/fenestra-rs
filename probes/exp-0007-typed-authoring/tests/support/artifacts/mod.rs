use std::fmt::Write as _;

use fenestra_ui_authoring::prototype::{AnchorKindV1, CompiledAuthoringV1};
use fenestra_ui_ir::prototype::SourceSpan;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MapArtifactLimitKindV1 {
    ArtifactBytes,
    LineBytes,
    Records,
}

#[derive(Clone, Copy)]
pub struct MapArtifactLimitsV1 {
    artifact_bytes: usize,
    line_bytes: usize,
    records: usize,
}

impl MapArtifactLimitsV1 {
    pub const fn new(artifact_bytes: usize, line_bytes: usize, records: usize) -> Self {
        Self {
            artifact_bytes,
            line_bytes,
            records,
        }
    }
}

pub const REGISTERED_MAP_ARTIFACT_LIMITS_V1: MapArtifactLimitsV1 =
    MapArtifactLimitsV1::new(4_096, 128, 36);

#[derive(Debug)]
pub enum MapArtifactEncodeErrorV1 {
    LimitExceeded(MapArtifactLimitKindV1),
    InvalidCatalog,
    InvalidEntry,
    InvalidLabel,
    Arithmetic,
    Formatting,
}

impl MapArtifactEncodeErrorV1 {
    pub const fn limit_kind(&self) -> Option<MapArtifactLimitKindV1> {
        match self {
            Self::LimitExceeded(limit) => Some(*limit),
            Self::InvalidCatalog
            | Self::InvalidEntry
            | Self::InvalidLabel
            | Self::Arithmetic
            | Self::Formatting => None,
        }
    }
}

pub fn encode_fen_map_v1(
    compiled: &CompiledAuthoringV1,
    limits: MapArtifactLimitsV1,
) -> Result<String, MapArtifactEncodeErrorV1> {
    encode_map_v1(compiled, limits, MapLaneV1::Fen)
}

pub fn encode_ui_map_v1(
    compiled: &CompiledAuthoringV1,
    limits: MapArtifactLimitsV1,
) -> Result<String, MapArtifactEncodeErrorV1> {
    encode_map_v1(compiled, limits, MapLaneV1::Ui)
}

#[derive(Clone, Copy)]
enum MapLaneV1 {
    Fen,
    Ui,
}

fn encode_map_v1(
    compiled: &CompiledAuthoringV1,
    limits: MapArtifactLimitsV1,
    lane: MapLaneV1,
) -> Result<String, MapArtifactEncodeErrorV1> {
    let entries = compiled.source_map().entries();
    let catalog = compiled.logical_source_catalog();
    if catalog.len() != entries.len() || !catalog.iter().all(|byte| *byte == b'@') {
        return Err(MapArtifactEncodeErrorV1::InvalidCatalog);
    }

    let mut artifact = BoundedArtifactV1::new(limits);
    artifact.push_line(match lane {
        MapLaneV1::Fen => "fenestra-authoring-map|1|fen",
        MapLaneV1::Ui => "fenestra-authoring-map|1|ui",
    })?;

    let mut line = String::new();
    write!(&mut line, "catalog|0|{}|", catalog.len())
        .map_err(|_| MapArtifactEncodeErrorV1::Formatting)?;
    for byte in catalog {
        line.push(char::from(*byte));
    }
    artifact.push_line(&line)?;

    for (ordinal, entry) in entries.iter().enumerate() {
        let ordinal = u32::try_from(ordinal).map_err(|_| MapArtifactEncodeErrorV1::Arithmetic)?;
        let logical_end = ordinal
            .checked_add(1)
            .ok_or(MapArtifactEncodeErrorV1::Arithmetic)?;
        match entry.logical_span() {
            SourceSpan::Bytes { source, start, end }
                if source.get() == 0 && start == ordinal && end == logical_end => {}
            SourceSpan::Synthetic | SourceSpan::Bytes { .. } => {
                return Err(MapArtifactEncodeErrorV1::InvalidEntry);
            }
        }

        let label = entry.canonical_label();
        if label.is_empty()
            || !label.is_ascii()
            || label
                .bytes()
                .any(|byte| byte == b'|' || byte == b'\r' || byte == b'\n')
        {
            return Err(MapArtifactEncodeErrorV1::InvalidLabel);
        }

        line.clear();
        write!(
            &mut line,
            "anchor|{ordinal}|{ordinal}|{logical_end}|{}|{label}",
            anchor_kind_name(entry.anchor_kind())
        )
        .map_err(|_| MapArtifactEncodeErrorV1::Formatting)?;
        match lane {
            MapLaneV1::Fen => {
                let source = entry
                    .physical_origin()
                    .source_id()
                    .ok_or(MapArtifactEncodeErrorV1::InvalidEntry)?;
                let (start, end) = entry
                    .physical_origin()
                    .fen_byte_range()
                    .ok_or(MapArtifactEncodeErrorV1::InvalidEntry)?;
                write!(&mut line, "|{}|{start}|{end}", source.get())
                    .map_err(|_| MapArtifactEncodeErrorV1::Formatting)?;
            }
            MapLaneV1::Ui => {
                if entry.physical_origin().source_id().is_some()
                    || entry.physical_origin().fen_byte_range().is_some()
                {
                    return Err(MapArtifactEncodeErrorV1::InvalidEntry);
                }
            }
        }
        artifact.push_line(&line)?;
    }

    Ok(artifact.finish())
}

fn anchor_kind_name(kind: AnchorKindV1) -> &'static str {
    match kind {
        AnchorKindV1::Document => "document",
        AnchorKindV1::Schema => "schema",
        AnchorKindV1::Component => "component",
        AnchorKindV1::Property => "property",
        AnchorKindV1::Construction => "construction",
        AnchorKindV1::Template => "template",
        AnchorKindV1::InitialProperty => "initial-property",
        AnchorKindV1::StaticChild => "static-child",
        AnchorKindV1::RegionChild => "region-child",
        AnchorKindV1::Region => "region",
        AnchorKindV1::InitialKey => "initial-key",
        AnchorKindV1::Style => "style",
        AnchorKindV1::StyleAssignment => "style-assignment",
    }
}

struct BoundedArtifactV1 {
    output: String,
    limits: MapArtifactLimitsV1,
    records: usize,
}

impl BoundedArtifactV1 {
    const fn new(limits: MapArtifactLimitsV1) -> Self {
        Self {
            output: String::new(),
            limits,
            records: 0,
        }
    }

    fn push_line(&mut self, line: &str) -> Result<(), MapArtifactEncodeErrorV1> {
        if self.records >= self.limits.records {
            return Err(MapArtifactEncodeErrorV1::LimitExceeded(
                MapArtifactLimitKindV1::Records,
            ));
        }
        if line.len() > self.limits.line_bytes {
            return Err(MapArtifactEncodeErrorV1::LimitExceeded(
                MapArtifactLimitKindV1::LineBytes,
            ));
        }
        let next_bytes = self
            .output
            .len()
            .checked_add(line.len())
            .and_then(|value| value.checked_add(1))
            .ok_or(MapArtifactEncodeErrorV1::Arithmetic)?;
        if next_bytes > self.limits.artifact_bytes {
            return Err(MapArtifactEncodeErrorV1::LimitExceeded(
                MapArtifactLimitKindV1::ArtifactBytes,
            ));
        }
        if !line.bytes().all(|byte| (0x20..=0x7e).contains(&byte)) {
            return Err(MapArtifactEncodeErrorV1::InvalidLabel);
        }

        self.output.push_str(line);
        self.output.push('\n');
        self.records += 1;
        Ok(())
    }

    fn finish(self) -> String {
        self.output
    }
}
