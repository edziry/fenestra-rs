use super::{RuntimeArtifactEncodeErrorV1, RuntimeArtifactFaultV1, RuntimeArtifactModelV1};

pub(super) fn inject(
    model: &RuntimeArtifactModelV1,
    fault: RuntimeArtifactFaultV1,
) -> Result<RuntimeArtifactModelV1, RuntimeArtifactEncodeErrorV1> {
    let _ = fault;
    Ok(model.clone())
}
