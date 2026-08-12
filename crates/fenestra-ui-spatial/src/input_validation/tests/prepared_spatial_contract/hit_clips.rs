use super::hit_support::*;
use super::validator_support::{ClipRow, GeometryRow, ShapeItemRow, validate};
use crate::coverage::SpatialFillRuleV2;

const D: i128 = (S as i128) * (S as i128);

#[test]
fn full_three_link_chain_is_exact_and_each_hit_uses_its_terminal_clip() {
    use super::super::validated_clip_support::clip;
    use super::super::validated_shape_support::{circle_values, rect_values};

    let source = owned_fixture(
        root_and_owners(1, 20, 20),
        vec![
            rect_values(0, 1, 0, 0, 10 * S, 10 * S),
            rect_values(1, 1, 0, 0, 10 * S, 10 * S),
            circle_values(2, 1, 5 * S, 5 * S, 4 * S),
            rect_values(3, 1, S, S, 8 * S, 8 * S),
        ],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![
            clip(0, 1, None, 1, SpatialFillRuleV2::NonZero),
            clip(1, 1, Some(0), 2, SpatialFillRuleV2::EvenOdd),
            clip(2, 1, Some(1), 3, SpatialFillRuleV2::NonZero),
        ],
        vec![
            accepting_fill(1, 0, 0, Some(0), SpatialFillRuleV2::NonZero),
            accepting_fill(1, 1, 0, Some(2), SpatialFillRuleV2::NonZero),
        ],
    );
    let snapshot = reference_snapshot(source);

    assert_hit(
        snapshot.hit_test(point(5 * S, 5 * S)),
        1,
        1,
        1,
        point(5 * S, 5 * S),
    );
    assert_hit(
        snapshot.hit_test(point(8 * S, 8 * S)),
        0,
        1,
        0,
        point(8 * S, 8 * S),
    );
    assert_hit(
        snapshot.hit_test(point(9 * S, 5 * S)),
        0,
        1,
        0,
        point(9 * S, 5 * S),
    );
}

#[test]
fn clip_fill_rule_uses_the_retained_even_odd_winding() {
    use super::super::flattened_path_support::{line_to, move_to, path};
    use super::super::validated_clip_support::clip;
    use super::super::validated_shape_support::{path_shape, rect_values};
    use crate::path::SpatialPathVerbV2;

    let winding = vec![
        move_to(0, 0),
        line_to(4 * S, 0),
        line_to(4 * S, 4 * S),
        line_to(0, 4 * S),
        SpatialPathVerbV2::Close,
    ];
    let mut verbs = winding.clone();
    verbs.extend(winding);
    let source = owned_fixture(
        root_and_owners(1, 10, 10),
        vec![rect_values(0, 1, 0, 0, 4 * S, 4 * S), path_shape(1, 1, 0)],
        Vec::new(),
        vec![path(0, 0, 10)],
        verbs,
        vec![
            clip(0, 1, None, 1, SpatialFillRuleV2::NonZero),
            clip(1, 1, None, 1, SpatialFillRuleV2::EvenOdd),
        ],
        vec![
            accepting_fill(1, 0, 0, Some(0), SpatialFillRuleV2::NonZero),
            accepting_fill(1, 1, 0, Some(1), SpatialFillRuleV2::NonZero),
        ],
    );
    assert_hit(
        reference_snapshot(source).hit_test(point(2 * S, 2 * S)),
        0,
        1,
        0,
        point(2 * S, 2 * S),
    );
}

