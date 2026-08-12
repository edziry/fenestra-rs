use std::collections::HashMap;

use fenestra_ui_ir::prototype::{PropertyId, PropertyValue};
use fenestra_ui_spatial::prototype::SpatialViewportV2;

use crate::logical_tree::NodeId;

use super::change::{StateEditError, StructuralTracker};
use super::error::{TransactionError, TransactionErrorKind};
use super::expand::ExpansionContext;
use super::fragment::FragmentId;
use super::headless::HeadlessSurface;
use super::mutation::{
    KeyInsert, KeyMove, KeyRemove, ManifestItem, MutationRecord, PropertyChange,
};
use super::state::RuntimeState;
use super::transaction::{Operation, UiRuntime};

mod resize;

struct DraftApplication<'a> {
    runtime: &'a UiRuntime,
    state: &'a mut RuntimeState,
    records: Vec<MutationRecord>,
    property_records: HashMap<(NodeId, PropertyId), usize>,
    property_sources: HashMap<(NodeId, PropertyId), usize>,
    created_sources: HashMap<NodeId, usize>,
    candidate_surface: Option<HeadlessSurface>,
    surface_record: Option<usize>,
    surface_source: Option<usize>,
    candidate_spatial_viewport: Option<SpatialViewportV2>,
    spatial_viewport_record: Option<usize>,
    structural: StructuralTracker,
}

pub(super) struct AppliedDraft {
    pub(super) records: Vec<MutationRecord>,
    property_sources: HashMap<(NodeId, PropertyId), usize>,
    created_sources: HashMap<NodeId, usize>,
    candidate_surface: Option<HeadlessSurface>,
    surface_source: Option<usize>,
    candidate_spatial_viewport: Option<SpatialViewportV2>,
}

impl AppliedDraft {
    pub(super) fn operation_index(&self, node: NodeId, property: PropertyId) -> Option<usize> {
        let retained_property = self.records.iter().any(|record| {
            matches!(
                record,
                MutationRecord::PropertyChanged(change)
                    if change.node == node && change.property == property
            )
        });
        retained_property
            .then(|| self.property_sources.get(&(node, property)).copied())
            .flatten()
            .or_else(|| self.created_sources.get(&node).copied())
    }

    pub(super) const fn candidate_surface(&self) -> Option<HeadlessSurface> {
        self.candidate_surface
    }

    pub(super) const fn surface_operation_index(&self) -> Option<usize> {
        self.surface_source
    }

    pub(super) const fn candidate_spatial_viewport(&self) -> Option<SpatialViewportV2> {
        self.candidate_spatial_viewport
    }
}

impl UiRuntime {
    pub(super) fn apply_operations(
        &self,
        state: &mut RuntimeState,
        operations: Vec<Operation>,
    ) -> Result<AppliedDraft, TransactionError> {
        let candidate_surface = state.headless_surface();
        let candidate_spatial_viewport = state.spatial_viewport();
        let mut application = DraftApplication {
            runtime: self,
            state,
            records: Vec::new(),
            property_records: HashMap::new(),
            property_sources: HashMap::new(),
            created_sources: HashMap::new(),
            candidate_surface,
            surface_record: None,
            surface_source: None,
            candidate_spatial_viewport,
            spatial_viewport_record: None,
            structural: StructuralTracker::new(self.capacity.structural_changes()),
        };
        for (index, operation) in operations.into_iter().enumerate() {
            application.apply(operation, index)?;
        }
        Ok(AppliedDraft {
            records: application.records,
            property_sources: application.property_sources,
            created_sources: application.created_sources,
            candidate_surface: application.candidate_surface,
            surface_source: application.surface_source,
            candidate_spatial_viewport: application.candidate_spatial_viewport,
        })
    }
}

