use fenestra_ui_testkit::prototype::NormalizedHeadlessProjectionV1;

use super::projection::encode_projection;
use super::receipt::encode_receipt;
use super::state::encode_state;
use super::{
    LaneLog, RuntimeArtifactEncodeErrorV1, RuntimeArtifactLimitKindV1, RuntimeArtifactLimitsV1,
    RuntimeArtifactModelV1, RuntimeGenerationArtifactV1, invalid_log, limit_exceeded,
};

const GENERATIONS: usize = 6;

pub(super) fn model(log: &LaneLog) -> Result<RuntimeArtifactModelV1, RuntimeArtifactEncodeErrorV1> {
    if log.receipts().len() != GENERATIONS
        || log.states().len() != GENERATIONS
        || log.projections().len() != GENERATIONS
    {
        return Err(invalid_log());
    }
    validate_final_keys(log)?;

    let mut generations = Vec::with_capacity(GENERATIONS);
    for generation in 0..GENERATIONS {
        let mut receipt = ArtifactWriterV1::new();
        encode_receipt(&mut receipt, generation, &log.receipts()[generation])?;
        let mut state = ArtifactWriterV1::new();
        encode_state(&mut state, &log.states()[generation])?;
        let projection = collect_projection(&log.projections()[generation])?;
        generations.push(RuntimeGenerationArtifactV1 {
            receipt: receipt.finish(),
            state: state.finish(),
            projection,
        });
    }
    Ok(RuntimeArtifactModelV1 {
        generations,
        projection_sources: log.projections().to_vec(),
        final_keys: log.final_keys().to_vec(),
    })
}

pub(super) fn encode_model(
    model: &RuntimeArtifactModelV1,
    limits: RuntimeArtifactLimitsV1,
) -> Result<String, RuntimeArtifactEncodeErrorV1> {
    let lines = artifact_lines(model)?;
    preflight(&lines, limits)?;
    let capacity = lines
        .iter()
        .try_fold(0usize, |bytes, line| {
            bytes.checked_add(line.len())?.checked_add(1)
        })
        .ok_or_else(invalid_log)?;
    let mut output = String::with_capacity(capacity);
    for line in lines {
        output.push_str(&line);
        output.push('\n');
    }
    Ok(output)
}

pub(super) fn collect_projection(
    projection: &NormalizedHeadlessProjectionV1,
) -> Result<Vec<String>, RuntimeArtifactEncodeErrorV1> {
    let mut writer = ArtifactWriterV1::new();
    encode_projection(&mut writer, projection)?;
    Ok(writer.finish())
}

fn artifact_lines(
    model: &RuntimeArtifactModelV1,
) -> Result<Vec<String>, RuntimeArtifactEncodeErrorV1> {
    if model.generations.len() != GENERATIONS || model.projection_sources.len() != GENERATIONS {
        return Err(invalid_log());
    }
    let mut lines = Vec::new();
    for (generation, artifact) in model.generations.iter().enumerate() {
        lines.push(format!("generation|1|{generation}|begin"));
        lines.extend(artifact.receipt.iter().cloned());
        lines.extend(artifact.state.iter().cloned());
        lines.extend(artifact.projection.iter().cloned());
        lines.push(format!("generation|1|{generation}|end"));
    }
    Ok(lines)
}

fn preflight(
    lines: &[String],
    limits: RuntimeArtifactLimitsV1,
) -> Result<(), RuntimeArtifactEncodeErrorV1> {
    if lines.len() > limits.limit(RuntimeArtifactLimitKindV1::Records) {
        return Err(limit_exceeded(RuntimeArtifactLimitKindV1::Records));
    }
    if lines
        .iter()
        .any(|line| line.len() > limits.limit(RuntimeArtifactLimitKindV1::LineBytes))
    {
        return Err(limit_exceeded(RuntimeArtifactLimitKindV1::LineBytes));
    }
    let bytes = lines.iter().try_fold(0usize, |bytes, line| {
        bytes.checked_add(line.len())?.checked_add(1)
    });
    match bytes {
        Some(bytes) if bytes <= limits.limit(RuntimeArtifactLimitKindV1::ArtifactBytes) => Ok(()),
        Some(_) => Err(limit_exceeded(RuntimeArtifactLimitKindV1::ArtifactBytes)),
        None => Err(invalid_log()),
    }
}

fn validate_final_keys(log: &LaneLog) -> Result<(), RuntimeArtifactEncodeErrorV1> {
    let final_state = log.states().last().ok_or_else(invalid_log)?;
    let fragment = final_state.fragments().first().ok_or_else(invalid_log)?;
    let keys = fragment
        .members()
        .iter()
        .map(|member| member.key())
        .collect::<Vec<_>>();
    if keys != log.final_keys() {
        return Err(invalid_log());
    }
    Ok(())
}

pub(super) struct ArtifactWriterV1 {
    lines: Vec<String>,
}

impl ArtifactWriterV1 {
    const fn new() -> Self {
        Self { lines: Vec::new() }
    }

    pub(super) fn push(&mut self, line: &str) -> Result<(), RuntimeArtifactEncodeErrorV1> {
        if !line.bytes().all(|byte| (0x20..=0x7e).contains(&byte)) {
            return Err(invalid_log());
        }
        self.lines.push(line.to_owned());
        Ok(())
    }

    fn finish(self) -> Vec<String> {
        self.lines
    }
}
