mod artifact;
mod identity;
mod receipt;
mod runner;

use fenestra_ui_ir::prototype::{InvalidationClass, InvalidationSet, PropertyValue};
use fenestra_ui_testkit::prototype::{
    FragmentPathV1, HeadlessFixtureV1, NodePathV1, SemanticOperationV1,
};

pub use artifact::{
    REGISTERED_RUNTIME_ARTIFACT_LIMITS_V1, RuntimeArtifactEncodeErrorKindV1,
    RuntimeArtifactFaultV1, RuntimeArtifactLimitKindV1, RuntimeArtifactLimitsV1,
    encode_runtime_artifact_v1, inject_runtime_artifact_fault_v1,
};
pub use receipt::{NormalizedManifestEntry, NormalizedMutation, NormalizedReceipt};
pub use runner::{LaneLog, oracle_projection_log, run_lane, validate_programs};

const INSERTED_KEY: u64 = 30;
const REMOVED_KEY: u64 = 20;

pub fn registered_operations(fixture: &HeadlessFixtureV1) -> Vec<SemanticOperationV1> {
    vec![
        SemanticOperationV1::SetProperty {
            node: control_path(),
            property: fixture.spec().color(),
            value: PropertyValue::Rgba8([20, 30, 40, 255]),
        },
        SemanticOperationV1::InsertKeyed {
            fragment: items_path(),
            key: INSERTED_KEY,
            final_index: 1,
        },
        SemanticOperationV1::MoveKeyed {
            fragment: items_path(),
            key: INSERTED_KEY,
            final_index: 2,
        },
        SemanticOperationV1::UpdateKeyed {
            fragment: items_path(),
            key: INSERTED_KEY,
            property: fixture.spec().height(),
            value: PropertyValue::ScalarI32(14),
        },
        SemanticOperationV1::RemoveKeyed {
            fragment: items_path(),
            key: REMOVED_KEY,
        },
    ]
}

pub fn expected_receipts(fixture: &HeadlessFixtureV1) -> Vec<NormalizedReceipt> {
    vec![
        NormalizedReceipt::new(0, Vec::new(), InvalidationSet::NONE),
        NormalizedReceipt::new(
            1,
            vec![NormalizedMutation::PropertyChanged {
                node: control_path(),
                property: fixture.spec().color(),
                old_value: PropertyValue::Rgba8([10, 20, 30, 255]),
                new_value: PropertyValue::Rgba8([20, 30, 40, 255]),
            }],
            paint(),
        ),
        NormalizedReceipt::new(
            2,
            vec![NormalizedMutation::KeyInserted {
                fragment: items_path(),
                key: INSERTED_KEY,
                root: item_path(INSERTED_KEY),
                final_index: 1,
                created: vec![NormalizedManifestEntry::Node(item_path(INSERTED_KEY))],
            }],
            region_invalidation(),
        ),
        NormalizedReceipt::new(
            3,
            vec![NormalizedMutation::KeyMoved {
                fragment: items_path(),
                key: INSERTED_KEY,
                root: item_path(INSERTED_KEY),
                old_index: 1,
                final_index: 2,
            }],
            region_invalidation(),
        ),
        NormalizedReceipt::new(
            4,
            vec![NormalizedMutation::PropertyChanged {
                node: item_path(INSERTED_KEY),
                property: fixture.spec().height(),
                old_value: PropertyValue::ScalarI32(12),
                new_value: PropertyValue::ScalarI32(14),
            }],
            dimension_invalidation(),
        ),
        NormalizedReceipt::new(
            5,
            vec![NormalizedMutation::KeyRemoved {
                fragment: items_path(),
                key: REMOVED_KEY,
                root: item_path(REMOVED_KEY),
                old_index: 1,
                retired: vec![NormalizedManifestEntry::Node(item_path(REMOVED_KEY))],
            }],
            region_invalidation(),
        ),
    ]
}

pub(super) fn control_path() -> NodePathV1 {
    NodePathV1::root().static_child(0).static_child(0)
}

pub(super) fn item_path(key: u64) -> NodePathV1 {
    NodePathV1::root().static_child(0).member(1, key)
}

pub(super) fn items_path() -> FragmentPathV1 {
    FragmentPathV1::new(NodePathV1::root().static_child(0), 1)
}

fn paint() -> InvalidationSet {
    InvalidationSet::from_class(InvalidationClass::Paint)
}

fn dimension_invalidation() -> InvalidationSet {
    invalidation(&[
        InvalidationClass::Layout,
        InvalidationClass::Semantics,
        InvalidationClass::HitTest,
        InvalidationClass::Paint,
        InvalidationClass::Composition,
    ])
}

fn region_invalidation() -> InvalidationSet {
    invalidation(&[
        InvalidationClass::Structure,
        InvalidationClass::Layout,
        InvalidationClass::Semantics,
        InvalidationClass::HitTest,
        InvalidationClass::Paint,
        InvalidationClass::Composition,
    ])
}

fn invalidation(classes: &[InvalidationClass]) -> InvalidationSet {
    classes.iter().fold(InvalidationSet::NONE, |set, class| {
        set.union(InvalidationSet::from_class(*class))
    })
}
