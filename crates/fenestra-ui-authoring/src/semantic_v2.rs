mod catalog;
mod content;
mod count;
mod encode;
mod field;
mod spatial;
mod types;
mod value;

pub use types::{
    SemanticArtifactErrorKindV2, SemanticArtifactErrorV2, SemanticArtifactLimitKindV2,
    SemanticArtifactLimitsV2, SemanticArtifactV2,
};

/// Exact bounded profile measured from the format-2 reference fixture.
///
/// This experiment profile is not an unbounded default or a product budget.
pub const REFERENCE_SEMANTIC_ARTIFACT_LIMITS_V2: SemanticArtifactLimitsV2 =
    SemanticArtifactLimitsV2::new(30_743, 157, 380);

use crate::compiled_v2::CompiledAuthoringV2;

/// Produces a deterministic semantic observation from a format-2 compilation.
///
/// # Errors
///
/// Returns a typed error when the artifact exceeds `limits` or the retained
/// compiler model violates its private invariants.
pub fn canonical_semantics_v2(
    compiled: &CompiledAuthoringV2,
    limits: SemanticArtifactLimitsV2,
) -> Result<SemanticArtifactV2, SemanticArtifactErrorV2> {
    Ok(SemanticArtifactV2::new(encode::encode(compiled, limits)?))
}
