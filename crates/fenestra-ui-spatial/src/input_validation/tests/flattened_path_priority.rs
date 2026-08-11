use super::flattened_path_support::{
    DEPTH_16_NONFLAT_HEIGHT, expect_limit, expect_nonflat, fixture, limits, line_to, move_to, path,
    point, quadratic, validate,
};
use super::local_transform_support::VIEWPORT;
use super::validated_hit_support::stroke as hit_stroke;
use super::validated_paint_support::{destination, image_paint, source};
use super::validated_semantic_support::{fixture_with_items, semantic};
use super::validated_shape_support::{path_shape, rect, rect_values};
use crate::content_diagnostic::SpatialContentReferenceV2;
use crate::content_item::SpatialInputPolicyV2;
use crate::coverage::SpatialFillRuleV2;
use crate::item_field::SpatialSemanticFieldV2;
use crate::limits::SpatialLimitKindV2;
use crate::model::SpatialScalarV2;
use crate::path::SpatialPathVerbV2;

#[test]
fn paths_are_completed_record_major_in_both_failure_directions() {
    let mut later_nonflat = vec![move_to(0, 0), line_to(1, 0), line_to(2, 0)];
    later_nonflat.extend(quadratic(DEPTH_16_NONFLAT_HEIGHT));
    let later_fixture = fixture(vec![path(0, 0, 3), path(1, 3, 2)], later_nonflat);
    expect_limit(
        validate(&later_fixture, limits(1, usize::MAX)),
        SpatialLimitKindV2::FlattenedSegmentsPerPath,
        0,
        2,
        2,
        1,
    );

    let mut earlier_nonflat = quadratic(DEPTH_16_NONFLAT_HEIGHT).to_vec();
    earlier_nonflat.extend([move_to(0, 0), line_to(1, 0), line_to(2, 0)]);
    let earlier_fixture = fixture(vec![path(0, 0, 2), path(1, 2, 3)], earlier_nonflat);
    expect_nonflat(validate(&earlier_fixture, limits(1, usize::MAX)), 0, 1);
}

#[test]
fn a_prior_total_failure_beats_a_later_per_path_failure() {
    let used_fixture = fixture(
        vec![path(0, 0, 2), path(1, 2, 3)],
        vec![
            move_to(0, 0),
            line_to(1, 0),
            move_to(2, 0),
            line_to(3, 0),
            line_to(4, 0),
        ],
    );
    expect_limit(
        validate(&used_fixture, limits(1, 0)),
        SpatialLimitKindV2::FlattenedSegmentsTotal,
        0,
        1,
        1,
        0,
    );
}

#[test]
fn source_verbs_keep_line_before_nonflat_and_nonflat_before_later_line() {
    let height = DEPTH_16_NONFLAT_HEIGHT;
    let line_first = fixture(
        vec![path(0, 0, 3)],
        vec![
            move_to(-height, 0),
            line_to(-height, 0),
            SpatialPathVerbV2::QuadraticTo {
                control: point(0, height),
                to: point(height, 0),
            },
        ],
    );
    expect_limit(
        validate(&line_first, limits(0, usize::MAX)),
        SpatialLimitKindV2::FlattenedSegmentsPerPath,
        0,
        1,
        1,
        0,
    );

    let curve_first = fixture(
        vec![path(0, 0, 3)],
        vec![
            move_to(-height, 0),
            SpatialPathVerbV2::QuadraticTo {
                control: point(0, height),
                to: point(height, 0),
            },
            line_to(height, 0),
        ],
    );
    expect_nonflat(validate(&curve_first, limits(0, usize::MAX)), 0, 1);
}

#[test]
fn every_declared_path_flattens_in_key_order_not_shape_use_order() {
    let mut verbs = vec![move_to(0, 0), line_to(1, 0), line_to(2, 0)];
    verbs.extend(quadratic(DEPTH_16_NONFLAT_HEIGHT));
    let used_fixture = fixture(vec![path(0, 0, 3), path(1, 3, 2)], verbs)
        .with_shapes(vec![path_shape(0, 1, 1), path_shape(1, 2, 0)], Vec::new());
    expect_limit(
        validate(&used_fixture, limits(1, usize::MAX)),
        SpatialLimitKindV2::FlattenedSegmentsPerPath,
        0,
        2,
        2,
        1,
    );

    let unreferenced = fixture(
        vec![path(0, 0, 2)],
        quadratic(DEPTH_16_NONFLAT_HEIGHT).to_vec(),
    );
    expect_nonflat(
        validate(&unreferenced, limits(usize::MAX, usize::MAX)),
        0,
        1,
    );
}

#[test]
fn every_item_stage_precedes_k2_flattening() {
    let fixture = fixture_with_items(
        Vec::new(),
        Vec::new(),
        vec![semantic(1, 0, 0, SpatialFillRuleV2::EvenOdd, Some(99))],
    )
    .with_paths(
        vec![path(0, 0, 2)],
        quadratic(DEPTH_16_NONFLAT_HEIGHT).to_vec(),
    );
    super::validated_semantic_support::expect_reference(
        prepare_flattened_paths!(&fixture, VIEWPORT, limits(0, 0)),
        SpatialContentReferenceV2::Clip,
        0,
        SpatialSemanticFieldV2::Clip,
    );
}

#[test]
fn k2_failure_precedes_base_stroke_and_image_bounds() {
    let maximum = SpatialScalarV2::MAX_RAW;
    let paints = vec![image_paint(
        1,
        0,
        0,
        source(0, 0, 1, 1),
        destination(maximum, maximum, maximum, maximum),
        None,
    )];
    let hits = vec![hit_stroke(
        1,
        0,
        0,
        maximum,
        None,
        SpatialInputPolicyV2::Ignore,
    )];
    let semantics = vec![semantic(1, 0, 0, SpatialFillRuleV2::NonZero, None)];
    let fixture = fixture_with_items(paints, hits, semantics)
        .with_paths(vec![path(0, 0, 2)], vec![move_to(0, 0), line_to(1, 1)])
        .with_shapes(
            vec![
                rect_values(0, 1, maximum, maximum, maximum, maximum),
                rect(1, 2),
                rect(2, 3),
                rect(3, 4),
            ],
            Vec::new(),
        );
    expect_limit(
        validate(&fixture, limits(0, 0)),
        SpatialLimitKindV2::FlattenedSegmentsPerPath,
        0,
        1,
        1,
        0,
    );
}