impl DraftApplication<'_> {
    fn apply(&mut self, operation: Operation, index: usize) -> Result<(), TransactionError> {
        match operation {
            Operation::SetProperty {
                node,
                property,
                value,
            } => self.property(node, property, value, index),
            Operation::InsertKeyed {
                fragment,
                key,
                final_index,
            } => self.insert(fragment, key, final_index, index),
            Operation::MoveKeyed {
                fragment,
                key,
                final_index,
            } => self.move_key(fragment, key, final_index, index),
            Operation::UpdateKeyed {
                fragment,
                key,
                property,
                value,
            } => self.keyed_property(fragment, key, property, value, index),
            Operation::RemoveKeyed { fragment, key } => self.remove(fragment, key, index),
            Operation::ResizeHeadless { surface } => self.resize(surface, index),
            Operation::ResizeSpatial { viewport } => self.resize_spatial(viewport, index),
        }
    }

    fn property(
        &mut self,
        node: NodeId,
        property: PropertyId,
        value: PropertyValue,
        operation_index: usize,
    ) -> Result<(), TransactionError> {
        let runtime_node = self.state.tree.value(node).ok_or_else(|| {
            TransactionError::new(TransactionErrorKind::MissingNode, Some(operation_index))
        })?;
        let slot_index = runtime_node
            .properties
            .iter()
            .position(|slot| slot.id == property)
            .ok_or_else(|| {
                TransactionError::new(TransactionErrorKind::UnknownProperty, Some(operation_index))
            })?;
        let slot = &runtime_node.properties[slot_index];
        if slot.value.value_type() != value.value_type() {
            return Err(TransactionError::new(
                TransactionErrorKind::PropertyTypeMismatch,
                Some(operation_index),
            ));
        }
        if slot.value == value {
            return Ok(());
        }
        let old_value = slot.value.clone();
        let invalidation = slot.invalidation;
        self.state
            .tree
            .value_mut_for_transaction(node)
            .ok_or_else(|| TransactionError::new(TransactionErrorKind::InvariantViolation, None))?
            .properties[slot_index]
            .value = value.clone();

        if let Some(record_index) = self.property_records.get(&(node, property)).copied() {
            let MutationRecord::PropertyChanged(change) = &mut self.records[record_index] else {
                return Err(TransactionError::new(
                    TransactionErrorKind::InvariantViolation,
                    None,
                ));
            };
            change.new_value = value;
        } else {
            self.property_records
                .insert((node, property), self.records.len());
            self.records
                .push(MutationRecord::PropertyChanged(PropertyChange {
                    node,
                    property,
                    old_value,
                    new_value: value,
                    invalidation,
                }));
        }
        self.property_sources
            .insert((node, property), operation_index);
        Ok(())
    }

    fn insert(
        &mut self,
        fragment: FragmentId,
        key: u64,
        final_index: usize,
        operation_index: usize,
    ) -> Result<(), TransactionError> {
        let stored = self.state.fragments.get(fragment).ok_or_else(|| {
            TransactionError::new(TransactionErrorKind::MissingFragment, Some(operation_index))
        })?;
        if stored.members.iter().any(|member| member.key == key) {
            return Err(TransactionError::new(
                TransactionErrorKind::DuplicateKey,
                Some(operation_index),
            ));
        }
        if final_index > stored.members.len() {
            return Err(TransactionError::new(
                TransactionErrorKind::IndexOutOfBounds,
                Some(operation_index),
            ));
        }
        let invalidation = self.region_invalidation(stored.descriptor)?;
        let expansion = match &self.runtime.spatial {
            Some(spatial) => ExpansionContext::styled(&self.runtime.construction, spatial.style()),
            None => {
                ExpansionContext::new(&self.runtime.construction, self.runtime.headless.as_ref())
            }
        };
        let (root, created) = self
            .state
            .insert_member(
                expansion,
                self.runtime.capacity,
                &mut self.structural,
                fragment,
                key,
                final_index,
            )
            .map_err(|error| Self::edit_error(error, operation_index))?;
        for item in &created {
            if let ManifestItem::Node(node) = item {
                self.created_sources.insert(*node, operation_index);
            }
        }
        self.records.push(MutationRecord::KeyInserted(KeyInsert {
            fragment,
            key,
            root,
            final_index,
            created,
            invalidation,
        }));
        Ok(())
    }

    fn move_key(
        &mut self,
        fragment: FragmentId,
        key: u64,
        final_index: usize,
        operation_index: usize,
    ) -> Result<(), TransactionError> {
        let stored = self.state.fragments.get(fragment).ok_or_else(|| {
            TransactionError::new(TransactionErrorKind::MissingFragment, Some(operation_index))
        })?;
        let old_index = stored
            .members
            .iter()
            .position(|member| member.key == key)
            .ok_or_else(|| {
                TransactionError::new(TransactionErrorKind::MissingKey, Some(operation_index))
            })?;
        if final_index >= stored.members.len() {
            return Err(TransactionError::new(
                TransactionErrorKind::IndexOutOfBounds,
                Some(operation_index),
            ));
        }
        if old_index == final_index {
            return Ok(());
        }
        let invalidation = self.region_invalidation(stored.descriptor)?;
        let root = self
            .state
            .move_member(fragment, old_index, final_index)
            .map_err(|error| Self::edit_error(error, operation_index))?;
        self.records.push(MutationRecord::KeyMoved(KeyMove {
            fragment,
            key,
            root,
            old_index,
            final_index,
            invalidation,
        }));
        Ok(())
    }

    fn keyed_property(
        &mut self,
        fragment: FragmentId,
        key: u64,
        property: PropertyId,
        value: PropertyValue,
        operation_index: usize,
    ) -> Result<(), TransactionError> {
        let root = self
            .state
            .fragments
            .get(fragment)
            .ok_or_else(|| {
                TransactionError::new(TransactionErrorKind::MissingFragment, Some(operation_index))
            })?
            .members
            .iter()
            .find_map(|member| (member.key == key).then_some(member.root))
            .ok_or_else(|| {
                TransactionError::new(TransactionErrorKind::MissingKey, Some(operation_index))
            })?;
        self.property(root, property, value, operation_index)
    }

    fn remove(
        &mut self,
        fragment: FragmentId,
        key: u64,
        operation_index: usize,
    ) -> Result<(), TransactionError> {
        let stored = self.state.fragments.get(fragment).ok_or_else(|| {
            TransactionError::new(TransactionErrorKind::MissingFragment, Some(operation_index))
        })?;
        let old_index = stored
            .members
            .iter()
            .position(|member| member.key == key)
            .ok_or_else(|| {
                TransactionError::new(TransactionErrorKind::MissingKey, Some(operation_index))
            })?;
        let invalidation = self.region_invalidation(stored.descriptor)?;
        let (root, retired) = self
            .state
            .remove_member(&mut self.structural, fragment, old_index)
            .map_err(|error| Self::edit_error(error, operation_index))?;
        self.records.push(MutationRecord::KeyRemoved(KeyRemove {
            fragment,
            key,
            root,
            old_index,
            retired,
            invalidation,
        }));
        Ok(())
    }

    fn region_invalidation(
        &self,
        descriptor: fenestra_ui_ir::prototype::StructuralRegionId,
    ) -> Result<fenestra_ui_ir::prototype::InvalidationSet, TransactionError> {
        self.runtime
            .construction
            .region(descriptor)
            .map(|region| region.invalidation())
            .ok_or_else(|| TransactionError::new(TransactionErrorKind::InvariantViolation, None))
    }

    fn edit_error(error: StateEditError, operation_index: usize) -> TransactionError {
        match error {
            StateEditError::Capacity(kind) => TransactionError::new(
                TransactionErrorKind::CapacityExceeded(kind),
                Some(operation_index),
            ),
            StateEditError::Headless(kind) => {
                TransactionError::new(TransactionErrorKind::Headless(kind), Some(operation_index))
            }
            StateEditError::Invariant => {
                TransactionError::new(TransactionErrorKind::InvariantViolation, None)
            }
        }
    }
}
