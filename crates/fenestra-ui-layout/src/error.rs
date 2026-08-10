use std::error::Error;
use std::fmt;

use crate::limits::LayoutLimitKindV1;
use crate::vocabulary::{
    LayoutArithmeticOperationV1, LayoutConstraintFieldV1, LayoutExtentV1, LayoutOutputFieldV1,
    LayoutPaddingSideV1,
};

/// Closed core-input failure taxonomy for layout contract version 1.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LayoutInputErrorKindV1 {
    /// An inclusive resource bound was exceeded.
    LimitExceeded(LayoutLimitKindV1),
    /// No root node was supplied.
    EmptyInput,
    /// The first node does not use key zero.
    InvalidRootKey,
    /// The root declares a parent.
    RootHasParent,
    /// A later node key does not equal its input ordinal.
    NonDenseKey,
    /// A later node omits its parent or names a key outside the input.
    MissingParent,
    /// A later node names itself or a later input node as its parent.
    ForwardParent,
    /// A later node reopens a completed preorder subtree.
    InvalidPreorder,
    /// One viewport extent is negative.
    NegativeViewport(LayoutExtentV1),
    /// One dimension constraint field is negative.
    NegativeConstraint {
        /// Width or height constraint.
        extent: LayoutExtentV1,
        /// Minimum, preferred, or maximum field.
        field: LayoutConstraintFieldV1,
    },
    /// A dimension minimum exceeds its maximum.
    InvertedConstraint(LayoutExtentV1),
    /// One padding side is negative.
    NegativePadding(LayoutPaddingSideV1),
    /// Combined padding exceeds its resolved border-box extent.
    PaddingExceedsExtent(LayoutExtentV1),
    /// The inter-child gap is negative.
    NegativeGap,
}

impl LayoutInputErrorKindV1 {
    /// Every input failure in deterministic validation order.
    pub const ALL: [Self; 27] = [
        Self::LimitExceeded(LayoutLimitKindV1::Nodes),
        Self::EmptyInput,
        Self::InvalidRootKey,
        Self::RootHasParent,
        Self::NonDenseKey,
        Self::MissingParent,
        Self::ForwardParent,
        Self::InvalidPreorder,
        Self::LimitExceeded(LayoutLimitKindV1::Depth),
        Self::LimitExceeded(LayoutLimitKindV1::ChildrenPerNode),
        Self::NegativeViewport(LayoutExtentV1::Width),
        Self::NegativeViewport(LayoutExtentV1::Height),
        Self::NegativeConstraint {
            extent: LayoutExtentV1::Width,
            field: LayoutConstraintFieldV1::Minimum,
        },
        Self::NegativeConstraint {
            extent: LayoutExtentV1::Width,
            field: LayoutConstraintFieldV1::Preferred,
        },
        Self::NegativeConstraint {
            extent: LayoutExtentV1::Width,
            field: LayoutConstraintFieldV1::Maximum,
        },
        Self::InvertedConstraint(LayoutExtentV1::Width),
        Self::NegativeConstraint {
            extent: LayoutExtentV1::Height,
            field: LayoutConstraintFieldV1::Minimum,
        },
        Self::NegativeConstraint {
            extent: LayoutExtentV1::Height,
            field: LayoutConstraintFieldV1::Preferred,
        },
        Self::NegativeConstraint {
            extent: LayoutExtentV1::Height,
            field: LayoutConstraintFieldV1::Maximum,
        },
        Self::InvertedConstraint(LayoutExtentV1::Height),
        Self::NegativePadding(LayoutPaddingSideV1::Left),
        Self::NegativePadding(LayoutPaddingSideV1::Right),
        Self::NegativePadding(LayoutPaddingSideV1::Top),
        Self::NegativePadding(LayoutPaddingSideV1::Bottom),
        Self::PaddingExceedsExtent(LayoutExtentV1::Width),
        Self::PaddingExceedsExtent(LayoutExtentV1::Height),
        Self::NegativeGap,
    ];

