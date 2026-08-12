use super::raster_support::*;
use super::validator_support::{ClipRow, GeometryRow, PaintRow, validate};
use crate::coverage::SpatialFillRuleV2;

const D: i128 = (S as i128) * (S as i128);

#[test]
fn accepted_paint_and_clip_rows_each_supply_distinct_authoritative_projection() {
    use super::super::validated_clip_support::root_clip;
    use super::super::validated_shape_support::rect_values;
    use super::super::world_transform_support::{free, identity, root};
    use crate::model::SpatialAnchorTargetV2;

    let nodes = vec![
        root(),
        free(
            1,
            0,
            SpatialAnchorTargetV2::Viewport,
            0,
            0,
            4,
            4,
            identity(),
        ),
        free(
            2,
            1,
            SpatialAnchorTargetV2::Viewport,
            0,
            0,
            4,
            4,
            identity(),
        ),
    ];
    let source = owned_fixture(
        viewport(1, 1),
        nodes,
        vec![rect_values(0, 1, 0, 0, S, S), rect_values(1, 2, 0, 0, S, S)],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![solid(0, color(255, 0, 0, 255))],
        Vec::new(),
        Vec::new(),
        vec![root_clip(0, 1, 0)],
        vec![fill(2, 0, 1, 0, 255, Some(0), SpatialFillRuleV2::NonZero)],
    );
    let (prepared, mut rows) = candidate_case(source);
    for (index, tx) in [(1, S / 4), (2, S / 2)] {
        let mut geometry = GeometryRow::read(rows.geometry[index]);
        geometry.world = [S / 2, 0, 0, S, tx, 0];
        geometry.determinant = D / 2;
        geometry.aabb = (false, [tx, 0, tx + 2 * S, 4 * S]);
        rows.geometry[index] = geometry.build();
    }
    let mut clip = ClipRow::read(rows.clips[0]);
    clip.world = [S / 2, 0, 0, S, S / 4, 0];
    clip.determinant = D / 2;
    clip.aabb = (false, [S / 4, 0, 3 * S / 4, S]);
    rows.clips[0] = clip.build();
    let mut paint = PaintRow::read(rows.paints[0]);
    paint.world = [S / 2, 0, 0, S, S / 2, 0];
    paint.determinant = D / 2;
    paint.aabb = (false, [S / 2, 0, S, S]);
    rows.paints[0] = paint.build();

    let raster = validate(prepared, &rows)
        .expect("candidate projection is structural")
        .rasterize_reference(limits(1))
        .expect("candidate-authoritative raster");
    assert_raster(&raster, 1, 1, &[64, 0, 0, 64]);
}

#[test]
fn full_clip_chain_maps_the_original_scene_sample_through_every_link() {
    use super::super::validated_clip_support::clip;
    use super::super::validated_shape_support::rect_values;
    use super::super::world_transform_support::{free, identity, root};
    use crate::model::SpatialAnchorTargetV2;

    let mut nodes = vec![root()];
    for (key, parent) in [(1, 0), (2, 1), (3, 2)] {
        nodes.push(free(
            key,
            parent,
            SpatialAnchorTargetV2::Viewport,
            0,
            0,
            4,
            4,
            identity(),
        ));
    }
    let source = owned_fixture(
        viewport(1, 1),
        nodes,
        vec![
            rect_values(0, 1, 0, 0, S / 2, S),
            rect_values(1, 2, 0, 0, S / 2, S),
            rect_values(2, 3, 0, 0, S, S),
        ],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![solid(0, color(255, 255, 255, 255))],
        Vec::new(),
        Vec::new(),
        vec![
            clip(0, 1, None, 0, SpatialFillRuleV2::NonZero),
            clip(1, 2, Some(0), 1, SpatialFillRuleV2::NonZero),
        ],
        vec![fill(3, 0, 2, 0, 255, Some(1), SpatialFillRuleV2::NonZero)],
    );
    let (prepared, mut rows) = candidate_case(source);
    for (index, tx) in [(1, 0), (2, S / 4), (3, 0)] {
        let mut geometry = GeometryRow::read(rows.geometry[index]);
        geometry.world = [S, 0, 0, S, tx, 0];
        geometry.determinant = D;
        geometry.aabb = (false, [tx, 0, tx + 4 * S, 4 * S]);
        rows.geometry[index] = geometry.build();
    }
    for (index, tx) in [(0, 0), (1, S / 4)] {
        let mut clip = ClipRow::read(rows.clips[index]);
        clip.world = [S, 0, 0, S, tx, 0];
        clip.determinant = D;
        clip.aabb = (false, [tx, 0, tx + S / 2, S]);
        rows.clips[index] = clip.build();
    }
    let mut paint = PaintRow::read(rows.paints[0]);
    paint.world = [S, 0, 0, S, 0, 0];
    paint.determinant = D;
    paint.aabb = (false, [0, 0, S, S]);
    rows.paints[0] = paint.build();

    let raster = validate(prepared, &rows)
        .expect("candidate clip chain is structural")
        .rasterize_reference(limits(1))
        .expect("full exact clip chain");
    assert_raster(&raster, 1, 1, &[64, 64, 64, 64]);
}

