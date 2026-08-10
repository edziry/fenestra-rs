use std::panic::{RefUnwindSafe, UnwindSafe};

use crate::error::{LayoutEngineErrorV1, LayoutErrorV1};
use crate::limits::LayoutLimitsV1;
use crate::model::{LayoutInputV1, LayoutNodeV1, LayoutOutputV1, LayoutViewportV1};
use crate::reference::compute_reference_stack_v1;
use crate::validation::{validate_input_v1, validate_output_v1};

/// Opaque proof that complete version-1 core input validation accepted one
/// borrowed input.
#[derive(Clone, Copy)]
pub struct ValidatedLayoutInputV1<'a> {
    input: LayoutInputV1<'a>,
}

impl<'a> ValidatedLayoutInputV1<'a> {
    const fn new(input: LayoutInputV1<'a>) -> Self {
        Self { input }
    }

    /// Returns the validated present logical viewport.
    #[must_use]
    pub const fn viewport(self) -> LayoutViewportV1 {
        self.input.viewport()
    }

    /// Returns validated nodes in dense authored preorder.
    #[must_use]
    pub const fn nodes(self) -> &'a [LayoutNodeV1] {
        self.input.nodes()
    }
}

/// Call-local candidate-neutral layout computation boundary.
pub trait LayoutEngineV1: Send + Sync + Unpin + UnwindSafe + RefUnwindSafe + 'static {
    /// Computes one owned raw output from fully validated core input.
    fn compute(
        &self,
        input: ValidatedLayoutInputV1<'_>,
    ) -> Result<LayoutOutputV1, LayoutEngineErrorV1>;
}

/// Validates core input, invokes one engine, and validates its owned output.
pub fn compute_layout_v1<E: LayoutEngineV1 + ?Sized>(
    engine: &E,
    input: LayoutInputV1<'_>,
    limits: LayoutLimitsV1,
) -> Result<LayoutOutputV1, LayoutErrorV1> {
    validate_input_v1(input, limits)
        .map_err(|(kind, location)| LayoutErrorV1::input(kind, location))?;
    let output = engine
        .compute(ValidatedLayoutInputV1::new(input))
        .map_err(LayoutErrorV1::from_engine)?;
    validate_output_v1(input.nodes(), output)
        .map_err(|(kind, location)| LayoutErrorV1::output(kind, location))
}

/// Owned integer reference engine reserved for the version-1 stack contract.
#[derive(Clone, Copy, Debug, Default)]
pub struct ReferenceStackEngineV1;

impl ReferenceStackEngineV1 {
    /// Creates the stateless reference engine.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl LayoutEngineV1 for ReferenceStackEngineV1 {
    fn compute(
        &self,
        input: ValidatedLayoutInputV1<'_>,
    ) -> Result<LayoutOutputV1, LayoutEngineErrorV1> {
        compute_reference_stack_v1(input)
    }
}
