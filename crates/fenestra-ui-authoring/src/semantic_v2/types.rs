use std::error::Error;
use std::fmt;

/// Inclusive resource category bounded by format-2 semantic observation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SemanticArtifactLimitKindV2 {
    /// Bytes in the complete artifact, including every line feed.
    ArtifactBytes,
    /// Bytes before the line feed in one artifact line.
    LineBytes,
    /// Semantic records, excluding the version header.
    Records,
}

impl SemanticArtifactLimitKindV2 {
    /// Every semantic artifact bound in deterministic enforcement order.
    pub const ALL: [Self; 3] = [Self::Records, Self::LineBytes, Self::ArtifactBytes];

    const fn as_str(self) -> &'static str {
        match self {
            Self::ArtifactBytes => "artifact-bytes",
            Self::LineBytes => "line-bytes",
            Self::Records => "records",
        }
    }
}

/// Complete inclusive limits for one format-2 semantic artifact.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SemanticArtifactLimitsV2 {
    values: [usize; 3],
}

impl SemanticArtifactLimitsV2 {
    /// Creates a complete explicit semantic artifact limit set.
    #[must_use]
    pub const fn new(artifact_bytes: usize, line_bytes: usize, records: usize) -> Self {
        Self {
            values: [artifact_bytes, line_bytes, records],
        }
    }

    /// Returns the inclusive bound for one semantic artifact resource.
    #[must_use]
    pub const fn limit(self, kind: SemanticArtifactLimitKindV2) -> usize {
        self.values[kind as usize]
    }
}

/// Closed failure categories for format-2 semantic artifact generation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SemanticArtifactErrorKindV2 {
    /// One explicit semantic artifact resource bound was exceeded.
    LimitExceeded(SemanticArtifactLimitKindV2),
    /// The retained private model violated a compiler invariant.
    InvalidCompiledDocument,
}

impl SemanticArtifactErrorKindV2 {
    /// Every semantic artifact failure in deterministic vocabulary order.
    pub const ALL: [Self; 4] = [
        Self::LimitExceeded(SemanticArtifactLimitKindV2::Records),
        Self::LimitExceeded(SemanticArtifactLimitKindV2::LineBytes),
        Self::LimitExceeded(SemanticArtifactLimitKindV2::ArtifactBytes),
        Self::InvalidCompiledDocument,
    ];
}

/// Privacy-safe failure produced while observing a format-2 document.
pub struct SemanticArtifactErrorV2 {
    kind: SemanticArtifactErrorKindV2,
}

impl SemanticArtifactErrorV2 {
    pub(crate) const fn new(kind: SemanticArtifactErrorKindV2) -> Self {
        Self { kind }
    }

    /// Returns the closed semantic artifact failure category.
    #[must_use]
    pub const fn kind(&self) -> SemanticArtifactErrorKindV2 {
        self.kind
    }
}

impl fmt::Display for SemanticArtifactErrorV2 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            SemanticArtifactErrorKindV2::LimitExceeded(limit) => {
                write!(formatter, "limit-exceeded({})", limit.as_str())
            }
            SemanticArtifactErrorKindV2::InvalidCompiledDocument => {
                formatter.write_str("invalid-compiled-document")
            }
        }
    }
}

impl fmt::Debug for SemanticArtifactErrorV2 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "SemanticArtifactErrorV2({self})")
    }
}

impl Error for SemanticArtifactErrorV2 {}

/// Opaque canonical semantic observation of one format-2 document.
pub struct SemanticArtifactV2 {
    source: Box<str>,
}

impl SemanticArtifactV2 {
    pub(crate) const fn new(source: Box<str>) -> Self {
        Self { source }
    }

    /// Returns the canonical ASCII artifact with exactly one final line feed.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.source
    }

    /// Returns the canonical artifact bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        self.source.as_bytes()
    }
}

impl fmt::Debug for SemanticArtifactV2 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SemanticArtifactV2")
            .field("bytes", &self.source.len())
            .finish()
    }
}
