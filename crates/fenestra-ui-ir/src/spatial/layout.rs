use crate::source::SourceSpan;

use super::{SpatialAxisV2, SpatialBindingV2, SpatialFieldV2, SpatialPointRecipeV2};

/// Symbolic affine transform recipe with an explicit origin.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpatialTransformRecipeV2 {
    a: SpatialFieldV2<SpatialBindingV2<i64>>,
    b: SpatialFieldV2<SpatialBindingV2<i64>>,
    c: SpatialFieldV2<SpatialBindingV2<i64>>,
    d: SpatialFieldV2<SpatialBindingV2<i64>>,
    tx: SpatialFieldV2<SpatialBindingV2<i64>>,
    ty: SpatialFieldV2<SpatialBindingV2<i64>>,
    origin: SpatialPointRecipeV2,
}

impl SpatialTransformRecipeV2 {
    /// Creates a symbolic affine transform recipe.
    #[must_use]
    pub const fn new(
        a: SpatialFieldV2<SpatialBindingV2<i64>>,
        b: SpatialFieldV2<SpatialBindingV2<i64>>,
        c: SpatialFieldV2<SpatialBindingV2<i64>>,
        d: SpatialFieldV2<SpatialBindingV2<i64>>,
        tx: SpatialFieldV2<SpatialBindingV2<i64>>,
        ty: SpatialFieldV2<SpatialBindingV2<i64>>,
        origin: SpatialPointRecipeV2,
    ) -> Self {
        Self {
            a,
            b,
            c,
            d,
            tx,
            ty,
            origin,
        }
    }

    /// Returns the horizontal scale coefficient recipe.
    #[must_use]
    pub const fn a(self) -> SpatialFieldV2<SpatialBindingV2<i64>> {
        self.a
    }

    /// Returns the vertical shear coefficient recipe.
    #[must_use]
    pub const fn b(self) -> SpatialFieldV2<SpatialBindingV2<i64>> {
        self.b
    }

    /// Returns the horizontal shear coefficient recipe.
    #[must_use]
    pub const fn c(self) -> SpatialFieldV2<SpatialBindingV2<i64>> {
        self.c
    }

    /// Returns the vertical scale coefficient recipe.
    #[must_use]
    pub const fn d(self) -> SpatialFieldV2<SpatialBindingV2<i64>> {
        self.d
    }

    /// Returns the horizontal translation recipe.
    #[must_use]
    pub const fn tx(self) -> SpatialFieldV2<SpatialBindingV2<i64>> {
        self.tx
    }

    /// Returns the vertical translation recipe.
    #[must_use]
    pub const fn ty(self) -> SpatialFieldV2<SpatialBindingV2<i64>> {
        self.ty
    }

    /// Returns the transform origin recipe.
    #[must_use]
    pub const fn origin(self) -> SpatialPointRecipeV2 {
        self.origin
    }
}

/// Literal container settings for the program viewport.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpatialViewportContainerV2 {
    axis: SpatialAxisV2,
    left: SpatialFieldV2<i32>,
    right: SpatialFieldV2<i32>,
    top: SpatialFieldV2<i32>,
    bottom: SpatialFieldV2<i32>,
    gap: SpatialFieldV2<i32>,
    span: SourceSpan,
}

impl SpatialViewportContainerV2 {
    /// Creates viewport container settings.
    #[must_use]
    pub const fn new(
        axis: SpatialAxisV2,
        left: SpatialFieldV2<i32>,
        right: SpatialFieldV2<i32>,
        top: SpatialFieldV2<i32>,
        bottom: SpatialFieldV2<i32>,
        gap: SpatialFieldV2<i32>,
        span: SourceSpan,
    ) -> Self {
        Self {
            axis,
            left,
            right,
            top,
            bottom,
            gap,
            span,
        }
    }

    /// Returns the viewport container axis.
    #[must_use]
    pub const fn axis(self) -> SpatialAxisV2 {
        self.axis
    }

    /// Returns the left viewport padding.
    #[must_use]
    pub const fn left(self) -> SpatialFieldV2<i32> {
        self.left
    }

    /// Returns the right viewport padding.
    #[must_use]
    pub const fn right(self) -> SpatialFieldV2<i32> {
        self.right
    }

    /// Returns the top viewport padding.
    #[must_use]
    pub const fn top(self) -> SpatialFieldV2<i32> {
        self.top
    }

    /// Returns the bottom viewport padding.
    #[must_use]
    pub const fn bottom(self) -> SpatialFieldV2<i32> {
        self.bottom
    }

    /// Returns the viewport child gap.
    #[must_use]
    pub const fn gap(self) -> SpatialFieldV2<i32> {
        self.gap
    }

    /// Returns the source span for the viewport container record.
    #[must_use]
    pub const fn span(self) -> SourceSpan {
        self.span
    }
}
