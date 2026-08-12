use std::error::Error;
use std::fmt;

use fenestra_ui_ir::prototype::SourceSpan;
use fenestra_ui_spatial::prototype::{SpatialNodeKeyV2, SpatialResolveErrorV2};

/// Closed failure vocabulary for symbolic spatial materialization.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeSpatialIrErrorKindV2 {
    /// Checked materialization arithmetic was exhausted.
    ArithmeticExhausted,
    /// Validated symbolic input violated an internal runtime invariant.
    InvariantViolation,
    /// Complete spatial resolution rejected materialized input.
    Resolve(SpatialResolveErrorV2),
}

/// Stored redacted diagnostic for one failed symbolic spatial materialization.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct RuntimeSpatialIrErrorV2 {
    kind: RuntimeSpatialIrErrorKindV2,
    span: SourceSpan,
}

impl RuntimeSpatialIrErrorV2 {
    #[allow(dead_code, reason = "reserved for symbolic spatial materialization")]
    pub(crate) const fn new(kind: RuntimeSpatialIrErrorKindV2, span: SourceSpan) -> Self {
        Self { kind, span }
    }

    /// Returns the closed failure kind.
    #[must_use]
    pub const fn kind(self) -> RuntimeSpatialIrErrorKindV2 {
        self.kind
    }

    /// Returns the authored source anchor associated with the failure.
    #[must_use]
    pub const fn span(self) -> SourceSpan {
        self.span
    }
}

impl fmt::Display for RuntimeSpatialIrErrorV2 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self.kind {
            RuntimeSpatialIrErrorKindV2::ArithmeticExhausted => "arithmetic-exhausted",
            RuntimeSpatialIrErrorKindV2::InvariantViolation => "invariant-violation",
            RuntimeSpatialIrErrorKindV2::Resolve(_) => "resolve",
        };
        write!(formatter, "runtime-spatial-ir-error({label})")
    }
}

impl fmt::Debug for RuntimeSpatialIrErrorV2 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "RuntimeSpatialIrErrorV2({self})")
    }
}

impl Error for RuntimeSpatialIrErrorV2 {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

/// Failure to validate or resolve one runtime spatial publication.
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum RuntimeSpatialErrorV2 {
    /// The program returned input for a different viewport.
    ViewportMismatch,
    /// The logical mapping does not cover every non-sentinel spatial node.
    MappingLengthMismatch,
    /// A mapped logical node is absent, foreign, or stale.
    MissingLogicalNode {
        #[doc = "Spatial key whose mapped logical node is absent."]
        key: SpatialNodeKeyV2,
    },
    /// A logical node appears at more than one spatial key.
    DuplicateLogicalNode {
        #[doc = "Second spatial key that repeats an earlier logical node."]
        key: SpatialNodeKeyV2,
    },
    /// Symbolic spatial materialization failed before publication.
    Ir(RuntimeSpatialIrErrorV2),
    /// Complete spatial resolution rejected the program input.
    Resolve(SpatialResolveErrorV2),
}

impl fmt::Display for RuntimeSpatialErrorV2 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::ViewportMismatch => "viewport-mismatch",
            Self::MappingLengthMismatch => "mapping-length-mismatch",
            Self::MissingLogicalNode { .. } => "missing-logical-node",
            Self::DuplicateLogicalNode { .. } => "duplicate-logical-node",
            Self::Ir(_) => "ir",
            Self::Resolve(_) => "resolve",
        };
        write!(formatter, "runtime-spatial-error({label})")
    }
}

impl fmt::Debug for RuntimeSpatialErrorV2 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "RuntimeSpatialErrorV2({self})")
    }
}

impl Error for RuntimeSpatialErrorV2 {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}
