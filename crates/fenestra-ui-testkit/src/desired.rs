use std::collections::BTreeMap;

use fenestra_ui_ir::prototype::{PropertyId, PropertyValue, ValidatedConstruction};

use crate::case::SemanticOperationV1;
use crate::error::{HarnessError, HarnessErrorKind, HarnessLimitKind};
use crate::fixture::HarnessLimitsV1;
use crate::model::clean_rebuild_v1;
use crate::semantic::{FragmentPathV1, NodePathV1, PathSegmentV1};

/// Semantic logical state used as the independent input to clean reconstruction.
#[derive(Clone, Debug)]
pub struct DesiredStateV1 {
    pub(crate) construction: ValidatedConstruction,
    pub(crate) property_overrides: BTreeMap<(NodePathV1, PropertyId), PropertyValue>,
    pub(crate) region_keys: BTreeMap<FragmentPathV1, Vec<u64>>,
    live_incarnations: BTreeMap<NodePathV1, u64>,
    next_incarnation: u64,
}

impl DesiredStateV1 {
    /// Creates the initial desired state for one exact validated construction.
    pub fn from_construction(
        construction: &ValidatedConstruction,
        limits: HarnessLimitsV1,
    ) -> Result<Self, HarnessError> {
        let mut desired = Self {
            construction: construction.clone(),
            property_overrides: BTreeMap::new(),
            region_keys: BTreeMap::new(),
            live_incarnations: BTreeMap::new(),
            next_incarnation: 0,
        };
        let initial = clean_rebuild_v1(construction, &desired, limits)?;
        desired.refresh_incarnations(&initial)?;
        Ok(desired)
    }

    pub(crate) fn incarnation_token(&self, path: &NodePathV1) -> Option<Vec<u64>> {
        let mut prefix = NodePathV1::root();
        let mut token = Vec::new();
        for segment in path.segments() {
            match segment {
                PathSegmentV1::Static { authored_slot } => {
                    prefix = prefix.static_child(*authored_slot);
                }
                PathSegmentV1::Member { region_slot, key } => {
                    prefix = prefix.member(*region_slot, *key);
                    token.push(*self.live_incarnations.get(&prefix)?);
                }
            }
        }
        Some(token)
    }

    pub(crate) fn preserves_incarnation(&self, draft: &Self, path: &NodePathV1) -> bool {
        self.incarnation_token(path)
            .is_some_and(|token| draft.incarnation_token(path).as_ref() == Some(&token))
    }

    pub(crate) fn apply_operation(
        &mut self,
        operation: &SemanticOperationV1,
        limits: HarnessLimitsV1,
    ) -> Result<(), HarnessError> {
        let construction = self.construction.clone();
        let current = clean_rebuild_v1(&construction, self, limits)?;
        match operation {
            SemanticOperationV1::SetProperty {
                node,
                property,
                value,
            } => {
                let slot = current
                    .node(node)
                    .and_then(|found| {
                        found
                            .properties()
                            .iter()
                            .find(|found| found.property() == *property)
                    })
                    .ok_or_else(invalid_operation)?;
                if slot.value().value_type() != value.value_type() {
                    return Err(invalid_operation());
                }
                self.property_overrides
                    .insert((node.clone(), *property), value.clone());
            }
            SemanticOperationV1::InsertKeyed {
                fragment,
                key,
                final_index,
            } => {
                let found = current.fragment(fragment).ok_or_else(invalid_operation)?;
                if found.members().iter().any(|member| member.key() == *key) {
                    return Err(invalid_operation());
                }
                let index = usize::try_from(*final_index).map_err(|_| arithmetic_error())?;
                if index > found.members().len() {
                    return Err(invalid_operation());
                }
                let memberships = current
                    .fragments()
                    .iter()
                    .try_fold(0_usize, |count, item| {
                        count.checked_add(item.members().len())
                    })
                    .ok_or_else(arithmetic_error)?;
                if memberships >= limits.live_memberships() {
                    return Err(HarnessError::limit(HarnessLimitKind::LiveMemberships));
                }
                let mut keys: Vec<_> = found.members().iter().map(|member| member.key()).collect();
                keys.insert(index, *key);
                self.region_keys.insert(fragment.clone(), keys);
            }
            SemanticOperationV1::MoveKeyed {
                fragment,
                key,
                final_index,
            } => {
                let found = current.fragment(fragment).ok_or_else(invalid_operation)?;
                let mut keys: Vec<_> = found.members().iter().map(|member| member.key()).collect();
                let old_index = keys
                    .iter()
                    .position(|candidate| candidate == key)
                    .ok_or_else(invalid_operation)?;
                let index = usize::try_from(*final_index).map_err(|_| arithmetic_error())?;
                if index >= keys.len() {
                    return Err(invalid_operation());
                }
                let moved = keys.remove(old_index);
                keys.insert(index, moved);
                self.region_keys.insert(fragment.clone(), keys);
            }
            SemanticOperationV1::UpdateKeyed {
                fragment,
                key,
                property,
                value,
            } => {
                let member = current
                    .fragment(fragment)
                    .and_then(|found| found.members().iter().find(|member| member.key() == *key))
                    .ok_or_else(invalid_operation)?;
                let slot = current
                    .node(member.node())
                    .and_then(|node| {
                        node.properties()
                            .iter()
                            .find(|slot| slot.property() == *property)
                    })
                    .ok_or_else(invalid_operation)?;
                if slot.value().value_type() != value.value_type() {
                    return Err(invalid_operation());
                }
                self.property_overrides
                    .insert((member.node().clone(), *property), value.clone());
            }
            SemanticOperationV1::RemoveKeyed { fragment, key } => {
                let found = current.fragment(fragment).ok_or_else(invalid_operation)?;
                let member = found
                    .members()
                    .iter()
                    .find(|member| member.key() == *key)
                    .ok_or_else(invalid_operation)?;
                let retired = member.node().clone();
                let keys = found
                    .members()
                    .iter()
                    .filter_map(|member| (member.key() != *key).then_some(member.key()))
                    .collect();
                self.property_overrides
                    .retain(|(node, _), _| !is_within(node, &retired));
                self.region_keys
                    .retain(|path, _| !is_within(path.owner(), &retired));
                self.region_keys.insert(fragment.clone(), keys);
            }
        }
        let construction = self.construction.clone();
        let updated = clean_rebuild_v1(&construction, self, limits)?;
        self.refresh_incarnations(&updated)
    }

    fn refresh_incarnations(
        &mut self,
        state: &crate::semantic::NormalizedStateV1,
    ) -> Result<(), HarnessError> {
        let live: Vec<_> = state
            .fragments()
            .iter()
            .flat_map(|fragment| fragment.members().iter())
            .map(|member| member.node().clone())
            .collect();
        self.live_incarnations
            .retain(|path, _| live.iter().any(|candidate| candidate == path));
        for path in live {
            if self.live_incarnations.contains_key(&path) {
                continue;
            }
            let incarnation = self.next_incarnation;
            self.next_incarnation = self
                .next_incarnation
                .checked_add(1)
                .ok_or_else(arithmetic_error)?;
            self.live_incarnations.insert(path, incarnation);
        }
        Ok(())
    }
}

fn is_within(candidate: &NodePathV1, ancestor: &NodePathV1) -> bool {
    candidate.segments().starts_with(ancestor.segments())
}

fn invalid_operation() -> HarnessError {
    HarnessError::new(HarnessErrorKind::InvalidOperation)
}

fn arithmetic_error() -> HarnessError {
    HarnessError::new(HarnessErrorKind::ArithmeticExhausted)
}
