use std::collections::{HashMap, HashSet};

use fenestra_ui_ir::prototype::{
    ChildFactory, PropertyId, PropertyValue, RegionFactory, TemplateFactory, ValidatedConstruction,
};
use fenestra_ui_runtime::prototype::{CommittedRuntimeSnapshot, FragmentId, NodeId};

use super::{ITEM_BODY, KEY, LIST, SECOND_KEY, VALUE, WIDTH};

#[derive(Default)]
struct ExpectedCounts {
    nodes: HashSet<NodeId>,
    fragments: HashSet<FragmentId>,
    property_slots: usize,
}

pub struct CleanModel {
    construction: ValidatedConstruction,
    root_width: Option<PropertyValue>,
    keys: Vec<u64>,
    member_values: HashMap<u64, PropertyValue>,
}

impl CleanModel {
    pub fn from_construction(construction: &ValidatedConstruction) -> Self {
        let region = construction.region(LIST).expect("list should resolve");
        Self {
            construction: construction.clone(),
            root_width: None,
            keys: region.initial_keys().map(|key| key.value()).collect(),
            member_values: HashMap::new(),
        }
    }

    pub fn set_root_width(&mut self, value: i32) {
        self.root_width = Some(PropertyValue::ScalarI32(value));
    }

    pub fn insert(&mut self, key: u64, final_index: usize) {
        self.keys.insert(final_index, key);
    }

    pub fn move_key(&mut self, key: u64, final_index: usize) {
        let old_index = self
            .keys
            .iter()
            .position(|candidate| *candidate == key)
            .expect("model key should exist");
        let key = self.keys.remove(old_index);
        self.keys.insert(final_index, key);
    }

    pub fn update(&mut self, key: u64, value: i32) {
        assert!(self.keys.contains(&key), "model key should exist");
        self.member_values
            .insert(key, PropertyValue::ScalarI32(value));
    }

    pub fn remove(&mut self, key: u64) {
        let index = self
            .keys
            .iter()
            .position(|candidate| *candidate == key)
            .expect("model key should exist");
        self.keys.remove(index);
        self.member_values.remove(&key);
    }

    pub fn assert_matches(&self, committed: &CommittedRuntimeSnapshot) {
        let mut counts = ExpectedCounts::default();
        self.assert_reconstructed_node(
            committed,
            committed.root(),
            self.construction.root_factory(),
            None,
            &mut counts,
        );
        assert_eq!(committed.node_count(), counts.nodes.len());
        assert_eq!(committed.fragment_count(), counts.fragments.len());
        assert_eq!(committed.property_slot_count(), counts.property_slots);
    }

    pub fn initial_keys_are_present(&self) {
        assert_eq!(self.keys, vec![KEY, SECOND_KEY]);
    }

    fn assert_reconstructed_node(
        &self,
        committed: &CommittedRuntimeSnapshot,
        node: NodeId,
        template: TemplateFactory<'_>,
        list_key: Option<u64>,
        counts: &mut ExpectedCounts,
    ) {
        assert!(counts.nodes.insert(node), "node identity visited twice");
        assert_eq!(committed.template(node), Some(template.id()));
        assert_eq!(committed.component(node), Some(template.component().id()));

        for property in template.component().properties() {
            counts.property_slots = counts
                .property_slots
                .checked_add(1)
                .expect("reference property count should remain bounded");
            assert_eq!(
                committed.property(node, property.id()),
                Some(self.expected_value(template, property.id(), list_key))
            );
        }

        let children = committed
            .children(node)
            .expect("reconstructed node should remain live");
        let mut offset = 0usize;
        for child in template.children() {
            match child {
                ChildFactory::Static { template, .. } => {
                    let child_node = children[offset];
                    assert_eq!(committed.parent(child_node), Some(node));
                    self.assert_reconstructed_node(committed, child_node, template, None, counts);
                    offset += 1;
                }
                ChildFactory::Region { region, .. } => {
                    let fragment = committed
                        .fragment(node, region.id())
                        .expect("reconstructed region should remain live");
                    assert!(
                        counts.fragments.insert(fragment),
                        "fragment identity visited twice"
                    );
                    let members = committed
                        .keyed_members(fragment)
                        .expect("reconstructed fragment should remain live")
                        .collect::<Vec<_>>();
                    let keys = self.expected_keys(region);
                    assert_eq!(
                        members.iter().map(|(key, _)| *key).collect::<Vec<_>>(),
                        keys
                    );
                    for (index, (key, member)) in members.into_iter().enumerate() {
                        assert_eq!(children[offset + index], member);
                        assert_eq!(committed.parent(member), Some(node));
                        self.assert_reconstructed_node(
                            committed,
                            member,
                            region.repeat_body(),
                            (region.id() == LIST).then_some(key),
                            counts,
                        );
                    }
                    offset += keys.len();
                }
            }
        }
        assert_eq!(offset, children.len());
    }

    fn expected_keys(&self, region: RegionFactory<'_>) -> Vec<u64> {
        if region.id() == LIST {
            self.keys.clone()
        } else {
            region.initial_keys().map(|key| key.value()).collect()
        }
    }

    fn expected_value<'a>(
        &'a self,
        template: TemplateFactory<'a>,
        property: PropertyId,
        list_key: Option<u64>,
    ) -> &'a PropertyValue {
        if template.id() == self.construction.root_factory().id()
            && property == WIDTH
            && let Some(value) = &self.root_width
        {
            return value;
        }
        if template.id() == ITEM_BODY
            && property == VALUE
            && let Some(value) = list_key.and_then(|key| self.member_values.get(&key))
        {
            return value;
        }
        template
            .effective_value(property)
            .expect("reconstructed property should resolve")
    }
}
