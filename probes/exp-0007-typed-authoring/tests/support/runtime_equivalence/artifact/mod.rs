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
    MutationKind,
    MutationPath,
    MutationProperty,
    MutationOldValue,
    MutationNewValue,
    MutationKey,
    MutationRoot,
    MutationIndices,
    CreatedManifest,
    RetiredManifest,
    StateNodePath,
    StateNodeParent,
    StateNodeTemplate,
    StateNodeComponent,
    StateNodeOrder,
    StatePropertyOrder,
    StatePropertyId,
    StatePropertyValue,
    StateChildOrder,
    StateChildKind,
    StateChildTarget,
    StateFragmentPath,
    StateFragmentDescriptor,
    StateMemberOrder,
    StateMemberKey,
    StateMemberPath,
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
    generations: Vec<RuntimeGenerationArtifactV1>,
    final_keys: Vec<u64>,
}

#[derive(Clone, Eq, PartialEq)]
struct RuntimeGenerationArtifactV1 {
    receipt: Vec<String>,
    state: Vec<String>,
    projection: Vec<String>,
}

impl RuntimeArtifactModelV1 {
    pub fn same_slice(&self, other: &Self, slice: RuntimeArtifactSliceV1) -> bool {
        match slice {
            RuntimeArtifactSliceV1::Receipt => self
                .generations
                .iter()
                .map(|generation| &generation.receipt)
                .eq(other
                    .generations
                    .iter()
                    .map(|generation| &generation.receipt)),
            RuntimeArtifactSliceV1::State => self
                .generations
                .iter()
                .map(|generation| &generation.state)
                .eq(other.generations.iter().map(|generation| &generation.state)),
            RuntimeArtifactSliceV1::Projection => self
                .generations
                .iter()
                .map(|generation| &generation.projection)
                .eq(other
                    .generations
                    .iter()
                    .map(|generation| &generation.projection)),
            RuntimeArtifactSliceV1::FinalKeys => self.final_keys == other.final_keys,
        }
    }
}

pub fn encode_runtime_artifact_v1(
    log: &LaneLog,
    limits: RuntimeArtifactLimitsV1,
) -> Result<String, RuntimeArtifactEncodeErrorV1> {
    let model = encode::model(log)?;
    encode::encode_model(&model, limits)
}

pub fn runtime_artifact_model_v1(
    log: &LaneLog,
) -> Result<RuntimeArtifactModelV1, RuntimeArtifactEncodeErrorV1> {
    encode::model(log)
}

pub fn encode_runtime_artifact_model_v1(
    model: &RuntimeArtifactModelV1,
    limits: RuntimeArtifactLimitsV1,
) -> Result<String, RuntimeArtifactEncodeErrorV1> {
    encode::encode_model(model, limits)
}

pub fn inject_runtime_artifact_fault_v1(
    log: &LaneLog,
    fault: RuntimeArtifactFaultV1,
) -> Result<LaneLog, RuntimeArtifactEncodeErrorV1> {
    fault::inject(log, fault)
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
