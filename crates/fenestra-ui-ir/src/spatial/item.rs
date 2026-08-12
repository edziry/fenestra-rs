use crate::ids::{
    SpatialBrushSymbolV2, SpatialClipSymbolV2, SpatialImageSymbolV2, SpatialShapeSymbolV2,
};
use crate::source::SourceSpan;
use crate::value::InputPolicy;

use super::{
    SpatialBindingV2, SpatialClipAddressV2, SpatialFieldV2, SpatialFillRuleV2, SpatialPointRecipeV2,
};

/// Coverage recipe shared by paint and hit-test items.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialCoverageRecipeV2 {
    /// Fills the interior of a shape.
    Fill {
        /// Node-local shape symbol.
        shape: SpatialFieldV2<SpatialShapeSymbolV2>,
        /// Fill rule used to determine coverage.
        rule: SpatialFillRuleV2,
    },
    /// Covers a round-joined and round-capped stroke.
    RoundStroke {
        /// Node-local shape symbol.
        shape: SpatialFieldV2<SpatialShapeSymbolV2>,
        /// Stroke width recipe.
        width: SpatialFieldV2<SpatialBindingV2<i64>>,
    },
}

/// Node-local clip declaration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpatialClipDeclarationV2 {
    symbol: SpatialFieldV2<SpatialClipSymbolV2>,
    parent: Option<SpatialClipAddressV2>,
    shape: SpatialFieldV2<SpatialShapeSymbolV2>,
    fill_rule: SpatialFillRuleV2,
    span: SourceSpan,
}

impl SpatialClipDeclarationV2 {
    /// Creates a node-local clip declaration.
    #[must_use]
    pub const fn new(
        symbol: SpatialFieldV2<SpatialClipSymbolV2>,
        parent: Option<SpatialClipAddressV2>,
        shape: SpatialFieldV2<SpatialShapeSymbolV2>,
        fill_rule: SpatialFillRuleV2,
        span: SourceSpan,
    ) -> Self {
        Self {
            symbol,
            parent,
            shape,
            fill_rule,
            span,
        }
    }

    /// Returns the node-local clip symbol.
    #[must_use]
    pub const fn symbol(self) -> SpatialFieldV2<SpatialClipSymbolV2> {
        self.symbol
    }

    /// Returns the optional parent clip address.
    #[must_use]
    pub const fn parent(self) -> Option<SpatialClipAddressV2> {
        self.parent
    }

    /// Returns the node-local shape symbol used by this clip.
    #[must_use]
    pub const fn shape(self) -> SpatialFieldV2<SpatialShapeSymbolV2> {
        self.shape
    }

    /// Returns the clip fill rule.
    #[must_use]
    pub const fn fill_rule(self) -> SpatialFillRuleV2 {
        self.fill_rule
    }

    /// Returns the clip declaration span.
    #[must_use]
    pub const fn span(self) -> SourceSpan {
        self.span
    }
}

/// One paint item in authored rendering order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialPaintRecipeV2 {
    /// Paints shape coverage with a brush.
    CoveragePaint {
        /// Coverage recipe to paint.
        coverage: SpatialCoverageRecipeV2,
        /// Node-local brush symbol.
        brush: SpatialFieldV2<SpatialBrushSymbolV2>,
        /// Paint opacity.
        opacity: SpatialFieldV2<u8>,
        /// Optional clip address.
        clip: Option<SpatialClipAddressV2>,
        /// Source span for this paint item.
        span: SourceSpan,
    },
    /// Paints an image subrectangle into a symbolic destination.
    ImagePaint {
        /// Program-local image symbol.
        image: SpatialFieldV2<SpatialImageSymbolV2>,
        /// Source rectangle horizontal origin.
        source_x: SpatialFieldV2<u32>,
        /// Source rectangle vertical origin.
        source_y: SpatialFieldV2<u32>,
        /// Source rectangle width.
        source_width: SpatialFieldV2<u32>,
        /// Source rectangle height.
        source_height: SpatialFieldV2<u32>,
        /// Destination rectangle origin.
        destination_origin: SpatialPointRecipeV2,
        /// Destination rectangle width recipe.
        destination_width: SpatialFieldV2<SpatialBindingV2<i64>>,
        /// Destination rectangle height recipe.
        destination_height: SpatialFieldV2<SpatialBindingV2<i64>>,
        /// Paint opacity.
        opacity: SpatialFieldV2<u8>,
        /// Optional clip address.
        clip: Option<SpatialClipAddressV2>,
        /// Source span for this paint item.
        span: SourceSpan,
    },
}

impl SpatialPaintRecipeV2 {
    /// Returns the source span for this paint item.
    #[must_use]
    pub const fn span(&self) -> SourceSpan {
        match self {
            Self::CoveragePaint { span, .. } | Self::ImagePaint { span, .. } => *span,
        }
    }
}

/// One hit-test item in authored hit-test order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpatialHitRecipeV2 {
    coverage: SpatialCoverageRecipeV2,
    clip: Option<SpatialClipAddressV2>,
    input_policy: SpatialFieldV2<SpatialBindingV2<InputPolicy>>,
    span: SourceSpan,
}

impl SpatialHitRecipeV2 {
    /// Creates a hit-test item.
    #[must_use]
    pub const fn new(
        coverage: SpatialCoverageRecipeV2,
        clip: Option<SpatialClipAddressV2>,
        input_policy: SpatialFieldV2<SpatialBindingV2<InputPolicy>>,
        span: SourceSpan,
    ) -> Self {
        Self {
            coverage,
            clip,
            input_policy,
            span,
        }
    }

    /// Returns the hit coverage recipe.
    #[must_use]
    pub const fn coverage(self) -> SpatialCoverageRecipeV2 {
        self.coverage
    }

    /// Returns the optional clip address.
    #[must_use]
    pub const fn clip(self) -> Option<SpatialClipAddressV2> {
        self.clip
    }

    /// Returns the input policy recipe.
    #[must_use]
    pub const fn input_policy(self) -> SpatialFieldV2<SpatialBindingV2<InputPolicy>> {
        self.input_policy
    }

    /// Returns the hit-test item span.
    #[must_use]
    pub const fn span(self) -> SourceSpan {
        self.span
    }
}

/// One semantic coverage item in authored order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpatialSemanticRecipeV2 {
    shape: SpatialFieldV2<SpatialShapeSymbolV2>,
    fill_rule: SpatialFillRuleV2,
    clip: Option<SpatialClipAddressV2>,
    span: SourceSpan,
}

impl SpatialSemanticRecipeV2 {
    /// Creates a semantic coverage item.
    #[must_use]
    pub const fn new(
        shape: SpatialFieldV2<SpatialShapeSymbolV2>,
        fill_rule: SpatialFillRuleV2,
        clip: Option<SpatialClipAddressV2>,
        span: SourceSpan,
    ) -> Self {
        Self {
            shape,
            fill_rule,
            clip,
            span,
        }
    }

    /// Returns the node-local shape symbol.
    #[must_use]
    pub const fn shape(self) -> SpatialFieldV2<SpatialShapeSymbolV2> {
        self.shape
    }

    /// Returns the semantic fill rule.
    #[must_use]
    pub const fn fill_rule(self) -> SpatialFillRuleV2 {
        self.fill_rule
    }

    /// Returns the optional clip address.
    #[must_use]
    pub const fn clip(self) -> Option<SpatialClipAddressV2> {
        self.clip
    }

    /// Returns the semantic item span.
    #[must_use]
    pub const fn span(self) -> SourceSpan {
        self.span
    }
}
