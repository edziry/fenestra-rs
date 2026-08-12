use fenestra_ui_ir::prototype::{
    InputPolicy, PropertyId, SourceId, SourceSpan, SpatialBindingV2, SpatialBrushSymbolV2,
    SpatialClipAddressV2, SpatialClipSymbolV2, SpatialFieldV2, SpatialImageSymbolV2,
    SpatialNodeSymbolV2, SpatialPointRecipeV2, SpatialShapeSymbolV2, TemplateNodeId,
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

pub(super) const fn rgba_lit(
    value: [u8; 4],
    anchor: u32,
) -> SpatialFieldV2<SpatialBindingV2<[u8; 4]>> {
    field(SpatialBindingV2::Literal(value), anchor)
}

pub(super) const fn rgba_prop(
    property: u32,
    anchor: u32,
) -> SpatialFieldV2<SpatialBindingV2<[u8; 4]>> {
    field(
        SpatialBindingV2::Property(PropertyId::new(property)),
        anchor,
    )
}

pub(super) const fn input_lit(
    value: InputPolicy,
    anchor: u32,
) -> SpatialFieldV2<SpatialBindingV2<InputPolicy>> {
    field(SpatialBindingV2::Literal(value), anchor)
}

pub(super) const fn input_prop(
    property: u32,
    anchor: u32,
) -> SpatialFieldV2<SpatialBindingV2<InputPolicy>> {
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

pub(super) const fn shape(value: u32, anchor: u32) -> SpatialFieldV2<SpatialShapeSymbolV2> {
    field(SpatialShapeSymbolV2::new(value), anchor)
}

pub(super) const fn brush(value: u32, anchor: u32) -> SpatialFieldV2<SpatialBrushSymbolV2> {
    field(SpatialBrushSymbolV2::new(value), anchor)
}

pub(super) const fn clip(value: u32, anchor: u32) -> SpatialFieldV2<SpatialClipSymbolV2> {
    field(SpatialClipSymbolV2::new(value), anchor)
}

pub(super) const fn image(value: u32, anchor: u32) -> SpatialFieldV2<SpatialImageSymbolV2> {
    field(SpatialImageSymbolV2::new(value), anchor)
}

pub(super) const fn address(
    owner: u32,
    owner_anchor: u32,
    clip_value: u32,
    clip_anchor: u32,
) -> SpatialClipAddressV2 {
    SpatialClipAddressV2::new(node(owner, owner_anchor), clip(clip_value, clip_anchor))
}
