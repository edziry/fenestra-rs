use super::hit_support::*;
use super::validator_support::{ClipRow, GeometryRow, ShapeItemRow, affine, validate};
use crate::coverage::SpatialFillRuleV2;

const D: i128 = (S as i128) * (S as i128);

#[test]
fn accepted_candidate_hit_and_clip_affines_are_authoritative() {
    use super::super::validated_clip_support::root_clip;
    use super::super::validated_shape_support::rect_values;
    use super::super::world_transform_support::{free, identity, root};
    use crate::model::SpatialAnchorTargetV2;

    let source = owned_fixture(
        vec![
            root(),
            free(
                1,
                0,
                SpatialAnchorTargetV2::Viewport,
                0,
                0,
                20,
                20,
                identity(),
            ),
            free(
                2,
                1,
                SpatialAnchorTargetV2::Viewport,
                0,
                0,
                20,
                20,
                identity(),
            ),
        ],
        vec![
            rect_values(0, 2, 0, 0, 4 * S, 5 * S),
            rect_values(1, 1, 3 * S, 2 * S, 4 * S, 5 * S),
        ],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![root_clip(0, 1, 1)],
        vec![accepting_fill(2, 0, 0, Some(0), SpatialFillRuleV2::NonZero)],
    );
    let (prepared, mut rows) = candidate_case(source);
    let world = [-S, 0, 0, S, 10 * S, 20 * S];

    let mut clip_owner = GeometryRow::read(rows.geometry[1]);
    clip_owner.world = [S, 0, 0, S, 5 * S, 20 * S];
    clip_owner.determinant = D;
    clip_owner.aabb = (false, [5 * S, 20 * S, 25 * S, 40 * S]);
    rows.geometry[1] = clip_owner.build();

    let mut geometry = GeometryRow::read(rows.geometry[2]);
    geometry.world = world;
    geometry.determinant = -D;
    geometry.aabb = (false, [-10 * S, 20 * S, 10 * S, 40 * S]);
    rows.geometry[2] = geometry.build();

    let mut clip = ClipRow::read(rows.clips[0]);
    clip.world = [S, 0, 0, S, 5 * S, 20 * S];
    clip.determinant = D;
    clip.aabb = (false, [8 * S, 22 * S, 12 * S, 27 * S]);
    rows.clips[0] = clip.build();

    let mut hit = ShapeItemRow::read_hit(rows.hits[0]);
    hit.world = world;
    hit.determinant = -D;
    hit.aabb = (false, [6 * S, 20 * S, 10 * S, 25 * S]);
    rows.hits[0] = hit.build_hit();

    let snapshot = validate(prepared, &rows).expect("candidate projection is structural");
    drop(rows);
    assert_hit(
        snapshot.hit_test(point(8 * S, 23 * S)),
        0,
        2,
        0,
        point(2 * S, 3 * S),
    );
    assert_eq!(snapshot.hit_test(point(2 * S, 3 * S)), None);
}

#[test]
fn inverse_rounding_uses_nearest_ties_away_for_both_signs() {
    use super::super::validated_shape_support::rect_values;

    let source = owned_fixture(
        root_and_owners(1, 20, 20),
        vec![rect_values(0, 1, -2, 0, 4, 2 * S)],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![accepting_fill(1, 0, 0, None, SpatialFillRuleV2::NonZero)],
    );
    let (prepared, mut rows) = candidate_case(source);
    let world = [2 * S, 0, 0, S, 0, 0];
    let mut geometry = GeometryRow::read(rows.geometry[1]);
    geometry.world = world;
    geometry.determinant = 2 * D;
    geometry.aabb = (false, [0, 0, 40 * S, 20 * S]);
    rows.geometry[1] = geometry.build();
    let mut hit = ShapeItemRow::read_hit(rows.hits[0]);
    hit.world = world;
    hit.determinant = 2 * D;
    hit.aabb = (false, [-4, 0, 4, 2 * S]);
    rows.hits[0] = hit.build_hit();
    let snapshot = validate(prepared, &rows).expect("scale-two candidate is structural");

    for raw in [-1, 1] {
        assert_hit(snapshot.hit_test(point(raw, S)), 0, 1, 0, point(raw, S));
    }
}

