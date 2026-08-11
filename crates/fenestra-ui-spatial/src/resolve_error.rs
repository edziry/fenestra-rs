//! Stored aggregate spatial-resolution diagnostics.

use std::error::Error;
use std::fmt;

use fenestra_ui_layout::prototype::{
    LayoutEngineErrorKindV1, LayoutOutputErrorKindV1, LayoutOutputFieldV1,
};

use crate::content_error::SpatialContentErrorKindV2;
use crate::error::{SpatialDependencyErrorKindV2, SpatialErrorLocationV2, SpatialInputErrorKindV2};
use crate::limits::SpatialLimitKindV2;
use crate::numeric_error::{SpatialArithmeticOperationV2, SpatialTransformErrorKindV2};
use crate::vocabulary::SpatialExtentV2;

/// Closed layout-bridge failure vocabulary for spatial resolution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialLayoutErrorKindV2 {
    /// The layout engine returned a closed failure.
    Engine(LayoutEngineErrorKindV1),
    /// Successful layout output failed boundary validation.
    Output(LayoutOutputErrorKindV1),
    /// The synthetic-root output differs from its required rectangle.
    SyntheticRootMismatch(LayoutOutputFieldV1),
    /// The spatial-to-layout bridge violated an internal invariant.
    BridgeInvariant,
}

impl SpatialLayoutErrorKindV2 {
    /// Every layout-bridge failure in deterministic vocabulary order.
    pub const ALL: [Self; 22] = layout_error_kinds();
}

/// Closed candidate-output failure vocabulary for spatial resolution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialOutputErrorKindV2 {
    /// Candidate record count differs from the required count.
    RecordCountMismatch,
    /// A candidate key differs from the required stable key.
    KeyMismatch,
    /// One candidate scalar lies outside the registered domain.
    ScalarOutOfDomain,
    /// One candidate base-box extent is negative.
    NegativeBaseExtent(SpatialExtentV2),
    /// One world transform has an invalid determinant.
    InvalidWorldDeterminant,
    /// One conservative candidate bound is invalid.
    InvalidAabb,
    /// One resolved clip chain is invalid.
    InvalidClipChain,
    /// Candidate projection order is invalid.
    InvalidProjectionOrder,
    /// One candidate reference is invalid.
    InvalidReference,
}

impl SpatialOutputErrorKindV2 {
    /// Every candidate-output failure in validation order.
    pub const ALL: [Self; 10] = [
        Self::RecordCountMismatch,
        Self::KeyMismatch,
        Self::ScalarOutOfDomain,
        Self::NegativeBaseExtent(SpatialExtentV2::Width),
        Self::NegativeBaseExtent(SpatialExtentV2::Height),
        Self::InvalidWorldDeterminant,
        Self::InvalidAabb,
        Self::InvalidClipChain,
        Self::InvalidProjectionOrder,
        Self::InvalidReference,
    ];
}

/// Closed top-level spatial-resolution failure vocabulary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialResolveErrorKindV2 {
    /// A checked caller capacity was exceeded.
    LimitExceeded(SpatialLimitKindV2),
    /// Raw spatial topology is invalid.
    Input(SpatialInputErrorKindV2),
    /// Raw spatial content is invalid.
    Content(SpatialContentErrorKindV2),
    /// The placement dependency graph is invalid.
    Dependency(SpatialDependencyErrorKindV2),
    /// A raw or composed transform is invalid.
    Transform(SpatialTransformErrorKindV2),
    /// Layout computation or bridging failed.
    Layout(SpatialLayoutErrorKindV2),
    /// Checked spatial arithmetic failed.
    Arithmetic(SpatialArithmeticOperationV2),
    /// Candidate output failed boundary validation.
    Output(SpatialOutputErrorKindV2),
}

impl SpatialResolveErrorKindV2 {
    /// Every resolver failure in top-level vocabulary order.
    pub const ALL: [Self; 192] = resolve_error_kinds();
}

/// Stored redacted diagnostic for one failed spatial resolution.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct SpatialResolveErrorV2 {
    kind: SpatialResolveErrorKindV2,
    location: SpatialErrorLocationV2,
    observed: Option<u128>,
    maximum: Option<u128>,
}

impl SpatialResolveErrorV2 {
    /// Returns the closed failure kind.
    #[must_use]
    pub const fn kind(self) -> SpatialResolveErrorKindV2 {
        self.kind
    }

    /// Returns the trusted diagnostic location.
    #[must_use]
    pub const fn location(self) -> SpatialErrorLocationV2 {
        self.location
    }

