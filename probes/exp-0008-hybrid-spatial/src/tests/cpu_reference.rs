use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::lanes::cpu_reference::{
    CpuCandidateV2, CpuFaultKindV2, CpuObligationV2, CpuOutcomeV2, classify_cpu_run_v2,
    cpu_candidate_registry_v2, cpu_cases_v2, cpu_faults_v2, literal_cpu_run_v2, raqote_cpu_run_v2,
    tiny_skia_cpu_run_v2,
};

#[test]
fn cpu_candidate_registry_and_profiles_are_exact() {
    let registry = cpu_candidate_registry_v2();
    assert_eq!(
        registry
            .iter()
            .map(|candidate| (
                candidate.kind,
                candidate.name,
                candidate.version,
                candidate.features
            ))
            .collect::<Vec<_>>(),
        vec![
            (CpuCandidateV2::TinySkia, "tiny-skia", "0.12.0", "std"),
            (CpuCandidateV2::Raqote, "raqote", "0.8.5", "-"),
        ]
    );
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let manifest = fs::read_to_string(root.join("Cargo.toml")).expect("manifest");
    for declaration in [
        "cpu-reference = [\"dep:raqote\", \"dep:tiny-skia\"]",
        "tiny-skia = { version = \"=0.12.0\", default-features = false, features = [\"std\"], optional = true }",
        "raqote = { version = \"=0.8.5\", default-features = false, optional = true }",
    ] {
        assert!(manifest.contains(declaration), "missing {declaration}");
    }
    let lock = fs::read_to_string(root.join("../../Cargo.lock")).expect("lock");
    for package in [
        "name = \"tiny-skia\"\nversion = \"0.12.0\"",
        "name = \"raqote\"\nversion = \"0.8.5\"",
    ] {
        assert_eq!(lock.matches(package).count(), 1, "{package}");
    }
}

#[test]
fn cpu_cases_cover_the_closed_rich_raster_obligations() {
    let cases = cpu_cases_v2();
    assert_eq!(
        cases.iter().map(|case| case.ordinal).collect::<Vec<_>>(),
        (0..cases.len() as u8).collect::<Vec<_>>()
    );
    assert_eq!(
        cases
            .iter()
            .flat_map(|case| case.obligations.iter().copied())
            .collect::<BTreeSet<_>>(),
        CpuObligationV2::ALL.into_iter().collect()
    );
    assert!(cases.iter().all(|case| case.width > 0 && case.height > 0));
}

#[test]
fn cpu_candidates_are_fresh_deterministic_and_exactly_classified() {
    let first_cases = cpu_cases_v2();
    let second_cases = cpu_cases_v2();
    assert_eq!(first_cases, second_cases);
    let literal = literal_cpu_run_v2(&first_cases).expect("literal CPU run");
    assert_eq!(
        literal_cpu_run_v2(&second_cases).expect("fresh literal CPU run"),
        literal
    );
    assert!(
        literal
            .cases
            .iter()
            .all(|case| { case.bytes.len() == case.width as usize * case.height as usize * 4 })
    );

    for (candidate, first, second) in [
        (
            CpuCandidateV2::TinySkia,
            tiny_skia_cpu_run_v2(&first_cases).expect("Tiny-Skia run"),
            tiny_skia_cpu_run_v2(&second_cases).expect("fresh Tiny-Skia run"),
        ),
        (
            CpuCandidateV2::Raqote,
            raqote_cpu_run_v2(&first_cases).expect("Raqote run"),
            raqote_cpu_run_v2(&second_cases).expect("fresh Raqote run"),
        ),
    ] {
        assert_eq!(first, second, "{candidate:?} is nondeterministic");
        let classification = classify_cpu_run_v2(&literal, &first);
        assert_eq!(classification.candidate, candidate);
        assert_eq!(classification.outcome, CpuOutcomeV2::Stop);
        assert_eq!(classification.reason, "mismatch");
        let mismatch = classification
            .first_mismatch
            .expect("registered CPU mismatch");
        let expected_mismatch = match candidate {
            CpuCandidateV2::TinySkia => (2, 48, 0, 40),
            CpuCandidateV2::Raqote => (2, 40, 40, 0),
        };
        assert_eq!(
            (
                mismatch.case,
                mismatch.byte,
                mismatch.expected,
                mismatch.observed,
            ),
            expected_mismatch
        );
        match classification.outcome {
            CpuOutcomeV2::Pass => assert_eq!(classification.reason, "-"),
            CpuOutcomeV2::Adapt => assert_eq!(classification.reason, "premultiplied-rgba8"),
            CpuOutcomeV2::Stop => assert_eq!(classification.reason, "mismatch"),
        }
        if classification.outcome == CpuOutcomeV2::Pass {
            assert_eq!(first, literal);
        } else {
            assert!(classification.first_mismatch.is_some());
        }
    }
}

