use std::collections::BTreeSet;

use fenestra_ui_ir::prototype::{
    ChildFactory, StructuralRegionId, TemplateNodeId, ValidatedConstruction,
};

use crate::desired::DesiredStateV1;
use crate::error::{HarnessError, HarnessErrorKind, HarnessLimitKind};
use crate::fixture::HarnessLimitsV1;
use crate::semantic::{
    FragmentPathV1, NodePathV1, NormalizedChildGroupV1, NormalizedFragmentV1, NormalizedMemberV1,
    NormalizedNodeV1, NormalizedPropertyV1, NormalizedStateV1,
};

enum RebuildWork {
    Node {
        path: NodePathV1,
        parent: Option<NodePathV1>,
        template: TemplateNodeId,
    },
    Fragment {
        path: FragmentPathV1,
        descriptor: StructuralRegionId,
    },
}

/// Reconstructs the complete normalized state without calling runtime behavior.
pub fn clean_rebuild_v1(
    construction: &ValidatedConstruction,
    desired: &DesiredStateV1,
    limits: HarnessLimitsV1,
) -> Result<NormalizedStateV1, HarnessError> {
    if !desired.construction.shares_domain_with(construction) {
        return Err(HarnessError::new(HarnessErrorKind::InvalidSemanticPath));
    }

    let mut work = vec![RebuildWork::Node {
        path: NodePathV1::root(),
        parent: None,
        template: construction.root_factory().id(),
    }];
    let mut nodes = Vec::new();
    let mut fragments = Vec::new();
    let mut visited_nodes = BTreeSet::new();
    let mut visited_fragments = BTreeSet::new();
    let mut property_slots = 0_usize;
    let mut memberships = 0_usize;
    let mut used_property_overrides = 0_usize;
    let mut used_region_overrides = 0_usize;

    while let Some(next) = work.pop() {
        match next {
            RebuildWork::Node {
                path,
                parent,
                template,
            } => {
                ensure_path_depth(&path, limits)?;
                if !visited_nodes.insert(path.clone()) {
                    return Err(HarnessError::new(HarnessErrorKind::StateMismatch));
                }
                ensure_next_count(
                    nodes.len(),
                    limits.normalized_nodes(),
                    HarnessLimitKind::NormalizedNodes,
                )?;

                let factory = construction
                    .template(template)
                    .ok_or_else(|| HarnessError::new(HarnessErrorKind::StateMismatch))?;
                let component = factory.component();
                let mut properties = Vec::new();
                for property in component.properties() {
                    property_slots = checked_increment(
                        property_slots,
                        limits.normalized_properties(),
                        HarnessLimitKind::NormalizedProperties,
                    )?;
                    let value = if let Some(value) = desired
                        .property_overrides
                        .get(&(path.clone(), property.id()))
                    {
                        used_property_overrides = used_property_overrides
                            .checked_add(1)
                            .ok_or_else(arithmetic_error)?;
                        value
                    } else {
                        factory
                            .effective_value(property.id())
                            .ok_or_else(|| HarnessError::new(HarnessErrorKind::StateMismatch))?
                    };
                    properties.push(NormalizedPropertyV1::new(property.id(), value.clone()));
                }

                let mut child_groups = Vec::new();
                let mut child_work = Vec::new();
                for (slot, child) in factory.children().enumerate() {
                    let slot = u16::try_from(slot).map_err(|_| arithmetic_error())?;
                    match child {
                        ChildFactory::Static { template, .. } => {
                            let child_path = path.clone().static_child(slot);
                            ensure_path_depth(&child_path, limits)?;
                            child_groups.push(NormalizedChildGroupV1::Static(child_path.clone()));
                            child_work.push(RebuildWork::Node {
                                path: child_path,
                                parent: Some(path.clone()),
                                template: template.id(),
                            });
                        }
                        ChildFactory::Region { region, .. } => {
                            let fragment_path = FragmentPathV1::new(path.clone(), slot);
                            child_groups
                                .push(NormalizedChildGroupV1::Region(fragment_path.clone()));
                            child_work.push(RebuildWork::Fragment {
                                path: fragment_path,
                                descriptor: region.id(),
                            });
                        }
                    }
                }
                work.extend(child_work.into_iter().rev());
                nodes.push(NormalizedNodeV1::new(
                    path,
                    parent,
                    template,
                    component.id(),
                    properties,
                    child_groups,
                ));
            }
            RebuildWork::Fragment { path, descriptor } => {
                ensure_path_depth(path.owner(), limits)?;
                if !visited_fragments.insert(path.clone()) {
                    return Err(HarnessError::new(HarnessErrorKind::StateMismatch));
                }
                ensure_next_count(
                    fragments.len(),
                    limits.normalized_fragments(),
                    HarnessLimitKind::NormalizedFragments,
                )?;

                let region = construction
                    .region(descriptor)
                    .ok_or_else(|| HarnessError::new(HarnessErrorKind::StateMismatch))?;
                let initial_keys: Vec<_> = region.initial_keys().map(|key| key.value()).collect();
                let keys = desired
                    .region_keys
                    .get(&path)
                    .map_or(initial_keys.as_slice(), Vec::as_slice);
                if desired.region_keys.contains_key(&path) {
                    used_region_overrides = used_region_overrides
                        .checked_add(1)
                        .ok_or_else(arithmetic_error)?;
                }
                memberships = memberships
                    .checked_add(keys.len())
                    .ok_or_else(arithmetic_error)?;
                if memberships > limits.live_memberships() {
                    return Err(HarnessError::limit(HarnessLimitKind::LiveMemberships));
                }

                let mut unique_keys = BTreeSet::new();
                let mut members = Vec::with_capacity(keys.len());
                let mut member_work = Vec::with_capacity(keys.len());
                for &key in keys {
                    if !unique_keys.insert(key) {
                        return Err(HarnessError::new(HarnessErrorKind::InvalidOperation));
                    }
                    let member_path = path.owner().clone().member(path.region_slot(), key);
                    ensure_path_depth(&member_path, limits)?;
                    members.push(NormalizedMemberV1::new(key, member_path.clone()));
                    member_work.push(RebuildWork::Node {
                        path: member_path,
                        parent: Some(path.owner().clone()),
                        template: region.repeat_body().id(),
                    });
                }
                work.extend(member_work.into_iter().rev());
                fragments.push(NormalizedFragmentV1::new(path, descriptor, members));
            }
        }
    }

    if used_property_overrides != desired.property_overrides.len()
        || used_region_overrides != desired.region_keys.len()
    {
        return Err(HarnessError::new(HarnessErrorKind::InvalidSemanticPath));
    }

    Ok(NormalizedStateV1::new(nodes, fragments))
}

fn ensure_path_depth(path: &NodePathV1, limits: HarnessLimitsV1) -> Result<(), HarnessError> {
    if path.depth() > limits.path_depth() {
        Err(HarnessError::limit(HarnessLimitKind::PathDepth))
    } else {
        Ok(())
    }
}

fn ensure_next_count(
    current: usize,
    limit: usize,
    kind: HarnessLimitKind,
) -> Result<(), HarnessError> {
    checked_increment(current, limit, kind).map(|_| ())
}

fn checked_increment(
    current: usize,
    limit: usize,
    kind: HarnessLimitKind,
) -> Result<usize, HarnessError> {
    let next = current.checked_add(1).ok_or_else(arithmetic_error)?;
    if next > limit {
        Err(HarnessError::limit(kind))
    } else {
        Ok(next)
    }
}

fn arithmetic_error() -> HarnessError {
    HarnessError::new(HarnessErrorKind::ArithmeticExhausted)
}
