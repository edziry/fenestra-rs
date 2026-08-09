#![allow(dead_code)]

use fenestra_ui_ir::prototype::{
    ComponentTypeId, InputPolicy, PropertyId, PropertyValue, StructuralRegionId, TemplateNodeId,
};
use fenestra_ui_runtime::prototype::{
    CommittedRuntimeSnapshot, FragmentId, HeadlessRect, NodeId, UiRuntime,
};
use fenestra_ui_testkit::prototype::{
    FragmentPathV1, HeadlessFixtureV1, HeadlessOracleV1, NodePathV1,
    NormalizedHeadlessProjectionV1, ObservedHeadlessProjectionV1, PathSegmentV1,
    SemanticOperationV1, compare_headless_projection_v1, observe_headless_projection_v1,
};

pub const WIDTH: PropertyId = PropertyId::new(0);
pub const HEIGHT: PropertyId = PropertyId::new(1);
pub const COLOR: PropertyId = PropertyId::new(2);
pub const VISIBLE: PropertyId = PropertyId::new(3);
pub const INPUT: PropertyId = PropertyId::new(4);
pub const COMPONENT: ComponentTypeId = ComponentTypeId::new(0);

pub const ROOT_TEMPLATE: TemplateNodeId = TemplateNodeId::new(0);
pub const CONTAINER_TEMPLATE: TemplateNodeId = TemplateNodeId::new(1);
pub const CONTROL_TEMPLATE: TemplateNodeId = TemplateNodeId::new(2);
pub const ITEM_TEMPLATE: TemplateNodeId = TemplateNodeId::new(3);
pub const ITEMS_REGION: StructuralRegionId = StructuralRegionId::new(0);

pub const FIRST_KEY: u64 = 10;
pub const SECOND_KEY: u64 = 20;
pub const INSERTED_KEY: u64 = 30;

pub fn root_path() -> NodePathV1 {
    NodePathV1::root()
}

pub fn container_path() -> NodePathV1 {
    root_path().static_child(0)
}

pub fn control_path() -> NodePathV1 {
    container_path().static_child(0)
}

pub fn item_path(key: u64) -> NodePathV1 {
    container_path().member(1, key)
}

pub fn items_path() -> FragmentPathV1 {
    FragmentPathV1::new(container_path(), 1)
}

pub fn runtime(fixture: &HeadlessFixtureV1) -> UiRuntime {
    UiRuntime::new_headless(
        fixture.style().clone(),
        fixture.spec(),
        fixture.surface(),
        fixture.runtime_capacity(),
    )
    .expect("registered headless fixture should initialize")
}

pub fn assert_oracle_matches(
    fixture: &HeadlessFixtureV1,
    oracle: &HeadlessOracleV1,
    snapshot: &CommittedRuntimeSnapshot,
) -> ObservedHeadlessProjectionV1 {
    let expected = oracle.rebuild().expect("clean rebuild should succeed");
    let observed = observe_headless_projection_v1(fixture, snapshot)
        .expect("committed headless projection should normalize");
    assert_eq!(observed.generation(), snapshot.generation());
    assert_eq!(expected.surface(), observed.projection().surface());
    assert_eq!(
        compare_headless_projection_v1(&expected, observed.projection())
            .expect("matching surfaces should compare"),
        None
    );
    observed
}

pub fn apply_operation(
    fixture: &HeadlessFixtureV1,
    runtime: &mut UiRuntime,
    oracle: &mut HeadlessOracleV1,
    operation: &SemanticOperationV1,
) -> CommittedRuntimeSnapshot {
    let before = runtime.committed();
    let mut transaction = runtime.begin_transaction();
    match operation {
        SemanticOperationV1::SetProperty {
            node,
            property,
            value,
        } => transaction
            .set_property(node_id(&before, node), *property, value.clone())
            .expect("valid direct operation should stage"),
        SemanticOperationV1::InsertKeyed {
            fragment,
            key,
            final_index,
        } => transaction
            .insert_keyed(
                fragment_id(&before, fragment),
                *key,
                usize::try_from(*final_index).expect("fixture index should fit usize"),
            )
            .expect("valid insertion should stage"),
        SemanticOperationV1::MoveKeyed {
            fragment,
            key,
            final_index,
        } => transaction
            .move_keyed(
                fragment_id(&before, fragment),
                *key,
                usize::try_from(*final_index).expect("fixture index should fit usize"),
            )
            .expect("valid move should stage"),
        SemanticOperationV1::UpdateKeyed {
            fragment,
            key,
            property,
            value,
        } => transaction
            .update_keyed(
                fragment_id(&before, fragment),
                *key,
                *property,
                value.clone(),
            )
            .expect("valid keyed update should stage"),
        SemanticOperationV1::RemoveKeyed { fragment, key } => transaction
            .remove_keyed(fragment_id(&before, fragment), *key)
            .expect("valid removal should stage"),
    }
    oracle
        .apply_operation(operation)
        .expect("valid desired operation should apply");
    runtime
        .commit(transaction)
        .expect("valid candidate operation should publish");
    let after = runtime.committed();
    assert_oracle_matches(fixture, oracle, &after);
    after
}

pub fn node_id(snapshot: &CommittedRuntimeSnapshot, path: &NodePathV1) -> NodeId {
    let mut node = snapshot.root();
    for segment in path.segments() {
        node = match segment {
            PathSegmentV1::Static { authored_slot: 0 } => snapshot
                .children(node)
                .expect("static owner should be live")
                .first()
                .copied()
                .expect("static child should be present"),
            PathSegmentV1::Static { .. } => panic!("fixture has no other static slot"),
            PathSegmentV1::Member {
                region_slot: 1,
                key,
            } => {
                let fragment = snapshot
                    .fragment(node, ITEMS_REGION)
                    .expect("item fragment should be live");
                snapshot
                    .keyed_member(fragment, *key)
                    .expect("keyed member should be live")
            }
            PathSegmentV1::Member { .. } => panic!("fixture has no other region slot"),
        };
    }
    node
}

pub fn fragment_id(snapshot: &CommittedRuntimeSnapshot, path: &FragmentPathV1) -> FragmentId {
    assert_eq!(path, &items_path());
    snapshot
        .fragment(node_id(snapshot, path.owner()), ITEMS_REGION)
        .expect("item fragment should be live")
}

pub fn computed_tuples(
    projection: &NormalizedHeadlessProjectionV1,
) -> Vec<(NodePathV1, i32, i32, [u8; 4], bool, InputPolicy)> {
    projection
        .computed_styles()
        .iter()
        .map(|record| {
            (
                record.path().clone(),
                record.width(),
                record.height(),
                record.color(),
                record.visible(),
                record.input(),
            )
        })
        .collect()
}

pub fn geometry_tuples(
    projection: &NormalizedHeadlessProjectionV1,
) -> Vec<(NodePathV1, HeadlessRect, HeadlessRect)> {
    projection
        .geometries()
        .iter()
        .map(|record| (record.path().clone(), record.bounds(), record.clip()))
        .collect()
}

pub fn path_order<'a>(paths: impl Iterator<Item = &'a NodePathV1>) -> Vec<NodePathV1> {
    paths.cloned().collect()
}

pub fn rgba(value: [u8; 4]) -> PropertyValue {
    PropertyValue::Rgba8(value)
}
