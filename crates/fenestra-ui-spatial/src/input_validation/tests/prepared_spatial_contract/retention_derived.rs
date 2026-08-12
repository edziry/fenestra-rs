use super::super::world_aabb_support::{aabb, fact};
use super::super::world_transform_support::{SCALE, placement, world};
use super::support::{requested_limits, rich_engine, rich_owned};
use super::*;

#[test]
fn prepared_state_owns_topology_geometry_and_every_phase_ten_projection() {
    let engine = rich_engine();
    let prepared = prepare_spatial_v2(&engine, rich_owned(), requested_limits())
        .expect("rich owned input prepares successfully");

    assert_eq!(
        prepared.topology_facts(),
        vec![
            (0, None, 1),
            (1, Some(0), 2),
            (2, Some(1), 3),
            (3, Some(2), 4)
        ]
    );
    assert_eq!(
        prepared.base_geometry_facts(),
        vec![
            (0, 0, 0, 20, 20),
            (1, 0, 10 * SCALE, 10, 10),
            (2, SCALE, 12 * SCALE, 3, 4),
            (3, SCALE, 6 * SCALE, 10, 10)
        ]
    );
    assert_eq!(
        prepared.world_transform_facts(),
        vec![
            world(0, [SCALE, 0, 0, SCALE, 0, 0]),
            world(1, [SCALE, 0, 0, SCALE, 0, 10 * SCALE]),
            world(2, [SCALE, 0, 0, SCALE, SCALE, 12 * SCALE]),
            world(3, [SCALE, 0, 0, SCALE, SCALE, 6 * SCALE]),
        ]
    );
    assert_eq!(
        prepared.geometry_world_aabb_facts(),
        vec![
            fact(0, aabb(0, 0, 20 * SCALE, 20 * SCALE)),
            fact(1, aabb(0, 10 * SCALE, 10 * SCALE, 20 * SCALE)),
            fact(2, aabb(SCALE, 12 * SCALE, 4 * SCALE, 16 * SCALE)),
            fact(3, aabb(SCALE, 6 * SCALE, 11 * SCALE, 16 * SCALE)),
        ]
    );
    assert_eq!(
        prepared.clip_world_aabb_facts(),
        vec![
            fact(0, aabb(SCALE, 12 * SCALE, 4 * SCALE, 16 * SCALE)),
            fact(1, aabb(0, 10 * SCALE, 10 * SCALE, 20 * SCALE)),
            fact(2, aabb(0, 10 * SCALE, 10 * SCALE, 20 * SCALE)),
        ]
    );
    assert_eq!(
        prepared.effective_clip_world_aabb_facts(),
        vec![
            fact(0, aabb(SCALE, 12 * SCALE, 4 * SCALE, 16 * SCALE)),
            fact(1, aabb(SCALE, 12 * SCALE, 4 * SCALE, 16 * SCALE)),
            fact(2, aabb(SCALE, 12 * SCALE, 4 * SCALE, 16 * SCALE)),
        ]
    );
    assert_eq!(
        prepared.paint_world_aabb_facts(),
        vec![
            fact(0, aabb(SCALE, 12 * SCALE, 4 * SCALE, 16 * SCALE)),
            fact(1, aabb(10 * SCALE, 30 * SCALE, 13 * SCALE, 34 * SCALE)),
            fact(2, aabb(-2 * SCALE, 10 * SCALE, 6 * SCALE, 18 * SCALE)),
        ]
    );
    assert_eq!(
        prepared.hit_world_aabb_facts(),
        vec![
            fact(0, aabb(-SCALE, 11 * SCALE, 5 * SCALE, 17 * SCALE)),
            fact(1, aabb(0, 5 * SCALE, 4 * SCALE, 7 * SCALE)),
            fact(2, aabb(-SCALE, 4 * SCALE, 5 * SCALE, 8 * SCALE)),
        ]
    );
    assert_eq!(
        prepared.semantic_world_aabb_facts(),
        vec![
            fact(0, aabb(SCALE, 6 * SCALE, 3 * SCALE, 6 * SCALE)),
            fact(1, aabb(SCALE, 6 * SCALE, 3 * SCALE, 6 * SCALE)),
        ]
    );
    assert_eq!(prepared.limits(), requested_limits());
    assert_eq!(
        engine.calls(),
        vec![(10, 10, vec![(0, None, 10, 10), (1, Some(0), 3, 4)],)]
    );

    assert_eq!(
        prepared.base_geometry_facts()[2],
        base_from_placement(placement(
            2,
            SCALE,
            12 * SCALE,
            3,
            4,
            4 * SCALE,
            16 * SCALE,
            SCALE,
            2 * SCALE,
        ))
    );
}

fn base_from_placement(
    fact: (u32, i64, i64, i32, i32, i64, i64, i64, i64),
) -> (u32, i64, i64, i32, i32) {
    (fact.0, fact.1, fact.2, fact.3, fact.4)
}
