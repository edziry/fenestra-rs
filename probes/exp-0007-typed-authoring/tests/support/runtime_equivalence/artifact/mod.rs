use fenestra_ui_testkit::prototype::HeadlessProjectionFaultV1;

use super::LaneLog;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeArtifactLimitKindV1 {
    ArtifactBytes,
    LineBytes,
    Records,
}

impl RuntimeArtifactLimitKindV1 {
    pub const ALL: [Self; 3] = [Self::ArtifactBytes, Self::LineBytes, Self::Records];
}

#[derive(Clone, Copy)]
pub struct RuntimeArtifactLimitsV1 {
    values: [usize; 3],
}

impl RuntimeArtifactLimitsV1 {
    pub const fn new(artifact_bytes: usize, line_bytes: usize, records: usize) -> Self {
        Self {
            values: [artifact_bytes, line_bytes, records],
        }
    }

    pub const fn limit(self, kind: RuntimeArtifactLimitKindV1) -> usize {
        self.values[kind as usize]
    }
}

pub const REGISTERED_RUNTIME_ARTIFACT_LIMITS_V1: RuntimeArtifactLimitsV1 =
    RuntimeArtifactLimitsV1::new(32_768, 512, 512);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeArtifactEncodeErrorKindV1 {
    LimitExceeded(RuntimeArtifactLimitKindV1),
    InvalidLog,
}

#[derive(Debug)]
pub struct RuntimeArtifactEncodeErrorV1 {
    kind: RuntimeArtifactEncodeErrorKindV1,
}

impl RuntimeArtifactEncodeErrorV1 {
    pub const fn kind(&self) -> RuntimeArtifactEncodeErrorKindV1 {
        self.kind
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeArtifactFaultV1 {
    Receipt,
    Manifest,
    StateOrder,
    Projection(HeadlessProjectionFaultV1),
}

pub fn encode_runtime_artifact_v1(
    _log: &LaneLog,
    _limits: RuntimeArtifactLimitsV1,
) -> Result<String, RuntimeArtifactEncodeErrorV1> {
    Ok(String::new())
}

pub fn inject_runtime_artifact_fault_v1(
    log: &LaneLog,
    _fault: RuntimeArtifactFaultV1,
) -> Result<LaneLog, RuntimeArtifactEncodeErrorV1> {
    Ok(log.clone())
}
