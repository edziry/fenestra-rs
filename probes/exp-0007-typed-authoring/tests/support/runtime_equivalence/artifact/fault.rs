use fenestra_ui_ir::prototype::{
    InputPolicy, InvalidationClass, InvalidationSet, PropertyId, PropertyValue,
};
use fenestra_ui_runtime::prototype::HeadlessSurface;
use fenestra_ui_testkit::prototype::{
    FragmentPathV1, NodePathV1, NormalizedStateFaultV1, inject_headless_projection_fault_v1,
    inject_headless_surface_fault_v1, inject_normalized_state_fault_v1,
};

use super::super::{NormalizedManifestEntry, NormalizedMutation, NormalizedReceipt};
use super::{LaneLog, RuntimeArtifactEncodeErrorV1, RuntimeArtifactFaultV1, invalid_log};

pub(super) fn inject(
    log: &LaneLog,
    fault: RuntimeArtifactFaultV1,
) -> Result<LaneLog, RuntimeArtifactEncodeErrorV1> {
    let mut receipts = log.receipts().to_vec();
    let mut states = log.states().to_vec();
    let mut projections = log.projections().to_vec();
    match fault {
        RuntimeArtifactFaultV1::ReceiptGeneration => {
            let receipt = receipt(&receipts, 1)?;
            receipts[1] = NormalizedReceipt::new(
                perturb_u64(receipt.generation()),
                receipt.mutations().to_vec(),
                receipt.invalidation(),
            );
        }
        RuntimeArtifactFaultV1::ReceiptInvalidation => {
            let receipt = receipt(&receipts, 1)?;
            receipts[1] = NormalizedReceipt::new(
                receipt.generation(),
                receipt.mutations().to_vec(),
                InvalidationSet::from_class(InvalidationClass::Layout),
            );
        }
        RuntimeArtifactFaultV1::MutationKind => mutate_receipt(&mut receipts, 1, |mutation| {
            *mutation = NormalizedMutation::HeadlessSurfaceChanged {
                old_surface: HeadlessSurface::new(120, 90),
                new_surface: HeadlessSurface::new(121, 90),
            };
            Ok(())
        })?,
        RuntimeArtifactFaultV1::MutationPath => mutate_receipt(&mut receipts, 1, |mutation| {
            let NormalizedMutation::PropertyChanged { node, .. } = mutation else {
                return Err(invalid_log());
            };
            *node = perturb_node_path(node);
            Ok(())
        })?,
        RuntimeArtifactFaultV1::MutationProperty => mutate_receipt(&mut receipts, 1, |mutation| {
            let NormalizedMutation::PropertyChanged { property, .. } = mutation else {
                return Err(invalid_log());
            };
            *property = PropertyId::new(perturb_u32(property.get()));
            Ok(())
        })?,
        RuntimeArtifactFaultV1::MutationOldValue => mutate_receipt(&mut receipts, 1, |mutation| {
            let NormalizedMutation::PropertyChanged { old_value, .. } = mutation else {
                return Err(invalid_log());
            };
            *old_value = perturb_value(old_value);
            Ok(())
        })?,
        RuntimeArtifactFaultV1::MutationNewValue => mutate_receipt(&mut receipts, 1, |mutation| {
            let NormalizedMutation::PropertyChanged { new_value, .. } = mutation else {
                return Err(invalid_log());
            };
            *new_value = perturb_value(new_value);
            Ok(())
        })?,
        RuntimeArtifactFaultV1::MutationKey => mutate_receipt(&mut receipts, 2, |mutation| {
            let NormalizedMutation::KeyInserted { key, .. } = mutation else {
                return Err(invalid_log());
            };
            *key = perturb_u64(*key);
            Ok(())
        })?,
        RuntimeArtifactFaultV1::MutationRoot => mutate_receipt(&mut receipts, 2, |mutation| {
            let NormalizedMutation::KeyInserted { root, .. } = mutation else {
                return Err(invalid_log());
            };
            *root = perturb_node_path(root);
            Ok(())
        })?,
        RuntimeArtifactFaultV1::MutationIndices => mutate_receipt(&mut receipts, 3, |mutation| {
            let NormalizedMutation::KeyMoved {
                old_index,
                final_index,
                ..
            } = mutation
            else {
                return Err(invalid_log());
            };
            if old_index == final_index {
                *final_index = final_index.checked_add(1).ok_or_else(invalid_log)?;
            } else {
                std::mem::swap(old_index, final_index);
            }
            Ok(())
        })?,
        RuntimeArtifactFaultV1::CreatedManifest => mutate_receipt(&mut receipts, 2, |mutation| {
            let NormalizedMutation::KeyInserted { created, .. } = mutation else {
                return Err(invalid_log());
            };
            perturb_manifest(created.first_mut().ok_or_else(invalid_log)?)
        })?,
        RuntimeArtifactFaultV1::RetiredManifest => mutate_receipt(&mut receipts, 5, |mutation| {
            let NormalizedMutation::KeyRemoved { retired, .. } = mutation else {
                return Err(invalid_log());
            };
            perturb_manifest(retired.first_mut().ok_or_else(invalid_log)?)
        })?,
        RuntimeArtifactFaultV1::StateNodePath => {
            fault_state(&mut states, NormalizedStateFaultV1::NodePath)?
        }
        RuntimeArtifactFaultV1::StateNodeParent => {
            fault_state(&mut states, NormalizedStateFaultV1::NodeParent)?
        }
        RuntimeArtifactFaultV1::StateNodeTemplate => {
            fault_state(&mut states, NormalizedStateFaultV1::NodeTemplate)?
        }
        RuntimeArtifactFaultV1::StateNodeComponent => {
            fault_state(&mut states, NormalizedStateFaultV1::NodeComponent)?
        }
        RuntimeArtifactFaultV1::StateNodeOrder => {
            fault_state(&mut states, NormalizedStateFaultV1::NodeOrder)?
        }
        RuntimeArtifactFaultV1::StatePropertyOrder => {
            fault_state(&mut states, NormalizedStateFaultV1::PropertyOrder)?
        }
        RuntimeArtifactFaultV1::StatePropertyId => {
            fault_state(&mut states, NormalizedStateFaultV1::PropertyId)?
        }
        RuntimeArtifactFaultV1::StatePropertyValue => {
            fault_state(&mut states, NormalizedStateFaultV1::PropertyValue)?
        }
        RuntimeArtifactFaultV1::StateChildOrder => {
            fault_state(&mut states, NormalizedStateFaultV1::ChildOrder)?
        }
        RuntimeArtifactFaultV1::StateChildKind => {
            fault_state(&mut states, NormalizedStateFaultV1::ChildKind)?
        }
        RuntimeArtifactFaultV1::StateChildTarget => {
            fault_state(&mut states, NormalizedStateFaultV1::ChildTarget)?
        }
        RuntimeArtifactFaultV1::StateFragmentPath => {
            fault_state(&mut states, NormalizedStateFaultV1::FragmentPath)?
        }
        RuntimeArtifactFaultV1::StateFragmentDescriptor => {
            fault_state(&mut states, NormalizedStateFaultV1::FragmentDescriptor)?
        }
        RuntimeArtifactFaultV1::StateMemberOrder => {
            fault_state(&mut states, NormalizedStateFaultV1::MemberOrder)?
        }
        RuntimeArtifactFaultV1::StateMemberKey => {
            fault_state(&mut states, NormalizedStateFaultV1::MemberKey)?
        }
        RuntimeArtifactFaultV1::StateMemberPath => {
            fault_state(&mut states, NormalizedStateFaultV1::MemberPath)?
        }
        RuntimeArtifactFaultV1::Surface => {
            let projection = projections.first().ok_or_else(invalid_log)?;
            projections[0] =
                inject_headless_surface_fault_v1(projection).map_err(|_| invalid_log())?;
        }
        RuntimeArtifactFaultV1::Projection(projection_fault) => {
            let projection = projections.first().ok_or_else(invalid_log)?;
            projections[0] = inject_headless_projection_fault_v1(projection, projection_fault)
                .map_err(|_| invalid_log())?;
        }
    }
    Ok(LaneLog::from_parts(
        receipts,
        states,
        projections,
        log.final_keys().to_vec(),
    ))
}

