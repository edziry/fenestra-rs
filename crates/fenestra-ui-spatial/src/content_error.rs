//! Closed aggregate failure vocabulary for raw spatial content.

use crate::content_diagnostic::{
    SpatialClipErrorV2, SpatialContentReferenceV2, SpatialGradientErrorV2, SpatialImageErrorV2,
    SpatialKeyedContentTableV2, SpatialOrderedItemTableV2, SpatialPathGrammarErrorV2,
    SpatialPayloadTableV2, SpatialShapeErrorV2, SpatialStrokeErrorV2,
};
use crate::vocabulary::SpatialAxisV2;

/// Closed aggregate failure vocabulary for raw spatial content.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialContentErrorKindV2 {
    /// A keyed table is not dense from zero.
    NonDenseKey(SpatialKeyedContentTableV2),
    /// A payload range is outside its table.
    InvalidRange(SpatialPayloadTableV2),
    /// An authored reference is absent or invalid.
    InvalidReference(SpatialContentReferenceV2),
    /// An ordered-item table is not in canonical order.
    InvalidOrder(SpatialOrderedItemTableV2),
    /// One authored scalar lies outside the registered domain.
    ScalarOutOfDomain,
    /// A path violates the closed path grammar.
    InvalidPathGrammar(SpatialPathGrammarErrorV2),
    /// A shape violates its geometry contract.
    InvalidShape(SpatialShapeErrorV2),
    /// A stroke width violates its coverage contract.
    InvalidStroke(SpatialStrokeErrorV2),
    /// A gradient violates its stop contract.
    InvalidGradient(SpatialGradientErrorV2),
    /// An image or image paint violates its contract.
    InvalidImage(SpatialImageErrorV2),
    /// A clip violates its ownership or ordering contract.
    InvalidClip(SpatialClipErrorV2),
    /// Path flattening remains non-flat at the maximum depth.
    NonFlatAtMaximumDepth,
    /// One derived local-bound coordinate lies outside the scalar domain.
    LocalBoundsOutOfDomain(SpatialAxisV2),
}

impl SpatialContentErrorKindV2 {
    /// Every content failure in deterministic vocabulary order.
    pub const ALL: [Self; 52] = content_error_kinds();
}

const fn content_error_kinds() -> [SpatialContentErrorKindV2; 52] {
    let mut values = [SpatialContentErrorKindV2::ScalarOutOfDomain; 52];
    let mut output = 0;
    let mut input = 0;

    while input < SpatialKeyedContentTableV2::ALL.len() {
        values[output] =
            SpatialContentErrorKindV2::NonDenseKey(SpatialKeyedContentTableV2::ALL[input]);
        output += 1;
        input += 1;
    }
    input = 0;
    while input < SpatialPayloadTableV2::ALL.len() {
        values[output] = SpatialContentErrorKindV2::InvalidRange(SpatialPayloadTableV2::ALL[input]);
        output += 1;
        input += 1;
    }
    input = 0;
    while input < SpatialContentReferenceV2::ALL.len() {
        values[output] =
            SpatialContentErrorKindV2::InvalidReference(SpatialContentReferenceV2::ALL[input]);
        output += 1;
        input += 1;
    }
    input = 0;
    while input < SpatialOrderedItemTableV2::ALL.len() {
        values[output] =
            SpatialContentErrorKindV2::InvalidOrder(SpatialOrderedItemTableV2::ALL[input]);
        output += 1;
        input += 1;
    }
    values[output] = SpatialContentErrorKindV2::ScalarOutOfDomain;
    output += 1;
    input = 0;
    while input < SpatialPathGrammarErrorV2::ALL.len() {
        values[output] =
            SpatialContentErrorKindV2::InvalidPathGrammar(SpatialPathGrammarErrorV2::ALL[input]);
        output += 1;
        input += 1;
    }
    input = 0;
    while input < SpatialShapeErrorV2::ALL.len() {
        values[output] = SpatialContentErrorKindV2::InvalidShape(SpatialShapeErrorV2::ALL[input]);
        output += 1;
        input += 1;
    }
    input = 0;
    while input < SpatialStrokeErrorV2::ALL.len() {
        values[output] = SpatialContentErrorKindV2::InvalidStroke(SpatialStrokeErrorV2::ALL[input]);
        output += 1;
        input += 1;
    }
    input = 0;
    while input < SpatialGradientErrorV2::ALL.len() {
        values[output] =
            SpatialContentErrorKindV2::InvalidGradient(SpatialGradientErrorV2::ALL[input]);
        output += 1;
        input += 1;
    }
    input = 0;
    while input < SpatialImageErrorV2::ALL.len() {
        values[output] = SpatialContentErrorKindV2::InvalidImage(SpatialImageErrorV2::ALL[input]);
        output += 1;
        input += 1;
    }
    input = 0;
    while input < SpatialClipErrorV2::ALL.len() {
        values[output] = SpatialContentErrorKindV2::InvalidClip(SpatialClipErrorV2::ALL[input]);
        output += 1;
        input += 1;
    }
    values[output] = SpatialContentErrorKindV2::NonFlatAtMaximumDepth;
    output += 1;
    values[output] = SpatialContentErrorKindV2::LocalBoundsOutOfDomain(SpatialAxisV2::X);
    output += 1;
    values[output] = SpatialContentErrorKindV2::LocalBoundsOutOfDomain(SpatialAxisV2::Y);

    values
}
