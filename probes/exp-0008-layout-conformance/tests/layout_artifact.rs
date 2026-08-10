#![forbid(unsafe_code)]

#[path = "layout_artifact/direct.rs"]
mod direct;
// This target reuses the encoder but not its dedicated negative-test accessors.
#[allow(dead_code)]
#[path = "layout_artifact/encode.rs"]
mod encode;
#[path = "layout_artifact/final/lines.rs"]
mod lines;
#[path = "layout_artifact/final/model.rs"]
mod model;
// This target reuses runtime model construction and rows, not its fault-control API.
#[allow(dead_code, unused_imports)]
#[path = "layout_artifact/runtime.rs"]
mod runtime;
#[path = "runtime_candidate/script.rs"]
mod script;
#[path = "runtime_candidate/support.rs"]
mod support;

use direct::{direct_lines_v1, registered_direct_artifact_v1};
use encode::{REGISTERED_LAYOUT_ARTIFACT_LIMITS_V1, encode_layout_artifact_v1};
use lines::final_artifact_lines_v1;
use model::{
    FinalArtifactV1, FinalClassificationV1, fault_row_v1, inject_metadata_fault_v1,
    registered_final_artifact_v1, registered_metadata_faults_v1,
};
use runtime::{build_runtime_artifact_v1, runtime_lines_v1};
use script::{run_candidate_lane_v1, run_reference_lane_v1};

const GOLDEN: &[u8] = include_bytes!("artifacts/layout-conformance-v1.txt");
const METADATA_ROWS: [&str; 15] = [
    "fenestra-layout-conformance|1",
    "work-unit|WU-0011|experiment=EXP-0008",
    "formats|layout-contract=1|layout-corpus=1",
    "package|name=fenestra-ui-exp-0008-layout-conformance|version=0.1.1|edition=2024|publish=false",
    "package|name=fenestra-ui-layout|version=0.1.1|edition=2024|publish=false",
    "package|name=fenestra-ui-ir|version=0.1.1|edition=2024|publish=false",
    "package|name=fenestra-ui-runtime|version=0.1.1|edition=2024|publish=false",
    "package|name=fenestra-ui-testkit|version=0.1.1|edition=2024|publish=false",
    "candidate|name=taffy|version=0.13.0|role=disposable-probe|product-selected=false",
    "layout-limits|nodes=32|depth=8|children-per-node=16",
    "candidate-limits|input-scalar=4096|output-edge=524288",
    "artifact-limits|records=512|line-bytes=512|artifact-bytes=65536|priority=records,line-bytes,artifact-bytes",
    "candidate-features|default=false|std=true|taffy_tree=true|flexbox=true",
    "dependency-facts|candidate-scope=probe-normal-private|candidate-requirement=exact|active-transitives=arrayvec@0.7.8,slotmap@1.1.1,version_check@0.9.5|lock-only=taffy>serde,serde>serde_derive|runtime-acceptance=fenestra-ui-ir,fenestra-ui-runtime,fenestra-ui-testkit|runtime-acceptance-scope=dev-only|other-taffy-manifests=0|native=false|ffi=false",
    "artifact-counts|metadata=15|direct=286|runtime=109|closing=2|records=412|cases=23|inputs=120|outputs=120|steps=7|generations=7|geometry=38|hit=24|scene=38",
];

#[test]
fn canonical_layout_conformance_artifact_matches_the_versioned_golden() {
    assert_reused_section_seams();
    let first = build_registered_artifact();
    let second = build_registered_artifact();
    assert_eq!(first, second, "two fresh typed builds must match");
    assert_eq!(
        FinalClassificationV1::ALL,
        [
            FinalClassificationV1::Pass,
            FinalClassificationV1::Adapt,
            FinalClassificationV1::Stop,
        ]
    );

    let first_lines = final_artifact_lines_v1(&first);
    let second_lines = final_artifact_lines_v1(&second);
    assert_eq!(first_lines, second_lines, "two fresh row builds must match");
    assert_canonical_rows(&first_lines);

    let first_bytes = encode(&first_lines);
    let second_bytes = encode(&second_lines);
    assert_eq!(
        first_bytes, second_bytes,
        "two fresh byte builds must match"
    );
    assert_canonical_bytes(&first_bytes);
    assert_metadata_faults(&first, &first_lines, &first_bytes);

    assert!(GOLDEN.ends_with(b"\n"));
    assert!(!GOLDEN.ends_with(b"\n\n"));
    assert!(GOLDEN.iter().all(u8::is_ascii));
    assert!(
        first_bytes == GOLDEN,
        "canonical artifact differs from its versioned golden (actual-bytes={};golden-bytes={})",
        first_bytes.len(),
        GOLDEN.len()
    );
}

