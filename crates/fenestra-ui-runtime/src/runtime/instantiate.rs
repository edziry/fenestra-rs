use fenestra_ui_ir::prototype::{ValidatedConstruction, ValidatedStyleProgram};

use crate::logical_tree::LogicalTree;

use super::capacity::RuntimeCapacity;
use super::change::StateEditError;
use super::error::{RuntimeInitializationError, RuntimeInitializationErrorKind};
use super::expand::ExpansionContext;
use super::fragment::FragmentStore;
use super::headless::{HeadlessRuntimeConfig, HeadlessSurface};
use super::state::{RuntimeGeneration, RuntimeState};

impl RuntimeState {
    pub(crate) fn initialize(
        construction: &ValidatedConstruction,
        capacity: RuntimeCapacity,
    ) -> Result<Self, RuntimeInitializationError> {
        Self::initialize_with(construction, capacity, None, None, None)
    }

    pub(crate) fn initialize_headless(
        construction: &ValidatedConstruction,
        capacity: RuntimeCapacity,
        config: &HeadlessRuntimeConfig,
        surface: HeadlessSurface,
    ) -> Result<Self, RuntimeInitializationError> {
        Self::initialize_with(construction, capacity, None, Some(config), Some(surface))
    }

    pub(crate) fn initialize_styled(
        construction: &ValidatedConstruction,
        capacity: RuntimeCapacity,
        style: &ValidatedStyleProgram,
    ) -> Result<Self, RuntimeInitializationError> {
        Self::initialize_with(construction, capacity, Some(style), None, None)
    }

    fn initialize_with(
        construction: &ValidatedConstruction,
        capacity: RuntimeCapacity,
        style: Option<&ValidatedStyleProgram>,
        headless: Option<&HeadlessRuntimeConfig>,
        surface: Option<HeadlessSurface>,
    ) -> Result<Self, RuntimeInitializationError> {
        let mut state = Self {
            generation: RuntimeGeneration::INITIAL,
            tree: LogicalTree::new(),
            fragments: FragmentStore::new(),
            property_slot_count: 0,
            headless: None,
            spatial: None,
        };
        let root_factory = construction.root_factory();
        let expansion = style.map_or_else(
            || ExpansionContext::new(construction, headless),
            |style| ExpansionContext::styled(construction, style),
        );
        let footprint = Self::factory_footprint(root_factory);
        state
            .preflight_initial(footprint, capacity)
            .map_err(Self::initial_error)?;
        let root_value = state
            .build_expanded_node(root_factory, expansion)
            .map_err(Self::initial_error)?;
        let root = state.tree.insert_root(root_value).map_err(|_| {
            RuntimeInitializationError::new(RuntimeInitializationErrorKind::InvariantViolation)
        })?;
        state
            .populate_expansion(root, root_factory, expansion, &mut None)
            .map_err(Self::initial_error)?;
        state.validate(construction, capacity).map_err(|()| {
            RuntimeInitializationError::new(RuntimeInitializationErrorKind::InvariantViolation)
        })?;
        if let Some(config) = headless {
            let surface = surface.ok_or_else(|| {
                RuntimeInitializationError::new(RuntimeInitializationErrorKind::InvariantViolation)
            })?;
            state.headless = Some(state.build_headless_projection(config, surface).map_err(
                |failure| {
                    RuntimeInitializationError::new(RuntimeInitializationErrorKind::Headless(
                        failure.kind(),
                    ))
                },
            )?);
        }
        Ok(state)
    }

    fn initial_error(error: StateEditError) -> RuntimeInitializationError {
        match error {
            StateEditError::Capacity(kind) => RuntimeInitializationError::new(
                RuntimeInitializationErrorKind::CapacityExceeded(kind),
            ),
            StateEditError::Headless(kind) => {
                RuntimeInitializationError::new(RuntimeInitializationErrorKind::Headless(kind))
            }
            StateEditError::Invariant => {
                RuntimeInitializationError::new(RuntimeInitializationErrorKind::InvariantViolation)
            }
        }
    }
}
