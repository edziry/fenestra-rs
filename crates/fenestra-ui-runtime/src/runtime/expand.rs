use fenestra_ui_ir::prototype::{
    ChildFactory, PropertyId, PropertyValue, TemplateFactory, ValidatedConstruction,
    ValidatedStyleProgram,
};

use crate::logical_tree::NodeId;

use super::capacity::RuntimeCapacity;
use super::change::{StateEditError, StructuralTracker};
use super::error::CapacityKind;
use super::fragment::{Fragment, FragmentId, KeyedMember};
use super::headless::{HeadlessProjectionErrorKind, HeadlessRuntimeConfig};
use super::mutation::ManifestItem;
use super::state::{ChildGroup, PropertySlot, RuntimeNode, RuntimeState};

#[derive(Clone, Copy)]
pub(crate) struct ExpansionContext<'a> {
    construction: &'a ValidatedConstruction,
    style: Option<&'a ValidatedStyleProgram>,
    headless: Option<&'a HeadlessRuntimeConfig>,
}

impl<'a> ExpansionContext<'a> {
    pub(crate) fn new(
        construction: &'a ValidatedConstruction,
        headless: Option<&'a HeadlessRuntimeConfig>,
    ) -> Self {
        Self {
            construction,
            style: headless.map(HeadlessRuntimeConfig::style),
            headless,
        }
    }

    pub(crate) const fn styled(
        construction: &'a ValidatedConstruction,
        style: &'a ValidatedStyleProgram,
    ) -> Self {
        Self {
            construction,
            style: Some(style),
            headless: None,
        }
    }

    pub(crate) const fn construction(self) -> &'a ValidatedConstruction {
        self.construction
    }

    fn preflight_fixed_records(
        self,
        projected_nodes: Option<usize>,
    ) -> Result<(), HeadlessProjectionErrorKind> {
        match self.headless {
            Some(config) => config.preflight_fixed_records(projected_nodes),
            None => Ok(()),
        }
    }

    fn materialized_value(
        self,
        template: TemplateFactory<'_>,
        property: PropertyId,
    ) -> Option<PropertyValue> {
        self.style
            .and_then(|style| style.assignment(template.id(), property))
            .map(|assignment| assignment.replacement().clone())
            .or_else(|| template.effective_value(property).cloned())
    }
}

#[derive(Clone, Copy)]
pub(crate) struct FactoryFootprint {
    nodes: MeasuredCount,
    fragments: MeasuredCount,
    property_slots: MeasuredCount,
}

#[derive(Clone, Copy, Default)]
struct MeasuredCount {
    value: usize,
    overflowed: bool,
}

impl MeasuredCount {
    fn from_value(value: usize) -> Self {
        Self {
            value,
            overflowed: false,
        }
    }

    fn add(self, other: Self) -> Self {
        Self {
            value: self.value.saturating_add(other.value),
            overflowed: self.overflowed
                || other.overflowed
                || self.value.checked_add(other.value).is_none(),
        }
    }

    fn multiply(self, factor: usize) -> Self {
        if self.value == 0 || factor == 0 {
            return Self::default();
        }
        Self {
            value: self.value.saturating_mul(factor),
            overflowed: self.overflowed || self.value.checked_mul(factor).is_none(),
        }
    }

    fn exceeds(self, current: usize, limit: usize) -> bool {
        self.overflowed
            || current
                .checked_add(self.value)
                .is_none_or(|total| total > limit)
    }

    fn projected(self, current: usize) -> Option<usize> {
        (!self.overflowed)
            .then(|| current.checked_add(self.value))
            .flatten()
    }
}

impl FactoryFootprint {
    fn structural_count(self) -> MeasuredCount {
        self.nodes.add(self.fragments)
    }

    pub(crate) fn structural(self) -> Result<usize, StateEditError> {
        let measured = self.structural_count();
        (!measured.overflowed)
            .then_some(measured.value)
            .ok_or(StateEditError::Capacity(CapacityKind::StructuralChanges))
    }
}

enum PopulateTask<'a> {
    Node {
        node: NodeId,
        children: Vec<ChildFactory<'a>>,
        next: usize,
    },
    Region {
        fragment: FragmentId,
        repeat_body: TemplateFactory<'a>,
        keys: Vec<u64>,
        next: usize,
    },
}

impl RuntimeState {
    pub(crate) fn factory_footprint(root: TemplateFactory<'_>) -> FactoryFootprint {
        let mut footprint = FactoryFootprint {
            nodes: MeasuredCount::default(),
            fragments: MeasuredCount::default(),
            property_slots: MeasuredCount::default(),
        };
        let mut pending = vec![(root, MeasuredCount::from_value(1))];
        while let Some((template, multiplicity)) = pending.pop() {
            footprint.nodes = footprint.nodes.add(multiplicity);
            let property_slots = multiplicity.multiply(template.component().properties().count());
            footprint.property_slots = footprint.property_slots.add(property_slots);
            let children = template.children().collect::<Vec<_>>();
            for child in children.into_iter().rev() {
                match child {
                    ChildFactory::Static { template, .. } => {
                        pending.push((template, multiplicity));
                    }
                    ChildFactory::Region { region, .. } => {
                        footprint.fragments = footprint.fragments.add(multiplicity);
                        let key_count = region.initial_keys().count();
                        if key_count != 0 {
                            let repeated = multiplicity.multiply(key_count);
                            pending.push((region.repeat_body(), repeated));
                        }
                    }
                }
            }
        }
        footprint
    }

    pub(crate) fn preflight_initial(
        &self,
        footprint: FactoryFootprint,
        capacity: RuntimeCapacity,
    ) -> Result<(), StateEditError> {
        self.check_live_footprint(footprint, capacity)
    }

