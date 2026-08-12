#[cfg(test)]
use fenestra_ui_ir::prototype::InvalidationSet;

use super::state::RuntimeState;

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum CommitCheckpoint {
    Draft,
    Apply,
    Validation,
    Preparation,
}

#[cfg(test)]
#[derive(Clone, Copy)]
enum DraftCorruption {
    Properties,
    Tree,
    Fragment,
}

#[derive(Clone, Copy)]
pub(super) struct CommitControl {
    panic_at: Option<CommitCheckpoint>,
    #[cfg(test)]
    corruption: Option<DraftCorruption>,
    // Validated IR rejects empty authored invalidation, so tests override only
    // the aggregate after applying a real effective mutation.
    #[cfg(test)]
    empty_invalidation: bool,
}

impl CommitControl {
    pub(super) const NONE: Self = Self {
        panic_at: None,
        #[cfg(test)]
        corruption: None,
        #[cfg(test)]
        empty_invalidation: false,
    };

    pub(super) fn panic_if(self, checkpoint: CommitCheckpoint) {
        if self.panic_at == Some(checkpoint) {
            panic!("injected commit panic");
        }
    }

    pub(super) fn before_validation(self, _state: &mut RuntimeState) {
        #[cfg(test)]
        match self.corruption {
            Some(DraftCorruption::Properties) => _state.corrupt_properties_for_test(),
            Some(DraftCorruption::Tree) => _state.corrupt_tree_for_test(),
            Some(DraftCorruption::Fragment) => _state.corrupt_fragment_for_test(),
            None => {}
        }
    }

    #[cfg(test)]
    pub(super) const fn override_invalidation(
        self,
        invalidation: InvalidationSet,
    ) -> InvalidationSet {
        if self.empty_invalidation {
            return InvalidationSet::NONE;
        }
        invalidation
    }

    #[cfg(test)]
    const fn panic_at(checkpoint: CommitCheckpoint) -> Self {
        Self {
            panic_at: Some(checkpoint),
            corruption: None,
            empty_invalidation: false,
        }
    }

    #[cfg(test)]
    const fn corrupt(corruption: DraftCorruption) -> Self {
        Self {
            panic_at: None,
            corruption: Some(corruption),
            empty_invalidation: false,
        }
    }

    #[cfg(test)]
    const fn empty_invalidation() -> Self {
        Self {
            panic_at: None,
            corruption: None,
            empty_invalidation: true,
        }
    }
}

#[cfg(test)]
#[derive(Clone, Copy)]
pub(super) enum CommitTestHook {
    PanicAfterDraft,
    PanicAfterApply,
    PanicAfterValidation,
    PanicAfterPreparation,
    CorruptPropertiesBeforeValidation,
    CorruptTreeBeforeValidation,
    CorruptFragmentBeforeValidation,
    EmptyInvalidationBeforeRebuild,
}

#[cfg(test)]
impl CommitTestHook {
    pub(super) const fn control(self) -> CommitControl {
        match self {
            Self::PanicAfterDraft => CommitControl::panic_at(CommitCheckpoint::Draft),
            Self::PanicAfterApply => CommitControl::panic_at(CommitCheckpoint::Apply),
            Self::PanicAfterValidation => CommitControl::panic_at(CommitCheckpoint::Validation),
            Self::PanicAfterPreparation => CommitControl::panic_at(CommitCheckpoint::Preparation),
            Self::CorruptPropertiesBeforeValidation => {
                CommitControl::corrupt(DraftCorruption::Properties)
            }
            Self::CorruptTreeBeforeValidation => CommitControl::corrupt(DraftCorruption::Tree),
            Self::CorruptFragmentBeforeValidation => {
                CommitControl::corrupt(DraftCorruption::Fragment)
            }
            Self::EmptyInvalidationBeforeRebuild => CommitControl::empty_invalidation(),
        }
    }
}
