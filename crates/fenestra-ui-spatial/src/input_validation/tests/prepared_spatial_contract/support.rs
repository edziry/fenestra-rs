use std::sync::Arc;

use fenestra_ui_layout::prototype::{
    LayoutEngineErrorKindV1, LayoutEngineErrorV1, LayoutErrorLocationV1,
};

use super::super::fixture::RawInputFixture;
use super::super::validated_hit_support::{fill as hit_fill, stroke as hit_stroke};
use super::super::validated_paint_support::{
    destination, fill, image_paint, source, stroke as paint_stroke,
};
use super::super::validated_semantic_support::semantic;
use super::super::world_aabb_support::{MAXIMUM, owner_node, transform};
use super::super::world_transform_support::{
    SCALE, ScriptedLayoutEngine, VIEWPORT, free, identity, limits, logical, output, root,
};
use crate::content_item::SpatialInputPolicyV2;
use crate::coverage::SpatialFillRuleV2;
use crate::limits::SpatialLimitsV2;
use crate::model::SpatialAnchorTargetV2;
use crate::owned_input::SpatialOwnedInputV2;

pub(super) fn direct_limit_owned() -> Arc<SpatialOwnedInputV2> {
    Arc::new(
        RawInputFixture::with_nodes(vec![super::super::world_transform_support::root()])
            .into_owned(VIEWPORT),
    )
}

pub(super) fn direct_limit_limits() -> SpatialLimitsV2 {
    let mut direct = [usize::MAX; super::super::DIRECT_COUNT];
    direct[0] = 0;
    super::super::limits_with_direct(direct)
}

pub(super) fn layout_failure_owned() -> Arc<SpatialOwnedInputV2> {
    Arc::new(
        super::super::world_transform_support::fixture(vec![
            super::super::world_transform_support::root(),
            super::super::world_transform_support::layout(1, 0, 1, 1, identity()),
        ])
        .into_owned(VIEWPORT),
    )
}

pub(super) fn late_failure_owned() -> Arc<SpatialOwnedInputV2> {
    let nodes = vec![
        super::super::world_transform_support::root(),
        super::super::world_transform_support::layout(1, 0, 1, 1, identity()),
        owner_node(2, transform([MAXIMUM, 0, 0, 65_536, 0, 0], 0, 0), 2, 1),
    ];
    Arc::new(
        super::super::world_aabb_support::fixture_with(
            nodes,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        )
        .into_owned(VIEWPORT),
    )
}

pub(super) fn rich_owned() -> Arc<SpatialOwnedInputV2> {
    Arc::new(rich_fixture().into_owned(VIEWPORT))
}

pub(super) fn root_only_owned() -> Arc<SpatialOwnedInputV2> {
    Arc::new(
        super::super::dependency_support::fixture(vec![super::super::dependency_support::root()])
            .into_owned(VIEWPORT),
    )
}

pub(super) fn distinct_viewport_root_owned() -> Arc<SpatialOwnedInputV2> {
    Arc::new(
        super::super::dependency_support::fixture(vec![super::super::dependency_support::root()])
            .into_owned(crate::model::SpatialViewportV2::new(7, 9)),
    )
}

pub(super) fn computed_layout_owned() -> Arc<SpatialOwnedInputV2> {
    Arc::new(
        super::super::world_transform_support::fixture(vec![
            root(),
            super::super::world_transform_support::layout(1, 0, 3, 4, identity()),
        ])
        .into_owned(VIEWPORT),
    )
}

pub(super) fn computed_layout_engine() -> ScriptedLayoutEngine {
    ScriptedLayoutEngine::new(vec![Ok(output(&[(0, 0, 0, 20, 20), (1, 7, 8, 5, 6)]))])
}

