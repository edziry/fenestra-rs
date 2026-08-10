use fenestra_ui_testkit::prototype::HeadlessProjectionFaultV1;

use super::LaneLog;

mod encode;
mod fault;
mod path;
mod projection;
mod receipt;
mod state;
mod value;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeArtifactLimitKindV1 {
    ArtifactBytes,
    LineBytes,
    Records,
}

impl RuntimeArtifactLimitKindV1 {
    pub const ALL: [Self; 3] = [Self::Records, Self::LineBytes, Self::ArtifactBytes];
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
    ReceiptGeneration,
    ReceiptInvalidation,
    MutationPath,
    MutationProperty,
    MutationValue,
    MutationKey,
    MutationRoot,
    MutationIndices,
    CreatedManifest,
    RetiredManifest,
    StateNodeParent,
    StateNodeTemplate,
    StateNodeComponent,
    StateNodeOrder,
    StatePropertyId,
    StatePropertyValue,
    StateChildKind,
    StateChildTarget,
    StateFragmentDescriptor,
    StateMemberKey,
    StateMemberPath,
    StateMemberOrder,
    Surface,
    Projection(HeadlessProjectionFaultV1),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeArtifactSliceV1 {
    Receipt,
    State,
    Projection,
    FinalKeys,
}

impl RuntimeArtifactSliceV1 {
    pub const ALL: [Self; 4] = [
        Self::Receipt,
        Self::State,
        Self::Projection,
        Self::FinalKeys,
    ];
}

#[derive(Clone)]
pub struct RuntimeArtifactModelV1 {
    log: LaneLog,
}

impl RuntimeArtifactModelV1 {
    pub fn same_slice(&self, other: &Self, slice: RuntimeArtifactSliceV1) -> bool {
        match slice {
            RuntimeArtifactSliceV1::Receipt => self.log.receipts() == other.log.receipts(),
            RuntimeArtifactSliceV1::State => self.log.states() == other.log.states(),
            RuntimeArtifactSliceV1::Projection => self.log.projections() == other.log.projections(),
            RuntimeArtifactSliceV1::FinalKeys => self.log.final_keys() == other.log.final_keys(),
        }
    }
}

pub fn encode_runtime_artifact_v1(
    log: &LaneLog,
    limits: RuntimeArtifactLimitsV1,
) -> Result<String, RuntimeArtifactEncodeErrorV1> {
    encode::encode(log, limits)
}

pub fn runtime_artifact_model_v1(
    log: &LaneLog,
) -> Result<RuntimeArtifactModelV1, RuntimeArtifactEncodeErrorV1> {
    Ok(RuntimeArtifactModelV1 { log: log.clone() })
}

pub fn encode_runtime_artifact_model_v1(
    model: &RuntimeArtifactModelV1,
    limits: RuntimeArtifactLimitsV1,
) -> Result<String, RuntimeArtifactEncodeErrorV1> {
    encode::encode(&model.log, limits)
}

pub fn inject_runtime_artifact_fault_v1(
    model: &RuntimeArtifactModelV1,
    fault: RuntimeArtifactFaultV1,
) -> Result<RuntimeArtifactModelV1, RuntimeArtifactEncodeErrorV1> {
    fault::inject(model, fault)
}

fn invalid_log() -> RuntimeArtifactEncodeErrorV1 {
    RuntimeArtifactEncodeErrorV1 {
        kind: RuntimeArtifactEncodeErrorKindV1::InvalidLog,
    }
}

fn limit_exceeded(limit: RuntimeArtifactLimitKindV1) -> RuntimeArtifactEncodeErrorV1 {
    RuntimeArtifactEncodeErrorV1 {
        kind: RuntimeArtifactEncodeErrorKindV1::LimitExceeded(limit),
    }
}
