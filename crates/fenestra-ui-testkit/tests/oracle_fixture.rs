use fenestra_ui_ir::prototype::{
    ComponentTypeId, InputPolicy, PropertyId, PropertyValue, StructuralRegionId, TemplateNodeId,
};
use fenestra_ui_runtime::prototype::UiRuntime;
use fenestra_ui_testkit::prototype::{
    DesiredStateV1, FragmentPathV1, NodePathV1, NormalizedChildGroupV1, NormalizedStateV1,
    RuntimeOracleFixtureV1, clean_rebuild_v1, observe_snapshot_v1,
};

#[test]
fn initial_clean_state_matches_public_snapshot() {
    let fixture = RuntimeOracleFixtureV1::build().expect("fixture should validate");
    let desired =
        DesiredStateV1::from_construction(fixture.construction(), fixture.harness_limits())
            .expect("initial desired state should fit");
    let expected = clean_rebuild_v1(fixture.construction(), &desired, fixture.harness_limits())
        .expect("clean state should rebuild");

    let runtime = UiRuntime::new(
        fixture.construction().clone(),
        fixture.replay_config().runtime_capacity(),
    )
    .expect("fixture should initialize");
    let observed = observe_snapshot_v1(
        fixture.construction(),
        &runtime.committed(),
        fixture.harness_limits(),
    )
    .expect("snapshot should normalize");

    assert_eq!(observed, expected);
    assert_eq!(expected.node_count(), 9);
    assert_eq!(expected.fragment_count(), 4);
    assert_eq!(expected.property_slot_count(), 14);
    assert_exact_initial_state(&expected);
}

#[test]
fn semantic_paths_distinguish_adjacent_and_nested_fragments() {
    let fixture = RuntimeOracleFixtureV1::build().expect("fixture should validate");
    let desired =
        DesiredStateV1::from_construction(fixture.construction(), fixture.harness_limits())
            .expect("initial desired state should fit");
    let expected = clean_rebuild_v1(fixture.construction(), &desired, fixture.harness_limits())
        .expect("clean state should rebuild");

    let root = NodePathV1::root();
    let primary = FragmentPathV1::new(root.clone(), 1);
    let secondary = FragmentPathV1::new(root.clone(), 2);
    let primary_member = root.clone().member(1, 7);
    let secondary_member = root.member(2, 7);
    let nested = FragmentPathV1::new(primary_member.clone(), 1);

    assert_ne!(primary, secondary);
    assert_ne!(primary_member, secondary_member);
    assert!(expected.contains_fragment(&primary));
    assert!(expected.contains_fragment(&secondary));
    assert!(expected.contains_fragment(&nested));
    assert!(expected.contains_node(&primary_member));
    assert!(expected.contains_node(&secondary_member));

    let runtime = UiRuntime::new(
        fixture.construction().clone(),
        fixture.replay_config().runtime_capacity(),
    )
    .expect("fixture should initialize");
    let snapshot = runtime.committed();
    let primary_id = snapshot
        .fragment(snapshot.root(), StructuralRegionId::new(0))
        .expect("primary fragment should resolve");
    let secondary_id = snapshot
        .fragment(snapshot.root(), StructuralRegionId::new(2))
        .expect("secondary fragment should resolve");
    assert_ne!(primary_id, secondary_id);
}

fn assert_exact_initial_state(state: &NormalizedStateV1) {
    let root = NodePathV1::root();
    let static_child = root.clone().static_child(0);
    let primary_seven = root.clone().member(1, 7);
    let primary_eight = root.clone().member(1, 8);
    let secondary_seven = root.clone().member(2, 7);
    let expected_nodes = vec![
        root.clone(),
        static_child,
        primary_seven.clone(),
        primary_seven.clone().static_child(0),
        primary_seven.clone().member(1, 1),
        primary_eight.clone(),
        primary_eight.clone().static_child(0),
        primary_eight.clone().member(1, 1),
        secondary_seven,
    ];
    assert_eq!(
        state
            .nodes()
            .iter()
            .map(|node| node.path())
            .collect::<Vec<_>>(),
        expected_nodes.iter().collect::<Vec<_>>()
    );

    let root_node = state.node(&root).expect("root should normalize");
    assert_eq!(root_node.template(), TemplateNodeId::new(0));
    assert_eq!(root_node.component(), ComponentTypeId::new(0));
    assert_eq!(
        root_node
            .properties()
            .iter()
            .map(|property| (property.property(), property.value().clone()))
            .collect::<Vec<_>>(),
        vec![
            (PropertyId::new(0), PropertyValue::ScalarI32(120)),
            (PropertyId::new(1), PropertyValue::Bool(true)),
            (PropertyId::new(2), PropertyValue::Rgba8([0, 0, 0, 255]),),
            (
                PropertyId::new(3),
                PropertyValue::InputPolicy(InputPolicy::Accept),
            ),
        ]
    );
    assert_eq!(
        root_node.child_groups(),
        [
            NormalizedChildGroupV1::Static(root.clone().static_child(0)),
            NormalizedChildGroupV1::Region(FragmentPathV1::new(root.clone(), 1)),
            NormalizedChildGroupV1::Region(FragmentPathV1::new(root.clone(), 2)),
        ]
    );

    let expected_fragments = [
        (FragmentPathV1::new(root.clone(), 1), 0, vec![7, 8]),
        (FragmentPathV1::new(primary_seven, 1), 1, vec![1]),
        (FragmentPathV1::new(primary_eight, 1), 1, vec![1]),
        (FragmentPathV1::new(root, 2), 2, vec![7]),
    ];
    for (found, (path, descriptor, keys)) in state.fragments().iter().zip(expected_fragments) {
        assert_eq!(found.path(), &path);
        assert_eq!(found.descriptor(), StructuralRegionId::new(descriptor));
        assert_eq!(
            found
                .members()
                .iter()
                .map(|member| member.key())
                .collect::<Vec<_>>(),
            keys
        );
    }
}
