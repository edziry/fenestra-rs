use crate::ids::{SpatialNodeSymbolV2, TemplateNodeId};
use crate::source::SourceSpan;

use super::{
    SpatialBrushDeclarationV2, SpatialClipDeclarationV2, SpatialContainerRecipeV2, SpatialFieldV2,
    SpatialHitRecipeV2, SpatialNodeParentV2, SpatialPaintRecipeV2, SpatialPlacementRecipeV2,
    SpatialSemanticRecipeV2, SpatialShapeDeclarationV2,
};

/// One symbolic spatial node and its node-local resources and items.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpatialNodeDeclarationV2 {
    symbol: SpatialFieldV2<SpatialNodeSymbolV2>,
    template: SpatialFieldV2<TemplateNodeId>,
    parent: SpatialNodeParentV2,
    placement: SpatialPlacementRecipeV2,
    container: SpatialContainerRecipeV2,
    shapes: Vec<SpatialShapeDeclarationV2>,
    brushes: Vec<SpatialBrushDeclarationV2>,
    clips: Vec<SpatialClipDeclarationV2>,
    paint_items: Vec<SpatialPaintRecipeV2>,
    hit_items: Vec<SpatialHitRecipeV2>,
    semantic_items: Vec<SpatialSemanticRecipeV2>,
    span: SourceSpan,
}

impl SpatialNodeDeclarationV2 {
    /// Creates a symbolic spatial node declaration.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        symbol: SpatialFieldV2<SpatialNodeSymbolV2>,
        template: SpatialFieldV2<TemplateNodeId>,
        parent: SpatialNodeParentV2,
        placement: SpatialPlacementRecipeV2,
        container: SpatialContainerRecipeV2,
        shapes: Vec<SpatialShapeDeclarationV2>,
        brushes: Vec<SpatialBrushDeclarationV2>,
        clips: Vec<SpatialClipDeclarationV2>,
        paint_items: Vec<SpatialPaintRecipeV2>,
        hit_items: Vec<SpatialHitRecipeV2>,
        semantic_items: Vec<SpatialSemanticRecipeV2>,
        span: SourceSpan,
    ) -> Self {
        Self {
            symbol,
            template,
            parent,
            placement,
            container,
            shapes,
            brushes,
            clips,
            paint_items,
            hit_items,
            semantic_items,
            span,
        }
    }

    /// Returns the program-local node symbol.
    #[must_use]
    pub const fn symbol(&self) -> SpatialFieldV2<SpatialNodeSymbolV2> {
        self.symbol
    }

    /// Returns the construction template represented by this node.
    #[must_use]
    pub const fn template(&self) -> SpatialFieldV2<TemplateNodeId> {
        self.template
    }

    /// Returns the declared spatial parent.
    #[must_use]
    pub const fn parent(&self) -> SpatialNodeParentV2 {
        self.parent
    }

    /// Returns the placement recipe.
    #[must_use]
    pub const fn placement(&self) -> SpatialPlacementRecipeV2 {
        self.placement
    }

    /// Returns the child container recipe.
    #[must_use]
    pub const fn container(&self) -> SpatialContainerRecipeV2 {
        self.container
    }

    /// Returns node-local shape declarations.
    #[must_use]
    pub fn shapes(&self) -> &[SpatialShapeDeclarationV2] {
        &self.shapes
    }

    /// Returns node-local brush declarations.
    #[must_use]
    pub fn brushes(&self) -> &[SpatialBrushDeclarationV2] {
        &self.brushes
    }

    /// Returns node-local clip declarations.
    #[must_use]
    pub fn clips(&self) -> &[SpatialClipDeclarationV2] {
        &self.clips
    }

    /// Returns paint items in authored order.
    #[must_use]
    pub fn paint_items(&self) -> &[SpatialPaintRecipeV2] {
        &self.paint_items
    }

    /// Returns hit-test items in authored order.
    #[must_use]
    pub fn hit_items(&self) -> &[SpatialHitRecipeV2] {
        &self.hit_items
    }

    /// Returns semantic items in authored order.
    #[must_use]
    pub fn semantic_items(&self) -> &[SpatialSemanticRecipeV2] {
        &self.semantic_items
    }

    /// Returns the node declaration span.
    #[must_use]
    pub const fn span(&self) -> SourceSpan {
        self.span
    }
}
