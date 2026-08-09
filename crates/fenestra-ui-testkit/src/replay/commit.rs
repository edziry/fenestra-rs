use fenestra_ui_ir::prototype::InvalidationSet;
use fenestra_ui_runtime::prototype::{CommitReceipt, CommittedRuntimeSnapshot};

use crate::error::{HarnessError, HarnessErrorKind};

#[derive(Clone, Copy)]
pub(super) struct CommitShapeV1 {
    receipt_empty: bool,
    same_state: bool,
    before_generation: u64,
    after_generation: u64,
    receipt_generation: u64,
    mutation_count: usize,
    invalidation: InvalidationSet,
}

#[derive(Clone, Copy)]
pub(super) struct RejectionShapeV1 {
    same_state: bool,
    before_generation: u64,
    after_generation: u64,
}

impl RejectionShapeV1 {
    pub(crate) const fn new(
        same_state: bool,
        before_generation: u64,
        after_generation: u64,
    ) -> Self {
        Self {
            same_state,
            before_generation,
            after_generation,
        }
    }

    pub(super) fn observe(
        before: &CommittedRuntimeSnapshot,
        after: &CommittedRuntimeSnapshot,
    ) -> Self {
        Self::new(
            before.shares_state_with(after),
            before.generation().get(),
            after.generation().get(),
        )
    }
}

impl CommitShapeV1 {
    #[allow(clippy::too_many_arguments)]
    pub(crate) const fn new(
        receipt_empty: bool,
        same_state: bool,
        before_generation: u64,
        after_generation: u64,
        receipt_generation: u64,
        mutation_count: usize,
        invalidation: InvalidationSet,
    ) -> Self {
        Self {
            receipt_empty,
            same_state,
            before_generation,
            after_generation,
            receipt_generation,
            mutation_count,
            invalidation,
        }
    }

    pub(super) fn observe(
        before: &CommittedRuntimeSnapshot,
        after: &CommittedRuntimeSnapshot,
        receipt: &CommitReceipt,
    ) -> Self {
        Self::new(
            receipt.is_empty(),
            before.shares_state_with(after),
            before.generation().get(),
            after.generation().get(),
            receipt.generation().get(),
            receipt.mutations().count(),
            receipt.invalidation(),
        )
    }

    pub(super) const fn before_generation(self) -> u64 {
        self.before_generation
    }

    pub(super) const fn after_generation(self) -> u64 {
        self.after_generation
    }

    pub(super) const fn mutation_count(self) -> usize {
        self.mutation_count
    }

    pub(super) const fn invalidation(self) -> InvalidationSet {
        self.invalidation
    }
}

pub(super) fn verify_commit_shape(shape: CommitShapeV1) -> Result<bool, HarnessError> {
    if shape.receipt_empty {
        if !shape.same_state
            || shape.after_generation != shape.before_generation
            || shape.receipt_generation != shape.after_generation
            || shape.mutation_count != 0
            || !shape.invalidation.is_empty()
        {
            return Err(trace_error());
        }
        return Ok(false);
    }

    let next_generation = shape
        .before_generation
        .checked_add(1)
        .ok_or_else(trace_error)?;
    if shape.same_state
        || shape.after_generation != next_generation
        || shape.receipt_generation != shape.after_generation
        || shape.mutation_count == 0
        || shape.invalidation.is_empty()
    {
        return Err(trace_error());
    }
    Ok(true)
}

pub(super) fn observe_after_verified_commit_v1<T>(
    shape: CommitShapeV1,
    observe: impl FnOnce() -> Result<T, HarnessError>,
) -> Result<T, HarnessError> {
    verify_commit_shape(shape)?;
    observe()
}

pub(super) fn verify_rejection_shape(shape: RejectionShapeV1) -> Result<(), HarnessError> {
    if !shape.same_state || shape.before_generation != shape.after_generation {
        return Err(trace_error());
    }
    Ok(())
}

fn trace_error() -> HarnessError {
    HarnessError::new(HarnessErrorKind::TraceMismatch)
}
