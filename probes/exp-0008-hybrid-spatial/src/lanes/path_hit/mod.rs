mod candidates;
mod faults;
mod input;
mod oracle;
mod types;

pub(crate) use candidates::{kurbo_run as kurbo_path_hit_run_v2, lyon_run as lyon_path_hit_run_v2};
pub(crate) use faults::path_hit_faults_v2;
pub(crate) use input::path_hit_cases_v2;
pub(crate) use oracle::literal_path_hit_run_v2;
pub(crate) use types::{
    PathHitCandidateV2, PathHitFaultKindV2, PathHitObligationV2, PathHitOutcomeV2,
};

use types::{PathHitCandidateRegistrationV2, PathHitCaseV2, PathHitRecordV2, PathHitRunV2};

pub(crate) const fn path_hit_candidate_registry_v2() -> [PathHitCandidateRegistrationV2; 2] {
    [
        PathHitCandidateRegistrationV2 {
            kind: PathHitCandidateV2::Kurbo,
            name: "kurbo",
            version: "0.13.1",
            features: "std",
            outcome: PathHitOutcomeV2::Adapt,
            reason: "edge-rounding",
        },
        PathHitCandidateRegistrationV2 {
            kind: PathHitCandidateV2::Lyon,
            name: "lyon-tessellation",
            version: "1.0.20",
            features: "std",
            outcome: PathHitOutcomeV2::Pass,
            reason: "-",
        },
    ]
}

fn finish_run(cases: &[PathHitCaseV2], records: Vec<PathHitRecordV2>) -> PathHitRunV2 {
    PathHitRunV2 {
        triangle_witnesses: records.iter().map(|record| record.layer_hits.len()).sum(),
        reverse_painter_queries: cases
            .iter()
            .filter(|case| case.layers.len() > 1)
            .map(|case| case.queries.len())
            .sum(),
        nonrectangular_aabb_misses: cases
            .iter()
            .flat_map(|case| &case.queries)
            .filter(|query| query.nonrectangular_aabb_miss)
            .count(),
        records,
    }
}
