use std::collections::BTreeMap;

use super::closure::closure_sha256_v2;
use super::{BASELINE, BASELINE_SHA256, sha256_hex_v2};

const MAX_RECORDS: usize = 4_096;
const MAX_LINE_BYTES: usize = 1_024;
const MAX_ARTIFACT_BYTES: usize = 1_048_576;

pub(crate) fn verify_lane_artifact_v2(bytes: &[u8]) -> Result<(), &'static str> {
    if bytes.len() > MAX_ARTIFACT_BYTES {
        return Err("lane artifact exceeds byte limit");
    }
    if bytes.last() != Some(&b'\n') || !bytes.is_ascii() {
        return Err("lane artifact is not canonical ASCII with final LF");
    }
    let text = std::str::from_utf8(bytes).map_err(|_| "lane artifact is not UTF-8")?;
    let lines = text.lines().collect::<Vec<_>>();
    if lines.len() > MAX_RECORDS {
        return Err("lane artifact exceeds record limit");
    }
    if lines.iter().any(|line| line.len() > MAX_LINE_BYTES) {
        return Err("lane artifact exceeds line limit");
    }
    if lines.first() != Some(&"spatial-v2|artifact=2|contract=2|corpus=2|kind=lane") {
        return Err("invalid lane artifact header");
    }
    let candidates = indexed_lines(&lines, "candidate|");
    let classifications = indexed_lines(&lines, "classification|");
    if candidates.is_empty() || candidates.len() != classifications.len() {
        return Err("invalid lane candidate count");
    }
    verify_positions(&lines, &candidates, &classifications)?;
    verify_counts(&lines, candidates.len())?;
    verify_candidates(&candidates, &classifications)?;
    verify_baseline_body(&lines, candidates.len())
}

fn verify_positions(
    lines: &[&str],
    candidates: &[(usize, &str)],
    classifications: &[(usize, &str)],
) -> Result<(), &'static str> {
    let profile = index_of(lines, "profile|")?;
    let result = index_of(lines, "result|")?;
    if candidates
        .iter()
        .enumerate()
        .any(|(offset, (index, _))| *index != profile + 1 + offset)
    {
        return Err("candidate rows are not directly after profile");
    }
    if classifications
        .iter()
        .enumerate()
        .any(|(offset, (index, _))| *index + classifications.len() - offset != result)
    {
        return Err("classification rows are not directly before result");
    }
    Ok(())
}

fn verify_counts(lines: &[&str], expected: usize) -> Result<(), &'static str> {
    for prefix in ["profile|", "result|"] {
        let line = lines[index_of(lines, prefix)?];
        let count = field_value(line, "candidate-count")?
            .parse::<usize>()
            .map_err(|_| "candidate count is invalid")?;
        if count != expected {
            return Err("candidate count does not match rows");
        }
    }
    Ok(())
}

fn field_value<'a>(line: &'a str, name: &str) -> Result<&'a str, &'static str> {
    let mut matches = line.split('|').skip(1).filter_map(|field| {
        let (field_name, value) = field.split_once('=')?;
        (field_name == name).then_some(value)
    });
    let value = matches.next().ok_or("required field is absent")?;
    if matches.next().is_some() {
        return Err("required field is duplicated");
    }
    Ok(value)
}