fn receipt(
    receipts: &[NormalizedReceipt],
    index: usize,
) -> Result<&NormalizedReceipt, RuntimeArtifactEncodeErrorV1> {
    receipts.get(index).ok_or_else(invalid_log)
}

fn mutate_receipt(
    receipts: &mut [NormalizedReceipt],
    index: usize,
    mutate: impl FnOnce(&mut NormalizedMutation) -> Result<(), RuntimeArtifactEncodeErrorV1>,
) -> Result<(), RuntimeArtifactEncodeErrorV1> {
    let source = receipt(receipts, index)?;
    let generation = source.generation();
    let invalidation = source.invalidation();
    let mut mutations = source.mutations().to_vec();
    mutate(mutations.first_mut().ok_or_else(invalid_log)?)?;
    receipts[index] = NormalizedReceipt::new(generation, mutations, invalidation);
    Ok(())
}

fn fault_state(
    states: &mut [fenestra_ui_testkit::prototype::NormalizedStateV1],
    fault: NormalizedStateFaultV1,
) -> Result<(), RuntimeArtifactEncodeErrorV1> {
    let state = states.first().ok_or_else(invalid_log)?;
    states[0] = inject_normalized_state_fault_v1(state, fault).map_err(|_| invalid_log())?;
    Ok(())
}

