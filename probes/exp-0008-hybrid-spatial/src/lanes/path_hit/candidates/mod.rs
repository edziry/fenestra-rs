mod kurbo;
mod lyon;

use super::finish_run;
use super::types::{
    FillRuleV2, PathCoverageV2, PathHitCaseV2, PathHitFaultKindV2, PathHitRecordV2,
    PathHitResultV2, PathHitRunV2, PathLayerV2, PathVerbV2,
};

pub(crate) fn kurbo_run(cases: &[PathHitCaseV2]) -> PathHitResultV2<PathHitRunV2> {
    run_cases(cases, kurbo::hit)
}

pub(crate) fn lyon_run(cases: &[PathHitCaseV2]) -> PathHitResultV2<PathHitRunV2> {
    run_cases(cases, lyon::hit)
}

fn run_cases(
    cases: &[PathHitCaseV2],
    mut hit: impl FnMut(&PathLayerV2, [i32; 2]) -> PathHitResultV2<bool>,
) -> PathHitResultV2<PathHitRunV2> {
    let mut records = Vec::new();
    for case in cases {
        for layer in case.layers.iter().chain(case.clip.iter()) {
            validate_layer(layer)?;
        }
        for (query, input) in case.queries.iter().enumerate() {
            let clip = case
                .clip
                .as_ref()
                .map_or(Ok(true), |clip| hit(clip, input.point))?;
            let layer_hits = case
                .layers
                .iter()
                .map(|layer| hit(layer, input.point).map(|value| value && clip))
                .collect::<PathHitResultV2<Vec<_>>>()?;
            let topmost = layer_hits
                .iter()
                .rposition(|value| *value)
                .map(|index| index as u8);
            if input.nonrectangular_aabb_miss && topmost.is_some() {
                return Err(PathHitFaultKindV2::TessellationLimit);
            }
            records.push(PathHitRecordV2 {
                case: case.ordinal,
                query: query as u8,
                layer_hits,
                topmost,
            });
        }
    }
    Ok(finish_run(cases, records))
}

fn validate_layer(layer: &PathLayerV2) -> PathHitResultV2<()> {
    if !matches!(layer.verbs.first(), Some(PathVerbV2::Move(_))) {
        return Err(PathHitFaultKindV2::MissingMove);
    }
    if layer.verbs.len() > 4096 {
        return Err(PathHitFaultKindV2::TessellationLimit);
    }
    if matches!(layer.coverage, PathCoverageV2::RoundStroke { width } if width <= 0) {
        return Err(PathHitFaultKindV2::InvalidStrokeWidth);
    }
    if matches!(layer.coverage, PathCoverageV2::Fill(_)) && has_open_subpath(&layer.verbs) {
        return Err(PathHitFaultKindV2::OpenFillSubpath);
    }
    Ok(())
}

fn has_open_subpath(verbs: &[PathVerbV2]) -> bool {
    let mut open = false;
    for verb in verbs {
        match verb {
            PathVerbV2::Move(_) if open => return true,
            PathVerbV2::Move(_) => open = true,
            PathVerbV2::Close => open = false,
            _ => {}
        }
    }
    open
}

pub(super) fn detects_with(
    kind: PathHitFaultKindV2,
    nonfinite_rejected: bool,
    singular_candidate_check: bool,
) -> bool {
    match kind {
        PathHitFaultKindV2::MissingMove => validate_layer(&PathLayerV2 {
            verbs: vec![PathVerbV2::Line([0, 0])],
            coverage: PathCoverageV2::Fill(FillRuleV2::NonZero),
        })
        .is_err(),
        PathHitFaultKindV2::OpenFillSubpath => validate_layer(&PathLayerV2 {
            verbs: vec![PathVerbV2::Move([0, 0]), PathVerbV2::Line([256, 0])],
            coverage: PathCoverageV2::Fill(FillRuleV2::NonZero),
        })
        .is_err(),
        PathHitFaultKindV2::NonFiniteCoordinate => nonfinite_rejected,
        PathHitFaultKindV2::TessellationLimit => 4097 > 4096 && singular_candidate_check,
        PathHitFaultKindV2::InvalidStrokeWidth => validate_layer(&PathLayerV2 {
            verbs: vec![PathVerbV2::Move([0, 0])],
            coverage: PathCoverageV2::RoundStroke { width: 0 },
        })
        .is_err(),
    }
}

pub(super) fn kurbo_detects(kind: PathHitFaultKindV2) -> bool {
    kurbo::detects(kind)
}

pub(super) fn lyon_detects(kind: PathHitFaultKindV2) -> bool {
    lyon::detects(kind)
}
