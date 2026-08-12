use super::candidates::{kurbo_detects, lyon_detects};
use super::oracle;
use super::types::{PathHitFaultKindV2, PathHitFaultV2};

pub(crate) fn path_hit_faults_v2() -> Vec<PathHitFaultV2> {
    [
        PathHitFaultKindV2::MissingMove,
        PathHitFaultKindV2::OpenFillSubpath,
        PathHitFaultKindV2::NonFiniteCoordinate,
        PathHitFaultKindV2::TessellationLimit,
        PathHitFaultKindV2::InvalidStrokeWidth,
    ]
    .into_iter()
    .map(|kind| PathHitFaultV2 {
        kind,
        literal: oracle::detects(kind),
        kurbo: kurbo_detects(kind),
        lyon: lyon_detects(kind),
    })
    .collect()
}
