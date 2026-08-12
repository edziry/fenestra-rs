use super::island_support::{
    expect_input, expect_limit, expect_plan, fixture, free, island_limits, layout, node, root,
};
use crate::error::{SpatialErrorLocationV2, SpatialInputErrorKindV2};
use crate::limits::SpatialLimitKindV2;
use crate::topology::SpatialPlacementV2;
use crate::vocabulary::SpatialNodeFieldV2;

#[test]
fn island_limits_use_complete_counts_and_three_global_passes() {
    let input = capacity_fixture();

    expect_limit(
        prepare_island_plan!(&input, island_limits(1, 0, 0)),
        SpatialLimitKindV2::Islands,
        SpatialErrorLocationV2::Input,
        3,
        1,
    );
    expect_limit(
        prepare_island_plan!(&input, island_limits(3, 2, 0)),
        SpatialLimitKindV2::LayoutInputRecordsPerIsland,
        SpatialErrorLocationV2::Island { index: 1 },
        4,
        2,
    );
    expect_limit(
        prepare_island_plan!(&input, island_limits(3, 4, 3)),
        SpatialLimitKindV2::LayoutInputRecordsTotal,
        SpatialErrorLocationV2::Input,
        9,
        3,
    );

    let plan = expect_plan(prepare_island_plan!(&input, island_limits(3, 4, 9)));
    assert_eq!(
        plan.island_facts(),
        vec![
            (0, 2, 0, vec![(2, 0)]),
            (1, 5, 4, vec![(5, 0), (6, 1), (7, 1)]),
            (2, 9, 8, vec![(9, 0), (10, 1)]),
        ]
    );
    assert_eq!(
        plan.item_facts(),
        vec![
            (1, None, vec![1]),
            (2, Some(0), vec![0, 2]),
            (3, None, vec![3]),
            (5, Some(1), vec![4, 5, 6, 7]),
            (9, Some(2), vec![8, 9, 10]),
        ]
    );
}

#[test]
fn per_island_failure_precedes_a_stricter_total_limit() {
    let input = capacity_fixture();

    expect_limit(
        prepare_island_plan!(&input, island_limits(3, 2, 0)),
        SpatialLimitKindV2::LayoutInputRecordsPerIsland,
        SpatialErrorLocationV2::Island { index: 1 },
        4,
        2,
    );
}

#[test]
fn phase_four_failure_precedes_island_derivation_and_limits() {
    let input = fixture(vec![
        root(),
        node(1, 0, SpatialPlacementV2::Root),
        layout(2, 0),
    ]);

    expect_input(
        prepare_island_plan!(&input, island_limits(0, 0, 0)).map(|_| ()),
        SpatialInputErrorKindV2::RootPlacementOnNonRoot,
        SpatialErrorLocationV2::NodeField {
            index: 1,
            field: SpatialNodeFieldV2::Placement,
        },
    );
}

#[cfg(target_pointer_width = "64")]
#[test]
fn island_record_counts_accept_the_reachable_u32_capacity() {
    let capacity = u32::MAX as u128 + 1;
    let exact = usize::try_from(capacity).expect("64-bit targets represent the row capacity");
    let below = exact - 1;

    expect_plan(super::check_island_fact(
        SpatialLimitKindV2::LayoutInputRecordsPerIsland,
        Some(0),
        capacity,
        island_limits(usize::MAX, exact, usize::MAX),
    ));
    expect_limit(
        super::check_island_fact(
            SpatialLimitKindV2::LayoutInputRecordsPerIsland,
            Some(0),
            capacity,
            island_limits(usize::MAX, below, usize::MAX),
        ),
        SpatialLimitKindV2::LayoutInputRecordsPerIsland,
        SpatialErrorLocationV2::Island { index: 0 },
        capacity,
        below as u128,
    );

    expect_plan(super::check_island_fact(
        SpatialLimitKindV2::LayoutInputRecordsTotal,
        None,
        capacity,
        island_limits(usize::MAX, usize::MAX, exact),
    ));
    expect_limit(
        super::check_island_fact(
            SpatialLimitKindV2::LayoutInputRecordsTotal,
            None,
            capacity,
            island_limits(usize::MAX, usize::MAX, below),
        ),
        SpatialLimitKindV2::LayoutInputRecordsTotal,
        SpatialErrorLocationV2::Input,
        capacity,
        below as u128,
    );
}

fn capacity_fixture() -> super::fixture::RawInputFixture {
    fixture(vec![
        root(),
        free(1, 0),
        layout(2, 0),
        free(3, 0),
        free(4, 0),
        layout(5, 4),
        layout(6, 5),
        layout(7, 5),
        free(8, 0),
        layout(9, 8),
        layout(10, 9),
    ])
}
