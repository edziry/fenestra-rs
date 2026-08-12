use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::lanes::image_resource::{
    ImageCandidateV2, ImageFaultKindV2, ImageObligationV2, ImageOutcomeV2, classify_image_run_v2,
    image_candidate_registry_v2, image_cases_v2, image_crate_run_v2, image_faults_v2,
    literal_image_run_v2, png_image_run_v2,
};

#[test]
fn image_candidate_registry_and_profiles_are_exact() {
    assert_eq!(
        image_candidate_registry_v2()
            .iter()
            .map(|candidate| (
                candidate.kind,
                candidate.name,
                candidate.version,
                candidate.features,
            ))
            .collect::<Vec<_>>(),
        vec![
            (ImageCandidateV2::Png, "png", "0.18.1", "-"),
            (ImageCandidateV2::Image, "image", "0.25.10", "png"),
        ]
    );
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let manifest = fs::read_to_string(root.join("Cargo.toml")).expect("manifest");
    for declaration in [
        "image-resource = [\"dep:image\", \"dep:png\"]",
        "png = { version = \"=0.18.1\", default-features = false, optional = true }",
        "image = { version = \"=0.25.10\", default-features = false, features = [\"png\"], optional = true }",
    ] {
        assert!(manifest.contains(declaration), "missing {declaration}");
    }
    let lock = fs::read_to_string(root.join("../../Cargo.lock")).expect("lock");
    for package in [
        "name = \"png\"\nversion = \"0.18.1\"",
        "name = \"image\"\nversion = \"0.25.10\"",
    ] {
        assert_eq!(lock.matches(package).count(), 1, "{package}");
    }
}

#[test]
fn image_cases_cover_the_closed_resource_obligations() {
    let cases = image_cases_v2();
    assert_eq!(
        cases.iter().map(|case| case.ordinal).collect::<Vec<_>>(),
        (0..cases.len() as u8).collect::<Vec<_>>()
    );
    assert_eq!(
        cases
            .iter()
            .flat_map(|case| case.obligations.iter().copied())
            .collect::<BTreeSet<_>>(),
        ImageObligationV2::ALL.into_iter().collect()
    );
    assert!(
        cases
            .iter()
            .all(|case| case.png_bytes.starts_with(b"\x89PNG\r\n\x1a\n"))
    );
    assert!(cases.iter().all(|case| case.width > 0 && case.height > 0));
}

#[test]
fn image_candidates_are_fresh_deterministic_and_exactly_classified() {
    let first_cases = image_cases_v2();
    let second_cases = image_cases_v2();
    assert_eq!(first_cases, second_cases);
    let literal = literal_image_run_v2(&first_cases).expect("literal image run");
    assert_eq!(
        literal_image_run_v2(&second_cases).expect("fresh literal image run"),
        literal
    );
    for record in &literal.records {
        assert_eq!(record.stride, record.width * 4);
        assert_eq!(
            record.rgba8.len(),
            record.stride as usize * record.height as usize
        );
    }

    for (candidate, expected, first, second) in [
        (
            ImageCandidateV2::Png,
            (ImageOutcomeV2::Adapt, "orientation-normalization"),
            png_image_run_v2(&first_cases).expect("PNG run"),
            png_image_run_v2(&second_cases).expect("fresh PNG run"),
        ),
        (
            ImageCandidateV2::Image,
            (ImageOutcomeV2::Pass, "-"),
            image_crate_run_v2(&first_cases).expect("Image run"),
            image_crate_run_v2(&second_cases).expect("fresh Image run"),
        ),
    ] {
        assert_eq!(first, second, "{candidate:?} is nondeterministic");
        let classification = classify_image_run_v2(&literal, &first);
        assert_eq!(classification.candidate, candidate);
        assert_eq!((classification.outcome, classification.reason), expected);
        assert_eq!(classification.first_mismatch, None);
        assert_eq!(first.records, literal.records);
    }
}

#[test]
fn image_faults_are_typed_and_candidate_adapters_preflight_them() {
    let faults = image_faults_v2();
    assert_eq!(
        faults.iter().map(|fault| fault.kind).collect::<Vec<_>>(),
        vec![
            ImageFaultKindV2::MalformedSignature,
            ImageFaultKindV2::DimensionLimit,
            ImageFaultKindV2::StrideOverflow,
            ImageFaultKindV2::ByteBomb,
            ImageFaultKindV2::UnsupportedColor,
            ImageFaultKindV2::TruncatedData,
        ]
    );
    assert!(faults.iter().all(|fault| fault.literal));
    assert!(faults.iter().all(|fault| fault.png));
    assert!(faults.iter().all(|fault| fault.image));
}

#[test]
fn image_oracle_and_candidate_sources_are_private_and_separate() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/lanes/image_resource");
    let files = rust_files(&root);
    assert_eq!(
        files.keys().cloned().collect::<BTreeSet<_>>(),
        BTreeSet::from([
            "candidates/image.rs".to_owned(),
            "candidates/mod.rs".to_owned(),
            "candidates/png.rs".to_owned(),
            "compare.rs".to_owned(),
            "faults.rs".to_owned(),
            "input.rs".to_owned(),
            "mod.rs".to_owned(),
            "oracle.rs".to_owned(),
            "png_bytes.rs".to_owned(),
            "types.rs".to_owned(),
        ])
    );
    let oracle = &files["oracle.rs"];
    for forbidden in ["use image", "use png", "crate::baseline", "candidate"] {
        assert!(!oracle.contains(forbidden), "oracle contains {forbidden}");
    }
    for (path, source) in &files {
        assert!(!source.contains("pub "), "public item in {path}");
        assert!(!source.contains("unsafe"), "unsafe code in {path}");
        if !path.starts_with("candidates/") {
            assert!(!source.contains("use image"));
            assert!(!source.contains("use png"));
        }
    }
}

fn rust_files(root: &Path) -> BTreeMap<String, String> {
    let mut pending = vec![root.to_path_buf()];
    let mut files = BTreeMap::new();
    while let Some(directory) = pending.pop() {
        let mut entries = fs::read_dir(&directory)
            .expect("registered image lane source")
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