#[test]
fn accepted_off_diagonal_affine_produces_the_exact_local_point() {
    use super::super::validated_shape_support::rect_values;

    let source = owned_fixture(
        root_and_owners(1, 20, 20),
        vec![rect_values(0, 1, S, 2 * S, 2 * S, 3 * S)],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![accepting_fill(1, 0, 0, None, SpatialFillRuleV2::NonZero)],
    );
    let (prepared, mut rows) = candidate_case(source);
    let world = [0, S, -S, 0, 10 * S, 20 * S];
    let mut geometry = GeometryRow::read(rows.geometry[1]);
    geometry.world = world;
    geometry.determinant = D;
    geometry.aabb = (false, [-10 * S, 20 * S, 10 * S, 40 * S]);
    rows.geometry[1] = geometry.build();
    let mut hit = ShapeItemRow::read_hit(rows.hits[0]);
    hit.world = world;
    hit.determinant = D;
    hit.aabb = (false, [5 * S, 21 * S, 8 * S, 23 * S]);
    rows.hits[0] = hit.build_hit();

    let snapshot = validate(prepared, &rows).expect("quarter-turn candidate is structural");
    assert_hit(
        snapshot.hit_test(point(7 * S, 22 * S)),
        0,
        1,
        0,
        point(2 * S, 3 * S),
    );
}

#[test]
fn out_of_domain_scene_points_are_total_misses() {
    use super::super::validated_shape_support::circle_values;
    use crate::model::SpatialScalarV2;

    let source = owned_fixture(
        root_and_owners(1, 20, 20),
        vec![circle_values(0, 1, SpatialScalarV2::MAX_RAW - 1, 0, 1)],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![accepting_fill(1, 0, 0, None, SpatialFillRuleV2::NonZero)],
    );
    let snapshot = reference_snapshot(source);
    assert_eq!(
        snapshot.hit_test(point(SpatialScalarV2::MAX_RAW + 1, 0)),
        None
    );
    assert_eq!(
        snapshot.hit_test(point(S, SpatialScalarV2::MIN_RAW - 1)),
        None
    );
}

#[test]
fn in_domain_scene_point_with_out_of_domain_inverse_is_a_miss() {
    use super::super::validated_shape_support::polygon;
    use crate::model::SpatialScalarV2;

    let maximum = SpatialScalarV2::MAX_RAW;
    let source = owned_fixture(
        root_and_owners(1, 0, 0),
        vec![polygon(0, 1, 0, 3)],
        vec![
            point(maximum, maximum),
            point(maximum - 2, maximum),
            point(maximum, maximum - 2),
        ],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![accepting_fill(1, 0, 0, None, SpatialFillRuleV2::NonZero)],
    );
    let (prepared, mut rows) = candidate_case(source);
    let world = [3 * S, 0, -2 * S, S, -maximum, -maximum];
    let determinant = 3 * D;
    let mut geometry = GeometryRow::read(rows.geometry[1]);
    geometry.world = world;
    geometry.determinant = determinant;
    geometry.aabb = (false, [-maximum, -maximum, -maximum, -maximum]);
    rows.geometry[1] = geometry.build();
    let mut hit = ShapeItemRow::read_hit(rows.hits[0]);
    hit.world = world;
    hit.determinant = determinant;
    hit.aabb = (false, [-6, -2, 4, 0]);
    rows.hits[0] = hit.build_hit();

    let snapshot = validate(prepared, &rows).expect("near-singular candidate is structural");
    assert_eq!(snapshot.hit_test(point(1, 0)), None);
}

#[test]
fn accepted_hit_aabb_is_reject_only_and_never_accepts_without_k4() {
    use super::super::validated_shape_support::circle_values;

    let source = owned_fixture(
        root_and_owners(1, 20, 20),
        vec![circle_values(0, 1, 5 * S, 5 * S, 3 * S)],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![accepting_fill(1, 0, 0, None, SpatialFillRuleV2::NonZero)],
    );
    let snapshot = reference_snapshot(source);
    assert_eq!(snapshot.hit_test(point(8 * S, 8 * S)), None);
    assert_eq!(
        snapshot.output().hits()[0].world_from_local(),
        affine([S, 0, 0, S, 0, 0])
    );
}