#[test]
fn every_link_uses_its_own_accepted_clip_affine() {
    use super::super::validated_clip_support::clip;
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
        free(
            3,
            2,
            SpatialAnchorTargetV2::Viewport,
            0,
            0,
            20,
            20,
            identity(),
        ),
    ];
    let source = owned_fixture(
        nodes,
        vec![
            rect_values(0, 3, 0, 0, 2 * S, 2 * S),
            rect_values(1, 1, 10 * S, 0, 2 * S, 2 * S),
            rect_values(2, 2, 5 * S, 0, 2 * S, 2 * S),
        ],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![
            clip(0, 1, None, 1, SpatialFillRuleV2::NonZero),
            clip(1, 2, Some(0), 2, SpatialFillRuleV2::NonZero),
        ],
        vec![accepting_fill(3, 0, 0, Some(1), SpatialFillRuleV2::NonZero)],
    );
    let (prepared, mut rows) = candidate_case(source);
    for (index, tx) in [(2, 5 * S), (3, 10 * S)] {
        let mut geometry = GeometryRow::read(rows.geometry[index]);
        geometry.world = [S, 0, 0, S, tx, 0];
        geometry.determinant = D;
        geometry.aabb = (false, [tx, 0, tx + 20 * S, 20 * S]);
        rows.geometry[index] = geometry.build();
    }
    let mut child = ClipRow::read(rows.clips[1]);
    child.world = [S, 0, 0, S, 5 * S, 0];
    child.determinant = D;
    child.aabb = (false, [10 * S, 0, 12 * S, 2 * S]);
    rows.clips[1] = child.build();
    let mut hit = ShapeItemRow::read_hit(rows.hits[0]);
    hit.world = [S, 0, 0, S, 10 * S, 0];
    hit.determinant = D;
    hit.aabb = (false, [10 * S, 0, 12 * S, 2 * S]);
    rows.hits[0] = hit.build_hit();

    let snapshot = validate(prepared, &rows).expect("candidate chain is structural");
    assert_hit(snapshot.hit_test(point(11 * S, S)), 0, 3, 0, point(S, S));
}

#[test]
fn disjoint_effective_clip_rejects_a_point_inside_the_child_primitive() {
    use super::super::validated_clip_support::clip;
    use super::super::validated_shape_support::rect_values;

    let source = owned_fixture(
        root_and_owners(1, 20, 20),
        vec![
            rect_values(0, 1, 0, 0, 20 * S, 20 * S),
            rect_values(1, 1, 0, 0, 2 * S, 2 * S),
            rect_values(2, 1, 10 * S, 10 * S, S, S),
        ],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![
            clip(0, 1, None, 1, SpatialFillRuleV2::NonZero),
            clip(1, 1, Some(0), 2, SpatialFillRuleV2::NonZero),
        ],
        vec![accepting_fill(1, 0, 0, Some(1), SpatialFillRuleV2::NonZero)],
    );
    let snapshot = reference_snapshot(source);
    assert!(snapshot.effective_clip_aabbs()[1].is_empty());
    assert_eq!(
        snapshot.hit_test(point(10 * S + S / 2, 10 * S + S / 2)),
        None
    );
}

#[test]
fn touching_effective_bounds_still_require_each_exact_clip_shape() {
    use super::super::validated_clip_support::clip;
    use super::super::validated_shape_support::rect_values;

    let source = owned_fixture(
        root_and_owners(1, 20, 20),
        vec![
            rect_values(0, 1, 0, 0, 20 * S, 20 * S),
            rect_values(1, 1, 0, 0, 10 * S, 10 * S),
            rect_values(2, 1, 10 * S, 0, 5 * S, 10 * S),
        ],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![
            clip(0, 1, None, 1, SpatialFillRuleV2::NonZero),
            clip(1, 1, Some(0), 2, SpatialFillRuleV2::NonZero),
        ],
        vec![accepting_fill(1, 0, 0, Some(1), SpatialFillRuleV2::NonZero)],
    );
    let snapshot = reference_snapshot(source);
    let effective = snapshot.effective_clip_aabbs()[1];
    assert!(!effective.is_empty());
    assert_eq!(effective.min_x(), effective.max_x());
    assert_eq!(snapshot.hit_test(point(10 * S, 5 * S)), None);
}
