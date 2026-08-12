use std::error::Error;
use std::fmt;

use fenestra_ui_spatial::prototype::{SpatialNodeKeyV2, SpatialResolveErrorV2};

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
