use std::fs;
use std::process::Command;

use fenestra_layout_inspector::evidence::{
    EvidenceError, EvidenceMilestone, EvidenceResult, LayoutInspectorEvidence, verify_artifact,
};
use fenestra_layout_inspector::{InspectorAction, LayoutInspector};

#[test]
fn evidence_builder_emits_a_verified_native_sequence() {
    let mut inspector = LayoutInspector::new().expect("the authored app should initialize");
    let mut evidence = LayoutInspectorEvidence::new();

    let initial = inspector.observe().expect("initial frame");
    evidence
        .record_initial(&initial)
        .expect("initial presentation");

    inspector
        .dispatch(InspectorAction::PointerMove { x: 4, y: 3 })
        .expect("pointer move");
    let moved = inspector.observe().expect("hover frame");
    evidence
        .record_pointer_move(4, 3, &moved)
        .expect("pointer hit");

    inspector
        .dispatch(InspectorAction::PointerPress)
        .expect("pointer press");
    let selected = inspector.observe().expect("selection frame");
    evidence.record_pointer_press(&selected).expect("selection");

    inspector
        .dispatch(InspectorAction::InsertTile { key: 30 })
        .expect("keyed insertion");
    let inserted = inspector.observe().expect("keyed frame");
    evidence
        .record_keyed_insert(30, &inserted)
        .expect("keyed insertion record");
    evidence
        .record_mutation_present(&inserted)
        .expect("mutation presentation");

    inspector
        .dispatch(InspectorAction::Resize {
            width: 224,
            height: 160,
        })
        .expect("resize");
    let resized = inspector.observe().expect("resized frame");
    evidence.record_resize(&resized).expect("resize record");
    evidence
        .record_resize_present(&resized)
        .expect("resize presentation");
    evidence.record_close().expect("close");

    let bytes = evidence.finish().expect("verified evidence");
    let verified = verify_artifact(&bytes).expect("independent verification");
    assert_eq!(verified.result(), EvidenceResult::Pass);
    assert_eq!(verified.record_count(), 10);
    assert_eq!(verified.final_generation(), Some(3));
    assert_eq!(verified.milestones(), EvidenceMilestone::ALL.as_slice());
    assert_eq!(
        std::str::from_utf8(&bytes).expect("ASCII evidence"),
        concat!(
            "fenestra-layout-inspector|artifact=1|wu=0015\n",
            "event|milestone=initial-present|generation=0|viewport=192x128|nodes=8|keys=10,20|hover=0|selected=0|raster-bytes=98304\n",
            "event|milestone=pointer-move|x=4|y=3|hit=1|generation=0\n",
            "event|milestone=pointer-press|generation=1|selected=1\n",
            "event|milestone=keyed-insert|key=30|generation=2|nodes=9|keys=10,20,30\n",
            "event|milestone=mutation-present|generation=2|viewport=192x128|raster-bytes=98304\n",
            "event|milestone=resize|viewport=224x160\n",
            "event|milestone=resize-present|generation=3|viewport=224x160|raster-bytes=143360\n",
            "event|milestone=close\n",
            "result|kind=pass|reason=complete\n",
        )
    );

    let path = std::env::temp_dir().join(format!(
        "fenestra-wu-0015-evidence-{}.txt",
        std::process::id()
    ));
    fs::write(&path, &bytes).expect("write evidence fixture");
    let output = Command::new(env!("CARGO_BIN_EXE_fenestra-layout-inspector-verify"))
        .arg(&path)
        .output()
        .expect("run standalone verifier");
    fs::remove_file(path).expect("remove evidence fixture");
    assert!(output.status.success());
    assert_eq!(
        output.stdout,
        format!("pass|records=10|bytes={}|generation=3\n", bytes.len()).as_bytes()
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn verifier_rejects_reordered_or_unbounded_evidence() {
    let mut inspector = LayoutInspector::new().expect("the authored app should initialize");
    let mut evidence = LayoutInspectorEvidence::new();
    evidence
        .record_initial(&inspector.observe().expect("initial frame"))
        .expect("initial presentation");
    let error = evidence
        .finish()
        .expect_err("an incomplete sequence must not pass");
    assert_eq!(error, EvidenceError::Incomplete);

    inspector
        .dispatch(InspectorAction::PointerMove { x: 4, y: 3 })
        .expect("pointer move");
    let moved = inspector.observe().expect("hover frame");
    let mut complete = LayoutInspectorEvidence::new();
    complete
        .record_initial(&inspector.observe().expect("initial frame"))
        .expect_err("a fresh initial frame is required by this fixture");
    let malformed = b"fenestra-layout-inspector|artifact=1|wu=0015\n";
    assert_eq!(verify_artifact(malformed), Err(EvidenceError::Grammar));
    assert!(moved.has_hover());
    assert_eq!(
        complete.next_required(),
        Some(EvidenceMilestone::InitialPresent)
    );
}

#[test]
fn committed_windows_artifact_is_still_independently_verified() {
    let bytes = include_bytes!("artifacts/windows-native-v1.txt");
    let verified = verify_artifact(bytes).expect("committed Windows artifact");
    assert_eq!(verified.result(), EvidenceResult::Pass);
    assert_eq!(verified.record_count(), 10);
    assert_eq!(verified.byte_count(), 608);
    assert_eq!(verified.final_generation(), Some(6));
}
