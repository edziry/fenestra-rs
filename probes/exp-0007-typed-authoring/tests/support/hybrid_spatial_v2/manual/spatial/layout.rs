use fenestra_ui_ir::prototype::{
    SpatialAxisV2, SpatialBindingV2, SpatialContainerRecipeV2, SpatialDimensionRecipeV2,
    SpatialFieldV2, SpatialLayoutPlacementRecipeV2, SpatialPaddingRecipeV2, SpatialPointRecipeV2,
    SpatialTransformRecipeV2,
};

use super::super::value::{fixed_lit, i32_lit};

pub(super) fn container(
    axis: SpatialAxisV2,
    edges: [SpatialFieldV2<SpatialBindingV2<i32>>; 4],
    gap: SpatialFieldV2<SpatialBindingV2<i32>>,
) -> SpatialContainerRecipeV2 {
    let [left, right, top, bottom] = edges;
    SpatialContainerRecipeV2::new(
        axis,
        SpatialPaddingRecipeV2::new(left, right, top, bottom),
        gap,
    )
}

pub(super) fn layout_placement(
    width: [SpatialFieldV2<SpatialBindingV2<i32>>; 3],
    height: [SpatialFieldV2<SpatialBindingV2<i32>>; 3],
    transform: SpatialTransformRecipeV2,
) -> SpatialLayoutPlacementRecipeV2 {
    SpatialLayoutPlacementRecipeV2::new(dimension(width), dimension(height), transform)
}

pub(super) fn transform(
    matrix: [SpatialFieldV2<SpatialBindingV2<i64>>; 6],
    origin: SpatialPointRecipeV2,
) -> SpatialTransformRecipeV2 {
    let [a, b, c, d, tx, ty] = matrix;
    SpatialTransformRecipeV2::new(a, b, c, d, tx, ty, origin)
}

pub(super) fn identity(field_start: u32) -> SpatialTransformRecipeV2 {
    transform(
        [
            fixed_lit(65_536, field_start),
            fixed_lit(0, field_start + 1),
            fixed_lit(0, field_start + 2),
            fixed_lit(65_536, field_start + 3),
            fixed_lit(0, field_start + 4),
            fixed_lit(0, field_start + 5),
        ],
        SpatialPointRecipeV2::new(fixed_lit(0, field_start + 6), fixed_lit(0, field_start + 7)),
    )
}

pub(super) fn zero_edges(start: u32) -> [SpatialFieldV2<SpatialBindingV2<i32>>; 4] {
    [
        i32_lit(0, start),
        i32_lit(0, start + 1),
        i32_lit(0, start + 2),
        i32_lit(0, start + 3),
    ]
}

fn dimension(fields: [SpatialFieldV2<SpatialBindingV2<i32>>; 3]) -> SpatialDimensionRecipeV2 {
    let [minimum, preferred, maximum] = fields;
    SpatialDimensionRecipeV2::new(minimum, preferred, maximum)
}