    /// Returns the observed widened value for a limit failure.
    #[must_use]
    pub const fn observed(self) -> Option<u128> {
        self.observed
    }

    /// Returns the effective widened maximum for a limit failure.
    #[must_use]
    pub const fn maximum(self) -> Option<u128> {
        self.maximum
    }
}

impl fmt::Display for SpatialResolveErrorV2 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let category = match self.kind {
            SpatialResolveErrorKindV2::LimitExceeded(_) => "limit-exceeded",
            SpatialResolveErrorKindV2::Input(_) => "input",
            SpatialResolveErrorKindV2::Content(_) => "content",
            SpatialResolveErrorKindV2::Dependency(_) => "dependency",
            SpatialResolveErrorKindV2::Transform(_) => "transform",
            SpatialResolveErrorKindV2::Layout(_) => "layout",
            SpatialResolveErrorKindV2::Arithmetic(_) => "arithmetic",
            SpatialResolveErrorKindV2::Output(_) => "output",
        };
        write!(formatter, "spatial-resolve-error({category})")
    }
}

impl fmt::Debug for SpatialResolveErrorV2 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "SpatialResolveErrorV2({self})")
    }
}

impl Error for SpatialResolveErrorV2 {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

const fn layout_error_kinds() -> [SpatialLayoutErrorKindV2; 22] {
    let mut values = [SpatialLayoutErrorKindV2::BridgeInvariant; 22];
    let mut output = 0;
    let mut input = 0;

    while input < LayoutEngineErrorKindV1::ALL.len() {
        values[output] = SpatialLayoutErrorKindV2::Engine(LayoutEngineErrorKindV1::ALL[input]);
        output += 1;
        input += 1;
    }
    input = 0;
    while input < LayoutOutputErrorKindV1::ALL.len() {
        values[output] = SpatialLayoutErrorKindV2::Output(LayoutOutputErrorKindV1::ALL[input]);
        output += 1;
        input += 1;
    }
    input = 0;
    while input < LayoutOutputFieldV1::ALL.len() {
        values[output] =
            SpatialLayoutErrorKindV2::SyntheticRootMismatch(LayoutOutputFieldV1::ALL[input]);
        output += 1;
        input += 1;
    }
    values[output] = SpatialLayoutErrorKindV2::BridgeInvariant;
    values
}

const fn resolve_error_kinds() -> [SpatialResolveErrorKindV2; 192] {
    let mut values = [SpatialResolveErrorKindV2::LimitExceeded(SpatialLimitKindV2::ALL[0]); 192];
    let mut output = 0;
    let mut input = 0;

    while input < SpatialLimitKindV2::ALL.len() {
        values[output] = SpatialResolveErrorKindV2::LimitExceeded(SpatialLimitKindV2::ALL[input]);
        output += 1;
        input += 1;
    }
    input = 0;
    while input < SpatialInputErrorKindV2::ALL.len() {
        values[output] = SpatialResolveErrorKindV2::Input(SpatialInputErrorKindV2::ALL[input]);
        output += 1;
        input += 1;
    }
    input = 0;
    while input < SpatialContentErrorKindV2::ALL.len() {
        values[output] = SpatialResolveErrorKindV2::Content(SpatialContentErrorKindV2::ALL[input]);
        output += 1;
        input += 1;
    }
    input = 0;
    while input < SpatialDependencyErrorKindV2::ALL.len() {
        values[output] =
            SpatialResolveErrorKindV2::Dependency(SpatialDependencyErrorKindV2::ALL[input]);
        output += 1;
        input += 1;
    }
    input = 0;
    while input < SpatialTransformErrorKindV2::ALL.len() {
        values[output] =
            SpatialResolveErrorKindV2::Transform(SpatialTransformErrorKindV2::ALL[input]);
        output += 1;
        input += 1;
    }
    input = 0;
    while input < SpatialLayoutErrorKindV2::ALL.len() {
        values[output] = SpatialResolveErrorKindV2::Layout(SpatialLayoutErrorKindV2::ALL[input]);
        output += 1;
        input += 1;
    }
    input = 0;
    while input < SpatialArithmeticOperationV2::ALL.len() {
        values[output] =
            SpatialResolveErrorKindV2::Arithmetic(SpatialArithmeticOperationV2::ALL[input]);
        output += 1;
        input += 1;
    }
    input = 0;
    while input < SpatialOutputErrorKindV2::ALL.len() {
        values[output] = SpatialResolveErrorKindV2::Output(SpatialOutputErrorKindV2::ALL[input]);
        output += 1;
        input += 1;
    }
    values
}