    const fn label(self) -> &'static str {
        match self {
            Self::LimitExceeded(_) => "limit-exceeded",
            Self::EmptyInput => "empty-input",
            Self::InvalidRootKey => "invalid-root-key",
            Self::RootHasParent => "root-has-parent",
            Self::NonDenseKey => "non-dense-key",
            Self::MissingParent => "missing-parent",
            Self::ForwardParent => "forward-parent",
            Self::InvalidPreorder => "invalid-preorder",
            Self::NegativeViewport(_) => "negative-viewport",
            Self::NegativeConstraint { .. } => "negative-constraint",
            Self::InvertedConstraint(_) => "inverted-constraint",
            Self::NegativePadding(_) => "negative-padding",
            Self::PaddingExceedsExtent(_) => "padding-exceeds-extent",
            Self::NegativeGap => "negative-gap",
        }
    }
}

/// Closed engine-owned failure taxonomy for layout contract version 1.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LayoutEngineErrorKindV1 {
    /// Checked engine arithmetic could not produce a logical coordinate.
    ArithmeticExhausted {
        /// Operation that overflowed.
        operation: LayoutArithmeticOperationV1,
        /// Width or height axis of the operation.
        extent: LayoutExtentV1,
    },
    /// A bounded engine profile rejected otherwise core-valid input.
    RejectedInput,
    /// Candidate output could not be represented by the integer boundary.
    UnrepresentableOutput,
    /// The engine violated an internal invariant.
    InvariantViolation,
}

impl LayoutEngineErrorKindV1 {
    /// Every engine failure in deterministic vocabulary order.
    pub const ALL: [Self; 9] = [
        Self::ArithmeticExhausted {
            operation: LayoutArithmeticOperationV1::FarEdge,
            extent: LayoutExtentV1::Width,
        },
        Self::ArithmeticExhausted {
            operation: LayoutArithmeticOperationV1::FarEdge,
            extent: LayoutExtentV1::Height,
        },
        Self::ArithmeticExhausted {
            operation: LayoutArithmeticOperationV1::ContentOrigin,
            extent: LayoutExtentV1::Width,
        },
        Self::ArithmeticExhausted {
            operation: LayoutArithmeticOperationV1::ContentOrigin,
            extent: LayoutExtentV1::Height,
        },
        Self::ArithmeticExhausted {
            operation: LayoutArithmeticOperationV1::GapAdvance,
            extent: LayoutExtentV1::Width,
        },
        Self::ArithmeticExhausted {
            operation: LayoutArithmeticOperationV1::GapAdvance,
            extent: LayoutExtentV1::Height,
        },
        Self::RejectedInput,
        Self::UnrepresentableOutput,
        Self::InvariantViolation,
    ];

    const fn label(self) -> &'static str {
        match self {
            Self::ArithmeticExhausted { .. } => "arithmetic-exhausted",
            Self::RejectedInput => "rejected-input",
            Self::UnrepresentableOutput => "unrepresentable-output",
            Self::InvariantViolation => "invariant-violation",
        }
    }
}

/// Closed successful-output failure taxonomy for layout contract version 1.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LayoutOutputErrorKindV1 {
    /// Output cardinality differs from input cardinality.
    RecordCountMismatch,
    /// A record key differs from the corresponding input key.
    KeyMismatch,
    /// One output rectangle scalar is negative.
    Negative(LayoutOutputFieldV1),
    /// A checked output far edge overflowed.
    FarEdgeArithmetic(LayoutExtentV1),
}

impl LayoutOutputErrorKindV1 {
    /// Every output failure in deterministic validation order.
    pub const ALL: [Self; 8] = [
        Self::RecordCountMismatch,
        Self::KeyMismatch,
        Self::Negative(LayoutOutputFieldV1::X),
        Self::Negative(LayoutOutputFieldV1::Y),
        Self::Negative(LayoutOutputFieldV1::Width),
        Self::Negative(LayoutOutputFieldV1::Height),
        Self::FarEdgeArithmetic(LayoutExtentV1::Width),
        Self::FarEdgeArithmetic(LayoutExtentV1::Height),
    ];

    const fn label(self) -> &'static str {
        match self {
            Self::RecordCountMismatch => "record-count-mismatch",
            Self::KeyMismatch => "key-mismatch",
            Self::Negative(_) => "negative-output",
            Self::FarEdgeArithmetic(_) => "far-edge-arithmetic",
        }
    }
}

/// Closed top-level failure taxonomy for one layout computation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LayoutErrorKindV1 {
    /// Core input validation failed before engine invocation.
    Input(LayoutInputErrorKindV1),
    /// The selected engine returned a closed failure.
    Engine(LayoutEngineErrorKindV1),
    /// Successful engine output failed boundary validation.
    Output(LayoutOutputErrorKindV1),
}

