use crate::ids::SourceId;

/// Opaque source location attached to one authored record.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SourceSpan {
    /// A source location synthesized without byte coordinates.
    Synthetic,
    /// A half-open byte range in an opaque source namespace.
    Bytes {
        /// Opaque source namespace.
        source: SourceId,
        /// Inclusive start byte offset.
        start: u32,
        /// Exclusive end byte offset.
        end: u32,
    },
}

impl SourceSpan {
    /// Creates a synthetic source anchor.
    #[must_use]
    pub const fn synthetic() -> Self {
        Self::Synthetic
    }

    /// Creates a half-open byte range without resolving source contents.
    #[must_use]
    pub const fn bytes(source: SourceId, start: u32, end: u32) -> Self {
        Self::Bytes { source, start, end }
    }

    pub(crate) const fn is_valid(self) -> bool {
        match self {
            Self::Synthetic => true,
            Self::Bytes { start, end, .. } => start <= end,
        }
    }
}
