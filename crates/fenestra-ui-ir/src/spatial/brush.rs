use crate::ids::SpatialBrushSymbolV2;
use crate::source::SourceSpan;

use super::{SpatialBindingV2, SpatialFieldV2, SpatialPointRecipeV2};

/// One source-bearing stop in a linear gradient.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpatialGradientStopV2 {
    offset: SpatialFieldV2<u16>,
    color: SpatialFieldV2<SpatialBindingV2<[u8; 4]>>,
    span: SourceSpan,
}

impl SpatialGradientStopV2 {
    /// Creates a gradient stop.
    #[must_use]
    pub const fn new(
        offset: SpatialFieldV2<u16>,
        color: SpatialFieldV2<SpatialBindingV2<[u8; 4]>>,
        span: SourceSpan,
    ) -> Self {
        Self {
            offset,
            color,
            span,
        }
    }

    /// Returns the normalized unsigned stop offset.
    #[must_use]
    pub const fn offset(self) -> SpatialFieldV2<u16> {
        self.offset
    }

    /// Returns the color recipe.
    #[must_use]
    pub const fn color(self) -> SpatialFieldV2<SpatialBindingV2<[u8; 4]>> {
        self.color
    }

    /// Returns the gradient stop span.
    #[must_use]
    pub const fn span(self) -> SourceSpan {
        self.span
    }
}

/// Content recipe for a symbolic brush.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SpatialBrushContentV2 {
    /// Uniform color brush.
    Solid {
        /// Color recipe.
        color: SpatialFieldV2<SpatialBindingV2<[u8; 4]>>,
    },
    /// Linear gradient brush.
    LinearGradient {
        /// Gradient start point.
        start: SpatialPointRecipeV2,
        /// Gradient end point.
        end: SpatialPointRecipeV2,
        /// Gradient stops in authored order.
        stops: Vec<SpatialGradientStopV2>,
    },
}

/// Node-local symbolic brush declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpatialBrushDeclarationV2 {
    symbol: SpatialFieldV2<SpatialBrushSymbolV2>,
    content: SpatialBrushContentV2,
    span: SourceSpan,
}

impl SpatialBrushDeclarationV2 {
    /// Creates a node-local brush declaration.
    #[must_use]
    pub fn new(
        symbol: SpatialFieldV2<SpatialBrushSymbolV2>,
        content: SpatialBrushContentV2,
        span: SourceSpan,
    ) -> Self {
        Self {
            symbol,
            content,
            span,
        }
    }

    /// Returns the node-local brush symbol.
    #[must_use]
    pub const fn symbol(&self) -> SpatialFieldV2<SpatialBrushSymbolV2> {
        self.symbol
    }

    /// Returns the brush content recipe.
    #[must_use]
    pub const fn content(&self) -> &SpatialBrushContentV2 {
        &self.content
    }

    /// Returns the brush declaration span.
    #[must_use]
    pub const fn span(&self) -> SourceSpan {
        self.span
    }
}