    pub(crate) fn preflight_created(
        &self,
        footprint: FactoryFootprint,
        capacity: RuntimeCapacity,
        expansion: ExpansionContext<'_>,
        structural: &mut StructuralTracker,
    ) -> Result<(), StateEditError> {
        let created = footprint.structural_count();
        if created.overflowed || created.value > structural.remaining() {
            return Err(StateEditError::Capacity(CapacityKind::StructuralChanges));
        }
        self.check_live_footprint(footprint, capacity)?;
        expansion
            .preflight_fixed_records(footprint.nodes.projected(self.tree.len()))
            .map_err(StateEditError::Headless)?;
        structural.reserve(created.value)
    }

    fn check_live_footprint(
        &self,
        footprint: FactoryFootprint,
        capacity: RuntimeCapacity,
    ) -> Result<(), StateEditError> {
        if footprint
            .nodes
            .exceeds(self.tree.len(), capacity.live_nodes())
        {
            return Err(StateEditError::Capacity(CapacityKind::LiveNodes));
        }
        if footprint
            .fragments
            .exceeds(self.fragments.len(), capacity.live_fragments())
        {
            return Err(StateEditError::Capacity(CapacityKind::LiveFragments));
        }
        if footprint
            .property_slots
            .exceeds(self.property_slot_count, capacity.live_property_slots())
        {
            return Err(StateEditError::Capacity(CapacityKind::LivePropertySlots));
        }
        Ok(())
    }

    pub(crate) fn build_expanded_node(
        &mut self,
        template: TemplateFactory<'_>,
        expansion: ExpansionContext<'_>,
    ) -> Result<RuntimeNode, StateEditError> {
        let component = template.component();
        let properties = component
            .properties()
            .map(|property| {
                let value = expansion
                    .materialized_value(template, property.id())
                    .ok_or(StateEditError::Invariant)?;
                Ok(PropertySlot {
                    id: property.id(),
                    value,
                    invalidation: property.invalidation(),
                })
            })
            .collect::<Result<Vec<_>, StateEditError>>()?;
        self.property_slot_count = self
            .property_slot_count
            .checked_add(properties.len())
            .ok_or(StateEditError::Invariant)?;
        Ok(RuntimeNode {
            template: template.id(),
            component: component.id(),
            properties,
            child_groups: Vec::new(),
        })
    }

    pub(crate) fn populate_expansion<'a>(
        &mut self,
        root: NodeId,
        root_factory: TemplateFactory<'a>,
        expansion: ExpansionContext<'_>,
        manifest: &mut Option<&mut Vec<ManifestItem>>,
    ) -> Result<(), StateEditError> {
        let mut pending = vec![PopulateTask::Node {
            node: root,
            children: root_factory.children().collect(),
            next: 0,
        }];
        while let Some(task) = pending.pop() {
            match task {
                PopulateTask::Node {
                    node,
                    children,
                    next,
                } => {
                    let Some(child) = children.get(next).copied() else {
                        continue;
                    };
                    pending.push(PopulateTask::Node {
                        node,
                        children,
                        next: next + 1,
                    });
                    match child {
                        ChildFactory::Static { template, .. } => {
                            let value = self.build_expanded_node(template, expansion)?;
                            let child_node = self
                                .tree
                                .append_child(node, value)
                                .map_err(|_| StateEditError::Invariant)?;
                            Self::record(manifest, ManifestItem::Node(child_node));
                            self.tree
                                .value_mut_for_transaction(node)
                                .ok_or(StateEditError::Invariant)?
                                .child_groups
                                .push(ChildGroup::Static(child_node));
                            pending.push(PopulateTask::Node {
                                node: child_node,
                                children: template.children().collect(),
                                next: 0,
                            });
                        }
                        ChildFactory::Region { region, .. } => {
                            let fragment = self.fragments.insert(Fragment {
                                owner: node,
                                descriptor: region.id(),
                                members: Vec::new(),
                            });
                            Self::record(manifest, ManifestItem::Fragment(fragment));
                            self.tree
                                .value_mut_for_transaction(node)
                                .ok_or(StateEditError::Invariant)?
                                .child_groups
                                .push(ChildGroup::Region(fragment));
                            pending.push(PopulateTask::Region {
                                fragment,
                                repeat_body: region.repeat_body(),
                                keys: region.initial_keys().map(|key| key.value()).collect(),
                                next: 0,
                            });
                        }
                    }
                }
                PopulateTask::Region {
                    fragment,
                    repeat_body,
                    keys,
                    next,
                } => {
                    let Some(key) = keys.get(next).copied() else {
                        continue;
                    };
                    pending.push(PopulateTask::Region {
                        fragment,
                        repeat_body,
                        keys,
                        next: next + 1,
                    });
                    let owner = self
                        .fragments
                        .get(fragment)
                        .map(|stored| stored.owner)
                        .ok_or(StateEditError::Invariant)?;
                    let value = self.build_expanded_node(repeat_body, expansion)?;
                    let member = self
                        .tree
                        .append_child(owner, value)
                        .map_err(|_| StateEditError::Invariant)?;
                    Self::record(manifest, ManifestItem::Node(member));
                    self.fragments
                        .get_mut(fragment)
                        .ok_or(StateEditError::Invariant)?
                        .members
                        .push(KeyedMember { key, root: member });
                    pending.push(PopulateTask::Node {
                        node: member,
                        children: repeat_body.children().collect(),
                        next: 0,
                    });
                }
            }
        }
        Ok(())
    }

    fn record(manifest: &mut Option<&mut Vec<ManifestItem>>, item: ManifestItem) {
        if let Some(records) = manifest.as_deref_mut() {
            records.push(item);
        }
    }
}
