use fenestra_ui_testkit::prototype::NormalizedHeadlessProjectionV1;

use super::{
    RuntimeArtifactV1, RuntimeGeometryV1, RuntimeHitV1, RuntimeMilestoneV1,
    RuntimeProjectionCountsV1, RuntimeProjectionV1, RuntimeReceiptV1, RuntimeSceneV1,
    RuntimeStepV1,
};
use crate::script::MilestoneV1;

pub(crate) fn build_runtime_artifact_v1(
    reference: &[MilestoneV1],
    candidate: &[MilestoneV1],
) -> RuntimeArtifactV1 {
    assert_eq!(reference.len(), RuntimeStepV1::ALL.len());
    assert_eq!(candidate.len(), RuntimeStepV1::ALL.len());

    let milestones = RuntimeStepV1::ALL
        .into_iter()
        .zip(reference)
        .zip(candidate)
        .map(|((step, reference), candidate)| {
            assert_eq!(reference.oracle_projection(), candidate.oracle_projection());
            RuntimeMilestoneV1 {
                step,
                reference_receipt: receipt(reference),
                candidate_receipt: receipt(candidate),
                oracle_projection: projection(reference.oracle_projection()),
                reference_projection: projection(reference.projection()),
                candidate_projection: projection(candidate.projection()),
            }
        })
        .collect();
    RuntimeArtifactV1 { milestones }
}

fn receipt(milestone: &MilestoneV1) -> RuntimeReceiptV1 {
    RuntimeReceiptV1 {
        receipt_generation: milestone.receipt_generation(),
        projection_generation: milestone.projection_generation(),
        invalidation: milestone.invalidation(),
        mutation_count: milestone.mutation_count(),
    }
}

fn projection(source: &NormalizedHeadlessProjectionV1) -> RuntimeProjectionV1 {
    let geometries = source
        .geometries()
        .iter()
        .map(|record| RuntimeGeometryV1 {
            path: record.path().clone(),
            bounds: record.bounds(),
            clip: record.clip(),
        })
        .collect();
    let hits = source
        .hit_regions()
        .iter()
        .map(|record| RuntimeHitV1 {
            path: record.path().clone(),
            clip: record.clip(),
        })
        .collect();
    let scenes = source
        .scene_rectangles()
        .iter()
        .map(|record| RuntimeSceneV1 {
            path: record.path().clone(),
            rectangle: record.rectangle(),
            color: record.color(),
        })
        .collect();
    RuntimeProjectionV1 {
        surface: source.surface(),
        counts: RuntimeProjectionCountsV1 {
            computed_styles: source.computed_styles().len(),
            geometry: source.geometries().len(),
            semantics: source.semantics().len(),
            hit_regions: source.hit_regions().len(),
            scene_rectangles: source.scene_rectangles().len(),
        },
        geometries,
        hits,
        scenes,
    }
}