fn perturb_manifest(
    entry: &mut NormalizedManifestEntry,
) -> Result<(), RuntimeArtifactEncodeErrorV1> {
    match entry {
        NormalizedManifestEntry::Node(path) => *path = perturb_node_path(path),
        NormalizedManifestEntry::Fragment(path) => *path = perturb_fragment_path(path),
    }
    Ok(())
}

fn perturb_node_path(path: &NodePathV1) -> NodePathV1 {
    path.clone().static_child(u16::MAX)
}

fn perturb_fragment_path(path: &FragmentPathV1) -> FragmentPathV1 {
    FragmentPathV1::new(path.owner().clone(), perturb_u16(path.region_slot()))
}

fn perturb_value(value: &PropertyValue) -> PropertyValue {
    match value {
        PropertyValue::Bool(value) => PropertyValue::Bool(!value),
        PropertyValue::ScalarI32(value) => PropertyValue::ScalarI32(perturb_i32(*value)),
        PropertyValue::Rgba8(value) => {
            let mut value = *value;
            value[0] ^= 1;
            PropertyValue::Rgba8(value)
        }
        PropertyValue::InputPolicy(InputPolicy::Accept) => {
            PropertyValue::InputPolicy(InputPolicy::Ignore)
        }
        PropertyValue::InputPolicy(InputPolicy::Ignore) => {
            PropertyValue::InputPolicy(InputPolicy::Accept)
        }
    }
}

const fn perturb_u16(value: u16) -> u16 {
    if value == u16::MAX {
        value - 1
    } else {
        value + 1
    }
}

const fn perturb_u32(value: u32) -> u32 {
    if value == u32::MAX {
        value - 1
    } else {
        value + 1
    }
}

const fn perturb_u64(value: u64) -> u64 {
    if value == u64::MAX {
        value - 1
    } else {
        value + 1
    }
}

const fn perturb_i32(value: i32) -> i32 {
    if value == i32::MAX {
        value - 1
    } else {
        value + 1
    }
}
