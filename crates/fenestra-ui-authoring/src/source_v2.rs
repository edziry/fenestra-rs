use fenestra_ui_ir::prototype::{SourceId, SourceSpan};
use proc_macro2::Span;

use crate::vocabulary_v2::AnchorKindV2;

/// Borrowed external `.fen` format-2 compiler input.
#[derive(Clone, Copy)]
pub struct FenSourceV2<'a> {
    source: SourceId,
    bytes: &'a [u8],
}

impl FenSourceV2<'_> {
    /// Creates a borrowed `.fen` input in one opaque source namespace.
    #[must_use]
    pub const fn new<'a>(source: SourceId, bytes: &'a [u8]) -> FenSourceV2<'a> {
        FenSourceV2 { source, bytes }
    }
}

impl<'a> FenSourceV2<'a> {
    pub(crate) const fn source(self) -> SourceId {
        self.source
    }

    pub(crate) const fn bytes(self) -> &'a [u8] {
        self.bytes
    }
}

/// Opaque physical source location for a format-2 authoring diagnostic.
#[derive(Clone, Copy)]
#[non_exhaustive]
pub struct PhysicalOriginV2 {
    kind: PhysicalOriginKindV2,
}

#[derive(Clone, Copy)]
enum PhysicalOriginKindV2 {
    FenBytes {
        source: SourceId,
        start: u32,
        end: u32,
    },
    UiToken {
        span: Span,
    },
}

impl PhysicalOriginV2 {
    pub(crate) const fn fen_bytes(source: SourceId, start: u32, end: u32) -> Self {
        Self {
            kind: PhysicalOriginKindV2::FenBytes { source, start, end },
        }
    }

    pub(crate) const fn ui_token(span: Span) -> Self {
        Self {
            kind: PhysicalOriginKindV2::UiToken { span },
        }
    }

    /// Returns the `.fen` source namespace, when the origin is byte-based.
    #[must_use]
    pub const fn source_id(&self) -> Option<SourceId> {
        match self.kind {
            PhysicalOriginKindV2::FenBytes { source, .. } => Some(source),
            PhysicalOriginKindV2::UiToken { span } => {
                let _ = span;
                None
            }
        }
    }

    /// Returns the half-open `.fen` byte range, when the origin is byte-based.
    #[must_use]
    pub const fn fen_byte_range(&self) -> Option<(u32, u32)> {
        match self.kind {
            PhysicalOriginKindV2::FenBytes { start, end, .. } => Some((start, end)),
            PhysicalOriginKindV2::UiToken { .. } => None,
        }
    }

    pub(crate) const fn ui_span(&self) -> Option<Span> {
        match self.kind {
            PhysicalOriginKindV2::FenBytes { .. } => None,
            PhysicalOriginKindV2::UiToken { span } => Some(span),
        }
    }
}

/// Location of one format-2 authoring diagnostic.
#[derive(Clone, Copy)]
pub enum DiagnosticLocationV2 {
    /// Physical failure before a semantic anchor exists.
    Physical(PhysicalOriginV2),
    /// Failure associated with an assigned logical semantic anchor.
    Anchored {
        /// Logical format-2 IR source span.
        logical: SourceSpan,
        /// Semantic role of the logical anchor.
        anchor_kind: AnchorKindV2,
        /// Frontend-specific physical origin used for reporting.
        physical: PhysicalOriginV2,
    },
}
