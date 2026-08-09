use super::*;

#[test]
fn recreated_paths_do_not_restore_retired_base_targets() {
    let fixture = RuntimeOracleFixtureV1::build().expect("fixture should validate");
    let limits = fixture.harness_limits();
    let mut desired = DesiredStateV1::from_construction(fixture.construction(), limits)
        .expect("desired state should build");
    let base =
        clean_rebuild_v1(fixture.construction(), &desired, limits).expect("base should rebuild");
    let base_nodes = base
        .nodes()
        .iter()
        .map(|node| target_node(&desired, node.path()))
        .collect::<Vec<_>>();
    let base_fragments = base
        .fragments()
        .iter()
        .map(|fragment| target_fragment(&desired, fragment.path()))
        .collect::<Vec<_>>();

    let root = NodePathV1::root();
    let retired = root.clone().member(1, 7);
    let primary = FragmentPathV1::new(root, 1);
    desired
        .apply_operation(
            &SemanticOperationV1::RemoveKeyed {
                fragment: primary.clone(),
                key: 7,
            },
            limits,
        )
        .expect("base member should remove");
    desired
        .apply_operation(
            &SemanticOperationV1::InsertKeyed {
                fragment: primary,
                key: 7,
                final_index: 1,
            },
            limits,
        )
        .expect("same semantic path should reinsert");

    let actions = applicable_actions(
        &fixture,
        &desired,
        &base_nodes,
        &base_fragments,
        GeneratorConfigV1::new(8, 2, 8),
    )
    .expect("remaining base targets should yield actions");
    assert!(!actions.is_empty());
    assert!(actions.iter().all(|operation| match operation {
        SemanticOperationV1::SetProperty { node, .. } => !is_within(node, &retired),
        SemanticOperationV1::InsertKeyed { fragment, .. }
        | SemanticOperationV1::MoveKeyed { fragment, .. }
        | SemanticOperationV1::UpdateKeyed { fragment, .. }
        | SemanticOperationV1::RemoveKeyed { fragment, .. } => {
            !is_within(fragment.owner(), &retired)
        }
    }));
}

fn target_node(desired: &DesiredStateV1, path: &NodePathV1) -> (NodePathV1, Vec<u64>) {
    (
        path.clone(),
        desired
            .incarnation_token(path)
            .expect("live node should have an incarnation token"),
    )
}

fn target_fragment(desired: &DesiredStateV1, path: &FragmentPathV1) -> (FragmentPathV1, Vec<u64>) {
    (
        path.clone(),
        desired
            .incarnation_token(path.owner())
            .expect("live owner should have an incarnation token"),
    )
}

fn is_within(candidate: &NodePathV1, ancestor: &NodePathV1) -> bool {
    candidate.segments().starts_with(ancestor.segments())
}
