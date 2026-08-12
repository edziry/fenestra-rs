use crate::lanes::cpu_reference::{
    CpuOutcomeV2, classify_cpu_run_v2, cpu_candidate_registry_v2, cpu_cases_v2, literal_cpu_run_v2,
    raqote_cpu_run_v2, tiny_skia_cpu_run_v2,
};
use crate::lanes::image_resource::{
    ImageCandidateV2, ImageOutcomeV2, classify_image_run_v2, image_candidate_registry_v2,
    image_cases_v2, image_crate_run_v2, literal_image_run_v2, png_image_run_v2,
};
use crate::lanes::native_renderer::{
    NativeOutcomeV2, classify_native_run_v2, literal_native_run_v2, native_candidate_registry_v2,
    native_cases_v2, vello_native_run_v2,
};
use crate::lanes::numeric_spatial::{NumericOutcomeV2, numeric_candidate_registry_v2};
use crate::lanes::path_hit::{PathHitOutcomeV2, path_hit_candidate_registry_v2};

use super::model::{
    CandidateEvidenceV2, LINUX, LaneEvidenceV2, VELLO_LINUX, VELLO_WINDOWS, WINDOWS,
};

const EUCLID: [(&str, &str); 1] = [("euclid", "0.22.14")];
const KURBO: [(&str, &str); 1] = [("kurbo", "0.13.1")];
const FIXED: [(&str, &str); 1] = [("fixed", "1.30.0")];
const LYON: [(&str, &str); 1] = [("lyon_tessellation", "1.0.20")];
const TINY_SKIA: [(&str, &str); 1] = [("tiny-skia", "0.12.0")];
const RAQOTE: [(&str, &str); 1] = [("raqote", "0.8.5")];
const VELLO: [(&str, &str); 2] = [("vello", "0.9.0"), ("wgpu", "29.0.3")];
const PNG: [(&str, &str); 1] = [("png", "0.18.1")];
const IMAGE: [(&str, &str); 1] = [("image", "0.25.10")];

pub(super) fn collect_lanes_v2() -> Result<Vec<LaneEvidenceV2>, &'static str> {
    Ok(vec![
        numeric_lane(),
        path_lane(),
        cpu_lane()?,
        native_lane()?,
        image_lane()?,
    ])
}

fn numeric_lane() -> LaneEvidenceV2 {
    let mut candidates = Vec::new();
    for registration in numeric_candidate_registry_v2() {
        let roots: &'static [(&'static str, &'static str)] = match registration.name {
            "euclid" => &EUCLID,
            "kurbo" => &KURBO,
            "fixed" => &FIXED,
            _ => unreachable!("closed numeric registry"),
        };
        push_pure(
            &mut candidates,
            "numeric-spatial",
            registration.name,
            registration.version,
            registration.features,
            roots,
            numeric_outcome(registration.outcome),
            registration.reason,
        );
    }
    LaneEvidenceV2 {
        file: "numeric-spatial-v2.txt",
        candidates,
    }
}

fn path_lane() -> LaneEvidenceV2 {
    let mut candidates = Vec::new();
    for registration in path_hit_candidate_registry_v2() {
        let roots: &'static [(&'static str, &'static str)] = match registration.name {
            "kurbo" => &KURBO,
            "lyon-tessellation" => &LYON,
            _ => unreachable!("closed path registry"),
        };
        push_pure(
            &mut candidates,
            "path-hit",
            registration.name,
            registration.version,
            registration.features,
            roots,
            path_outcome(registration.outcome),
            registration.reason,
        );
    }
    LaneEvidenceV2 {
        file: "path-hit-v2.txt",
        candidates,
    }
}

fn cpu_lane() -> Result<LaneEvidenceV2, &'static str> {
    let cases = cpu_cases_v2();
    let literal = literal_cpu_run_v2(&cases).map_err(|_| "literal CPU run failed")?;
    let observed = [
        tiny_skia_cpu_run_v2(&cases).map_err(|_| "Tiny-Skia run failed")?,
        raqote_cpu_run_v2(&cases).map_err(|_| "Raqote run failed")?,
    ];
    let mut candidates = Vec::new();
    for (registration, observed) in cpu_candidate_registry_v2().into_iter().zip(observed) {
        let classification = classify_cpu_run_v2(&literal, &observed);
        let roots: &'static [(&'static str, &'static str)] = match registration.name {
            "tiny-skia" => &TINY_SKIA,
            "raqote" => &RAQOTE,
            _ => unreachable!("closed CPU registry"),
        };
        push_pure(
            &mut candidates,
            "cpu-reference",
            registration.name,
            registration.version,
            registration.features,
            roots,
            cpu_outcome(classification.outcome),
            classification.reason,
        );
    }
    Ok(LaneEvidenceV2 {
        file: "cpu-reference-v2.txt",
        candidates,
    })
}

