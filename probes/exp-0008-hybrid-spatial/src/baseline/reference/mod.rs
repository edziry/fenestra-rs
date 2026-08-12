mod adapter;
mod normalize;
pub(crate) mod runtime;

use fenestra_ui_layout::prototype::ReferenceStackEngineV1;
use fenestra_ui_spatial::prototype::{REGISTERED_SPATIAL_LIMITS_V2, resolve_spatial_v2};

use super::corpus::registered_corpus_v2;
use super::model::{
    CaseResultV2, EvidenceBuildErrorV2, SpatialEvidenceCaseV2, SpatialEvidenceV2, WidthWitnessV2,
};
use super::model_projection::observation_from_projection_v2;

pub(crate) fn reconstruct_reference_v2() -> Result<SpatialEvidenceV2, EvidenceBuildErrorV2> {
    let corpus = registered_corpus_v2();
    runtime::verify_runtime_observations_v2(&corpus[12].observations)?;
    let engine = ReferenceStackEngineV1::new();
    let mut cases = Vec::with_capacity(corpus.len());
    for case in corpus {
        let mut observations = Vec::with_capacity(case.observations.len());
        for input in &case.observations {
            let source = adapter::owned_input(&input.scene);
            let resolved = resolve_spatial_v2(&engine, source, REGISTERED_SPATIAL_LIMITS_V2)
                .map_err(|_| EvidenceBuildErrorV2 {
                    location: "reference-resolve",
                })?;
            let projection = normalize::projection(&input.scene, &resolved)?;
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