impl LayoutErrorKindV1 {
    /// Every layout failure in deterministic phase and vocabulary order.
    pub const ALL: [Self; 44] = all_layout_error_kinds();
}

const fn all_layout_error_kinds() -> [LayoutErrorKindV1; 44] {
    let mut all = [LayoutErrorKindV1::Input(LayoutInputErrorKindV1::EmptyInput); 44];
    let mut output = 0;
    let mut input = 0;
    while input < LayoutInputErrorKindV1::ALL.len() {
        all[output] = LayoutErrorKindV1::Input(LayoutInputErrorKindV1::ALL[input]);
        input += 1;
        output += 1;
    }
    let mut engine = 0;
    while engine < LayoutEngineErrorKindV1::ALL.len() {
        all[output] = LayoutErrorKindV1::Engine(LayoutEngineErrorKindV1::ALL[engine]);
        engine += 1;
        output += 1;
    }
    let mut result = 0;
    while result < LayoutOutputErrorKindV1::ALL.len() {
        all[output] = LayoutErrorKindV1::Output(LayoutOutputErrorKindV1::ALL[result]);
        result += 1;
        output += 1;
    }
    all
}

/// Privacy-safe ordinal location for one layout failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LayoutErrorLocationV1 {
    /// Whole input rather than one trustworthy node.
    Input,
    /// Present logical viewport.
    Viewport,
    /// Node at one raw input ordinal.
    InputNode {
        /// Zero-based raw input ordinal.
        index: u32,
    },
    /// Whole output rather than one record.
    Output,
    /// Record at one output ordinal.
    OutputRecord {
        /// Zero-based output ordinal.
        index: u32,
    },
}

/// Engine-owned failure with no input or candidate payload.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct LayoutEngineErrorV1 {
    kind: LayoutEngineErrorKindV1,
    location: LayoutErrorLocationV1,
}

impl LayoutEngineErrorV1 {
    /// Creates one closed privacy-safe engine failure.
    #[must_use]
    pub const fn new(kind: LayoutEngineErrorKindV1, location: LayoutErrorLocationV1) -> Self {
        Self { kind, location }
    }

    /// Returns the closed failure category.
    #[must_use]
    pub const fn kind(self) -> LayoutEngineErrorKindV1 {
        self.kind
    }

    /// Returns the privacy-safe ordinal location.
    #[must_use]
    pub const fn location(self) -> LayoutErrorLocationV1 {
        self.location
    }
}

impl fmt::Display for LayoutEngineErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "layout-engine-error({})", self.kind.label())
    }
}

impl fmt::Debug for LayoutEngineErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "LayoutEngineErrorV1({self})")
    }
}

impl Error for LayoutEngineErrorV1 {}

/// Boundary failure with no input or candidate payload.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct LayoutErrorV1 {
    kind: LayoutErrorKindV1,
    location: LayoutErrorLocationV1,
}

impl LayoutErrorV1 {
    pub(crate) const fn input(
        kind: LayoutInputErrorKindV1,
        location: LayoutErrorLocationV1,
    ) -> Self {
        Self {
            kind: LayoutErrorKindV1::Input(kind),
            location,
        }
    }

    pub(crate) const fn from_engine(error: LayoutEngineErrorV1) -> Self {
        Self {
            kind: LayoutErrorKindV1::Engine(error.kind),
            location: error.location,
        }
    }

    pub(crate) const fn output(
        kind: LayoutOutputErrorKindV1,
        location: LayoutErrorLocationV1,
    ) -> Self {
        Self {
            kind: LayoutErrorKindV1::Output(kind),
            location,
        }
    }

    /// Returns the closed failure category.
    #[must_use]
    pub const fn kind(self) -> LayoutErrorKindV1 {
        self.kind
    }

    /// Returns the privacy-safe ordinal location.
    #[must_use]
    pub const fn location(self) -> LayoutErrorLocationV1 {
        self.location
    }
}

impl fmt::Display for LayoutErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self.kind {
            LayoutErrorKindV1::Input(kind) => kind.label(),
            LayoutErrorKindV1::Engine(kind) => kind.label(),
            LayoutErrorKindV1::Output(kind) => kind.label(),
        };
        write!(formatter, "layout-error({label})")
    }
}

impl fmt::Debug for LayoutErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "LayoutErrorV1({self})")
    }
}

impl Error for LayoutErrorV1 {}
