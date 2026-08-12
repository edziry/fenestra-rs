use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use crate::lanes::artifact::{all_lane_artifacts_v2, sha256_hex_v2, verify_lane_artifact_v2};

const BASELINE_SHA256: &str = "bc71d3f9167808984abf083613ea86a81eced60d8670d9b3133821dbb34d21a1";

#[test]
fn five_committed_lane_artifacts_are_exact_fresh_encodings() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/artifacts");
    let artifacts = all_lane_artifacts_v2().expect("fresh lane artifacts");
    assert_eq!(
        artifacts
            .iter()
            .map(|artifact| artifact.name)
            .collect::<Vec<_>>(),
        vec![
            "numeric-spatial-v2.txt",
            "path-hit-v2.txt",
            "cpu-reference-v2.txt",
            "native-renderer-v2.txt",
            "image-resource-v2.txt",
        ]
    );
    for artifact in artifacts {
        let committed = fs::read(root.join(artifact.name)).expect("committed lane artifact");
        assert_eq!(committed, artifact.bytes, "{} is stale", artifact.name);
        verify_lane_artifact_v2(&committed).expect("valid lane artifact");
    }
}

#[test]
fn lane_rows_and_classifications_are_complete_and_exact() {
    let expectations = [
        (
            "numeric-spatial",
            vec!["euclid", "euclid", "kurbo", "kurbo", "fixed", "fixed"],
            vec![("pass", "-"); 6],
        ),
        (
            "path-hit",
            vec!["kurbo", "kurbo", "lyon-tessellation", "lyon-tessellation"],
            vec![
                ("adapt", "edge-rounding"),
                ("adapt", "edge-rounding"),
                ("pass", "-"),
                ("pass", "-"),
            ],
        ),
        (
            "cpu-reference",
            vec!["tiny-skia", "tiny-skia", "raqote", "raqote"],
            vec![("stop", "mismatch"); 4],
        ),
        (
            "native-renderer",
            vec!["vello", "vello"],
            vec![("stop", "target-unavailable"); 2],
        ),
        (
            "image-resource",
            vec!["png", "png", "image", "image"],
            vec![
                ("adapt", "orientation-normalization"),
                ("adapt", "orientation-normalization"),
                ("stop", "mismatch"),
                ("stop", "mismatch"),
            ],
        ),
    ];
    for (artifact, (lane, names, outcomes)) in all_lane_artifacts_v2()
        .expect("lane artifacts")
        .into_iter()
        .zip(expectations)
    {
        let lines = text_lines(&artifact.bytes);
        let candidates = lines
            .iter()
            .filter(|line| line.starts_with("candidate|"))
            .map(|line| fields(line))
            .collect::<Vec<_>>();
        let classifications = lines
            .iter()
            .filter(|line| line.starts_with("classification|"))
            .map(|line| fields(line))
            .collect::<Vec<_>>();
        assert_eq!(candidates.len(), names.len());
        assert_eq!(classifications.len(), names.len());
        for (ordinal, ((candidate, classification), expected_name)) in candidates
            .iter()
            .zip(&classifications)
            .zip(names)
            .enumerate()
        {
            assert_eq!(candidate["ordinal"], ordinal.to_string());
            assert_eq!(candidate["lane"], lane);
            assert_eq!(candidate["name"], expected_name);
            assert_eq!(candidate["baseline-sha256"], BASELINE_SHA256);
            assert_hex64(candidate["closure-sha256"]);
            assert_eq!(classification["candidate"], ordinal.to_string());
            assert_eq!(
                (classification["outcome"], classification["reason"]),
                outcomes[ordinal]
            );
        }
        for pair in candidates.chunks_exact(2) {
            assert_eq!(pair[0]["closure-sha256"], pair[1]["closure-sha256"]);
            if lane != "native-renderer" {
                assert_eq!(pair[0]["target"], "x86_64-unknown-linux-gnu");
                assert_eq!(pair[1]["target"], "x86_64-pc-windows-msvc");
            }
        }
        if lane == "native-renderer" {
            assert_eq!(
                candidates[0]["target"],
                "x86_64-unknown-linux-gnu:vulkan-wayland"
            );
            assert_eq!(candidates[1]["target"], "x86_64-pc-windows-msvc:dx12-win32");
        }
    }
}

#[test]
fn lane_encoding_preserves_the_complete_baseline_body() {
    let baseline =
        fs::read(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/artifacts/spatial-v2.txt"))
            .expect("baseline artifact");
    assert_eq!(sha256_hex_v2(&baseline), BASELINE_SHA256);
    let baseline_lines = text_lines(&baseline);
    for artifact in all_lane_artifacts_v2().expect("lane artifacts") {
        let lines = text_lines(&artifact.bytes);
        let body = lines
            .into_iter()
            .filter(|line| !line.starts_with("candidate|") && !line.starts_with("classification|"))
            .map(|line| {
                line.replace("kind=lane", "kind=baseline").replace(
                    &format!(
                        "candidate-count={}",
                        artifact
                            .bytes
                            .split(|byte| *byte == b'\n')
                            .filter(|line| line.starts_with(b"candidate|"))
                            .count()
                    ),
                    "candidate-count=0",
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(body, baseline_lines);
    }
}

#[test]
fn sha256_is_real_and_encodings_are_fresh_deterministic() {
    assert_eq!(
        sha256_hex_v2(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_eq!(
        all_lane_artifacts_v2().expect("first fresh artifacts"),
        all_lane_artifacts_v2().expect("second fresh artifacts")
    );
}

fn text_lines(bytes: &[u8]) -> Vec<String> {
    assert_eq!(bytes.last(), Some(&b'\n'));
    std::str::from_utf8(bytes)
        .expect("ASCII artifact")
        .lines()
        .map(str::to_owned)
        .collect()
}

fn fields(line: &str) -> BTreeMap<&str, &str> {
    line.split('|')
        .skip(1)
        .map(|field| field.split_once('=').expect("named field"))
        .collect()
}

fn assert_hex64(value: &str) {
    assert_eq!(value.len(), 64);
    assert!(
        value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    );
}
