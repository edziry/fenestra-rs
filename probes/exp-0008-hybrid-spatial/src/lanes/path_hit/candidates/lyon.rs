use lyon_tessellation::geometry_builder::{VertexBuffers, simple_builder};
use lyon_tessellation::math::{Point, point};
use lyon_tessellation::path::Path;
use lyon_tessellation::{
    FillOptions, FillTessellator, LineCap, LineJoin, StrokeOptions, StrokeTessellator,
};

use super::super::types::{
    FillRuleV2, PATH_SCALE_V2, PathCoverageV2, PathHitFaultKindV2, PathHitResultV2, PathLayerV2,
    PathVerbV2,
};
use super::detects_with;

pub(super) fn hit(layer: &PathLayerV2, query: [i32; 2]) -> PathHitResultV2<bool> {
    let path = build(&layer.verbs)?;
    let mut geometry = VertexBuffers::<Point, u16>::new();
    match layer.coverage {
        PathCoverageV2::Fill(rule) => {
            let options = match rule {
                FillRuleV2::NonZero => FillOptions::non_zero(),
                FillRuleV2::EvenOdd => FillOptions::even_odd(),
            }
            .with_tolerance(0.001);
            FillTessellator::new()
                .tessellate_path(&path, &options, &mut simple_builder(&mut geometry))
                .map_err(|_| PathHitFaultKindV2::TessellationLimit)?;
        }
        PathCoverageV2::RoundStroke { width } => {
            let options = StrokeOptions::default()
                .with_line_width(scalar(width))
                .with_line_cap(LineCap::Round)
                .with_line_join(LineJoin::Round)
                .with_tolerance(0.001);
            StrokeTessellator::new()
                .tessellate_path(&path, &options, &mut simple_builder(&mut geometry))
                .map_err(|_| PathHitFaultKindV2::TessellationLimit)?;
        }
    }
    let query = point(scalar(query[0]), scalar(query[1]));
    Ok(geometry.indices.chunks_exact(3).any(|triangle| {
        contains(
            geometry.vertices[usize::from(triangle[0])],
            geometry.vertices[usize::from(triangle[1])],
            geometry.vertices[usize::from(triangle[2])],
            query,
        )
    }))
}

fn build(verbs: &[PathVerbV2]) -> PathHitResultV2<Path> {
    let mut builder = Path::builder();
    let mut open = false;
    for verb in verbs {
        match *verb {
            PathVerbV2::Move(to) => {
                if open {
                    builder.end(false);
                }
                builder.begin(to_point(to));
                open = true;
            }
            PathVerbV2::Line(to) => {
                builder.line_to(to_point(to));
            }
            PathVerbV2::Quadratic(control, to) => {
                builder.quadratic_bezier_to(to_point(control), to_point(to));
            }
            PathVerbV2::Cubic(first, second, to) => {
                builder.cubic_bezier_to(to_point(first), to_point(second), to_point(to));
            }
            PathVerbV2::Close => {
                builder.end(true);
                open = false;
            }
        }
    }
    if open {
        builder.end(false);
    }
    Ok(builder.build())
}

fn contains(a: Point, b: Point, c: Point, point: Point) -> bool {
    let first = cross(a, b, point);
    let second = cross(b, c, point);
    let third = cross(c, a, point);
    let negative = first < -1.0e-5 || second < -1.0e-5 || third < -1.0e-5;
    let positive = first > 1.0e-5 || second > 1.0e-5 || third > 1.0e-5;
    !(negative && positive)
}

fn cross(a: Point, b: Point, point: Point) -> f32 {
    (b.x - a.x) * (point.y - a.y) - (point.x - a.x) * (b.y - a.y)
}

fn to_point(value: [i32; 2]) -> Point {
    point(scalar(value[0]), scalar(value[1]))
}

fn scalar(value: i32) -> f32 {
    value as f32 / PATH_SCALE_V2 as f32
}

pub(super) fn detects(kind: PathHitFaultKindV2) -> bool {
    let mut builder = Path::builder();
    builder.begin(point(0.0, 0.0));
    builder.line_to(point(1.0, 0.0));
    builder.line_to(point(0.0, 1.0));
    builder.end(true);
    let path = builder.build();
    let mut geometry = VertexBuffers::<Point, u16>::new();
    let valid = FillTessellator::new()
        .tessellate_path(
            &path,
            &FillOptions::non_zero(),
            &mut simple_builder(&mut geometry),
        )
        .is_ok()
        && !geometry.indices.is_empty();
    detects_with(kind, !f32::NAN.is_finite(), valid)
}
