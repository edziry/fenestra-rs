use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::lanes::native_renderer::{
    NativeCandidateV2, NativeFaultKindV2, NativeObligationV2, NativeOutcomeV2,
    classify_native_run_v2, literal_native_run_v2, native_candidate_registry_v2, native_cases_v2,
    native_faults_v2, vello_native_run_v2,
};

#[test]
fn native_candidate_registry_and_profile_are_exact() {
    let registry = native_candidate_registry_v2();
    assert_eq!(registry.len(), 1);
    assert_eq!(registry[0].kind, NativeCandidateV2::Vello);
    assert_eq!(registry[0].name, "vello");
    assert_eq!(registry[0].version, "0.9.0");
    assert_eq!(registry[0].renderer_features, "wgpu");
    assert_eq!(registry[0].gpu_version, "29.0.3");
    assert_eq!(registry[0].gpu_features, "std,parking_lot,wgsl,vulkan,dx12");
    assert_eq!(
        registry[0].targets,
        "x86_64-unknown-linux-gnu:vulkan-wayland,x86_64-pc-windows-msvc:dx12-win32"
    );

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let manifest = fs::read_to_string(root.join("Cargo.toml")).expect("manifest");
    for declaration in [
        "native-renderer = [\"dep:vello\", \"dep:wgpu\"]",
        "vello = { version = \"=0.9.0\", default-features = false, features = [\"wgpu\"], optional = true }",
        "wgpu = { version = \"=29.0.3\", default-features = false, features = [\"std\", \"parking_lot\", \"wgsl\", \"vulkan\", \"dx12\"], optional = true }",
    ] {
        assert!(manifest.contains(declaration), "missing {declaration}");
    }
    let lock = fs::read_to_string(root.join("../../Cargo.lock")).expect("lock");
    for package in [
        "name = \"vello\"\nversion = \"0.9.0\"",
        "name = \"wgpu\"\nversion = \"29.0.3\"",
    ] {
        assert_eq!(lock.matches(package).count(), 1, "{package}");
    }
}

#[test]
fn native_cases_cover_the_closed_scene_and_protocol_obligations() {
    let cases = native_cases_v2();
    assert_eq!(
        cases.iter().map(|case| case.ordinal).collect::<Vec<_>>(),
        (0..cases.len() as u8).collect::<Vec<_>>()
    );
    assert_eq!(
        cases
            .iter()
            .flat_map(|case| case.obligations.iter().copied())
            .collect::<BTreeSet<_>>(),
        NativeObligationV2::ALL.into_iter().collect()
    );
    assert!(cases.iter().all(|case| case.width > 0 && case.height > 0));
    assert!(cases.iter().all(|case| !case.commands.is_empty()));
}

#[test]
fn vello_builds_two_fresh_exact_scenes_without_claiming_gpu_pixels() {
    let first_cases = native_cases_v2();
    let second_cases = native_cases_v2();
    assert_eq!(first_cases, second_cases);
    let literal = literal_native_run_v2(&first_cases).expect("literal native run");
    assert_eq!(
        literal_native_run_v2(&second_cases).expect("fresh literal native run"),
        literal
    );
    let first = vello_native_run_v2(&first_cases).expect("Vello scene build");
    let second = vello_native_run_v2(&second_cases).expect("fresh Vello scene build");
    assert_eq!(first, second);
    assert_eq!(first.records, literal.records);
    assert!(first.scene_fingerprint != 0);
    assert!(first.encoded_scene_bytes > 0);
    assert!(first.used_vello_scene);
    assert!(!first.executed_gpu);

    let classification = classify_native_run_v2(&literal, &first);
    assert_eq!(classification.candidate, NativeCandidateV2::Vello);
    assert_eq!(classification.outcome, NativeOutcomeV2::Stop);
    assert_eq!(classification.reason, "target-unavailable");
    assert_eq!(classification.first_mismatch, None);
}

#[test]
fn native_faults_are_typed_and_vello_preflights_them() {
    let faults = native_faults_v2();
    assert_eq!(
        faults.iter().map(|fault| fault.kind).collect::<Vec<_>>(),
        vec![
            NativeFaultKindV2::MissingCapability,
            NativeFaultKindV2::ZeroSurface,
            NativeFaultKindV2::ResourceLimit,
            NativeFaultKindV2::AdapterUnavailable,
            NativeFaultKindV2::SurfaceLost,
        ]
    );
    assert!(faults.iter().all(|fault| fault.literal));
    assert!(faults.iter().all(|fault| fault.vello));
}

#[test]
fn native_oracle_and_candidate_sources_are_private_and_separate() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/lanes/native_renderer");
    let files = rust_files(&root);
    assert_eq!(
        files.keys().cloned().collect::<BTreeSet<_>>(),
        BTreeSet::from([
            "candidates/mod.rs".to_owned(),
            "candidates/vello.rs".to_owned(),
            "compare.rs".to_owned(),
            "faults.rs".to_owned(),
            "input.rs".to_owned(),
            "mod.rs".to_owned(),
            "oracle.rs".to_owned(),
            "types.rs".to_owned(),
        ])
    );
    let oracle = &files["oracle.rs"];
    for forbidden in ["use vello", "use wgpu", "crate::baseline", "candidate"] {
        assert!(!oracle.contains(forbidden), "oracle contains {forbidden}");
    }
    let adapter = &files["candidates/vello.rs"];
    for required in [
        "vello::Scene",
        ".fill(",
        ".push_clip_layer(",
        ".draw_image(",
    ] {
        assert!(adapter.contains(required), "adapter misses {required}");
    }
    for (path, source) in &files {
        assert!(!source.contains("pub "), "public item in {path}");
        assert!(!source.contains("unsafe"), "unsafe code in {path}");
        if !path.starts_with("candidates/") {
            assert!(!source.contains("use vello"));
            assert!(!source.contains("use wgpu"));
        }
    }
}

fn rust_files(root: &Path) -> BTreeMap<String, String> {
    let mut pending = vec![root.to_path_buf()];
    let mut files = BTreeMap::new();
    while let Some(directory) = pending.pop() {
        let mut entries = fs::read_dir(&directory)
            .expect("registered native lane source")
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
