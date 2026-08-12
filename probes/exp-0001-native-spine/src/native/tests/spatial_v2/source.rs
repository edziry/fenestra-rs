use std::fs;
use std::path::{Path, PathBuf};

const FORBIDDEN_TREE: &[&str] = &[
    "fenestra_ui_testkit",
    "fenestra_ui_authoring",
    "fenestra_ui_layout",
    "CommittedRuntimeSnapshot",
    "RuntimeSpatialViewV2",
    "RuntimeSpatialBuildViewV2",
    "SpatialResolvedSnapshotV2",
];

const FORBIDDEN_PORT: &[&str] = &[
    "FrameWork",
    "UiScheduler",
    "LogicalTree",
    "NodeId",
    "winit",
    "softbuffer",
    "vello",
    "tiny_skia",
    "wgpu",
];

#[test]
fn spatial_v2_production_boundary_has_no_logical_layout_testkit_or_candidate_vocabulary() {
    let root = workspace_root().join("probes/exp-0001-native-spine/src/native/spatial_v2");
    let files = rust_files(&root);
    assert!(
        !files.is_empty(),
        "spatial V2 production module should exist"
    );
    let mut port_files = 0;
    for path in files {
        let source = read(&path);
        for forbidden in FORBIDDEN_TREE {
            assert!(
                !source.contains(forbidden),
                "{} contains forbidden boundary vocabulary {forbidden}",
                path.display()
            );
        }
        if source.contains("trait SpatialPresenterPortV2") {
            port_files += 1;
            for forbidden in FORBIDDEN_PORT {
                assert!(
                    !source.contains(forbidden),
                    "{} leaks {forbidden} above the presenter port",
                    path.display()
                );
            }
        }
    }
    assert_eq!(
        port_files, 1,
        "one private semantic port should be declared"
    );
}

#[test]
fn public_probe_surface_and_v1_native_lane_remain_byte_stable_controls() {
    let root = workspace_root();
    let library = read(&root.join("probes/exp-0001-native-spine/src/lib.rs"));
    assert_eq!(library.matches("pub fn run_native_probe_v1").count(), 1);
    assert!(!library.contains("pub fn run_native_probe_v2"));
    assert!(!library.contains("pub use spatial_v2"));

    let main = read(&root.join("probes/exp-0001-native-spine/src/main.rs"));
    assert_eq!(main.matches("run_native_probe_v1").count(), 2);
    assert!(!main.contains("spatial_v2"));

    let artifact = root.join("probes/exp-0001-native-spine/tests/artifacts/fedora-wayland-v1.txt");
    assert_eq!(
        fnv64(&fs::read(artifact).expect("V1 artifact should exist")),
        0xac23_7a8d_c844_3172
    );
}

fn rust_files(root: &Path) -> Vec<PathBuf> {
    let mut pending = vec![root.to_path_buf()];
    let mut files = Vec::new();
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(&directory)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", directory.display()))
        {
            let path = entry
                .expect("directory entry should remain readable")
                .path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                files.push(path);
            }
        }
    }
    files.sort();
    files
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("native probe should remain below workspace root")
        .to_path_buf()
}

fn read(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}

fn fnv64(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}