fn verify_candidates(
    candidates: &[(usize, &str)],
    classifications: &[(usize, &str)],
) -> Result<(), &'static str> {
    let mut lane = None;
    for ordinal in 0..candidates.len() {
        let candidate = fields(candidates[ordinal].1, CANDIDATE_FIELDS)?;
        let classification = fields(classifications[ordinal].1, CLASSIFICATION_FIELDS)?;
        exact_ordinal(&candidate, "ordinal", ordinal)?;
        exact_ordinal(&classification, "candidate", ordinal)?;
        let candidate_lane = required(&candidate, "lane")?;
        if lane
            .replace(candidate_lane)
            .is_some_and(|value| value != candidate_lane)
        {
            return Err("candidate rows mix lanes");
        }
        if required(&candidate, "baseline-sha256")? != BASELINE_SHA256 {
            return Err("candidate baseline digest is invalid");
        }
        let expected = expected_candidate(candidate_lane, ordinal)?;
        if required(&candidate, "name")? != expected.name
            || required(&candidate, "versions")? != expected.versions
            || required(&candidate, "features")? != expected.features
            || required(&classification, "outcome")? != expected.outcome
            || required(&classification, "reason")? != expected.reason
        {
            return Err("candidate tuple or classification is not registered");
        }
        let expected_closure = candidate_closure(&candidate)?;
        if required(&candidate, "closure-sha256")? != expected_closure {
            return Err("candidate closure digest is invalid");
        }
        verify_target(candidate_lane, required(&candidate, "target")?, ordinal)?;
        verify_classification(
            required(&classification, "outcome")?,
            required(&classification, "reason")?,
        )?;
    }
    let lane = lane.ok_or("lane has no candidates")?;
    if candidates.len() != expected_candidate_count(lane)? {
        return Err("lane candidate registry is incomplete");
    }
    Ok(())
}

struct ExpectedCandidateV2 {
    name: &'static str,
    versions: &'static str,
    features: &'static str,
    outcome: &'static str,
    reason: &'static str,
}

fn expected_candidate(lane: &str, ordinal: usize) -> Result<ExpectedCandidateV2, &'static str> {
    let pair = ordinal / 2;
    let expected = match (lane, pair) {
        ("numeric-spatial", 0) => expected("euclid", "0.22.14", "std", "pass", "-"),
        ("numeric-spatial", 1) => expected("kurbo", "0.13.1", "std", "pass", "-"),
        ("numeric-spatial", 2) => expected("fixed", "1.30.0", "-", "pass", "-"),
        ("path-hit", 0) => expected("kurbo", "0.13.1", "std", "adapt", "edge-rounding"),
        ("path-hit", 1) => expected("lyon-tessellation", "1.0.20", "std", "pass", "-"),
        ("cpu-reference", 0) => expected("tiny-skia", "0.12.0", "std", "stop", "mismatch"),
        ("cpu-reference", 1) => expected("raqote", "0.8.5", "-", "stop", "mismatch"),
        ("native-renderer", 0) => expected(
            "vello",
            "0.9.0,29.0.3",
            "wgpu,std,parking_lot,wgsl,vulkan,dx12",
            "stop",
            "target-unavailable",
        ),
        ("image-resource", 0) => {
            expected("png", "0.18.1", "-", "adapt", "orientation-normalization")
        }
        ("image-resource", 1) => expected("image", "0.25.10", "png", "stop", "mismatch"),
        _ => return Err("candidate ordinal is not registered for lane"),
    };
    Ok(expected)
}

const fn expected(
    name: &'static str,
    versions: &'static str,
    features: &'static str,
    outcome: &'static str,
    reason: &'static str,
) -> ExpectedCandidateV2 {
    ExpectedCandidateV2 {
        name,
        versions,
        features,
        outcome,
        reason,
    }
}

fn expected_candidate_count(lane: &str) -> Result<usize, &'static str> {
    match lane {
        "numeric-spatial" => Ok(6),
        "path-hit" | "cpu-reference" | "image-resource" => Ok(4),
        "native-renderer" => Ok(2),
        _ => Err("lane is not registered"),
    }
}

fn candidate_closure(candidate: &BTreeMap<&str, &str>) -> Result<String, &'static str> {
    let name = required(candidate, "name")?;
    let versions = required(candidate, "versions")?
        .split(',')
        .collect::<Vec<_>>();
    let roots = match (name, versions.as_slice()) {
        ("euclid", [version]) => vec![("euclid", *version)],
        ("kurbo", [version]) => vec![("kurbo", *version)],
        ("fixed", [version]) => vec![("fixed", *version)],
        ("lyon-tessellation", [version]) => vec![("lyon_tessellation", *version)],
        ("tiny-skia", [version]) => vec![("tiny-skia", *version)],
        ("raqote", [version]) => vec![("raqote", *version)],
        ("png", [version]) => vec![("png", *version)],
        ("image", [version]) => vec![("image", *version)],
        ("vello", [vello, wgpu]) => vec![("vello", *vello), ("wgpu", *wgpu)],
        _ => return Err("candidate package tuple is not registered"),
    };
    closure_sha256_v2(&roots)
}

