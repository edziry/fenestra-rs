use std::error::Error;
use std::fmt;

use crate::compiled::CompiledAuthoringV1;
use crate::resolved::ResolvedDocumentV1;

mod encode;
mod logical;
mod record;
mod value;
mod writer;

/// Inclusive resource category bounded by semantic artifact generation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SemanticArtifactLimitKindV1 {
    /// Bytes in the complete artifact, including every line feed.
    ArtifactBytes,
    /// Bytes before the line feed in one artifact line.
    LineBytes,
    /// Semantic records, excluding the version header.
    Records,
}

impl SemanticArtifactLimitKindV1 {
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

/// Complete inclusive limits for one semantic artifact.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SemanticArtifactLimitsV1 {
    values: [usize; 3],
}

/// Exact bounded profile for the WU-0010 semantic evidence artifact.
///
/// This experiment profile is not an unbounded default or a product budget.
pub const REFERENCE_SEMANTIC_ARTIFACT_LIMITS_V1: SemanticArtifactLimitsV1 =
    SemanticArtifactLimitsV1::new(8_192, 512, 64);

impl SemanticArtifactLimitsV1 {
    /// Creates a complete explicit semantic artifact limit set.
    #[must_use]
    pub const fn new(artifact_bytes: usize, line_bytes: usize, records: usize) -> Self {
        Self {
            values: [artifact_bytes, line_bytes, records],
        }
    }

    /// Returns the inclusive bound for one semantic artifact resource.
    #[must_use]
    pub const fn limit(self, kind: SemanticArtifactLimitKindV1) -> usize {
        self.values[kind as usize]
    }
}

/// Closed failure categories for semantic artifact generation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SemanticArtifactErrorKindV1 {
    /// One explicit semantic artifact resource bound was exceeded.
    LimitExceeded(SemanticArtifactLimitKindV1),
    /// The retained private model violated a compiler invariant.
    InvalidCompiledDocument,
}

impl SemanticArtifactErrorKindV1 {
    /// Every semantic artifact failure in deterministic vocabulary order.
    pub const ALL: [Self; 4] = [
        Self::LimitExceeded(SemanticArtifactLimitKindV1::Records),
        Self::LimitExceeded(SemanticArtifactLimitKindV1::LineBytes),
        Self::LimitExceeded(SemanticArtifactLimitKindV1::ArtifactBytes),
        Self::InvalidCompiledDocument,
    ];
}

/// Privacy-safe failure produced while observing a compiled document.
pub struct SemanticArtifactErrorV1 {
    kind: SemanticArtifactErrorKindV1,
}

impl SemanticArtifactErrorV1 {
    const fn new(kind: SemanticArtifactErrorKindV1) -> Self {
        Self { kind }
    }

    /// Returns the closed semantic artifact failure category.
    #[must_use]
    pub const fn kind(&self) -> SemanticArtifactErrorKindV1 {
        self.kind
    }
}

impl fmt::Display for SemanticArtifactErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            SemanticArtifactErrorKindV1::LimitExceeded(limit) => {
                write!(formatter, "limit-exceeded({})", limit.as_str())
            }
            SemanticArtifactErrorKindV1::InvalidCompiledDocument => {
                formatter.write_str("invalid-compiled-document")
            }
        }
    }
}

impl fmt::Debug for SemanticArtifactErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "SemanticArtifactErrorV1({self})")
    }
}

impl Error for SemanticArtifactErrorV1 {}

/// Opaque canonical semantic observation of one compiled document.
pub struct SemanticArtifactV1 {
    source: Box<str>,
}

impl SemanticArtifactV1 {
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

impl fmt::Debug for SemanticArtifactV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SemanticArtifactV1")
            .field("bytes", &self.source.len())
            .finish()
    }
}

/// Produces a deterministic semantic observation from the retained model.
///
/// # Errors
///
/// Returns a typed error when the artifact exceeds `limits` or the retained
/// compiler model violates its private invariants.
pub fn canonical_semantics_v1(
    compiled: &CompiledAuthoringV1,
    limits: SemanticArtifactLimitsV1,
) -> Result<SemanticArtifactV1, SemanticArtifactErrorV1> {
    observe_resolved_v1(compiled.resolved(), limits)
}

fn observe_resolved_v1(
    resolved: &ResolvedDocumentV1,
    limits: SemanticArtifactLimitsV1,
) -> Result<SemanticArtifactV1, SemanticArtifactErrorV1> {
    Ok(SemanticArtifactV1 {
        source: encode::encode_resolved_v1(resolved, limits)?,
    })
}

#[cfg(test)]
mod tests;
