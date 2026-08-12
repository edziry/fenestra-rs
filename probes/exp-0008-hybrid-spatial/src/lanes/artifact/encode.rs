use std::fmt::Write;

use super::closure::closure_sha256_v2;
use super::collect::collect_lanes_v2;
use super::model::{CandidateEvidenceV2, LaneEvidenceV2};
use super::{BASELINE, BASELINE_SHA256, LaneArtifactV2, verify_lane_artifact_v2};

pub(crate) fn all_lane_artifacts_v2() -> Result<Vec<LaneArtifactV2>, &'static str> {
    collect_lanes_v2()?.into_iter().map(encode_lane).collect()
}

fn encode_lane(lane: LaneEvidenceV2) -> Result<LaneArtifactV2, &'static str> {
    let mut output = String::with_capacity(BASELINE.len() + lane.candidates.len() * 400);
    for line in std::str::from_utf8(BASELINE)
        .map_err(|_| "baseline artifact is not UTF-8")?
        .lines()
    {
        if line.starts_with("spatial-v2|") {
            push_line(&mut output, &line.replace("kind=baseline", "kind=lane"));
        } else if line.starts_with("profile|") {
            push_line(
                &mut output,
                &line.replace(
                    "candidate-count=0",
                    &format!("candidate-count={}", lane.candidates.len()),
                ),
            );
            for (ordinal, candidate) in lane.candidates.iter().enumerate() {
                push_candidate(&mut output, ordinal, candidate)?;
            }
        } else if line.starts_with("result|") {
            for (ordinal, candidate) in lane.candidates.iter().enumerate() {
                writeln!(
                    output,
                    "classification|candidate={ordinal}|outcome={}|reason={}",
                    candidate.outcome, candidate.reason
                )
                .map_err(|_| "lane classification encoding failed")?;
            }
            push_line(
                &mut output,
                &line.replace(
                    "candidate-count=0",
                    &format!("candidate-count={}", lane.candidates.len()),
                ),
            );
        } else {
            push_line(&mut output, line);
        }
    }
    let artifact = LaneArtifactV2 {
        name: lane.file,
        bytes: output.into_bytes(),
    };
    verify_lane_artifact_v2(&artifact.bytes)?;
    Ok(artifact)
}

fn push_candidate(
    output: &mut String,
    ordinal: usize,
    candidate: &CandidateEvidenceV2,
) -> Result<(), &'static str> {
    let closure = closure_sha256_v2(candidate.roots)?;
    writeln!(
        output,
        "candidate|ordinal={ordinal}|lane={}|name={}|versions={}|features={}|target={}|closure-sha256={closure}|baseline-sha256={BASELINE_SHA256}",
        candidate.lane,
        candidate.name,
        candidate.versions,
        candidate.features,
        candidate.target,
    )
    .map_err(|_| "lane candidate encoding failed")
}

fn push_line(output: &mut String, line: &str) {
    output.push_str(line);
    output.push('\n');
}
