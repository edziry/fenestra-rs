use fenestra_ui_ir::prototype::{SourceId, SourceSpan};
use proc_macro2::Span;

use crate::vocabulary::AnchorKindV1;

/// Borrowed external `.fen` compiler input.
#[derive(Clone, Copy)]
pub struct FenSourceV1<'a> {
    source: SourceId,
    bytes: &'a [u8],
}

impl<'a> FenSourceV1<'a> {
    /// Creates a borrowed `.fen` input in one opaque source namespace.
    #[must_use]
    pub const fn new(source: SourceId, bytes: &'a [u8]) -> Self {
        Self { source, bytes }
    }

    pub(crate) const fn source(self) -> SourceId {
        self.source
    }

    pub(crate) const fn bytes(self) -> &'a [u8] {
        self.bytes
    }
}

/// Opaque physical source location for an authoring diagnostic.
#[derive(Clone, Copy)]
#[non_exhaustive]
pub struct PhysicalOriginV1 {
    kind: PhysicalOriginKindV1,
}

#[derive(Clone, Copy)]
enum PhysicalOriginKindV1 {
    FenBytes {
        source: SourceId,
        start: u32,
        end: u32,
    },
    UiToken {
        span: Span,
    },
}

impl PhysicalOriginV1 {
    pub(crate) const fn fen_bytes(source: SourceId, start: u32, end: u32) -> Self {
        Self {
            kind: PhysicalOriginKindV1::FenBytes { source, start, end },
        }
    }

    pub(crate) const fn ui_token(span: Span) -> Self {
        Self {
            kind: PhysicalOriginKindV1::UiToken { span },
        }
    }

    /// Returns the `.fen` source namespace, when the origin is byte-based.
    #[must_use]
    pub const fn source_id(&self) -> Option<SourceId> {
        match self.kind {
            PhysicalOriginKindV1::FenBytes { source, .. } => Some(source),
            PhysicalOriginKindV1::UiToken { span } => {
                let _ = span;
                None
            }
        }
    }

    /// Returns the half-open `.fen` byte range, when the origin is byte-based.
    #[must_use]
    pub const fn fen_byte_range(&self) -> Option<(u32, u32)> {
        match self.kind {
            PhysicalOriginKindV1::FenBytes { start, end, .. } => Some((start, end)),
            PhysicalOriginKindV1::UiToken { .. } => None,
        }
    }

    pub(crate) const fn ui_span(&self) -> Option<Span> {
        match self.kind {
            PhysicalOriginKindV1::FenBytes { .. } => None,
            PhysicalOriginKindV1::UiToken { span } => Some(span),
        }
    }
}

/// Location of one authoring diagnostic.
#[derive(Clone, Copy)]
pub enum DiagnosticLocationV1 {
    /// Physical failure before a semantic anchor exists.
    Physical(PhysicalOriginV1),
    /// Failure associated with an assigned logical semantic anchor.
    Anchored {
        /// Logical version-1 IR source span.
        logical: SourceSpan,
        /// Semantic role of the logical anchor.
        anchor_kind: AnchorKindV1,
        /// Frontend-specific physical origin used for reporting.
        physical: PhysicalOriginV1,
    },
}