fn native_lane() -> Result<LaneEvidenceV2, &'static str> {
    let cases = native_cases_v2();
    let literal = literal_native_run_v2(&cases).map_err(|_| "literal native run failed")?;
    let observed = vello_native_run_v2(&cases).map_err(|_| "Vello run failed")?;
    let classification = classify_native_run_v2(&literal, &observed);
    let registration = native_candidate_registry_v2()[0];
    let versions = "0.9.0,29.0.3";
    let features = "wgpu,std,parking_lot,wgsl,vulkan,dx12";
    debug_assert_eq!(registration.version, "0.9.0");
    debug_assert_eq!(registration.gpu_version, "29.0.3");
    debug_assert_eq!(registration.renderer_features, "wgpu");
    debug_assert_eq!(
        registration.gpu_features,
        "std,parking_lot,wgsl,vulkan,dx12"
    );
    let outcome = native_outcome(classification.outcome);
    let candidates = vec![
        candidate(
            "native-renderer",
            registration.name,
            versions,
            features,
            VELLO_LINUX,
            &VELLO,
            outcome,
            classification.reason,
        ),
        candidate(
            "native-renderer",
            registration.name,
            versions,
            features,
            VELLO_WINDOWS,
            &VELLO,
            outcome,
            classification.reason,
        ),
    ];
    Ok(LaneEvidenceV2 {
        file: "native-renderer-v2.txt",
        candidates,
    })
}

fn image_lane() -> Result<LaneEvidenceV2, &'static str> {
    let cases = image_cases_v2();
    let literal = literal_image_run_v2(&cases).map_err(|_| "literal image run failed")?;
    let observed = [
        png_image_run_v2(&cases).map_err(|_| "PNG run failed")?,
        image_crate_run_v2(&cases).map_err(|_| "Image run failed")?,
    ];
    let mut candidates = Vec::new();
    for (registration, observed) in image_candidate_registry_v2().into_iter().zip(observed) {
        let classification = classify_image_run_v2(&literal, &observed);
        let roots: &'static [(&'static str, &'static str)] = match registration.kind {
            ImageCandidateV2::Png => &PNG,
            ImageCandidateV2::Image => &IMAGE,
        };
        push_pure(
            &mut candidates,
            "image-resource",
            registration.name,
            registration.version,
            registration.features,
            roots,
            image_outcome(classification.outcome),
            classification.reason,
        );
    }
    Ok(LaneEvidenceV2 {
        file: "image-resource-v2.txt",
        candidates,
    })
}

#[allow(clippy::too_many_arguments)]
fn push_pure(
    candidates: &mut Vec<CandidateEvidenceV2>,
    lane: &'static str,
    name: &'static str,
    versions: &'static str,
    features: &'static str,
    roots: &'static [(&'static str, &'static str)],
    outcome: &'static str,
    reason: &'static str,
) {
    candidates.push(candidate(
        lane, name, versions, features, LINUX, roots, outcome, reason,
    ));
    candidates.push(candidate(
        lane, name, versions, features, WINDOWS, roots, outcome, reason,
    ));
}

#[allow(clippy::too_many_arguments)]
const fn candidate(
    lane: &'static str,
    name: &'static str,
    versions: &'static str,
    features: &'static str,
    target: &'static str,
    roots: &'static [(&'static str, &'static str)],
    outcome: &'static str,
    reason: &'static str,
) -> CandidateEvidenceV2 {
    CandidateEvidenceV2 {
        lane,
        name,
        versions,
        features,
        target,
        roots,
        outcome,
        reason,
    }
}

const fn numeric_outcome(outcome: NumericOutcomeV2) -> &'static str {
    match outcome {
        NumericOutcomeV2::Pass => "pass",
    }
}

const fn path_outcome(outcome: PathHitOutcomeV2) -> &'static str {
    match outcome {
        PathHitOutcomeV2::Pass => "pass",
        PathHitOutcomeV2::Adapt => "adapt",
    }
}

const fn cpu_outcome(outcome: CpuOutcomeV2) -> &'static str {
    match outcome {
        CpuOutcomeV2::Pass => "pass",
        CpuOutcomeV2::Adapt => "adapt",
        CpuOutcomeV2::Stop => "stop",
    }
}

const fn native_outcome(outcome: NativeOutcomeV2) -> &'static str {
    match outcome {
        NativeOutcomeV2::Pass => "pass",
        NativeOutcomeV2::Adapt => "adapt",
        NativeOutcomeV2::Stop => "stop",
    }
}

const fn image_outcome(outcome: ImageOutcomeV2) -> &'static str {
    match outcome {
        ImageOutcomeV2::Pass => "pass",
        ImageOutcomeV2::Adapt => "adapt",
        ImageOutcomeV2::Stop => "stop",
    }
}
