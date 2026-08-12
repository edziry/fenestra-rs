use fenestra_ui_ir::prototype::{
    PropertyId, SourceId, SourceSpan, SpatialBindingV2, SpatialFieldV2, SpatialNodeSymbolV2,
    SpatialPointRecipeV2, TemplateNodeId,
};

pub(super) const fn span(anchor: u32) -> SourceSpan {
    SourceSpan::bytes(SourceId::new(0), anchor, anchor + 1)
}

pub(super) const fn field<T>(value: T, anchor: u32) -> SpatialFieldV2<T> {
    SpatialFieldV2::new(value, span(anchor))
}

pub(super) const fn i32_lit(value: i32, anchor: u32) -> SpatialFieldV2<SpatialBindingV2<i32>> {
    field(SpatialBindingV2::Literal(value), anchor)
}

pub(super) const fn i32_prop(property: u32, anchor: u32) -> SpatialFieldV2<SpatialBindingV2<i32>> {
    field(
        SpatialBindingV2::Property(PropertyId::new(property)),
        anchor,
    )
}

pub(super) const fn fixed_lit(value: i64, anchor: u32) -> SpatialFieldV2<SpatialBindingV2<i64>> {
    field(SpatialBindingV2::Literal(value), anchor)
}

pub(super) const fn fixed_prop(
    property: u32,
    anchor: u32,
) -> SpatialFieldV2<SpatialBindingV2<i64>> {
    field(
        SpatialBindingV2::Property(PropertyId::new(property)),
        anchor,
    )
}

pub(super) const fn point(
    x: SpatialFieldV2<SpatialBindingV2<i64>>,
    y: SpatialFieldV2<SpatialBindingV2<i64>>,
) -> SpatialPointRecipeV2 {
    SpatialPointRecipeV2::new(x, y)
}

pub(super) const fn node(value: u32, anchor: u32) -> SpatialFieldV2<SpatialNodeSymbolV2> {
    field(SpatialNodeSymbolV2::new(value), anchor)
}

pub(super) const fn template(value: u32, anchor: u32) -> SpatialFieldV2<TemplateNodeId> {
    field(TemplateNodeId::new(value), anchor)
}
