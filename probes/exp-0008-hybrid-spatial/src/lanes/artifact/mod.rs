mod closure;
mod collect;
mod encode;
mod model;
mod sha256;
mod verify;

pub(crate) use encode::all_lane_artifacts_v2;
pub(crate) use model::LaneArtifactV2;
pub(crate) use sha256::sha256_hex_v2;
pub(crate) use verify::verify_lane_artifact_v2;

const BASELINE: &[u8] = include_bytes!("../../../tests/artifacts/spatial-v2.txt");
const BASELINE_SHA256: &str = "bc71d3f9167808984abf083613ea86a81eced60d8670d9b3133821dbb34d21a1";
