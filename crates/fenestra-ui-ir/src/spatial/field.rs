use crate::ids::PropertyId;
use crate::source::SourceSpan;

/// One source-bearing value in a symbolic spatial program.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpatialFieldV2<T> {
    value: T,
    span: SourceSpan,
}

impl<T> SpatialFieldV2<T> {
    /// Creates a source-bearing spatial field.
    #[must_use]
    pub const fn new(value: T, span: SourceSpan) -> Self {
        Self { value, span }
    }

    /// Returns the authored value.
    #[must_use]
    pub const fn value(&self) -> &T {
        &self.value
    }

    /// Returns the source span of the authored value.
    #[must_use]
    pub const fn span(&self) -> SourceSpan {
        self.span
    }
}

/// Literal value or schema property reference used by a spatial recipe.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialBindingV2<T> {
    /// Uses the literal payload directly.
    Literal(T),
    /// Resolves the value from a property of the owning component.
    Property(PropertyId),
}
