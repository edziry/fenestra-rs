use super::super::prepared_brush_support::color;
use super::super::validated_paint_support::{destination, source};
use super::super::world_aabb_support::aabb;
use super::super::world_transform_support::{SCALE, logical};
use super::support::{requested_limits, rich_engine, rich_owned};
use super::*;
use crate::brush::SpatialBrushKindV2;
use crate::content_item::SpatialInputPolicyV2;
use crate::coverage::{SpatialCoverageKindV2, SpatialFillRuleV2};
use crate::model::{SpatialPointV2, SpatialScalarV2};
use crate::paint::SpatialPaintKindV2;
use crate::shape::SpatialShapeKindV2;

#[test]
fn prepared_state_owns_normalized_resources_ranges_and_complete_item_plans() {
    let prepared = prepare_spatial_v2(&rich_engine(), rich_owned(), requested_limits())
        .expect("rich owned input prepares successfully");

    assert_eq!(prepared.path_range_facts(), vec![(0, 0, 2), (1, 2, 4)]);
    assert_eq!(prepared.validated_path_facts(), vec![(0, 2, 1), (1, 2, 1)]);
    assert_eq!(
        prepared.flattened_path_facts(),
        vec![
            (
                0,
                1,
                vec![(5 * SCALE, 5 * SCALE), (6 * SCALE, 5 * SCALE)],
                vec![(0, 2, false)],
            ),
            (1, 1, vec![(0, 0), (2 * SCALE, 0)], vec![(0, 2, false)],),
        ]
    );
    assert_eq!(prepared.polygon_range_facts(), vec![(1, 0, 3), (4, 3, 6)]);
    assert_eq!(
        prepared.shape_plan_facts(),
        vec![
            (
                0,
                1,
                SpatialShapeKindV2::Rect,
                None,
                0,
                aabb(SCALE, 2 * SCALE, 4 * SCALE, 6 * SCALE),
                aabb(SCALE, 2 * SCALE, 4 * SCALE, 6 * SCALE),
            ),
            (
                1,
                2,
                SpatialShapeKindV2::Polygon,
                None,
                3,
                aabb(-2 * SCALE, -SCALE, 4 * SCALE, 5 * SCALE),
                aabb(-2 * SCALE, -SCALE, 4 * SCALE, 5 * SCALE),
            ),
            (
                2,
                3,
                SpatialShapeKindV2::Path,
                Some(1),
                0,
                aabb(0, 0, 2 * SCALE, 0),
                aabb(0, 0, 2 * SCALE, 0),
            ),
            (
                3,
                1,
                SpatialShapeKindV2::Circle,
                None,
                0,
                aabb(0, 0, 10 * SCALE, 10 * SCALE),
                aabb(0, 0, 10 * SCALE, 10 * SCALE),
            ),
            (
                4,
                1,
                SpatialShapeKindV2::Polygon,
                None,
                3,
                aabb(20 * SCALE, 20 * SCALE, 22 * SCALE, 22 * SCALE),
                aabb(20 * SCALE, 20 * SCALE, 22 * SCALE, 22 * SCALE),
            ),
        ]
    );
    assert_eq!(prepared.gradient_range_facts(), vec![(1, 0, 2), (2, 2, 4)]);
    assert_eq!(
        prepared.prepared_brush_facts(),
        vec![
            (0, SpatialBrushKindV2::Solid, 0),
            (1, SpatialBrushKindV2::LinearGradient, 2),
            (2, SpatialBrushKindV2::LinearGradient, 2),
        ]
    );
    assert_eq!(prepared.prepared_solid_color(0), color(9, 48, 109, 137));
    assert_eq!(
        prepared.prepared_gradient_facts(1),
        (
            point(0, 0),
            point(1, 1),
            vec![(0, color(128, 0, 0, 128)), (u16::MAX, color(0, 0, 64, 64)),],
        )
    );
    assert_eq!(
        prepared.prepared_gradient_facts(2),
        (
            point(0, 0),
            point(1, 1),
            vec![
                (0, color(10, 20, 30, 255)),
                (u16::MAX, color(40, 50, 60, 255)),
            ],
        )
    );
    assert_eq!(
        prepared.image_plan_facts(),
        vec![(0, 1, 1, 4), (1, 2, 1, 8)]
    );
    assert_item_plans(&prepared);
}