pub(super) fn cross_axis_empty_owned() -> Arc<SpatialOwnedInputV2> {
    let q = 1_i64 << 32;
    let nodes = vec![
        root(),
        free(
            1,
            0,
            SpatialAnchorTargetV2::Viewport,
            3 * SCALE,
            4 * SCALE,
            0,
            0,
            transform([0, q, q, 0, 5 * SCALE, 7 * SCALE], 0, 0),
        ),
    ];
    Arc::new(
        super::super::world_aabb_support::fixture_with(
            nodes,
            vec![super::super::validated_shape_support::rect_values(
                0, 1, 0, 0, 0, SCALE,
            )],
            vec![super::super::validated_clip_support::root_clip(0, 1, 0)],
            vec![fill(1, 0, 0, 0, Some(0), SpatialFillRuleV2::NonZero)],
            vec![hit_fill(
                1,
                0,
                0,
                Some(0),
                SpatialFillRuleV2::EvenOdd,
                SpatialInputPolicyV2::Accept,
            )],
            vec![semantic(1, 0, 0, SpatialFillRuleV2::NonZero, Some(0))],
        )
        .into_owned(VIEWPORT),
    )
}

pub(super) fn disjoint_clips_owned() -> Arc<SpatialOwnedInputV2> {
    let fixture = super::super::effective_clip_aabb_support::clips_only_fixture(
        vec![
            super::super::effective_clip_aabb_support::rect_values(
                0,
                1,
                0,
                0,
                2 * SCALE,
                2 * SCALE,
            ),
            super::super::effective_clip_aabb_support::rect_values(
                1,
                1,
                10 * SCALE,
                10 * SCALE,
                SCALE,
                SCALE,
            ),
        ],
        vec![
            super::super::effective_clip_aabb_support::nonzero_clip(0, 1, None, 0),
            super::super::effective_clip_aabb_support::even_odd_clip(1, 1, Some(0), 1),
        ],
    );
    Arc::new(fixture.into_owned(VIEWPORT))
}

pub(super) fn rich_engine() -> ScriptedLayoutEngine {
    ScriptedLayoutEngine::new(vec![Ok(output(&[(0, 0, 0, 10, 10), (1, 1, 2, 3, 4)]))])
}

pub(super) fn successful_layout_engine() -> ScriptedLayoutEngine {
    ScriptedLayoutEngine::new(vec![Ok(output(&[(0, 0, 0, 20, 20), (1, 0, 0, 1, 1)]))])
}

pub(super) fn zero_call_engine() -> ScriptedLayoutEngine {
    ScriptedLayoutEngine::new(Vec::new())
}

pub(super) fn rejected_layout_engine() -> ScriptedLayoutEngine {
    ScriptedLayoutEngine::new(vec![Err(LayoutEngineErrorV1::new(
        LayoutEngineErrorKindV1::RejectedInput,
        LayoutErrorLocationV1::InputNode { index: 1 },
    ))])
}

pub(super) fn requested_limits() -> SpatialLimitsV2 {
    limits()
}

fn rich_fixture() -> RawInputFixture {
    let paints = vec![
        fill(1, 0, 0, 0, Some(2), SpatialFillRuleV2::EvenOdd),
        image_paint(
            1,
            1,
            1,
            source(0, 0, 1, 1),
            destination(logical(10), logical(20), logical(3), logical(4)),
            None,
        ),
        paint_stroke(2, 0, 1, 2 * SCALE, 1, None),
    ];

    super::super::effective_clip_aabb_support::retained_prepared_fixture(
        paints,
        vec![
            hit_fill(
                2,
                0,
                1,
                Some(0),
                SpatialFillRuleV2::NonZero,
                SpatialInputPolicyV2::Accept,
            ),
            hit_stroke(3, 0, 2, 2 * SCALE, Some(1), SpatialInputPolicyV2::Ignore),
            hit_stroke(3, 1, 2, 4 * SCALE, Some(2), SpatialInputPolicyV2::Accept),
        ],
        vec![
            semantic(3, 0, 2, SpatialFillRuleV2::EvenOdd, Some(2)),
            semantic(3, 1, 2, SpatialFillRuleV2::NonZero, Some(1)),
        ],
    )
}
