mod boundary;
mod decode;
mod encode;
mod fingerprint;
mod grammar;
mod model;
mod preflight;
mod reference;
mod state;
mod trace;
mod verify;

#[cfg(test)]
mod tests;

use super::error::{ArtifactDecodeError, ArtifactDecodeErrorKind, ArtifactLimitKind};
use super::scan::{ARTIFACT_BYTES_LIMIT, scan_lines};

pub(super) use boundary::EnvelopeBoundariesV1;
pub use decode::decode_failure_artifact_v1;
pub use encode::encode_failure_artifact_v1;
pub use model::{
    ArtifactFixtureMetadataV1, ArtifactReductionV1, ArtifactReplayConfigV1, FailureArtifactV1,
};
pub use verify::verify_failure_artifact_v1;

pub(super) fn scan_envelope_v1(
    bytes: &[u8],
) -> Result<EnvelopeBoundariesV1<'_>, ArtifactDecodeError> {
    if bytes.len() > ARTIFACT_BYTES_LIMIT {
        return Err(ArtifactDecodeError::new(
            ArtifactDecodeErrorKind::LimitExceeded(ArtifactLimitKind::ArtifactBytes),
            None,
        ));
    }
    let lines = scan_lines(bytes)?;
    let layout = state::scan_boundaries_v1(&lines)?;
    Ok(EnvelopeBoundariesV1::new(lines, layout))
}
