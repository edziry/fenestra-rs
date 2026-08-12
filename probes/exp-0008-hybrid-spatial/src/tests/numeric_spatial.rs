use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::lanes::numeric_spatial::{
    NumericCandidateV2, NumericFaultKindV2, NumericOutcomeV2, euclid_numeric_run_v2,
    fixed_numeric_run_v2, kurbo_numeric_run_v2, literal_numeric_run_v2,
    numeric_candidate_registry_v2, numeric_faults_v2, numeric_inputs_v2,
};

#[test]
fn numeric_candidate_registry_is_exact_and_closed() {
    let registry = numeric_candidate_registry_v2();
    assert_eq!(registry.len(), 3);
    assert_eq!(
        registry
            .iter()
            .map(|candidate| (
                candidate.kind,
                candidate.name,
                candidate.version,
                candidate.features,
            ))
            .collect::<Vec<_>>(),
        vec![
            (NumericCandidateV2::Euclid, "euclid", "0.22.14", "std"),
            (NumericCandidateV2::Kurbo, "kurbo", "0.13.1", "std"),
            (NumericCandidateV2::Fixed, "fixed", "1.30.0", "-"),
        ]
    );
    assert!(registry.iter().all(|candidate| {
        candidate.outcome == NumericOutcomeV2::Pass && candidate.reason == "-"
    }));
}

#[test]
fn numeric_manifest_and_lock_use_only_the_exact_candidate_profiles() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let manifest =
        fs::read_to_string(root.join("Cargo.toml")).expect("manifest should be readable");
    for declaration in [
        "numeric-spatial = [\"dep:euclid\", \"dep:fixed\", \"dep:kurbo\"]",
        "euclid = { version = \"=0.22.14\", default-features = false, features = [\"std\"], optional = true }",
        "fixed = { version = \"=1.30.0\", default-features = false, optional = true }",
        "kurbo = { version = \"=0.13.1\", default-features = false, features = [\"std\"], optional = true }",
    ] {
        assert!(manifest.contains(declaration), "missing {declaration}");
    }
    let lock = fs::read_to_string(root.join("../../Cargo.lock")).expect("lock should be readable");
    for package in [
        "name = \"euclid\"\nversion = \"0.22.14\"",
        "name = \"fixed\"\nversion = \"1.30.0\"",
        "name = \"kurbo\"\nversion = \"0.13.1\"",
    ] {
        assert_eq!(lock.matches(package).count(), 1, "{package}");
    }
}

#[test]
fn every_numeric_candidate_matches_two_fresh_literal_reconstructions() {
    let first_inputs = numeric_inputs_v2();
    let second_inputs = numeric_inputs_v2();
    assert_eq!(first_inputs, second_inputs);
    assert!(!first_inputs.is_empty());

    let first_literal = literal_numeric_run_v2(&first_inputs).expect("literal run should resolve");
    let second_literal =
        literal_numeric_run_v2(&second_inputs).expect("fresh literal run should resolve");
    assert_eq!(first_literal, second_literal);
    assert!(first_literal.proves_endpoints);
    assert!(first_literal.proves_rounding);
    assert!(first_literal.proves_composition);
    assert!(first_literal.proves_inverse);
    assert!(first_literal.proves_transform_origin);

    for run in [
        euclid_numeric_run_v2(&first_inputs),
        kurbo_numeric_run_v2(&first_inputs),
        fixed_numeric_run_v2(&first_inputs),
    ] {
        let run = run.expect("registered numeric input should be supported");
        assert_eq!(run, first_literal);
        assert!(run.typed_space_witnesses >= 4);
    }
    assert_eq!(
        euclid_numeric_run_v2(&second_inputs).expect("fresh Euclid run"),
        first_literal
    );
    assert_eq!(
        kurbo_numeric_run_v2(&second_inputs).expect("fresh Kurbo run"),
        first_literal
    );
    assert_eq!(
        fixed_numeric_run_v2(&second_inputs).expect("fresh Fixed run"),
        first_literal
    );
}

#[test]
fn numeric_faults_are_typed_complete_and_candidate_neutral() {
    let faults = numeric_faults_v2();
    assert_eq!(
        faults.iter().map(|fault| fault.kind).collect::<Vec<_>>(),
        vec![
            NumericFaultKindV2::BelowMinimum,
            NumericFaultKindV2::AboveMaximum,
            NumericFaultKindV2::CompositionOverflow,
            NumericFaultKindV2::SingularInverse,
            NumericFaultKindV2::NonFiniteCandidate,
        ]
    );
    assert!(faults.iter().all(|fault| fault.detected_by_literal));
    assert!(faults.iter().all(|fault| fault.detected_by_euclid));
    assert!(faults.iter().all(|fault| fault.detected_by_kurbo));
    assert!(faults.iter().all(|fault| fault.detected_by_fixed));
}

#[test]
fn numeric_oracle_and_candidate_sources_are_separate_and_private() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/lanes/numeric_spatial");
    let files = rust_files(&root);
    assert!(!files.is_empty());
    let expected = BTreeSet::from([
        "candidates/euclid.rs".to_owned(),
        "candidates/fixed.rs".to_owned(),
        "candidates/kurbo.rs".to_owned(),
        "candidates/mod.rs".to_owned(),
        "faults.rs".to_owned(),
        "input.rs".to_owned(),
        "mod.rs".to_owned(),
        "oracle.rs".to_owned(),
        "types.rs".to_owned(),
    ]);
    assert_eq!(files.keys().cloned().collect::<BTreeSet<_>>(), expected);

    let oracle = &files["oracle.rs"];
    for forbidden in ["euclid", "kurbo", "fixed::", "crate::baseline", "candidate"] {
        assert!(!oracle.contains(forbidden), "oracle contains {forbidden}");
    }
    for (path, source) in &files {
        assert!(
            !source.contains("pub "),
            "public surface leaked from {path}"
        );
        assert!(!source.contains("unsafe"), "unsafe code in {path}");
        if !path.starts_with("candidates/") {
            for forbidden in ["use euclid", "use kurbo", "use fixed"] {
                assert!(
                    !source.contains(forbidden),
                    "{forbidden} leaked into {path}"
                );
            }
        }
    }
}

fn rust_files(root: &Path) -> std::collections::BTreeMap<String, String> {
    let mut pending = vec![root.to_path_buf()];
    let mut files = std::collections::BTreeMap::new();
    while let Some(directory) = pending.pop() {
        let mut entries = fs::read_dir(&directory)
            .expect("registered numeric source directory should exist")
            .map(|entry| entry.expect("registered source entry should be readable"))
            .collect::<Vec<_>>();
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let kind = entry.file_type().expect("source kind should be readable");
            assert!(!kind.is_symlink());
            if kind.is_dir() {
                pending.push(entry.path());
            } else if entry.path().extension().and_then(|value| value.to_str()) == Some("rs") {
                let relative = entry
                    .path()
                    .strip_prefix(root)
                    .expect("source should remain below numeric root")
                    .to_string_lossy()
                    .replace('\\', "/");
                files.insert(
                    relative,
                    fs::read_to_string(entry.path()).expect("source should be UTF-8"),
                );
            }
        }
    }
    files
}