fn verify_target(lane: &str, target: &str, ordinal: usize) -> Result<(), &'static str> {
    let expected = if lane == "native-renderer" {
        [
            "x86_64-unknown-linux-gnu:vulkan-wayland",
            "x86_64-pc-windows-msvc:dx12-win32",
        ][ordinal % 2]
    } else {
        ["x86_64-unknown-linux-gnu", "x86_64-pc-windows-msvc"][ordinal % 2]
    };
    (target == expected)
        .then_some(())
        .ok_or("candidate target is not registered")
}

fn verify_classification(outcome: &str, reason: &str) -> Result<(), &'static str> {
    const ADAPT: [&str; 6] = [
        "fixed16-conversion",
        "edge-rounding",
        "painter-reorder",
        "premultiplied-rgba8",
        "orientation-normalization",
        "profile-rejection",
    ];
    const STOP: [&str; 8] = [
        "mismatch",
        "unsupported",
        "nondeterministic",
        "dependency-policy",
        "target-unavailable",
        "unsafe-boundary",
        "build-boundary",
        "resource-bound",
    ];
    let valid = match outcome {
        "pass" => reason == "-",
        "adapt" => ADAPT.contains(&reason),
        "stop" => STOP.contains(&reason),
        _ => false,
    };
    valid.then_some(()).ok_or("invalid lane classification")
}

fn verify_baseline_body(lines: &[&str], candidate_count: usize) -> Result<(), &'static str> {
    let mut body = String::with_capacity(BASELINE.len());
    for line in lines {
        if line.starts_with("candidate|") || line.starts_with("classification|") {
            continue;
        }
        let normalized = line.replace("kind=lane", "kind=baseline").replace(
            &format!("candidate-count={candidate_count}"),
            "candidate-count=0",
        );
        body.push_str(&normalized);
        body.push('\n');
    }
    if body.as_bytes() != BASELINE || sha256_hex_v2(BASELINE) != BASELINE_SHA256 {
        return Err("lane artifact changed the baseline body");
    }
    Ok(())
}

fn indexed_lines<'a>(lines: &[&'a str], prefix: &str) -> Vec<(usize, &'a str)> {
    lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| line.starts_with(prefix).then_some((index, *line)))
        .collect()
}

fn index_of(lines: &[&str], prefix: &str) -> Result<usize, &'static str> {
    let mut matches = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| line.starts_with(prefix));
    let (index, _) = matches.next().ok_or("required lane row is absent")?;
    if matches.next().is_some() {
        return Err("required lane row is duplicated");
    }
    Ok(index)
}

const CANDIDATE_FIELDS: &[&str] = &[
    "ordinal",
    "lane",
    "name",
    "versions",
    "features",
    "target",
    "closure-sha256",
    "baseline-sha256",
];
const CLASSIFICATION_FIELDS: &[&str] = &["candidate", "outcome", "reason"];

fn fields<'a>(
    line: &'a str,
    expected: &[&str],
) -> Result<BTreeMap<&'a str, &'a str>, &'static str> {
    let raw = line.split('|').skip(1).collect::<Vec<_>>();
    if raw.len() != expected.len() {
        return Err("lane row field count is not exact");
    }
    let fields = raw
        .into_iter()
        .map(|field| field.split_once('=').ok_or("lane field has no value"))
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    if fields.len() != expected.len() || expected.iter().any(|name| !fields.contains_key(name)) {
        return Err("lane row fields are not exact");
    }
    Ok(fields)
}

fn required<'a>(fields: &BTreeMap<&str, &'a str>, name: &str) -> Result<&'a str, &'static str> {
    fields.get(name).copied().ok_or("required field is absent")
}

fn exact_ordinal(
    fields: &BTreeMap<&str, &str>,
    name: &str,
    expected: usize,
) -> Result<(), &'static str> {
    let observed = required(fields, name)?
        .parse::<usize>()
        .map_err(|_| "candidate ordinal is invalid")?;
    (observed == expected)
        .then_some(())
        .ok_or("candidate ordinals are not dense")
}
