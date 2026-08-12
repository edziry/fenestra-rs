use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::lanes::path_hit::{
    PathHitCandidateV2, PathHitFaultKindV2, PathHitObligationV2, PathHitOutcomeV2,
    kurbo_path_hit_run_v2, literal_path_hit_run_v2, lyon_path_hit_run_v2,
    path_hit_candidate_registry_v2, path_hit_cases_v2, path_hit_faults_v2,
};

#[test]
fn path_hit_candidate_registry_and_profiles_are_exact() {
    let registry = path_hit_candidate_registry_v2();
    assert_eq!(registry.len(), 2);
    assert_eq!(
        registry
            .iter()
            .map(|candidate| (
                candidate.kind,
                candidate.name,
                candidate.version,
                candidate.features,
                candidate.outcome,
                candidate.reason,
            ))
            .collect::<Vec<_>>(),
        vec![
            (
                PathHitCandidateV2::Kurbo,
                "kurbo",
                "0.13.1",
                "std",
                PathHitOutcomeV2::Adapt,
                "edge-rounding",
            ),
            (
                PathHitCandidateV2::Lyon,
                "lyon-tessellation",
                "1.0.20",
                "std",
                PathHitOutcomeV2::Pass,
                "-",
            ),
        ]
    );

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let manifest = fs::read_to_string(root.join("Cargo.toml")).expect("manifest");
    for declaration in [
        "path-hit = [\"dep:kurbo\", \"dep:lyon_tessellation\"]",
        "lyon_tessellation = { version = \"=1.0.20\", default-features = false, features = [\"std\"], optional = true }",
    ] {
        assert!(manifest.contains(declaration), "missing {declaration}");
    }
    let lock = fs::read_to_string(root.join("../../Cargo.lock")).expect("lock");
    assert_eq!(
        lock.matches("name = \"lyon_tessellation\"\nversion = \"1.0.20\"")
            .count(),
        1
    );
}

#[test]
fn registered_path_hit_cases_cover_every_required_obligation() {
    let cases = path_hit_cases_v2();
    assert!(cases.len() >= 10);
    assert_eq!(
        cases.iter().map(|case| case.ordinal).collect::<Vec<_>>(),
        (0..cases.len() as u8).collect::<Vec<_>>()
    );
    let observed = cases
        .iter()
        .flat_map(|case| case.obligations.iter().copied())
        .collect::<BTreeSet<_>>();
    assert_eq!(observed, PathHitObligationV2::ALL.into_iter().collect());
    assert!(cases.iter().all(|case| !case.queries.is_empty()));
    assert!(cases.iter().any(|case| case.layers.len() > 1));
    assert!(cases.iter().any(|case| case.clip.is_some()));
}

#[test]
fn kurbo_and_lyon_match_two_fresh_literal_path_hit_runs() {
    let first_cases = path_hit_cases_v2();
    let second_cases = path_hit_cases_v2();
    assert_eq!(first_cases, second_cases);

    let first_literal = literal_path_hit_run_v2(&first_cases).expect("literal path run");
    let second_literal = literal_path_hit_run_v2(&second_cases).expect("fresh literal path run");
    assert_eq!(first_literal, second_literal);
    assert!(first_literal.triangle_witnesses > 0);
    assert!(first_literal.reverse_painter_queries > 0);
    assert!(first_literal.nonrectangular_aabb_misses > 0);

    let first_kurbo = kurbo_path_hit_run_v2(&first_cases).expect("Kurbo path run");
    let first_lyon = lyon_path_hit_run_v2(&first_cases).expect("Lyon path run");
    assert_eq!(first_kurbo, first_literal);
    assert_eq!(first_lyon, first_literal);
    assert_eq!(
        kurbo_path_hit_run_v2(&second_cases).expect("fresh Kurbo path run"),
        first_literal
    );
    assert_eq!(
        lyon_path_hit_run_v2(&second_cases).expect("fresh Lyon path run"),
        first_literal
    );
}

#[test]
fn path_hit_faults_are_closed_typed_and_detected_by_both_candidates() {
    let faults = path_hit_faults_v2();
    assert_eq!(
        faults.iter().map(|fault| fault.kind).collect::<Vec<_>>(),
        vec![
            PathHitFaultKindV2::MissingMove,
            PathHitFaultKindV2::OpenFillSubpath,
            PathHitFaultKindV2::NonFiniteCoordinate,
            PathHitFaultKindV2::TessellationLimit,
            PathHitFaultKindV2::InvalidStrokeWidth,
        ]
    );
    assert!(faults.iter().all(|fault| fault.literal));
    assert!(faults.iter().all(|fault| fault.kurbo));
    assert!(faults.iter().all(|fault| fault.lyon));
}

#[test]
fn path_hit_oracle_and_candidate_sources_are_isolated() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/lanes/path_hit");
    let files = rust_files(&root);
    let expected = BTreeSet::from([
        "candidates/kurbo.rs".to_owned(),
        "candidates/lyon.rs".to_owned(),
        "candidates/mod.rs".to_owned(),
        "faults.rs".to_owned(),
        "input.rs".to_owned(),
        "mod.rs".to_owned(),
        "oracle.rs".to_owned(),
        "types.rs".to_owned(),
    ]);
    assert_eq!(files.keys().cloned().collect::<BTreeSet<_>>(), expected);
    let oracle = &files["oracle.rs"];
    for forbidden in ["kurbo", "lyon", "crate::baseline", "candidate"] {
        assert!(!oracle.contains(forbidden), "oracle contains {forbidden}");
    }
    for (path, source) in &files {
        assert!(!source.contains("pub "), "public item in {path}");
        assert!(!source.contains("unsafe"), "unsafe code in {path}");
        if !path.starts_with("candidates/") {
            assert!(!source.contains("use kurbo"), "Kurbo leaked into {path}");
            assert!(!source.contains("use lyon"), "Lyon leaked into {path}");
        }
    }
}

fn rust_files(root: &Path) -> BTreeMap<String, String> {
    let mut pending = vec![root.to_path_buf()];
    let mut files = BTreeMap::new();
    while let Some(directory) = pending.pop() {
        let mut entries = fs::read_dir(&directory)
            .expect("registered path-hit source directory")
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
