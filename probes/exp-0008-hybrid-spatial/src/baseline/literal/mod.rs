use super::corpus::registered_corpus_v2;
use super::model::{
    CaseResultV2, EvidenceBuildErrorV2, SpatialEvidenceCaseV2, SpatialEvidenceV2, WidthWitnessV2,
};
use super::model_projection::observation_from_projection_v2;

mod coverage;
mod normalize;
mod numeric;
mod paint;
mod path;
mod resolve;
mod types;

pub(crate) fn reconstruct_literal_v2() -> Result<SpatialEvidenceV2, EvidenceBuildErrorV2> {
    let mut cases = Vec::new();
    for case in registered_corpus_v2() {
        let mut observations = Vec::new();
        for input in &case.observations {
            let plan = resolve::prepare(&input.scene);
            let projection = normalize::normalize(&plan);
            observations.push(observation_from_projection_v2(
                case.ordinal,
                input.step,
                &input.scene,
                &projection,
            ));
        }
        cases.push(SpatialEvidenceCaseV2 {
            ordinal: case.ordinal,
            name: case.name,
            observations,
            result: CaseResultV2::MATCH,
        });
    }
    Ok(SpatialEvidenceV2 {
        cases,
        width_witness: WidthWitnessV2::REGISTERED,
    })
}
