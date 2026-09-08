use fenestra_ui_ir::prototype::{
    PropertyId, SpatialAxisV2, SpatialBindingV2, SpatialBrushContentV2, SpatialBrushDeclarationV2,
    SpatialBrushSymbolV2, SpatialContainerRecipeV2, SpatialCoverageRecipeV2,
    SpatialDimensionRecipeV2, SpatialFieldV2, SpatialFillRuleV2, SpatialHitRecipeV2,
    SpatialLayoutPlacementRecipeV2, SpatialNodeDeclarationV2, SpatialNodeParentV2,
    SpatialNodeSymbolV2, SpatialPaddingRecipeV2, SpatialPaintRecipeV2, SpatialPlacementRecipeV2,
    SpatialPointRecipeV2, SpatialShapeDeclarationV2, SpatialShapeGeometryV2, SpatialShapeSymbolV2,
    SpatialTransformRecipeV2, SpatialViewportContainerV2, TemplateNodeId,
};

use super::{BACKGROUND, FlatElement, GAP, HEIGHT, INPUT, PADDING, SPAN, WIDTH};
use crate::model::ElementKind;

pub(super) fn node(id: u32, node: &FlatElement<'_>) -> SpatialNodeDeclarationV2 {
    let shape = SpatialShapeSymbolV2::new(0);
    let brush = SpatialBrushSymbolV2::new(0);
    let coverage = SpatialCoverageRecipeV2::Fill {
        shape: field(shape),
        rule: SpatialFillRuleV2::NonZero,
    };
    let parent = node.parent.map_or(SpatialNodeParentV2::Viewport, |parent| {
        SpatialNodeParentV2::Node(field(SpatialNodeSymbolV2::new(parent as u32)))
    });
    let axis = match node.element.kind {
        ElementKind::Row | ElementKind::Checkbox => SpatialAxisV2::Row,
        ElementKind::Column | ElementKind::Rect | ElementKind::Text | ElementKind::Button => {
            SpatialAxisV2::Column
        }
    };
    SpatialNodeDeclarationV2::new(
        field(SpatialNodeSymbolV2::new(id)),
        field(TemplateNodeId::new(id)),
        parent,
        SpatialPlacementRecipeV2::Layout(SpatialLayoutPlacementRecipeV2::new(
            dimension(WIDTH),
            dimension(HEIGHT),
            identity(),
        )),
        SpatialContainerRecipeV2::new(
            axis,
            SpatialPaddingRecipeV2::new(
                property(PADDING),
                property(PADDING),
                property(PADDING),
                property(PADDING),
            ),
            property(GAP),
        ),
        vec![SpatialShapeDeclarationV2::new(
            field(shape),
            SpatialShapeGeometryV2::Rect {
                origin: zero_point(),
                width: property(WIDTH),
                height: property(HEIGHT),
            },
            SPAN,
        )],
        vec![SpatialBrushDeclarationV2::new(
            field(brush),
            SpatialBrushContentV2::Solid {
                color: property(BACKGROUND),
            },
            SPAN,
        )],
        Vec::new(),
        vec![SpatialPaintRecipeV2::CoveragePaint {
            coverage,
            brush: field(brush),
            opacity: field(255),
            clip: None,
            span: SPAN,
        }],
        vec![SpatialHitRecipeV2::new(
            coverage,
            None,
            property(INPUT),
            SPAN,
        )],
        Vec::new(),
        SPAN,
    )
}

pub(super) fn viewport() -> SpatialViewportContainerV2 {
    SpatialViewportContainerV2::new(
        SpatialAxisV2::Column,
        field(0),
        field(0),
        field(0),
        field(0),
        field(0),
        SPAN,
    )
}

fn dimension(id: PropertyId) -> SpatialDimensionRecipeV2 {
    SpatialDimensionRecipeV2::new(property(id), property(id), property(id))
}

fn identity() -> SpatialTransformRecipeV2 {
    SpatialTransformRecipeV2::new(
        literal(65_536),
        literal(0),
        literal(0),
        literal(65_536),
        literal(0),
        literal(0),
        zero_point(),
    )
}

fn zero_point() -> SpatialPointRecipeV2 {
    SpatialPointRecipeV2::new(literal(0), literal(0))
}

fn property<T>(id: PropertyId) -> SpatialFieldV2<SpatialBindingV2<T>> {
    field(SpatialBindingV2::Property(id))
}

fn literal<T>(value: T) -> SpatialFieldV2<SpatialBindingV2<T>> {
    field(SpatialBindingV2::Literal(value))
}

fn field<T>(value: T) -> SpatialFieldV2<T> {
    SpatialFieldV2::new(value, SPAN)
}