#[test]
fn disjoint_and_touching_effective_clip_bounds_reject_registered_samples() {
    use super::super::validated_clip_support::clip;
    use super::super::validated_shape_support::rect_values;

    for (parent_x, child_x) in [(0, 2 * S), (-S + S / 8, S / 8)] {
        let source = owned_fixture(
            viewport(1, 1),
            root_and_owners(1, 4, 4),
            vec![
                rect_values(0, 1, 0, 0, S, S),
                rect_values(1, 1, parent_x, 0, S, S),
                rect_values(2, 1, child_x, 0, S, S),
            ],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![solid(0, color(255, 255, 255, 255))],
            Vec::new(),
            Vec::new(),
            vec![
                clip(0, 1, None, 1, SpatialFillRuleV2::NonZero),
                clip(1, 1, Some(0), 2, SpatialFillRuleV2::NonZero),
            ],
            vec![fill(1, 0, 0, 0, 255, Some(1), SpatialFillRuleV2::NonZero)],
        );
        let raster = snapshot(source)
            .rasterize_reference(limits(1))
            .expect("empty exact intersection");
        assert_raster(&raster, 1, 1, &[0, 0, 0, 0]);
    }
}

#[test]
fn conservative_circle_aabb_never_establishes_exact_coverage() {
    use super::super::validated_shape_support::circle_values;

    let source = owned_fixture(
        viewport(1, 1),
        root_and_owners(1, 0, 0),
        vec![circle_values(0, 1, S / 2, S / 2, S / 8)],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![solid(0, color(255, 255, 255, 255))],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![fill(1, 0, 0, 0, 255, None, SpatialFillRuleV2::NonZero)],
    );
    let raster = snapshot(source)
        .rasterize_reference(limits(1))
        .expect("exact conservative miss");
    assert_raster(&raster, 1, 1, &[0, 0, 0, 0]);
}

#[test]
fn in_domain_scene_samples_with_out_of_domain_inverse_are_uncovered() {
    use super::super::validated_shape_support::rect_values;
    use crate::model::SpatialScalarV2;

    let maximum = SpatialScalarV2::MAX_RAW;
    let source = owned_fixture(
        viewport(1, 1),
        root_and_owners(1, 0, 0),
        vec![rect_values(0, 1, maximum - S, maximum - S, S, S)],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![solid(0, color(255, 255, 255, 255))],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![fill(1, 0, 0, 0, 255, None, SpatialFillRuleV2::NonZero)],
    );
    let (prepared, mut rows) = candidate_case(source);
    let world = [3 * S, 0, -2 * S, -S, -maximum, maximum];
    let determinant = -3 * D;
    let mut geometry = GeometryRow::read(rows.geometry[1]);
    geometry.world = world;
    geometry.determinant = determinant;
    geometry.aabb = (false, [-maximum, maximum, -maximum, maximum]);
    rows.geometry[1] = geometry.build();
    let mut paint = PaintRow::read(rows.paints[0]);
    paint.world = world;
    paint.determinant = determinant;
    paint.aabb = (false, [-3 * S, 0, 2 * S, S]);
    rows.paints[0] = paint.build();

    let raster = validate(prepared, &rows)
        .expect("near-domain candidate is structural")
        .rasterize_reference(limits(1))
        .expect("inverse misses are not errors");
    assert_raster(&raster, 1, 1, &[191, 191, 191, 191]);
}

