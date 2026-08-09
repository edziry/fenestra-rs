use super::*;

#[test]
fn node_fields_follow_documented_schema_order() {
    let root = NodePathV1::root();
    let other_parent = root.clone().static_child(9);
    let expected = single_node_state(0, 0, &[(0, 10), (1, 20)], None, Vec::new());
    let cases = [
        (
            single_node_state(
                9,
                9,
                &[(0, 11), (1, 21)],
                Some(other_parent.clone()),
                Vec::new(),
            ),
            FingerprintFieldV1::Template,
            FingerprintSummaryV1::Template(TemplateNodeId::new(0)),
            FingerprintSummaryV1::Template(TemplateNodeId::new(9)),
        ),
        (
            single_node_state(
                0,
                9,
                &[(0, 11), (1, 21)],
                Some(other_parent.clone()),
                Vec::new(),
            ),
            FingerprintFieldV1::Component,
            FingerprintSummaryV1::Component(ComponentTypeId::new(0)),
            FingerprintSummaryV1::Component(ComponentTypeId::new(9)),
        ),
        (
            single_node_state(
                0,
                0,
                &[(0, 11), (1, 21)],
                Some(other_parent.clone()),
                Vec::new(),
            ),
            FingerprintFieldV1::Property,
            property_summary(0, 10),
            property_summary(0, 11),
        ),
        (
            single_node_state(
                0,
                0,
                &[(0, 10), (1, 21)],
                Some(other_parent.clone()),
                Vec::new(),
            ),
            FingerprintFieldV1::Property,
            property_summary(1, 20),
            property_summary(1, 21),
        ),
        (
            single_node_state(
                0,
                0,
                &[(0, 10), (1, 20)],
                Some(other_parent.clone()),
                Vec::new(),
            ),
            FingerprintFieldV1::Parent,
            FingerprintSummaryV1::None,
            FingerprintSummaryV1::Node(other_parent),
        ),
        (
            NormalizedStateV1::new(Vec::new(), Vec::new()),
            FingerprintFieldV1::Template,
            FingerprintSummaryV1::Template(TemplateNodeId::new(0)),
            FingerprintSummaryV1::None,
        ),
    ];
    for (observed, field, expected_summary, observed_summary) in cases {
        assert_state_mismatch(
            &expected,
            &observed,
            FingerprintLocationV1::Node(root.clone()),
            field,
            expected_summary,
            observed_summary,
        );
    }
}

#[test]
fn fragment_and_child_phases_follow_documented_order() {
    let root = NodePathV1::root();
    let primary = FragmentPathV1::new(root.clone(), 1);
    let member_seven = root.clone().member(1, 7);
    let other_child = root.clone().static_child(9);
    let expected_root = normalized_node(
        root.clone(),
        None,
        0,
        0,
        &[(0, 10)],
        vec![NormalizedChildGroupV1::Region(primary.clone())],
    );
    let expected_fragment =
        normalized_fragment(primary.clone(), 0, vec![(7, member_seven.clone())]);
    let expected =
        NormalizedStateV1::new(vec![expected_root.clone()], vec![expected_fragment.clone()]);
    let changed_groups = vec![NormalizedChildGroupV1::Static(other_child.clone())];

    let missing_fragment = NormalizedStateV1::new(
        vec![normalized_node(
            root.clone(),
            None,
            0,
            0,
            &[(0, 10)],
            changed_groups.clone(),
        )],
        Vec::new(),
    );
    assert_state_mismatch(
        &expected,
        &missing_fragment,
        FingerprintLocationV1::Fragment(primary.clone()),
        FingerprintFieldV1::FragmentBinding,
        FingerprintSummaryV1::BindingPresent,
        FingerprintSummaryV1::BindingAbsent,
    );

    let different_keys = NormalizedStateV1::new(
        vec![normalized_node(
            root.clone(),
            None,
            0,
            0,
            &[(0, 10)],
            changed_groups.clone(),
        )],
        vec![normalized_fragment(
            primary.clone(),
            0,
            vec![(8, root.clone().member(1, 8))],
        )],
    );
    assert_state_mismatch(
        &expected,
        &different_keys,
        FingerprintLocationV1::Fragment(primary.clone()),
        FingerprintFieldV1::KeyedOrder,
        FingerprintSummaryV1::Keys(vec![7]),
        FingerprintSummaryV1::Keys(vec![8]),
    );

    let different_groups = NormalizedStateV1::new(
        vec![normalized_node(
            root.clone(),
            None,
            0,
            0,
            &[(0, 10)],
            changed_groups.clone(),
        )],
        vec![expected_fragment.clone()],
    );
    assert_state_mismatch(
        &expected,
        &different_groups,
        FingerprintLocationV1::Node(root.clone()),
        FingerprintFieldV1::ChildOrder,
        FingerprintSummaryV1::Children(vec![NormalizedChildGroupV1::Region(primary.clone())]),
        FingerprintSummaryV1::Children(changed_groups),
    );

    let different_flat_order = NormalizedStateV1::new(
        vec![expected_root],
        vec![normalized_fragment(
            primary,
            0,
            vec![(7, other_child.clone())],
        )],
    );
    assert_state_mismatch(
        &expected,
        &different_flat_order,
        FingerprintLocationV1::Node(root),
        FingerprintFieldV1::ChildOrder,
        FingerprintSummaryV1::Nodes(vec![member_seven]),
        FingerprintSummaryV1::Nodes(vec![other_child]),
    );
}

#[test]
fn global_counts_follow_child_order_and_v1_count_order() {
    let root = NodePathV1::root();
    let extra_node_path = root.clone().static_child(9);
    let extra_fragment_path = FragmentPathV1::new(root.clone(), 9);
    let expected_root = normalized_node(root.clone(), None, 0, 0, &[(0, 10)], Vec::new());
    let expected = NormalizedStateV1::new(vec![expected_root.clone()], Vec::new());
    let root_with_extra_property =
        normalized_node(root.clone(), None, 0, 0, &[(0, 10), (1, 20)], Vec::new());
    let extra_node = normalized_node(extra_node_path, Some(root.clone()), 1, 1, &[], Vec::new());
    let extra_fragment = normalized_fragment(extra_fragment_path, 9, Vec::new());

    let all_counts_differ = NormalizedStateV1::new(
        vec![root_with_extra_property.clone(), extra_node],
        vec![extra_fragment.clone()],
    );
    assert_state_mismatch(
        &expected,
        &all_counts_differ,
        FingerprintLocationV1::Global,
        FingerprintFieldV1::NodeCount,
        FingerprintSummaryV1::Count(1),
        FingerprintSummaryV1::Count(2),
    );

    let fragment_and_property_counts_differ =
        NormalizedStateV1::new(vec![root_with_extra_property.clone()], vec![extra_fragment]);
    assert_state_mismatch(
        &expected,
        &fragment_and_property_counts_differ,
        FingerprintLocationV1::Global,
        FingerprintFieldV1::FragmentCount,
        FingerprintSummaryV1::Count(0),
        FingerprintSummaryV1::Count(1),
    );

    let property_count_differs = NormalizedStateV1::new(vec![root_with_extra_property], Vec::new());
    assert_state_mismatch(
        &expected,
        &property_count_differs,
        FingerprintLocationV1::Global,
        FingerprintFieldV1::PropertyCount,
        FingerprintSummaryV1::Count(1),
        FingerprintSummaryV1::Count(2),
    );
}
