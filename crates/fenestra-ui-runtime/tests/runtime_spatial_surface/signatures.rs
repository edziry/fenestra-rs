use std::sync::Arc;

use fenestra_ui_ir::prototype::{
    ComponentTypeId, PropertyId, PropertyValue, StructuralRegionId, TemplateNodeId,
    ValidatedStyleProgram,
};
use fenestra_ui_layout::prototype::LayoutEngineV1;
use fenestra_ui_spatial::prototype::{
    SpatialLimitsV2, SpatialNodeKeyV2, SpatialOwnedInputV2, SpatialResolvedSnapshotV2,
    SpatialViewportV2,
};

use crate::{
    RuntimeSpatialBuildViewV2, RuntimeSpatialInputV2, RuntimeSpatialProgramV2,
    RuntimeSpatialViewV2, SpatialViewportChangeViewV2,
};
use fenestra_ui_runtime::prototype::{
    CommittedRuntimeSnapshot, FragmentId, KeyedMemberIter, NodeId, RuntimeCapacity,
    RuntimeInitializationError, TransactionError, UiRuntime, UiTransaction,
};

struct ExactProgram;

impl RuntimeSpatialProgramV2 for ExactProgram {
    fn build(
        &self,
        _runtime: RuntimeSpatialBuildViewV2<'_>,
        _viewport: SpatialViewportV2,
    ) -> RuntimeSpatialInputV2 {
        panic!("signature probe does not invoke the program")
    }
}

#[test]
fn runtime_spatial_program_is_object_safe_with_the_exact_build_signature() {
    let program: Box<dyn RuntimeSpatialProgramV2> = Box::new(ExactProgram);
    drop(program);
}

#[test]
fn runtime_spatial_input_constructor_signature_is_exact() {
    let _: fn(Arc<SpatialOwnedInputV2>, Box<[NodeId]>) -> RuntimeSpatialInputV2 =
        RuntimeSpatialInputV2::new;
}

#[test]
fn runtime_spatial_build_view_signatures_are_exact() {
    assert_runtime_spatial_build_view_signatures();
}

#[test]
fn runtime_spatial_observation_signatures_are_exact() {
    assert_runtime_spatial_observation_signatures();
}

#[allow(clippy::type_complexity)]
#[test]
fn runtime_spatial_owner_method_signatures_are_exact() {
    let _: fn(
        ValidatedStyleProgram,
        Box<dyn RuntimeSpatialProgramV2>,
        SpatialViewportV2,
        SpatialLimitsV2,
        RuntimeCapacity,
    ) -> Result<UiRuntime, RuntimeInitializationError> = UiRuntime::new_spatial;
    let _: fn(
        ValidatedStyleProgram,
        Box<dyn RuntimeSpatialProgramV2>,
        SpatialViewportV2,
        SpatialLimitsV2,
        RuntimeCapacity,
        Box<dyn LayoutEngineV1>,
    ) -> Result<UiRuntime, RuntimeInitializationError> = UiRuntime::new_spatial_with_layout_engine;
    let _: fn(&mut UiTransaction, SpatialViewportV2) -> Result<(), TransactionError> =
        UiTransaction::resize_spatial;
    let _: for<'a> fn(&'a CommittedRuntimeSnapshot) -> Option<RuntimeSpatialViewV2<'a>> =
        CommittedRuntimeSnapshot::spatial;
}

fn assert_runtime_spatial_build_view_signatures<'a>() {
    let _: fn(RuntimeSpatialBuildViewV2<'a>) -> NodeId = RuntimeSpatialBuildViewV2::root;
    let _: fn(RuntimeSpatialBuildViewV2<'a>) -> usize = RuntimeSpatialBuildViewV2::node_count;
    let _: fn(RuntimeSpatialBuildViewV2<'a>, NodeId) -> Option<TemplateNodeId> =
        RuntimeSpatialBuildViewV2::template;
    let _: fn(RuntimeSpatialBuildViewV2<'a>, NodeId) -> Option<ComponentTypeId> =
        RuntimeSpatialBuildViewV2::component;
    let _: fn(RuntimeSpatialBuildViewV2<'a>, NodeId, PropertyId) -> Option<&'a PropertyValue> =
        RuntimeSpatialBuildViewV2::property;
    let _: fn(RuntimeSpatialBuildViewV2<'a>, NodeId) -> Option<NodeId> =
        RuntimeSpatialBuildViewV2::parent;
    let _: fn(RuntimeSpatialBuildViewV2<'a>, NodeId) -> Option<&'a [NodeId]> =
        RuntimeSpatialBuildViewV2::children;
    let _: fn(RuntimeSpatialBuildViewV2<'a>, NodeId, StructuralRegionId) -> Option<FragmentId> =
        RuntimeSpatialBuildViewV2::fragment;
    let _: fn(RuntimeSpatialBuildViewV2<'a>, FragmentId) -> Option<KeyedMemberIter<'a>> =
        RuntimeSpatialBuildViewV2::keyed_members;
    let _: fn(RuntimeSpatialBuildViewV2<'a>, FragmentId, u64) -> Option<NodeId> =
        RuntimeSpatialBuildViewV2::keyed_member;
}

fn assert_runtime_spatial_observation_signatures<'a>() {
    let _: fn(RuntimeSpatialViewV2<'a>) -> &'a SpatialResolvedSnapshotV2 =
        RuntimeSpatialViewV2::snapshot;
    let _: fn(RuntimeSpatialViewV2<'a>, SpatialNodeKeyV2) -> Option<NodeId> =
        RuntimeSpatialViewV2::logical_node;
    let _: fn(RuntimeSpatialViewV2<'a>, NodeId) -> Option<SpatialNodeKeyV2> =
        RuntimeSpatialViewV2::spatial_key;
    let _: fn(SpatialViewportChangeViewV2<'a>) -> SpatialViewportV2 =
        SpatialViewportChangeViewV2::old_viewport;
    let _: fn(SpatialViewportChangeViewV2<'a>) -> SpatialViewportV2 =
        SpatialViewportChangeViewV2::new_viewport;
}
