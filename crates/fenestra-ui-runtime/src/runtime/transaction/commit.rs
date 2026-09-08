use std::convert::Infallible;
use std::fmt;
use std::sync::Arc;

use fenestra_ui_ir::prototype::InvalidationSet;

use super::{CommitReceipt, UiRuntime, UiTransaction};
use crate::runtime::commit_control::{CommitCheckpoint, CommitControl};
use crate::runtime::error::{CapacityKind, TransactionError, TransactionErrorKind};
use crate::runtime::headless::HeadlessProjectionErrorKind;
use crate::runtime::mutation::MutationRecord;
use crate::runtime::state::RuntimeState;
use crate::runtime::view::CommittedRuntimeSnapshot;

/// Rejection before publishing a transaction and its prepared sidecar.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommitWithError<E> {
    /// The transaction or its runtime projection was rejected.
    Runtime(TransactionError),
    /// The caller rejected preparation against the candidate snapshot.
    Preparation(E),
}

impl<E> From<TransactionError> for CommitWithError<E> {
    fn from(error: TransactionError) -> Self {
        Self::Runtime(error)
    }
}

impl<E: fmt::Display> fmt::Display for CommitWithError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Runtime(error) => error.fmt(formatter),
            Self::Preparation(error) => {
                write!(formatter, "transaction preparation failed: {error}")
            }
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for CommitWithError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Runtime(error) => Some(error),
            Self::Preparation(error) => Some(error),
        }
    }
}

impl CommitWithError<Infallible> {
    pub(super) fn into_runtime(self) -> TransactionError {
        match self {
            Self::Runtime(error) => error,
            Self::Preparation(never) => match never {},
        }
    }
}

struct PreparedTransaction {
    draft: RuntimeState,
    records: Vec<MutationRecord>,
    invalidation: InvalidationSet,
}

impl UiRuntime {
    pub(super) fn commit_inner<T, E>(
        &mut self,
        transaction: UiTransaction,
        control: CommitControl,
        prepare: impl FnOnce(&CommittedRuntimeSnapshot) -> Result<T, E>,
    ) -> Result<(CommitReceipt, T), CommitWithError<E>> {
        let PreparedTransaction {
            draft,
            records,
            invalidation,
        } = self.prepare_transaction(transaction, control)?;
        let effective = !records.is_empty();
        if effective {
            self.retired.retain(|state| state.strong_count() != 0);
        }
        let candidate = self.candidate_snapshot(draft, effective, self.retired.len())?;
        let sidecar = prepare(&candidate).map_err(CommitWithError::Preparation)?;
        let generation = candidate.generation();
        let previous = if effective {
            self.retired.reserve(1);
            control.panic_if(CommitCheckpoint::Preparation);
            let previous = std::mem::replace(&mut self.state, candidate.state);
            self.retired.push(Arc::downgrade(&previous));
            Some(previous)
        } else {
            None
        };
        Ok((
            CommitReceipt {
                generation,
                records,
                invalidation,
                _retired_generation: previous,
            },
            sidecar,
        ))
    }

    pub(super) fn preview_inner(
        &self,
        transaction: UiTransaction,
    ) -> Result<CommittedRuntimeSnapshot, TransactionError> {
        let prepared = self.prepare_transaction(transaction, CommitControl::NONE)?;
        let retained = self
            .retired
            .iter()
            .filter(|state| state.strong_count() != 0)
            .count();
        self.candidate_snapshot(prepared.draft, !prepared.records.is_empty(), retained)
    }

    fn prepare_transaction(
        &self,
        transaction: UiTransaction,
        control: CommitControl,
    ) -> Result<PreparedTransaction, TransactionError> {
        if let Some(error) = transaction.poison {
            return Err(error);
        }
        if !Arc::ptr_eq(&self.state, &transaction.base) {
            return Err(TransactionError::new(TransactionErrorKind::StaleBase, None));
        }

        let mut draft = transaction.base.fork_for_transaction();
        control.panic_if(CommitCheckpoint::Draft);
        let mut applied = self.apply_operations(&mut draft, transaction.operations)?;
        control.panic_if(CommitCheckpoint::Apply);
        control.before_validation(&mut draft);
        draft
            .validate(&self.construction, self.capacity)
            .map_err(|()| TransactionError::new(TransactionErrorKind::InvariantViolation, None))?;
        control.panic_if(CommitCheckpoint::Validation);
        applied.records.retain(MutationRecord::is_effective);
        let invalidation = applied
            .records
            .iter()
            .fold(InvalidationSet::NONE, |set, record| {
                set.union(record.invalidation())
            });
        #[cfg(test)]
        let invalidation = control.override_invalidation(invalidation);
        if applied.records.is_empty() {
            return Ok(PreparedTransaction {
                draft,
                records: applied.records,
                invalidation,
            });
        }

        if let Some(headless) = &self.headless {
            let surface = applied.candidate_surface().ok_or_else(|| {
                TransactionError::new(TransactionErrorKind::InvariantViolation, None)
            })?;
            draft
                .rebuild_headless_projection(headless, surface)
                .map_err(|failure| {
                    let operation_index = match failure.kind() {
                        HeadlessProjectionErrorKind::InvalidSurface => {
                            applied.surface_operation_index()
                        }
                        _ => failure
                            .cause()
                            .and_then(|(node, property)| applied.operation_index(node, property)),
                    };
                    TransactionError::new(
                        TransactionErrorKind::Headless(failure.kind()),
                        operation_index,
                    )
                })?;
        }

        if let Some(spatial) = &self.spatial {
            let viewport = applied.candidate_spatial_viewport().ok_or_else(|| {
                TransactionError::new(TransactionErrorKind::InvariantViolation, None)
            })?;
            draft.spatial = Some(spatial.build(&draft, viewport).map_err(|error| {
                TransactionError::new(TransactionErrorKind::Spatial(error), None)
            })?);
        }

        Ok(PreparedTransaction {
            draft,
            records: applied.records,
            invalidation,
        })
    }

    fn candidate_snapshot(
        &self,
        mut draft: RuntimeState,
        effective: bool,
        retained: usize,
    ) -> Result<CommittedRuntimeSnapshot, TransactionError> {
        if !effective {
            return Ok(self.committed());
        }
        let retained = retained.checked_add(1).ok_or_else(|| {
            TransactionError::new(
                TransactionErrorKind::CapacityExceeded(CapacityKind::RetainedGenerations),
                None,
            )
        })?;
        if retained > self.capacity.retained_generations() {
            return Err(TransactionError::new(
                TransactionErrorKind::CapacityExceeded(CapacityKind::RetainedGenerations),
                None,
            ));
        }
        draft.generation = self.state.generation.next().ok_or_else(|| {
            TransactionError::new(TransactionErrorKind::GenerationExhausted, None)
        })?;
        Ok(CommittedRuntimeSnapshot {
            state: Arc::new(draft),
        })
    }
}
