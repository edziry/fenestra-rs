use super::projection::encode_projection;
use super::receipt::encode_receipt;
use super::state::encode_state;
use super::{
    LaneLog, RuntimeArtifactEncodeErrorV1, RuntimeArtifactLimitKindV1, RuntimeArtifactLimitsV1,
    invalid_log, limit_exceeded,
};

const GENERATIONS: usize = 6;

pub(super) fn encode(
    log: &LaneLog,
    limits: RuntimeArtifactLimitsV1,
) -> Result<String, RuntimeArtifactEncodeErrorV1> {
    if log.receipts().len() != GENERATIONS
        || log.states().len() != GENERATIONS
        || log.projections().len() != GENERATIONS
    {
        return Err(invalid_log());
    }
    validate_final_keys(log)?;

    let mut writer = ArtifactWriterV1::new(limits);
    for generation in 0..GENERATIONS {
        writer.push(&format!("generation|1|{generation}|begin"))?;
        encode_receipt(&mut writer, generation, &log.receipts()[generation])?;
        encode_state(&mut writer, &log.states()[generation])?;
        encode_projection(&mut writer, &log.projections()[generation])?;
        writer.push(&format!("generation|1|{generation}|end"))?;
    }
    Ok(writer.finish())
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
    output: String,
    limits: RuntimeArtifactLimitsV1,
    records: usize,
}

impl ArtifactWriterV1 {
    const fn new(limits: RuntimeArtifactLimitsV1) -> Self {
        Self {
            output: String::new(),
            limits,
            records: 0,
        }
    }

    pub(super) fn push(&mut self, line: &str) -> Result<(), RuntimeArtifactEncodeErrorV1> {
        if self.records >= self.limits.limit(RuntimeArtifactLimitKindV1::Records) {
            return Err(limit_exceeded(RuntimeArtifactLimitKindV1::Records));
        }
        if line.len() > self.limits.limit(RuntimeArtifactLimitKindV1::LineBytes) {
            return Err(limit_exceeded(RuntimeArtifactLimitKindV1::LineBytes));
        }
        let next = self
            .output
            .len()
            .checked_add(line.len())
            .and_then(|value| value.checked_add(1))
            .ok_or_else(invalid_log)?;
        if next > self.limits.limit(RuntimeArtifactLimitKindV1::ArtifactBytes) {
            return Err(limit_exceeded(RuntimeArtifactLimitKindV1::ArtifactBytes));
        }
        if !line.bytes().all(|byte| (0x20..=0x7e).contains(&byte)) {
            return Err(invalid_log());
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
