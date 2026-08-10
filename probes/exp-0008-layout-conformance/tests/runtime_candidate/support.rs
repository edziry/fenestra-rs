use fenestra_ui_ir::prototype::{InvalidationClass, InvalidationSet, StructuralRegionId};
use fenestra_ui_runtime::prototype::{CommittedRuntimeSnapshot, FragmentId, UiTransaction};
use fenestra_ui_testkit::prototype::{
    FragmentPathV1, NodePathV1, PathSegmentV1, SemanticOperationV1,
};

pub(super) const SECOND_KEY: u64 = 20;
pub(super) const INSERTED_KEY: u64 = 30;

const ITEMS_REGION: StructuralRegionId = StructuralRegionId::new(0);

pub(super) const PAINT_INVALIDATION: InvalidationSet =
    InvalidationSet::from_class(InvalidationClass::Paint);
pub(super) const DIMENSION_INVALIDATION: InvalidationSet = InvalidationSet::NONE
    .union(InvalidationSet::from_class(InvalidationClass::Layout))
    .union(InvalidationSet::from_class(InvalidationClass::Semantics))
    .union(InvalidationSet::from_class(InvalidationClass::HitTest))
    .union(InvalidationSet::from_class(InvalidationClass::Paint))
    .union(InvalidationSet::from_class(InvalidationClass::Composition));
pub(super) const REGION_INVALIDATION: InvalidationSet = InvalidationSet::NONE
    .union(InvalidationSet::from_class(InvalidationClass::Structure))
    .union(DIMENSION_INVALIDATION);
pub(super) const RESIZE_INVALIDATION: InvalidationSet = InvalidationSet::NONE
    .union(InvalidationSet::from_class(InvalidationClass::Surface))
    .union(DIMENSION_INVALIDATION);

pub(super) fn control_path() -> NodePathV1 {
    NodePathV1::root().static_child(0).static_child(0)
}

pub(super) fn items_path() -> FragmentPathV1 {
    FragmentPathV1::new(NodePathV1::root().static_child(0), 1)
}

pub(super) fn stage_operation(
    transaction: &mut UiTransaction,
    snapshot: &CommittedRuntimeSnapshot,
    operation: &SemanticOperationV1,
) {
    match operation {
        SemanticOperationV1::SetProperty {
            node,
            property,
            value,
        } => transaction
            .set_property(resolve_node(snapshot, node), *property, value.clone())
            .expect("registered direct update must stage"),
        SemanticOperationV1::InsertKeyed {
            fragment,
            key,
            final_index,
        } => transaction
            .insert_keyed(
                resolve_fragment(snapshot, fragment),
                *key,
                usize::try_from(*final_index).expect("registered index must fit usize"),
            )
            .expect("registered insertion must stage"),
        SemanticOperationV1::MoveKeyed {
            fragment,
            key,
            final_index,
        } => transaction
            .move_keyed(
                resolve_fragment(snapshot, fragment),
                *key,
                usize::try_from(*final_index).expect("registered index must fit usize"),
            )
            .expect("registered move must stage"),
        SemanticOperationV1::UpdateKeyed {
            fragment,
            key,
            property,
            value,
        } => transaction
            .update_keyed(
                resolve_fragment(snapshot, fragment),
                *key,
                *property,
                value.clone(),
            )
            .expect("registered keyed update must stage"),
        SemanticOperationV1::RemoveKeyed { fragment, key } => transaction
            .remove_keyed(resolve_fragment(snapshot, fragment), *key)
            .expect("registered removal must stage"),
    }
}

fn resolve_node(
    snapshot: &CommittedRuntimeSnapshot,
    path: &NodePathV1,
) -> fenestra_ui_runtime::prototype::NodeId {
    let mut node = snapshot.root();
    for segment in path.segments() {
        node = match segment {
            PathSegmentV1::Static { authored_slot: 0 } => snapshot
                .children(node)
                .expect("registered static owner must be live")
                .first()
                .copied()
                .expect("registered static child must exist"),
            PathSegmentV1::Member {
                region_slot: 1,
                key,
            } => snapshot
                .keyed_member(
                    snapshot
                        .fragment(node, ITEMS_REGION)
                        .expect("registered fragment must be live"),
                    *key,
                )
                .expect("registered keyed member must be live"),
            PathSegmentV1::Static { .. } | PathSegmentV1::Member { .. } => {
                panic!("path is outside the registered fixture")
            }
        };
    }
    node
}

fn resolve_fragment(snapshot: &CommittedRuntimeSnapshot, path: &FragmentPathV1) -> FragmentId {
    assert_eq!(path.region_slot(), 1);
    snapshot
        .fragment(resolve_node(snapshot, path.owner()), ITEMS_REGION)
        .expect("registered fragment must be live")
}
