use fenestra_ui_ir::prototype::ValidatedConstruction;

use crate::logical_tree::LogicalTree;

use super::capacity::RuntimeCapacity;
use super::change::StateEditError;
use super::error::{RuntimeInitializationError, RuntimeInitializationErrorKind};
use super::fragment::FragmentStore;
use super::state::{RuntimeGeneration, RuntimeState};

impl RuntimeState {
    pub(crate) fn initialize(
        construction: &ValidatedConstruction,
        capacity: RuntimeCapacity,
    ) -> Result<Self, RuntimeInitializationError> {
        let mut state = Self {
            generation: RuntimeGeneration::INITIAL,
            tree: LogicalTree::new(),
            fragments: FragmentStore::new(),
            property_slot_count: 0,
        };
        let root_factory = construction.root_factory();
        let footprint = Self::factory_footprint(root_factory);
        state
            .preflight_initial(footprint, capacity)
            .map_err(Self::initial_error)?;
        let root_value = state
            .build_expanded_node(root_factory)
            .map_err(Self::initial_error)?;
        let root = state.tree.insert_root(root_value).map_err(|_| {
            RuntimeInitializationError::new(RuntimeInitializationErrorKind::InvariantViolation)
        })?;
        state
            .populate_expansion(root, root_factory, &mut None)
            .map_err(Self::initial_error)?;
        state.validate(construction, capacity).map_err(|()| {
            RuntimeInitializationError::new(RuntimeInitializationErrorKind::InvariantViolation)
        })?;
        Ok(state)
    }

    fn initial_error(error: StateEditError) -> RuntimeInitializationError {
        match error {
            StateEditError::Capacity(kind) => RuntimeInitializationError::new(
                RuntimeInitializationErrorKind::CapacityExceeded(kind),
            ),
            StateEditError::Invariant => {
                RuntimeInitializationError::new(RuntimeInitializationErrorKind::InvariantViolation)
            }
        }
    }
}