fn assert_item_plans(prepared: &PreparedSpatialV2) {
    assert_eq!(
        prepared.validated_clip_facts(),
        vec![
            (0, 1, None, 0, SpatialFillRuleV2::NonZero, 1),
            (1, 1, Some(0), 3, SpatialFillRuleV2::EvenOdd, 2),
            (2, 1, Some(1), 3, SpatialFillRuleV2::NonZero, 3),
        ]
    );
    assert_eq!(
        prepared.validated_paint_facts(),
        vec![
            (0, 1, 0, SpatialPaintKindV2::CoveragePaint),
            (1, 1, 1, SpatialPaintKindV2::ImagePaint),
            (2, 2, 0, SpatialPaintKindV2::CoveragePaint),
        ]
    );
    assert_eq!(
        prepared.validated_fill_paint_facts(),
        vec![(0, 0, SpatialFillRuleV2::EvenOdd, 0, 173, Some(2))]
    );
    assert_eq!(
        prepared.validated_stroke_paint_facts(),
        vec![(2, 1, 2 * SCALE, 1, 173, None)]
    );
    assert_eq!(
        prepared.validated_image_paint_facts(),
        vec![(
            1,
            1,
            source(0, 0, 1, 1),
            destination(logical(10), logical(20), logical(3), logical(4)),
            211,
            None,
        )]
    );
    assert_eq!(
        prepared.finalized_image_paint_facts(),
        vec![(
            1,
            source(0, 0, 1, 1),
            destination(logical(10), logical(20), logical(3), logical(4)),
            211,
            aabb(10 * SCALE, 20 * SCALE, 13 * SCALE, 24 * SCALE),
        )]
    );
    assert_eq!(
        prepared.validated_hit_facts(),
        vec![
            (
                0,
                2,
                0,
                SpatialCoverageKindV2::Fill,
                SpatialInputPolicyV2::Accept,
                Some(0)
            ),
            (
                1,
                3,
                0,
                SpatialCoverageKindV2::RoundStroke,
                SpatialInputPolicyV2::Ignore,
                Some(1)
            ),
            (
                2,
                3,
                1,
                SpatialCoverageKindV2::RoundStroke,
                SpatialInputPolicyV2::Accept,
                Some(2)
            ),
        ]
    );
    assert_eq!(
        prepared.validated_fill_hit_facts(),
        vec![(0, 1, SpatialFillRuleV2::NonZero)]
    );
    assert_eq!(
        prepared.validated_stroke_hit_facts(),
        vec![(1, 2, 2 * SCALE), (2, 2, 4 * SCALE)]
    );
    assert_eq!(
        prepared.validated_semantic_facts(),
        vec![
            (0, 3, 0, 2, SpatialFillRuleV2::EvenOdd, Some(2)),
            (1, 3, 1, 2, SpatialFillRuleV2::NonZero, Some(1)),
        ]
    );

    let source_bytes = prepared.source_arc().as_input().resources().images()[1].bytes();
    let image_bytes = prepared
        .finalized_image_paint_bytes(1)
        .expect("P5 image key resolves through the retained source");
    assert_eq!(image_bytes.as_ptr(), source_bytes.as_ptr());
    assert_eq!(image_bytes.len(), source_bytes.len());
}

const fn point(x: i64, y: i64) -> SpatialPointV2 {
    SpatialPointV2::new(SpatialScalarV2::new(x), SpatialScalarV2::new(y))
}
