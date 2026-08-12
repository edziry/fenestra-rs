use fenestra_ui_ir::prototype::{
    SpatialAnchorComponentV2, SpatialAnchorTargetRecipeV2, SpatialAxisV2, SpatialContainerRecipeV2,
    SpatialDimensionRecipeV2, SpatialFreePlacementRecipeV2, SpatialLayoutPlacementRecipeV2,
    SpatialNodeParentV2, SpatialPaddingRecipeV2, SpatialPlacementRecipeV2, SpatialPointRecipeV2,
    SpatialTransformRecipeV2, SpatialViewportContainerV2,
};
use proc_macro2::TokenStream;

use crate::emitter::builder::{array, ir_call, ir_path};

use super::value::{field_i32, field_i32_binding, field_i64_binding, field_node, source_span};

pub(super) fn viewport_container(value: SpatialViewportContainerV2) -> TokenStream {
    ir_call(
        &["SpatialViewportContainerV2", "new"],
        vec![
            axis(value.axis()),
            field_i32(value.left()),
            field_i32(value.right()),
            field_i32(value.top()),
            field_i32(value.bottom()),
            field_i32(value.gap()),
            source_span(value.span()),
        ],
        true,
    )
}

pub(super) fn container(value: SpatialContainerRecipeV2) -> TokenStream {
    ir_call(
        &["SpatialContainerRecipeV2", "new"],
        vec![
            axis(value.axis()),
            padding(value.padding()),
            field_i32_binding(value.gap()),
        ],
        true,
    )
}

pub(super) fn placement(value: SpatialPlacementRecipeV2) -> TokenStream {
    match value {
        SpatialPlacementRecipeV2::Layout(value) => ir_call(
            &["SpatialPlacementRecipeV2", "Layout"],
            vec![layout_placement(value)],
            false,
        ),
        SpatialPlacementRecipeV2::Free(value) => ir_call(
            &["SpatialPlacementRecipeV2", "Free"],
            vec![free_placement(value)],
            false,
        ),
    }
}

pub(super) fn node_parent(value: SpatialNodeParentV2) -> TokenStream {
    match value {
        SpatialNodeParentV2::Viewport => ir_path(&["SpatialNodeParentV2", "Viewport"]),
        SpatialNodeParentV2::Node(node) => ir_call(
            &["SpatialNodeParentV2", "Node"],
            vec![field_node(node)],
            false,
        ),
    }
}

pub(super) fn point(value: SpatialPointRecipeV2) -> TokenStream {
    ir_call(
        &["SpatialPointRecipeV2", "new"],
        vec![field_i64_binding(value.x()), field_i64_binding(value.y())],
        true,
    )
}

fn padding(value: SpatialPaddingRecipeV2) -> TokenStream {
    ir_call(
        &["SpatialPaddingRecipeV2", "new"],
        vec![
            field_i32_binding(value.left()),
            field_i32_binding(value.right()),
            field_i32_binding(value.top()),
            field_i32_binding(value.bottom()),
        ],
        true,
    )
}

fn dimension(value: SpatialDimensionRecipeV2) -> TokenStream {
    ir_call(
        &["SpatialDimensionRecipeV2", "new"],
        vec![
            field_i32_binding(value.minimum()),
            field_i32_binding(value.preferred()),
            field_i32_binding(value.maximum()),
        ],
        true,
    )
}

fn transform(value: SpatialTransformRecipeV2) -> TokenStream {
    ir_call(
        &["SpatialTransformRecipeV2", "new"],
        vec![
            field_i64_binding(value.a()),
            field_i64_binding(value.b()),
            field_i64_binding(value.c()),
            field_i64_binding(value.d()),
            field_i64_binding(value.tx()),
            field_i64_binding(value.ty()),
            point(value.origin()),
        ],
        true,
    )
}

fn layout_placement(value: SpatialLayoutPlacementRecipeV2) -> TokenStream {
    ir_call(
        &["SpatialLayoutPlacementRecipeV2", "new"],
        vec![
            dimension(value.width()),
            dimension(value.height()),
            transform(value.transform()),
        ],
        true,
    )
}

fn free_placement(value: SpatialFreePlacementRecipeV2) -> TokenStream {
    ir_call(
        &["SpatialFreePlacementRecipeV2", "new"],
        vec![
            field_i32_binding(value.width()),
            field_i32_binding(value.height()),
            anchor_pair(value.self_anchor()),
            anchor_target(value.target()),
            anchor_pair(value.target_anchor()),
            point(value.offset()),
            transform(value.transform()),
        ],
        true,
    )
}

fn axis(value: SpatialAxisV2) -> TokenStream {
    let variant = match value {
        SpatialAxisV2::Row => "Row",
        SpatialAxisV2::Column => "Column",
    };
    ir_path(&["SpatialAxisV2", variant])
}

fn anchor_pair(value: [SpatialAnchorComponentV2; 2]) -> TokenStream {
    array(value.into_iter().map(anchor_component).collect(), false)
}

fn anchor_component(value: SpatialAnchorComponentV2) -> TokenStream {
    let variant = match value {
        SpatialAnchorComponentV2::Start => "Start",
        SpatialAnchorComponentV2::Center => "Center",
        SpatialAnchorComponentV2::End => "End",
    };
    ir_path(&["SpatialAnchorComponentV2", variant])
}

fn anchor_target(value: SpatialAnchorTargetRecipeV2) -> TokenStream {
    match value {
        SpatialAnchorTargetRecipeV2::Viewport => {
            ir_path(&["SpatialAnchorTargetRecipeV2", "Viewport"])
        }
        SpatialAnchorTargetRecipeV2::Parent => ir_path(&["SpatialAnchorTargetRecipeV2", "Parent"]),
        SpatialAnchorTargetRecipeV2::Node(node) => ir_call(
            &["SpatialAnchorTargetRecipeV2", "Node"],
            vec![field_node(node)],
            false,
        ),
    }
}
