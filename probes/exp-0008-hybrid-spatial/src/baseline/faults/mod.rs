mod input;
mod limits;
mod output;
mod raster;
mod types;

pub(crate) use types::{RawFaultEvidenceV2, RollbackEvidenceV2};

pub(crate) fn raw_fault_evidence_v2() -> RawFaultEvidenceV2 {
    RawFaultEvidenceV2 {
        raw_inputs: input::content_faults(),
        limits: limits::raw_limits(),
        output_faults: output::output_faults(),
        dependency_cycle: input::dependency_cycle(),
        singular: input::singular_transform(),
        raster: raster::raster_limit(),
        rollback: rollback_probe(),
        native_faults: 0,
        native_presenter_rows: 0,
        candidate_faults: 0,
    }
}

pub(crate) fn fault_control_count_v2(report: &RawFaultEvidenceV2) -> (u64, u64, bool) {
    let registered =
        report.raw_inputs.len() + report.limits.len() * 2 + report.output_faults.len() + 3 + 8;
    let registered = u64::try_from(registered).expect("fault control count should fit u64");
    let detected = report
        .raw_inputs
        .iter()
        .all(|fault| fault.observed.is_none() && fault.maximum.is_none())
        && report
            .limits
            .iter()
            .all(|boundary| boundary.equality_passes && boundary.observed == boundary.maximum + 1)
        && report.rollback.before_digest == report.rollback.after_digest
        && report.rollback.before_allocation == report.rollback.after_allocation
        && report.rollback.before_state == report.rollback.after_state;
    (registered, if detected { registered } else { 0 }, detected)
}

fn rollback_probe() -> RollbackEvidenceV2 {
    let corpus = super::corpus::registered_corpus_v2();
    let probe = super::reference::runtime::verify_runtime_observations_v2(&corpus[12].observations)
        .expect("registered runtime rollback probe must execute");
    RollbackEvidenceV2 {
        attempted_generation: probe.attempted_generation,
        retained_generation: probe.retained_generation,
        before_digest: probe.before_digest,
        after_digest: probe.after_digest,
        before_allocation: probe.before_allocation,
        after_allocation: probe.after_allocation,
        before_state: probe.before_state,
        after_state: probe.after_state,
    }
}
