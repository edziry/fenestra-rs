use fenestra_ui_testkit::prototype::inject_headless_projection_fault_v1;

use super::super::{NormalizedMutation, NormalizedReceipt};
use super::{LaneLog, RuntimeArtifactEncodeErrorV1, RuntimeArtifactFaultV1, invalid_log};

pub(super) fn inject(
    log: &LaneLog,
    fault: RuntimeArtifactFaultV1,
) -> Result<LaneLog, RuntimeArtifactEncodeErrorV1> {
    let mut receipts = log.receipts().to_vec();
    let mut states = log.states().to_vec();
    let mut projections = log.projections().to_vec();
    match fault {
        RuntimeArtifactFaultV1::Receipt => fault_receipt(&mut receipts)?,
        RuntimeArtifactFaultV1::Manifest => fault_manifest(&mut receipts)?,
        RuntimeArtifactFaultV1::StateOrder => {
            if states.len() < 2 {
                return Err(invalid_log());
            }
            states.swap(0, 1);
        }
        RuntimeArtifactFaultV1::Projection(fault) => {
            let projection = projections.first().ok_or_else(invalid_log)?;
            let faulted = inject_headless_projection_fault_v1(projection, fault)
                .map_err(|_| invalid_log())?;
            projections[0] = faulted;
        }
    }
    Ok(LaneLog::from_parts(
        receipts,
        states,
        projections,
        log.final_keys().to_vec(),
    ))
}

fn fault_receipt(receipts: &mut [NormalizedReceipt]) -> Result<(), RuntimeArtifactEncodeErrorV1> {
    let receipt = receipts.get(1).ok_or_else(invalid_log)?;
    receipts[1] = NormalizedReceipt::new(receipt.generation(), Vec::new(), receipt.invalidation());
    Ok(())
}

fn fault_manifest(receipts: &mut [NormalizedReceipt]) -> Result<(), RuntimeArtifactEncodeErrorV1> {
    let receipt = receipts.get(2).ok_or_else(invalid_log)?;
    let generation = receipt.generation();
    let invalidation = receipt.invalidation();
    let mut mutations = receipt.mutations().to_vec();
    let mutation = mutations.first_mut().ok_or_else(invalid_log)?;
    match mutation {
        NormalizedMutation::KeyInserted { created, .. } if !created.is_empty() => {
            created.clear();
        }
        NormalizedMutation::PropertyChanged { .. }
        | NormalizedMutation::KeyInserted { .. }
        | NormalizedMutation::KeyMoved { .. }
        | NormalizedMutation::KeyRemoved { .. }
        | NormalizedMutation::HeadlessSurfaceChanged { .. } => return Err(invalid_log()),
    }
    receipts[2] = NormalizedReceipt::new(generation, mutations, invalidation);
    Ok(())
}
