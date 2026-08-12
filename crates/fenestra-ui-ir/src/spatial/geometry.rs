use crate::ids::SpatialShapeSymbolV2;
use crate::source::SourceSpan;

use super::{SpatialBindingV2, SpatialFieldV2, SpatialPointRecipeV2};

/// One authored path verb recipe.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialPathVerbRecipeV2 {
    /// Starts a new subpath at a point.
    MoveTo {
        /// Destination point.
        to: SpatialPointRecipeV2,
        /// Source span for this verb.
        span: SourceSpan,
    },
    /// Adds a straight segment to a point.
    LineTo {
        /// Destination point.
        to: SpatialPointRecipeV2,
        /// Source span for this verb.
        span: SourceSpan,
    },
    /// Adds a quadratic Bezier segment.
    QuadraticTo {
        /// Control point.
        control: SpatialPointRecipeV2,
        /// Destination point.
        to: SpatialPointRecipeV2,
        /// Source span for this verb.
        span: SourceSpan,
    },
    /// Adds a cubic Bezier segment.
    CubicTo {
        /// First control point.
        control1: SpatialPointRecipeV2,
        /// Second control point.
        control2: SpatialPointRecipeV2,
        /// Destination point.
        to: SpatialPointRecipeV2,
        /// Source span for this verb.
        span: SourceSpan,
    },
    /// Closes the current subpath.
    Close {
        /// Source span for this verb.
        span: SourceSpan,
    },
}

impl SpatialPathVerbRecipeV2 {
    /// Returns the source span for this path verb.
    #[must_use]
    pub const fn span(&self) -> SourceSpan {
        match self {
            Self::MoveTo { span, .. }
            | Self::LineTo { span, .. }
            | Self::QuadraticTo { span, .. }
            | Self::CubicTo { span, .. }
            | Self::Close { span } => *span,
        }
    }
}

/// One source-bearing point in a polygon recipe.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpatialPolygonPointV2 {
    point: SpatialPointRecipeV2,
    span: SourceSpan,
}

impl SpatialPolygonPointV2 {
    /// Creates a polygon point.
    #[must_use]
    pub const fn new(point: SpatialPointRecipeV2, span: SourceSpan) -> Self {
        Self { point, span }
    }

    /// Returns the point recipe.
    #[must_use]
    pub const fn point(self) -> SpatialPointRecipeV2 {
        self.point
    }

    /// Returns the point record span.
    #[must_use]
    pub const fn span(self) -> SourceSpan {
        self.span
    }
}

/// Geometry recipe for one symbolic shape.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SpatialShapeGeometryV2 {
    /// Axis-aligned rectangle geometry.
    Rect {
        /// Rectangle origin.
        origin: SpatialPointRecipeV2,
        /// Rectangle width.
        width: SpatialFieldV2<SpatialBindingV2<i64>>,
        /// Rectangle height.
        height: SpatialFieldV2<SpatialBindingV2<i64>>,
    },
    /// Circle geometry.
    Circle {
        /// Circle center.
        center: SpatialPointRecipeV2,
        /// Circle radius.
        radius: SpatialFieldV2<SpatialBindingV2<i64>>,
    },
    /// Closed polygon geometry.
    Polygon {
        /// Polygon points in authored order.
        points: Vec<SpatialPolygonPointV2>,
    },
    /// General path geometry.
    Path {
        /// Path verbs in authored order.
        verbs: Vec<SpatialPathVerbRecipeV2>,
    },
}

/// Node-local symbolic shape declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpatialShapeDeclarationV2 {
    symbol: SpatialFieldV2<SpatialShapeSymbolV2>,
    geometry: SpatialShapeGeometryV2,
    span: SourceSpan,
}

impl SpatialShapeDeclarationV2 {
    /// Creates a node-local shape declaration.
    #[must_use]
    pub fn new(
        symbol: SpatialFieldV2<SpatialShapeSymbolV2>,
        geometry: SpatialShapeGeometryV2,
        span: SourceSpan,
    ) -> Self {
        Self {
            symbol,
            geometry,
            span,
        }
    }

    /// Returns the node-local shape symbol.
    #[must_use]
    pub const fn symbol(&self) -> SpatialFieldV2<SpatialShapeSymbolV2> {
        self.symbol
    }

    /// Returns the shape geometry recipe.
    #[must_use]
    pub const fn geometry(&self) -> &SpatialShapeGeometryV2 {
        &self.geometry
    }

    /// Returns the shape declaration span.
    #[must_use]
    pub const fn span(&self) -> SourceSpan {
        self.span
    }
}
