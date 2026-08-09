use std::error::Error;
use std::fmt;

/// Bounded encoder failures in deterministic diagnostic order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HeadlessArtifactEncodeErrorKindV1 {
    /// The complete artifact crossed its byte ceiling.
    ArtifactBytes,
    /// One record crossed its byte ceiling, excluding its line feed.
    LineBytes,
    /// The complete artifact crossed its line-count ceiling.
    Lines,
}

/// Privacy-safe failure to encode one headless V1 artifact.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct HeadlessArtifactEncodeErrorV1 {
    kind: HeadlessArtifactEncodeErrorKindV1,
}

impl HeadlessArtifactEncodeErrorV1 {
    pub(in crate::headless::artifact) const fn new(
        kind: HeadlessArtifactEncodeErrorKindV1,
    ) -> Self {
        Self { kind }
    }

    /// Returns the crossed output bound.
    #[must_use]
    pub const fn kind(self) -> HeadlessArtifactEncodeErrorKindV1 {
        self.kind
    }
}

impl fmt::Debug for HeadlessArtifactEncodeErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HeadlessArtifactEncodeErrorV1")
            .field("kind", &self.kind)
            .finish()
    }
}

impl fmt::Display for HeadlessArtifactEncodeErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "headless artifact encode failed: {:?}",
            self.kind
        )
    }
}

impl Error for HeadlessArtifactEncodeErrorV1 {}
