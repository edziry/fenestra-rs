//! Lifetime-free path, shape, brush, and image extraction.

use std::ops::Range;

use super::super::model::{
    PreparedBrushContent, PreparedBrushPlan, PreparedImagePlan, PreparedPathPlan,
    PreparedShapeGeometry, PreparedShapePlan,
};
use crate::geometry_kernel::{
    DerivedLocalBoundsK3, FlattenedPathK2, ValidatedPathK1, fill_bounds_k3,
};
use crate::input_validation::brush_structure::GradientRange;
use crate::input_validation::prepared_brushes::PreparedBrush;
use crate::input_validation::shape_structure::PolygonRange;
use crate::input_validation::validated_shapes::{ValidatedShape, ValidatedShapeGeometry};
use crate::paint_kernel::ValidatedImageP4;

pub(super) fn extract_paths(
    ranges: Vec<Range<usize>>,
    validated: Vec<ValidatedPathK1<'_>>,
    flattened: Vec<FlattenedPathK2>,
) -> Box<[PreparedPathPlan]> {
    assert_eq!(
        ranges.len(),
        validated.len(),
        "validated paths remain key aligned"
    );
    assert_eq!(
        ranges.len(),
        flattened.len(),
        "flattened paths remain key aligned"
    );
    ranges
        .into_iter()
        .zip(validated)
        .zip(flattened)
        .map(|((verb_range, validated), flattened)| PreparedPathPlan {
            verb_count: validated.verbs().len(),
            subpath_count: validated.subpath_count(),
            verb_range,
            flattened,
        })
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

pub(super) fn extract_shapes(
    validated: Vec<ValidatedShape<'_>>,
    polygon_ranges: Vec<PolygonRange>,
    bounds: Vec<DerivedLocalBoundsK3>,
) -> Box<[PreparedShapePlan]> {
    assert_eq!(
        validated.len(),
        bounds.len(),
        "shape bounds remain key aligned"
    );
    let mut ranges = vec![None; validated.len()];
    for range in polygon_ranges {
        let (shape, points) = range.into_parts();
        let slot = ranges
            .get_mut(trusted_index(shape, "polygon shape"))
            .expect("validated polygon ranges reference existing shapes");
        assert!(
            slot.replace(points).is_none(),
            "each polygon has one trusted range"
        );
    }

    validated
        .into_iter()
        .zip(bounds)
        .enumerate()
        .map(|(index, (shape, bounds))| {
            let (owner, geometry) = shape.into_parts();
            let geometry = match geometry {
                ValidatedShapeGeometry::Rect(rect) => PreparedShapeGeometry::Rect {
                    origin: rect.origin(),
                    width: rect.width(),
                    height: rect.height(),
                },
                ValidatedShapeGeometry::Circle(circle) => PreparedShapeGeometry::Circle {
                    center: circle.center(),
                    radius: circle.radius(),
                },
                ValidatedShapeGeometry::Polygon(polygon) => {
                    let point_range = ranges[index]
                        .take()
                        .expect("every validated polygon retained its trusted point range");
                    assert_eq!(
                        point_range.len(),
                        polygon.points().len(),
                        "polygon proof and trusted range remain aligned"
                    );
                    PreparedShapeGeometry::Polygon { point_range }
                }
                ValidatedShapeGeometry::Path(path) => {
                    PreparedShapeGeometry::Path { path: path.get() }
                }
            };
            PreparedShapePlan {
                owner,
                geometry,
                base_bounds: bounds.base_bounds(),
                fill_clip_bounds: fill_bounds_k3(&bounds),
            }
        })
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

pub(super) fn extract_brushes(
    prepared: Vec<PreparedBrush>,
    gradient_ranges: Vec<GradientRange>,
) -> Box<[PreparedBrushPlan]> {
    let mut ranges = vec![None; prepared.len()];
    for range in gradient_ranges {
        let (brush, stops) = range.into_parts();
        let slot = ranges
            .get_mut(trusted_index(brush, "gradient brush"))
            .expect("validated gradient ranges reference existing brushes");
        assert!(
            slot.replace(stops).is_none(),
            "each gradient has one trusted range"
        );
    }

    prepared
        .into_iter()
        .enumerate()
        .map(|(index, brush)| match brush {
            PreparedBrush::Solid(color) => {
                assert!(
                    ranges[index].is_none(),
                    "solid brushes have no gradient range"
                );
                PreparedBrushPlan {
                    gradient_range: None,
                    content: PreparedBrushContent::Solid(color),
                }
            }
            PreparedBrush::LinearGradient(gradient) => {
                let gradient_range = ranges[index]
                    .take()
                    .expect("every prepared gradient retained its trusted stop range");
                let (start, end, stops) = gradient.into_parts();
                assert_eq!(
                    gradient_range.len(),
                    stops.len(),
                    "prepared gradient and trusted range remain aligned"
                );
                PreparedBrushPlan {
                    gradient_range: Some(gradient_range),
                    content: PreparedBrushContent::LinearGradient {
                        start,
                        end,
                        stops: stops.into_boxed_slice(),
                    },
                }
            }
        })
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

pub(super) fn extract_images(images: Vec<ValidatedImageP4<'_>>) -> Box<[PreparedImagePlan]> {
    images
        .into_iter()
        .enumerate()
        .map(|(index, image)| {
            let (key, width, height, stride) = image.into_parts();
            assert_eq!(
                trusted_index(key, "image"),
                index,
                "validated images remain key aligned"
            );
            PreparedImagePlan {
                width,
                height,
                stride,
            }
        })
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

fn trusted_index(key: u32, table: &str) -> usize {
    usize::try_from(key).unwrap_or_else(|_| panic!("validated {table} key fits usize"))
}