#[test]
fn cpu_faults_are_typed_and_candidate_adapters_preflight_them() {
    let faults = cpu_faults_v2();
    assert_eq!(
        faults.iter().map(|fault| fault.kind).collect::<Vec<_>>(),
        vec![
            CpuFaultKindV2::ZeroDimension,
            CpuFaultKindV2::PixelLimit,
            CpuFaultKindV2::InvalidImageStride,
            CpuFaultKindV2::NonFiniteTransform,
            CpuFaultKindV2::UnsupportedSampling,
        ]
    );
    assert!(faults.iter().all(|fault| fault.literal));
    assert!(faults.iter().all(|fault| fault.tiny_skia));
    assert!(faults.iter().all(|fault| fault.raqote));
}

#[test]
fn cpu_oracle_and_candidate_sources_are_private_and_separate() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/lanes/cpu_reference");
    let files = rust_files(&root);
    let expected = BTreeSet::from([
        "candidates/mod.rs".to_owned(),
        "candidates/raqote.rs".to_owned(),
        "candidates/tiny_skia.rs".to_owned(),
        "compare.rs".to_owned(),
        "faults.rs".to_owned(),
        "input.rs".to_owned(),
        "mod.rs".to_owned(),
        "oracle.rs".to_owned(),
        "types.rs".to_owned(),
    ]);
    assert_eq!(files.keys().cloned().collect::<BTreeSet<_>>(), expected);
    let oracle = &files["oracle.rs"];
    for forbidden in ["tiny_skia", "raqote", "crate::baseline", "candidate"] {
        assert!(!oracle.contains(forbidden), "oracle contains {forbidden}");
    }
    for (path, source) in &files {
        assert!(!source.contains("pub "), "public item in {path}");
        assert!(!source.contains("unsafe"), "unsafe code in {path}");
        if !path.starts_with("candidates/") {
            assert!(!source.contains("use tiny_skia"));
            assert!(!source.contains("use raqote"));
        }
    }
}

fn rust_files(root: &Path) -> BTreeMap<String, String> {
    let mut pending = vec![root.to_path_buf()];
    let mut files = BTreeMap::new();
    while let Some(directory) = pending.pop() {
        let mut entries = fs::read_dir(&directory)
            .expect("registered CPU lane source")
            .map(|entry| entry.expect("registered source entry"))
            .collect::<Vec<_>>();
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let kind = entry.file_type().expect("source kind");
            assert!(!kind.is_symlink());
            if kind.is_dir() {
                pending.push(entry.path());
            } else if entry.path().extension().and_then(|value| value.to_str()) == Some("rs") {
                let relative = entry
                    .path()
                    .strip_prefix(root)
                    .expect("source remains below lane")
                    .to_string_lossy()
                    .replace('\\', "/");
                files.insert(
                    relative,
                    fs::read_to_string(entry.path()).expect("UTF-8 source"),
                );
            }
        }
    }
    files
}
