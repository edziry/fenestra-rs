use kurbo::{
    BezPath, Cap, Join, ParamCurveNearest, PathEl, Point, Shape, Stroke, StrokeOpts, stroke,
};

use super::super::types::{
    FillRuleV2, PATH_SCALE_V2, PathCoverageV2, PathHitFaultKindV2, PathHitResultV2, PathLayerV2,
    PathVerbV2,
};
use super::detects_with;

pub(super) fn hit(layer: &PathLayerV2, query: [i32; 2]) -> PathHitResultV2<bool> {
    let path = build(&layer.verbs)?;
    let point = point(query);
    let winding = match layer.coverage {
        PathCoverageV2::Fill(_) => path.winding(point),
        PathCoverageV2::RoundStroke { width } => {
            let style = Stroke::new(scalar(width))
                .with_join(Join::Round)
                .with_caps(Cap::Round);
            stroke(
                path.elements().iter().copied(),
                &style,
                &StrokeOpts::default(),
                0.001,
            )
            .winding(point)
        }
    };
    Ok(match layer.coverage {
        PathCoverageV2::Fill(_) if on_boundary(&path, point) => true,
        PathCoverageV2::Fill(FillRuleV2::EvenOdd) => winding.unsigned_abs() % 2 == 1,
        PathCoverageV2::Fill(FillRuleV2::NonZero) | PathCoverageV2::RoundStroke { .. } => {
            winding != 0
        }
    })
}

fn on_boundary(path: &BezPath, point: Point) -> bool {
    path.segments()
        .any(|segment| segment.nearest(point, 1.0e-9).distance_sq <= 1.0e-12)
}

fn build(verbs: &[PathVerbV2]) -> PathHitResultV2<BezPath> {
    let mut path = BezPath::new();
    for verb in verbs {
        match *verb {
            PathVerbV2::Move(to) => path.push(PathEl::MoveTo(point(to))),
            PathVerbV2::Line(to) => path.push(PathEl::LineTo(point(to))),
            PathVerbV2::Quadratic(control, to) => {
                path.push(PathEl::QuadTo(point(control), point(to)));
            }
            PathVerbV2::Cubic(first, second, to) => {
                path.push(PathEl::CurveTo(point(first), point(second), point(to)));
            }
            PathVerbV2::Close => path.push(PathEl::ClosePath),
        }
    }
    Ok(path)
}

fn point(value: [i32; 2]) -> Point {
    Point::new(scalar(value[0]), scalar(value[1]))
}

fn scalar(value: i32) -> f64 {
    f64::from(value) / f64::from(PATH_SCALE_V2)
}

pub(super) fn detects(kind: PathHitFaultKindV2) -> bool {
    let path = BezPath::from_vec(vec![
        PathEl::MoveTo(Point::ZERO),
        PathEl::LineTo(Point::new(1.0, 0.0)),
        PathEl::LineTo(Point::new(0.0, 1.0)),
        PathEl::ClosePath,
    ]);
    detects_with(
        kind,
        !f64::NAN.is_finite(),
        path.winding(Point::new(0.2, 0.2)) != 0,
    )
}
