#![forbid(unsafe_code)]

//! Unpublished candidate-neutral layout conformance boundary for Fenestra.

mod engine;
mod error;
mod limits;
mod model;
mod reference;
mod validation;
mod vocabulary;

/// Reserved unstable surface for layout feasibility work.
#[doc(hidden)]
pub mod prototype {
    pub use crate::engine::{
        LayoutEngineV1, ReferenceStackEngineV1, ValidatedLayoutInputV1, compute_layout_v1,
    };
    pub use crate::error::{
        LayoutEngineErrorKindV1, LayoutEngineErrorV1, LayoutErrorKindV1, LayoutErrorLocationV1,
        LayoutErrorV1, LayoutInputErrorKindV1, LayoutOutputErrorKindV1,
    };
    pub use crate::limits::{LayoutLimitKindV1, LayoutLimitsV1, REGISTERED_LAYOUT_LIMITS_V1};
    pub use crate::model::{
        LayoutDimensionV1, LayoutInputV1, LayoutNodeKeyV1, LayoutNodeV1, LayoutOutputV1,
        LayoutPaddingV1, LayoutRecordV1, LayoutRectV1, LayoutStyleV1, LayoutViewportV1,
    };
    pub use crate::vocabulary::{
        LayoutArithmeticOperationV1, LayoutAxisV1, LayoutConstraintFieldV1, LayoutExtentV1,
        LayoutOutputFieldV1, LayoutPaddingSideV1,
    };
}