#[test]
fn clip_inverse_failure_rejects_only_the_affected_samples() {
    use super::super::validated_clip_support::root_clip;
    use super::super::validated_shape_support::rect_values;
    use super::super::world_transform_support::{free, identity, root};
    use crate::model::{SpatialAnchorTargetV2, SpatialScalarV2};

    let maximum = SpatialScalarV2::MAX_RAW;
    let nodes = vec![
        root(),
        free(
            1,
            0,
            SpatialAnchorTargetV2::Viewport,
            0,
            0,
            0,
            0,
            identity(),
        ),
        free(
            2,
            1,
            SpatialAnchorTargetV2::Viewport,
            0,
            0,
            1,
            1,
            identity(),
        ),
    ];
    let source = owned_fixture(
        viewport(1, 1),
        nodes,
        vec![
            rect_values(0, 1, maximum - S, maximum - S, S, S),
            rect_values(1, 2, 0, 0, S, S),
        ],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![solid(0, color(255, 255, 255, 255))],
        Vec::new(),
        Vec::new(),
        vec![root_clip(0, 1, 0)],
        vec![fill(2, 0, 1, 0, 255, Some(0), SpatialFillRuleV2::NonZero)],
    );
    let (prepared, mut rows) = candidate_case(source);
    let world = [3 * S, 0, -2 * S, -S, -maximum, maximum];
    let determinant = -3 * D;
    let mut geometry = GeometryRow::read(rows.geometry[1]);
    geometry.world = world;
    geometry.determinant = determinant;
    geometry.aabb = (false, [-maximum, maximum, -maximum, maximum]);
    rows.geometry[1] = geometry.build();
    let mut clip = ClipRow::read(rows.clips[0]);
    clip.world = world;
    clip.determinant = determinant;
    clip.aabb = (false, [-3 * S, 0, 2 * S, S]);
    rows.clips[0] = clip.build();

    let raster = validate(prepared, &rows)
        .expect("near-domain clip candidate is structural")
        .rasterize_reference(limits(1))
        .expect("clip inverse misses are not errors");
    assert_raster(&raster, 1, 1, &[191, 191, 191, 191]);
}

#[test]
fn path_clips_preserve_nonzero_and_even_odd_exact_k4_winding() {
    use super::super::flattened_path_support::{line_to, move_to, path};
    use super::super::validated_clip_support::clip;
    use super::super::validated_shape_support::{path_shape, rect_values};
    use crate::path::SpatialPathVerbV2;

    let winding = vec![
        move_to(0, 0),
        line_to(S, 0),
        line_to(S, S),
        line_to(0, S),
        SpatialPathVerbV2::Close,
    ];
    let mut verbs = winding.clone();
    verbs.extend(winding);
    let source = owned_fixture(
        viewport(1, 1),
        root_and_owners(1, 1, 1),
        vec![rect_values(0, 1, 0, 0, S, S), path_shape(1, 1, 0)],
        Vec::new(),
        vec![path(0, 0, 10)],
        verbs,
        vec![
            solid(0, color(255, 0, 0, 255)),
            solid(1, color(0, 255, 0, 255)),
        ],
        Vec::new(),
        Vec::new(),
        vec![
            clip(0, 1, None, 1, SpatialFillRuleV2::NonZero),
            clip(1, 1, None, 1, SpatialFillRuleV2::EvenOdd),
        ],
        vec![
            fill(1, 0, 0, 0, 255, Some(0), SpatialFillRuleV2::NonZero),
            fill(1, 1, 0, 1, 255, Some(1), SpatialFillRuleV2::NonZero),
        ],
    );
    let raster = snapshot(source)
        .rasterize_reference(limits(1))
        .expect("exact path clip");
    assert_raster(&raster, 1, 1, &[255, 0, 0, 255]);
}
