use super::island_support::{expect_plan, fixture, free, island_limits, layout, root};

#[test]
fn maximal_islands_resume_across_omitted_free_subtrees() {
    let input = fixture(vec![
        root(),
        free(1, 0),
        layout(2, 0),
        layout(3, 2),
        free(4, 2),
        layout(5, 4),
        layout(6, 5),
        free(7, 5),
        layout(8, 2),
        free(9, 0),
        layout(10, 9),
        free(11, 0),
        layout(12, 0),
    ]);

    let plan = expect_plan(prepare_island_plan!(&input, island_limits(3, 5, 10)));

    assert_eq!(
        plan.island_facts(),
        vec![
            (0, 2, 0, vec![(2, 0), (3, 1), (8, 1), (12, 0)]),
            (1, 5, 4, vec![(5, 0), (6, 1)]),
            (2, 10, 9, vec![(10, 0)]),
        ]
    );
    assert_eq!(
        plan.item_facts(),
        vec![
            (1, None, vec![1]),
            (2, Some(0), vec![0, 2, 3, 8, 12]),
            (5, Some(1), vec![4, 5, 6]),
            (7, None, vec![7]),
            (10, Some(2), vec![9, 10]),
            (11, None, vec![11]),
        ]
    );

    let mut owners = plan
        .item_facts()
        .into_iter()
        .flat_map(|(_, _, owners)| owners)
        .collect::<Vec<_>>();
    owners.sort_unstable();
    assert_eq!(owners, (0..=12).collect::<Vec<_>>());
}

#[test]
fn island_indices_follow_first_members_instead_of_host_keys() {
    let input = fixture(vec![
        root(),
        free(1, 0),
        free(2, 1),
        layout(3, 2),
        layout(4, 1),
        layout(5, 0),
    ]);

    let plan = expect_plan(prepare_island_plan!(&input, island_limits(3, 2, 6)));

    assert_eq!(
        plan.island_facts(),
        vec![
            (0, 3, 2, vec![(3, 0)]),
            (1, 4, 1, vec![(4, 0)]),
            (2, 5, 0, vec![(5, 0)]),
        ]
    );
    assert_eq!(
        plan.item_facts(),
        vec![
            (3, Some(0), vec![2, 3]),
            (4, Some(1), vec![1, 4]),
            (5, Some(2), vec![0, 5]),
        ]
    );
}

#[test]
fn root_and_free_nodes_without_islands_are_singletons_only() {
    let input = fixture(vec![root(), free(1, 0), free(2, 1), free(3, 0)]);

    let plan = expect_plan(prepare_island_plan!(&input, island_limits(0, 0, 0)));

    assert!(plan.island_facts().is_empty());
    assert_eq!(
        plan.item_facts(),
        vec![
            (0, None, vec![0]),
            (1, None, vec![1]),
            (2, None, vec![2]),
            (3, None, vec![3]),
        ]
    );
}

#[test]
fn a_free_host_replaces_its_singleton_but_keeps_the_root_singleton() {
    let input = fixture(vec![root(), free(1, 0), layout(2, 1), free(3, 0)]);

    let plan = expect_plan(prepare_island_plan!(&input, island_limits(1, 2, 2)));

    assert_eq!(plan.island_facts(), vec![(0, 2, 1, vec![(2, 0)])]);
    assert_eq!(
        plan.item_facts(),
        vec![
            (0, None, vec![0]),
            (2, Some(0), vec![1, 2]),
            (3, None, vec![3]),
        ]
    );
}

#[test]
fn a_free_host_reuses_its_island_after_an_omitted_free_subtree() {
    let input = fixture(vec![
        root(),
        free(1, 0),
        layout(2, 1),
        free(3, 1),
        layout(4, 3),
        layout(5, 1),
    ]);

    let plan = expect_plan(prepare_island_plan!(&input, island_limits(2, 3, 5)));

    assert_eq!(
        plan.island_facts(),
        vec![(0, 2, 1, vec![(2, 0), (5, 0)]), (1, 4, 3, vec![(4, 0)]),]
    );
    assert_eq!(
        plan.item_facts(),
        vec![
            (0, None, vec![0]),
            (2, Some(0), vec![1, 2, 5]),
            (4, Some(1), vec![3, 4]),
        ]
    );
}

#[test]
fn nested_members_reference_their_exact_dense_layout_parent() {
    let input = fixture(vec![root(), layout(1, 0), layout(2, 1), layout(3, 2)]);

    let plan = expect_plan(prepare_island_plan!(&input, island_limits(1, 4, 4)));

    assert_eq!(
        plan.island_facts(),
        vec![(0, 1, 0, vec![(1, 0), (2, 1), (3, 2)])]
    );
    assert_eq!(plan.item_facts(), vec![(1, Some(0), vec![0, 1, 2, 3])]);
}
