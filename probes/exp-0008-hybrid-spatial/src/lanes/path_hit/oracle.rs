use super::finish_run;
use super::types::{
    FillRuleV2, PATH_SCALE_V2, PathCoverageV2, PathHitCaseV2, PathHitFaultKindV2, PathHitRecordV2,
    PathHitResultV2, PathHitRunV2, PathLayerV2, PathVerbV2,
};

#[derive(Clone)]
struct Subpath {
    points: Vec<[f64; 2]>,
    closed: bool,
}

pub(crate) fn literal_path_hit_run_v2(cases: &[PathHitCaseV2]) -> PathHitResultV2<PathHitRunV2> {
    let mut records = Vec::new();
    for case in cases {
        validate_case(case)?;
        for (query, input) in case.queries.iter().enumerate() {
            let clip = case
                .clip
                .as_ref()
                .map_or(Ok(true), |clip| hit_layer(clip, input.point))?;
            let layer_hits = case
                .layers
                .iter()
                .map(|layer| hit_layer(layer, input.point).map(|hit| hit && clip))
                .collect::<PathHitResultV2<Vec<_>>>()?;
            let topmost = layer_hits
                .iter()
                .rposition(|hit| *hit)
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

fn validate_case(case: &PathHitCaseV2) -> PathHitResultV2<()> {
    for layer in case.layers.iter().chain(case.clip.iter()) {
        validate_layer(layer)?;
    }
    Ok(())
}

fn validate_layer(layer: &PathLayerV2) -> PathHitResultV2<()> {
    if !matches!(layer.verbs.first(), Some(PathVerbV2::Move(_))) {
        return Err(PathHitFaultKindV2::MissingMove);
    }
    if matches!(layer.coverage, PathCoverageV2::RoundStroke { width } if width <= 0) {
        return Err(PathHitFaultKindV2::InvalidStrokeWidth);
    }
    if layer.verbs.len() > 4096 {
        return Err(PathHitFaultKindV2::TessellationLimit);
    }
    let paths = flatten(&layer.verbs)?;
    if matches!(layer.coverage, PathCoverageV2::Fill(_)) && paths.iter().any(|path| !path.closed) {
        return Err(PathHitFaultKindV2::OpenFillSubpath);
    }
    Ok(())
}

fn hit_layer(layer: &PathLayerV2, point: [i32; 2]) -> PathHitResultV2<bool> {
    let subpaths = flatten(&layer.verbs)?;
    let point = scaled(point);
    match layer.coverage {
        PathCoverageV2::Fill(rule) => Ok(fill_hit(&subpaths, point, rule)),
        PathCoverageV2::RoundStroke { width } => {
            let radius = f64::from(width) / f64::from(PATH_SCALE_V2) / 2.0;
            Ok(subpaths.iter().any(|path| stroke_hit(path, point, radius)))
        }
    }
}

fn fill_hit(subpaths: &[Subpath], point: [f64; 2], rule: FillRuleV2) -> bool {
    if subpaths.iter().any(|path| boundary(path, point)) {
        return true;
    }
    let winding = subpaths
        .iter()
        .map(|path| winding(path, point))
        .sum::<i32>();
    match rule {
        FillRuleV2::NonZero => winding != 0,
        FillRuleV2::EvenOdd => winding.unsigned_abs() % 2 == 1,
    }
}

fn winding(path: &Subpath, point: [f64; 2]) -> i32 {
    path.points.windows(2).fold(0, |value, segment| {
        let [from, to] = [segment[0], segment[1]];
        let side = cross(from, to, point);
        if from[1] <= point[1] && to[1] > point[1] && side > 0.0 {
            value + 1
        } else if from[1] > point[1] && to[1] <= point[1] && side < 0.0 {
            value - 1
        } else {
            value
        }
    })
}

fn boundary(path: &Subpath, point: [f64; 2]) -> bool {
    path.points
        .windows(2)
        .any(|segment| distance_squared(segment[0], segment[1], point) <= 1.0e-12)
}

fn stroke_hit(path: &Subpath, point: [f64; 2], radius: f64) -> bool {
    path.points
        .windows(2)
        .any(|segment| distance_squared(segment[0], segment[1], point) <= radius * radius)
}

fn distance_squared(from: [f64; 2], to: [f64; 2], point: [f64; 2]) -> f64 {
    let delta = [to[0] - from[0], to[1] - from[1]];
    let length = delta[0] * delta[0] + delta[1] * delta[1];
    let t = if length == 0.0 {
        0.0
    } else {
        (((point[0] - from[0]) * delta[0] + (point[1] - from[1]) * delta[1]) / length)
            .clamp(0.0, 1.0)
    };
    let nearest = [from[0] + t * delta[0], from[1] + t * delta[1]];
    (point[0] - nearest[0]).powi(2) + (point[1] - nearest[1]).powi(2)
}

fn flatten(verbs: &[PathVerbV2]) -> PathHitResultV2<Vec<Subpath>> {
    let mut paths = Vec::new();
    let mut current: Option<Subpath> = None;
    for verb in verbs {
        match *verb {
            PathVerbV2::Move(point) => {
                if let Some(path) = current.take() {
                    paths.push(path);
                }
                current = Some(Subpath {
                    points: vec![scaled(point)],
                    closed: false,
                });
            }
            PathVerbV2::Line(to) => current_path(&mut current)?.points.push(scaled(to)),
            PathVerbV2::Quadratic(control, to) => {
                let path = current_path(&mut current)?;
                let from = *path
                    .points
                    .last()
                    .expect("validated path has a current point");
                append_quadratic(&mut path.points, from, scaled(control), scaled(to));
            }
            PathVerbV2::Cubic(first, second, to) => {
                let path = current_path(&mut current)?;
                let from = *path
                    .points
                    .last()
                    .expect("validated path has a current point");
                append_cubic(
                    &mut path.points,
                    from,
                    scaled(first),
                    scaled(second),
                    scaled(to),
                );
            }
            PathVerbV2::Close => {
                let mut path = current.take().ok_or(PathHitFaultKindV2::MissingMove)?;
                let first = path.points[0];
                if path.points.last() != Some(&first) {
                    path.points.push(first);
                }
                path.closed = true;
                paths.push(path);
            }
        }
    }
    if let Some(path) = current {
        paths.push(path);
    }
    Ok(paths)
}

fn current_path(current: &mut Option<Subpath>) -> PathHitResultV2<&mut Subpath> {
    current.as_mut().ok_or(PathHitFaultKindV2::MissingMove)
}

fn append_quadratic(points: &mut Vec<[f64; 2]>, from: [f64; 2], control: [f64; 2], to: [f64; 2]) {
    for step in 1..=64 {
        let t = f64::from(step) / 64.0;
        let u = 1.0 - t;
        points.push([
            u * u * from[0] + 2.0 * u * t * control[0] + t * t * to[0],
            u * u * from[1] + 2.0 * u * t * control[1] + t * t * to[1],
        ]);
    }
}

fn append_cubic(
    points: &mut Vec<[f64; 2]>,
    from: [f64; 2],
    first: [f64; 2],
    second: [f64; 2],
    to: [f64; 2],
) {
    for step in 1..=64 {
        let t = f64::from(step) / 64.0;
        let u = 1.0 - t;
        points.push([
            u.powi(3) * from[0]
                + 3.0 * u * u * t * first[0]
                + 3.0 * u * t * t * second[0]
                + t.powi(3) * to[0],
            u.powi(3) * from[1]
                + 3.0 * u * u * t * first[1]
                + 3.0 * u * t * t * second[1]
                + t.powi(3) * to[1],
        ]);
    }
}

fn cross(from: [f64; 2], to: [f64; 2], point: [f64; 2]) -> f64 {
    (to[0] - from[0]) * (point[1] - from[1]) - (point[0] - from[0]) * (to[1] - from[1])
}

fn scaled(point: [i32; 2]) -> [f64; 2] {
    [
        f64::from(point[0]) / f64::from(PATH_SCALE_V2),
        f64::from(point[1]) / f64::from(PATH_SCALE_V2),
    ]
}

pub(super) fn detects(kind: PathHitFaultKindV2) -> bool {
    match kind {
        PathHitFaultKindV2::MissingMove => validate_layer(&PathLayerV2 {
            verbs: vec![PathVerbV2::Line([0, 0])],
            coverage: PathCoverageV2::Fill(FillRuleV2::NonZero),
        })
        .is_err(),
        PathHitFaultKindV2::OpenFillSubpath => validate_layer(&PathLayerV2 {
            verbs: vec![
                PathVerbV2::Move([0, 0]),
                PathVerbV2::Line([PATH_SCALE_V2, 0]),
            ],
            coverage: PathCoverageV2::Fill(FillRuleV2::NonZero),
        })
        .is_err(),
        PathHitFaultKindV2::NonFiniteCoordinate => !f64::NAN.is_finite(),
        PathHitFaultKindV2::TessellationLimit => 4097 > 4096,
        PathHitFaultKindV2::InvalidStrokeWidth => validate_layer(&PathLayerV2 {
            verbs: vec![PathVerbV2::Move([0, 0])],
            coverage: PathCoverageV2::RoundStroke { width: 0 },
        })
        .is_err(),
    }
}