fn build_registered_artifact() -> FinalArtifactV1 {
    let reference = run_reference_lane_v1();
    let candidate = run_candidate_lane_v1();
    let runtime = build_runtime_artifact_v1(reference.transcript(), candidate.transcript());
    registered_final_artifact_v1(registered_direct_artifact_v1(), runtime)
}

fn assert_canonical_rows(lines: &[String]) {
    assert_eq!(lines.len(), 412);
    assert_eq!(&lines[..METADATA_ROWS.len()], METADATA_ROWS);
    assert_eq!(
        lines[15],
        "case|index=0|name=single-fixed-root|viewport=7,5|nodes=1"
    );
    assert_eq!(lines[300], "case-result|case=22|classification=pass");
    assert!(lines[301].starts_with("runtime|steps=7|"));
    assert_eq!(lines[409], "runtime-result|classification=pass");
    assert_eq!(lines[410], "artifact-result|classification=pass");
    assert_eq!(lines[411], "end");
    for (tag, count) in [
        ("case|", 23),
        ("case-input|", 120),
        ("case-output|", 120),
        ("case-result|", 23),
        ("runtime|", 1),
        ("runtime-generation|", 7),
        ("runtime-geometry|", 38),
        ("runtime-hit|", 24),
        ("runtime-scene|", 38),
        ("runtime-result|", 1),
        ("dependency-facts|", 1),
        ("artifact-result|", 1),
        ("end", 1),
    ] {
        assert_eq!(
            lines.iter().filter(|line| line.starts_with(tag)).count(),
            count,
            "wrong {tag} count"
        );
    }
}

fn assert_canonical_bytes(bytes: &[u8]) {
    assert!(bytes.ends_with(b"\n"));
    assert!(!bytes.ends_with(b"\n\n"));
    assert_eq!(bytes.iter().filter(|byte| **byte == b'\n').count(), 412);
    assert!(
        bytes
            .iter()
            .all(|byte| *byte == b'\n' || (0x20..=0x7e).contains(byte))
    );
    let source = std::str::from_utf8(bytes).expect("printable ASCII is UTF-8");
    for forbidden in [
        "/home/",
        "C:\\",
        "target/",
        "NodeId",
        "FragmentId",
        "DefaultKey",
        "runtime-id",
        "candidate-id",
        "pointer",
        "address",
        "0x",
        "Debug",
        "username=",
        "clock=",
        "timestamp=",
        "hostname=",
        "source=",
        "payload=",
    ] {
        assert!(!source.contains(forbidden), "forbidden token: {forbidden}");
    }
}

fn assert_metadata_faults(model: &FinalArtifactV1, lines: &[String], bytes: &[u8]) {
    let retained_model = model.clone();
    let retained_lines = lines.to_vec();
    let retained_bytes = bytes.to_vec();
    let faults = registered_metadata_faults_v1();
    assert_eq!(faults.len(), 35);
    for fault in faults {
        let faulted = inject_metadata_fault_v1(model, fault);
        assert_eq!(faulted.direct, model.direct, "{fault:?}: direct changed");
        assert_eq!(faulted.runtime, model.runtime, "{fault:?}: runtime changed");
        let faulted_lines = final_artifact_lines_v1(&faulted);
        let changed_rows = lines
            .iter()
            .zip(&faulted_lines)
            .enumerate()
            .filter_map(|(index, (left, right))| (left != right).then_some(index))
            .collect::<Vec<_>>();
        assert_eq!(changed_rows, vec![fault_row_v1(fault)], "{fault:?}");
        assert_ne!(encode(&faulted_lines), bytes, "{fault:?}: bytes unchanged");
        assert_eq!(model, &retained_model, "{fault:?}: model not atomic");
        assert_eq!(lines, retained_lines, "{fault:?}: rows not atomic");
        assert_eq!(bytes, retained_bytes, "{fault:?}: bytes not atomic");
    }
}

fn encode(lines: &[String]) -> Vec<u8> {
    encode_layout_artifact_v1(lines, REGISTERED_LAYOUT_ARTIFACT_LIMITS_V1)
        .expect("the registered 412-row artifact must fit its inclusive limits")
        .as_bytes()
        .to_vec()
}

fn assert_reused_section_seams() {
    direct::assert_direct_artifact_contract(|model| encode(&direct_lines_v1(model)));
    let reference = run_reference_lane_v1();
    let candidate = run_candidate_lane_v1();
    let runtime = build_runtime_artifact_v1(reference.transcript(), candidate.transcript());
    assert_eq!(runtime_lines_v1(&runtime).len(), 109);
}
