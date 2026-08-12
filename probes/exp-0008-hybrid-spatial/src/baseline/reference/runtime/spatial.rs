use fenestra_ui_ir::prototype::{
    SUPPORTED_SPATIAL_FORMAT, SchemaNamespace, SchemaRevision, SpatialAxisV2, SpatialBindingV2,
    SpatialContainerRecipeV2, SpatialDimensionRecipeV2, SpatialFieldV2,
    SpatialLayoutPlacementRecipeV2, SpatialNodeDeclarationV2, SpatialNodeParentV2,
    SpatialPaddingRecipeV2, SpatialPlacementRecipeV2, SpatialProgramV2, SpatialTransformRecipeV2,
    SpatialViewportContainerV2,
};

use super::value::{fixed_lit, fixed_prop, i32_lit, i32_prop, node, point, span, template};

pub(super) fn program() -> SpatialProgramV2 {
    SpatialProgramV2::new(
        SUPPORTED_SPATIAL_FORMAT,
        SchemaNamespace::new(80_008),
        SchemaRevision::new(2),
        SpatialViewportContainerV2::new(
            SpatialAxisV2::Column,
            field_i32(0, 42),
            field_i32(0, 43),
            field_i32(0, 44),
            field_i32(0, 45),
            field_i32(0, 46),
            span(41),
        ),
        vec![root(), fixed_child(), keyed_child()],
        Vec::new(),
        span(40),
    )
}

fn root() -> SpatialNodeDeclarationV2 {
    declaration(
        0,
        0,
        SpatialNodeParentV2::Viewport,
        layout(
            [i32_lit(0, 55), i32_prop(0, 56), i32_lit(240, 57)],
            [i32_lit(0, 58), i32_prop(1, 59), i32_lit(180, 60)],
            transform_with_factor(62),
        ),
        50,
    )
}

fn fixed_child() -> SpatialNodeDeclarationV2 {
    declaration(
        1,
        1,
        SpatialNodeParentV2::Node(node(0, 72)),
        layout(
            [i32_lit(0, 73), i32_prop(0, 74), i32_lit(80, 75)],
            [i32_lit(0, 76), i32_prop(1, 77), i32_lit(60, 78)],
            identity(80),
        ),
        70,
    )
}

fn keyed_child() -> SpatialNodeDeclarationV2 {
    declaration(
        2,
        2,
        SpatialNodeParentV2::Node(node(0, 92)),
        layout(
            [i32_lit(0, 93), i32_prop(0, 94), i32_lit(32, 95)],
            [i32_lit(0, 96), i32_prop(1, 97), i32_lit(24, 98)],
            identity(100),
        ),
        90,
    )
}

fn declaration(
    symbol: u32,
    template_id: u32,
    parent: SpatialNodeParentV2,
    placement: SpatialPlacementRecipeV2,
    anchor: u32,
) -> SpatialNodeDeclarationV2 {
    SpatialNodeDeclarationV2::new(
        node(symbol, anchor + 1),
        template(template_id, anchor + 2),
        parent,
        placement,
        container(anchor + 3),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        span(anchor),
    )
}

fn container(anchor: u32) -> SpatialContainerRecipeV2 {
    SpatialContainerRecipeV2::new(
        SpatialAxisV2::Column,
        SpatialPaddingRecipeV2::new(
            i32_lit(0, anchor),
            i32_lit(0, anchor + 1),
            i32_lit(0, anchor + 2),
            i32_lit(0, anchor + 3),
        ),
        i32_lit(0, anchor + 4),
    )
}

fn layout(
    width: [SpatialFieldV2<SpatialBindingV2<i32>>; 3],
    height: [SpatialFieldV2<SpatialBindingV2<i32>>; 3],
    transform: SpatialTransformRecipeV2,
) -> SpatialPlacementRecipeV2 {
    SpatialPlacementRecipeV2::Layout(SpatialLayoutPlacementRecipeV2::new(
        dimension(width),
        dimension(height),
        transform,
    ))
}

fn dimension(fields: [SpatialFieldV2<SpatialBindingV2<i32>>; 3]) -> SpatialDimensionRecipeV2 {
    let [minimum, preferred, maximum] = fields;
    SpatialDimensionRecipeV2::new(minimum, preferred, maximum)
}

fn transform_with_factor(anchor: u32) -> SpatialTransformRecipeV2 {
    SpatialTransformRecipeV2::new(
        fixed_prop(3, anchor),
        fixed_lit(0, anchor + 1),
        fixed_lit(0, anchor + 2),
        fixed_lit(65_536, anchor + 3),
        fixed_lit(0, anchor + 4),
        fixed_lit(0, anchor + 5),
        point(fixed_lit(0, anchor + 6), fixed_lit(0, anchor + 7)),
    )
}

fn identity(anchor: u32) -> SpatialTransformRecipeV2 {
    SpatialTransformRecipeV2::new(
        fixed_lit(65_536, anchor),
        fixed_lit(0, anchor + 1),
        fixed_lit(0, anchor + 2),
        fixed_lit(65_536, anchor + 3),
        fixed_lit(0, anchor + 4),
        fixed_lit(0, anchor + 5),
        point(fixed_lit(0, anchor + 6), fixed_lit(0, anchor + 7)),
    )
}

const fn field_i32(value: i32, anchor: u32) -> SpatialFieldV2<i32> {
    SpatialFieldV2::new(value, span(anchor))
}
