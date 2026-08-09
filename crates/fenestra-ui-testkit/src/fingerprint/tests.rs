use fenestra_ui_ir::prototype::{
    ComponentTypeId, PropertyId, PropertyValue, StructuralRegionId, TemplateNodeId,
};

use super::{
    FailureFingerprintKindV1, FailureFingerprintV1, FingerprintFieldV1, FingerprintLocationV1,
    FingerprintSummaryV1, compare_normalized_state_v1,
};
use crate::semantic::{
    FragmentPathV1, NodePathV1, NormalizedChildGroupV1, NormalizedFragmentV1, NormalizedMemberV1,
    NormalizedNodeV1, NormalizedPropertyV1, NormalizedStateV1,
};

mod comparison_order;
mod keyed_move;
mod types;

#[test]
fn equal_normalized_states_have_no_fingerprint() {
    let state = single_node_state(0, 0, &[(0, 10)], None, Vec::new());

    let fingerprint =
        compare_normalized_state_v1(&state, &state).expect("equal state comparison should succeed");

    assert!(fingerprint.is_none());
}

fn assert_state_mismatch(
    expected_state: &NormalizedStateV1,
    observed_state: &NormalizedStateV1,
    location: FingerprintLocationV1,
    field: FingerprintFieldV1,
    expected: FingerprintSummaryV1,
    observed: FingerprintSummaryV1,
) {
    let fingerprint: FailureFingerprintV1 =
        compare_normalized_state_v1(expected_state, observed_state)
            .expect("state comparison should succeed")
            .expect("states should mismatch");
    assert_eq!(fingerprint.kind(), FailureFingerprintKindV1::StateMismatch);
    assert_eq!(fingerprint.location(), &location);
    assert_eq!(fingerprint.field(), field);
    assert_eq!(fingerprint.expected(), &expected);
    assert_eq!(fingerprint.observed(), &observed);
}

fn single_node_state(
    template: u32,
    component: u32,
    properties: &[(u32, i32)],
    parent: Option<NodePathV1>,
    child_groups: Vec<NormalizedChildGroupV1>,
) -> NormalizedStateV1 {
    NormalizedStateV1::new(
        vec![normalized_node(
            NodePathV1::root(),
            parent,
            template,
            component,
            properties,
            child_groups,
        )],
        Vec::new(),
    )
}

fn normalized_node(
    path: NodePathV1,
    parent: Option<NodePathV1>,
    template: u32,
    component: u32,
    properties: &[(u32, i32)],
    child_groups: Vec<NormalizedChildGroupV1>,
) -> NormalizedNodeV1 {
    NormalizedNodeV1::new(
        path,
        parent,
        TemplateNodeId::new(template),
        ComponentTypeId::new(component),
        properties
            .iter()
            .map(|&(property, value)| {
                NormalizedPropertyV1::new(
                    PropertyId::new(property),
                    PropertyValue::ScalarI32(value),
                )
            })
            .collect(),
        child_groups,
    )
}

fn normalized_fragment(
    path: FragmentPathV1,
    descriptor: u32,
    members: Vec<(u64, NodePathV1)>,
) -> NormalizedFragmentV1 {
    NormalizedFragmentV1::new(
        path,
        StructuralRegionId::new(descriptor),
        members
            .into_iter()
            .map(|(key, node)| NormalizedMemberV1::new(key, node))
            .collect(),
    )
}

fn property_summary(property: u32, value: i32) -> FingerprintSummaryV1 {
    FingerprintSummaryV1::Property(PropertyId::new(property), PropertyValue::ScalarI32(value))
}
