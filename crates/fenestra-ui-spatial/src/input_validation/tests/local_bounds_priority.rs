use super::flattened_path_support::{DEPTH_16_NONFLAT_HEIGHT, expect_nonflat, path, quadratic};
use super::local_bounds_support::{expect_bounds_error, fixture, limits, validate};
use super::local_transform_support::VIEWPORT;
use super::validated_hit_support::stroke as hit_stroke;
use super::validated_paint_support::{destination, fill, image_paint, source, stroke};
use super::validated_shape_support::{circle_values, rect_values};
use crate::content_item::SpatialInputPolicyV2;
use crate::coverage::SpatialFillRuleV2;
use crate::error::SpatialErrorLocationV2;
use crate::geometry_field::SpatialShapeFieldV2;
use crate::item_field::{SpatialHitFieldV2, SpatialPaintFieldV2};
use crate::model::SpatialScalarV2;
use crate::vocabulary::SpatialAxisV2;

#[test]
fn every_shape_finishes_before_any_paint_or_hit_bound() {
    let maximum = SpatialScalarV2::MAX_RAW;
    let shapes = vec![
        rect_values(0, 1, maximum, maximum, 0, 0),
        rect_values(1, 1, 0, maximum, 0, 1),
    ];
    let paints = vec![stroke(1, 0, 0, 1, 0, None)];
    let hits = vec![hit_stroke(1, 0, 0, 1, None, SpatialInputPolicyV2::Accept)];
    let fixture = fixture(shapes, Vec::new(), paints, hits);

    expect_bounds_error(
        validate(&fixture),
        SpatialAxisV2::Y,
        shape_location(1, SpatialShapeFieldV2::RectHeight),
    );
}

#[test]
fn shape_records_complete_before_later_variants_in_both_directions() {
    let maximum = SpatialScalarV2::MAX_RAW;
    let rect_first = fixture(
        vec![
            rect_values(0, 1, 0, maximum, 0, 1),
            circle_values(1, 1, maximum, maximum, 1),
        ],
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    expect_bounds_error(
        validate(&rect_first),
        SpatialAxisV2::Y,
        shape_location(0, SpatialShapeFieldV2::RectHeight),
    );

    let circle_first = fixture(
        vec![
            circle_values(0, 1, 0, maximum, 1),
            rect_values(1, 1, maximum, maximum, 1, 1),
        ],
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    expect_bounds_error(
        validate(&circle_first),
        SpatialAxisV2::Y,
        shape_location(0, SpatialShapeFieldV2::CircleRadius),
    );
}

#[test]
fn paint_records_complete_image_then_stroke_and_stroke_then_image() {
    let maximum = SpatialScalarV2::MAX_RAW;
    let shapes = vec![
        rect_values(0, 1, 0, maximum, 0, 0),
        rect_values(1, 1, maximum, maximum, 0, 0),
    ];
    let image_first = fixture(
        shapes.clone(),
        Vec::new(),
        vec![
            image_paint(
                1,
                0,
                0,
                source(0, 0, 1, 1),
                destination(0, maximum, 1, 1),
                None,
            ),
            stroke(1, 1, 1, 1, 0, None),
        ],
        Vec::new(),
    );
    expect_bounds_error(
        validate(&image_first),
        SpatialAxisV2::Y,
        paint_location(0, SpatialPaintFieldV2::DestinationHeight),
    );

    let stroke_first = fixture(
        shapes,
        Vec::new(),
        vec![
            stroke(1, 0, 0, 1, 0, None),
            image_paint(
                1,
                1,
                0,
                source(0, 0, 1, 1),
                destination(maximum, maximum, 1, 1),
                None,
            ),
        ],
        Vec::new(),
    );
    expect_bounds_error(
        validate(&stroke_first),
        SpatialAxisV2::Y,
        paint_location(0, SpatialPaintFieldV2::StrokeWidth),
    );
}

#[test]
fn the_complete_paint_table_precedes_the_first_hit_bound() {
    let maximum = SpatialScalarV2::MAX_RAW;
    let shapes = vec![
        rect_values(0, 1, 0, 0, 1, 1),
        rect_values(1, 1, 0, maximum, 0, 0),
        rect_values(2, 1, maximum, maximum, 0, 0),
    ];
    let paints = vec![
        fill(1, 0, 0, 0, None, SpatialFillRuleV2::NonZero),
        stroke(1, 1, 1, 1, 0, None),
    ];
    let hits = vec![hit_stroke(1, 0, 2, 1, None, SpatialInputPolicyV2::Accept)];
    let fixture = fixture(shapes, Vec::new(), paints, hits);

    expect_bounds_error(
        validate(&fixture),
        SpatialAxisV2::Y,
        paint_location(1, SpatialPaintFieldV2::StrokeWidth),
    );
}

#[test]
fn hit_records_complete_before_a_later_x_failure() {
    let maximum = SpatialScalarV2::MAX_RAW;
    let shapes = vec![
        rect_values(0, 1, 0, maximum, 0, 0),
        rect_values(1, 1, maximum, maximum, 0, 0),
    ];
    let hits = vec![
        hit_stroke(1, 0, 0, 1, None, SpatialInputPolicyV2::Accept),
        hit_stroke(1, 1, 1, 1, None, SpatialInputPolicyV2::Ignore),
    ];
    let fixture = fixture(shapes, Vec::new(), Vec::new(), hits);

    expect_bounds_error(
        validate(&fixture),
        SpatialAxisV2::Y,
        hit_location(0, SpatialHitFieldV2::StrokeWidth),
    );
}

#[test]
fn complete_k2_flattening_precedes_every_local_bound_pass() {
    let maximum = SpatialScalarV2::MAX_RAW;
    let fixture = fixture(
        vec![rect_values(0, 1, maximum, maximum, 1, 1)],
        Vec::new(),
        Vec::new(),
        Vec::new(),
    )
    .with_paths(
        vec![path(0, 0, 2)],
        quadratic(DEPTH_16_NONFLAT_HEIGHT).to_vec(),
    );

    expect_nonflat(prepare_local_bounds!(&fixture, VIEWPORT, limits()), 0, 1);
}

const fn shape_location(index: u32, field: SpatialShapeFieldV2) -> SpatialErrorLocationV2 {
    SpatialErrorLocationV2::Shape { index, field }
}

const fn paint_location(index: u32, field: SpatialPaintFieldV2) -> SpatialErrorLocationV2 {
    SpatialErrorLocationV2::Paint { index, field }
}

const fn hit_location(index: u32, field: SpatialHitFieldV2) -> SpatialErrorLocationV2 {
    SpatialErrorLocationV2::Hit { index, field }
}
