mod support;

use fenestra_ui_layout::prototype::{
    LayoutErrorLocationV1, LayoutInputErrorKindV1, LayoutLimitKindV1, LayoutLimitsV1,
};

use support::{GENEROUS_LIMITS, VIEWPORT, assert_invalid, assert_valid, input_node, node, root};

#[test]
fn empty_input_fails_without_invoking_the_engine() {
    assert_invalid(
        &[],
        VIEWPORT,
        GENEROUS_LIMITS,
        LayoutInputErrorKindV1::EmptyInput,
        LayoutErrorLocationV1::Input,
    );
}

#[test]
fn root_key_precedes_root_parent_validation() {
    assert_invalid(
        &[node(3, Some(9))],
        VIEWPORT,
        GENEROUS_LIMITS,
        LayoutInputErrorKindV1::InvalidRootKey,
        input_node(0),
    );
    assert_invalid(
        &[node(0, Some(9))],
        VIEWPORT,
        GENEROUS_LIMITS,
        LayoutInputErrorKindV1::RootHasParent,
        input_node(0),
    );
}

#[test]
fn later_node_shape_checks_follow_field_order() {
    let cases = [
        (
            vec![root(), node(2, None)],
            LayoutInputErrorKindV1::NonDenseKey,
            1,
        ),
        (
            vec![root(), node(1, None)],
            LayoutInputErrorKindV1::MissingParent,
            1,
        ),
        (
            vec![root(), node(1, Some(9))],
            LayoutInputErrorKindV1::MissingParent,
            1,
        ),
        (
            vec![root(), node(1, Some(1))],
            LayoutInputErrorKindV1::ForwardParent,
            1,
        ),
        (
            vec![root(), node(1, Some(2)), node(2, Some(0))],
            LayoutInputErrorKindV1::ForwardParent,
            1,
        ),
        (
            vec![root(), node(1, Some(0)), node(2, Some(0)), node(3, Some(1))],
            LayoutInputErrorKindV1::InvalidPreorder,
            3,
        ),
    ];

    for (nodes, expected, index) in cases {
        assert_invalid(
            &nodes,
            VIEWPORT,
            GENEROUS_LIMITS,
            expected,
            input_node(index),
        );
    }
}

#[test]
fn valid_authored_preorder_accepts_nested_and_sibling_subtrees() {
    let nodes = [
        root(),
        node(1, Some(0)),
        node(2, Some(1)),
        node(3, Some(1)),
        node(4, Some(0)),
        node(5, Some(4)),
    ];

    assert_valid(&nodes, VIEWPORT, LayoutLimitsV1::new(6, 3, 2));
}

#[test]
fn authored_preorder_can_pop_multiple_completed_subtrees_but_never_reopen_one() {
    let valid = [
        root(),
        node(1, Some(0)),
        node(2, Some(1)),
        node(3, Some(2)),
        node(4, Some(0)),
        node(5, Some(4)),
        node(6, Some(0)),
    ];
    assert_valid(&valid, VIEWPORT, LayoutLimitsV1::new(7, 4, 3));

    let reopened = [
        root(),
        node(1, Some(0)),
        node(2, Some(1)),
        node(3, Some(2)),
        node(4, Some(0)),
        node(5, Some(1)),
    ];
    assert_invalid(
        &reopened,
        VIEWPORT,
        GENEROUS_LIMITS,
        LayoutInputErrorKindV1::InvalidPreorder,
        input_node(5),
    );
}

#[test]
fn extreme_keys_and_parents_are_classified_without_indexing_them() {
    assert_invalid(
        &[root(), node(u32::MAX, Some(0))],
        VIEWPORT,
        GENEROUS_LIMITS,
        LayoutInputErrorKindV1::NonDenseKey,
        input_node(1),
    );
    assert_invalid(
        &[root(), node(1, Some(u32::MAX))],
        VIEWPORT,
        GENEROUS_LIMITS,
        LayoutInputErrorKindV1::MissingParent,
        input_node(1),
    );
    assert_invalid(
        &[root(), node(1, Some(2)), node(99, Some(0))],
        VIEWPORT,
        GENEROUS_LIMITS,
        LayoutInputErrorKindV1::MissingParent,
        input_node(1),
    );
}

#[test]
fn node_capacity_is_inclusive_and_rejects_one_over_first() {
    let nodes = [root(), node(1, Some(0)), node(2, Some(0))];
    assert_valid(&nodes, VIEWPORT, LayoutLimitsV1::new(3, 2, 2));
    assert_invalid(
        &nodes,
        VIEWPORT,
        LayoutLimitsV1::new(2, 64, 64),
        LayoutInputErrorKindV1::LimitExceeded(LayoutLimitKindV1::Nodes),
        LayoutErrorLocationV1::Input,
    );
}

#[test]
fn root_depth_one_is_inclusive() {
    assert_valid(&[root()], VIEWPORT, LayoutLimitsV1::new(1, 1, 0));
}

#[test]
fn depth_capacity_is_inclusive_and_reports_first_deep_node() {
    let nodes = [root(), node(1, Some(0)), node(2, Some(1))];
    assert_valid(&nodes, VIEWPORT, LayoutLimitsV1::new(3, 3, 1));
    assert_invalid(
        &nodes,
        VIEWPORT,
        LayoutLimitsV1::new(3, 2, 2),
        LayoutInputErrorKindV1::LimitExceeded(LayoutLimitKindV1::Depth),
        input_node(2),
    );
}

#[test]
fn child_capacity_is_inclusive_and_reports_the_first_parent() {
    let nodes = [root(), node(1, Some(0)), node(2, Some(0))];
    assert_valid(&nodes, VIEWPORT, LayoutLimitsV1::new(3, 2, 2));
    assert_invalid(
        &nodes,
        VIEWPORT,
        LayoutLimitsV1::new(3, 3, 1),
        LayoutInputErrorKindV1::LimitExceeded(LayoutLimitKindV1::ChildrenPerNode),
        input_node(0),
    );
}

#[test]
fn zero_depth_and_child_limits_report_the_trusted_owner() {
    assert_invalid(
        &[root()],
        VIEWPORT,
        LayoutLimitsV1::new(1, 0, 0),
        LayoutInputErrorKindV1::LimitExceeded(LayoutLimitKindV1::Depth),
        input_node(0),
    );
    assert_invalid(
        &[root(), node(1, Some(0))],
        VIEWPORT,
        LayoutLimitsV1::new(2, 2, 0),
        LayoutInputErrorKindV1::LimitExceeded(LayoutLimitKindV1::ChildrenPerNode),
        input_node(0),
    );
}

#[cfg(target_pointer_width = "64")]
#[test]
fn limits_larger_than_the_key_domain_do_not_narrow_small_inputs() {
    let wider_than_u32 = usize::try_from(u64::from(u32::MAX) + 1)
        .expect("64-bit usize represents one past the u32 key domain");
    assert_valid(
        &[root()],
        VIEWPORT,
        LayoutLimitsV1::new(wider_than_u32, wider_than_u32, wider_than_u32),
    );
}
