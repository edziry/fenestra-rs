use fenestra_ui_authoring::prototype::CompiledAuthoringV1;

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
pub struct MapArtifactEncodeErrorV1 {
    limit: Option<MapArtifactLimitKindV1>,
}

impl MapArtifactEncodeErrorV1 {
    pub const fn limit_kind(&self) -> Option<MapArtifactLimitKindV1> {
        self.limit
    }
}

pub fn encode_fen_map_v1(
    _compiled: &CompiledAuthoringV1,
    _limits: MapArtifactLimitsV1,
) -> Result<String, MapArtifactEncodeErrorV1> {
    panic!("FEN source-map artifact encoder is not implemented")
}

pub fn encode_ui_map_v1(
    _compiled: &CompiledAuthoringV1,
    _limits: MapArtifactLimitsV1,
) -> Result<String, MapArtifactEncodeErrorV1> {
    panic!("UI source-map artifact encoder is not implemented")
}
