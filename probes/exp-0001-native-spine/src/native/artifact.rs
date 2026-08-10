mod encode;
mod types;

pub(crate) use encode::encode_native_artifact_v1;
pub(crate) use types::{
    NativeArtifactCapabilitiesV1, NativeArtifactManifestV1, NativeArtifactTerminalV1,
    NativeOsFamilyV1, NativeProbeResultV1, NativeTargetV1, NativeWindowSystemV1,
};

pub(crate) const NATIVE_ARTIFACT_SCHEMA_REVISION_V1: u16 = 1;
pub(crate) const NATIVE_ARTIFACT_MAX_EVENTS_V1: usize = 128;
pub(crate) const NATIVE_ARTIFACT_MAX_LINES_V1: usize = NATIVE_ARTIFACT_MAX_EVENTS_V1 + 3;
pub(crate) const NATIVE_ARTIFACT_MAX_BYTES_V1: usize = 65_536;
