use fenestra_ui::native::{
    ImeEvent, Key, KeyState, KeyboardInput, Modifiers, WindowContent, WindowEvent,
};

use super::{EvidenceMilestone, InspectorAction, NativeApplication};
use crate::evidence::{EvidenceResult, verify_artifact};

#[test]
fn repeated_insertions_preserve_unique_keys() {
    let mut application = NativeApplication::new(false).expect("application initializes");
    for _ in 0..3 {
        application
            .insert_tile()
            .expect("each press inserts a tile");
    }
    let frame = application
        .inspector
        .observe()
        .expect("frame is observable");
    assert_eq!(frame.keyed_keys(), [10, 20, 30, 40, 50]);
}

#[test]
fn native_insertion_and_resize_preserve_evidence_sequence() {
    let mut application = NativeApplication::new(true).expect("application initializes");
    application.presented().expect("initial presentation");
    application
        .event(WindowEvent::PointerMoved { x: 4, y: 3 })
        .expect("pointer move");
    assert!(application.inspector.hovered().is_some());
    application
        .event(WindowEvent::PointerPressed)
        .expect("pointer press");
    assert_eq!(
        application.inspector.selected(),
        application.inspector.hovered()
    );

    application.insert_tile().expect("native insertion");
    assert_eq!(
        application
            .inspector
            .observe()
            .expect("inserted frame")
            .keyed_keys(),
        [10, 20, 30]
    );
    application.presented().expect("mutation presentation");
    application.resize(0, 0).expect("minimize during evidence");
    assert_eq!(
        application
            .evidence
            .as_ref()
            .expect("evidence enabled")
            .next_required(),
        Some(EvidenceMilestone::Resize)
    );
    application
        .resize(224, 160)
        .expect("restore during evidence");
    application.presented().expect("resize presentation");
    application
        .event(WindowEvent::CloseRequested)
        .expect("close evidence");
    let bytes = application.output.take().expect("complete evidence");
    let verified = verify_artifact(&bytes).expect("independent verification");
    assert_eq!(verified.result(), EvidenceResult::Pass);
    assert_eq!(verified.final_generation(), Some(3));
}

#[test]
fn native_construction_preserves_the_supplied_inspector() {
    let mut inspector = crate::LayoutInspector::new().expect("application initializes");
    inspector
        .dispatch(InspectorAction::Resize {
            width: 240,
            height: 160,
        })
        .expect("custom viewport");
    inspector
        .dispatch(InspectorAction::InsertTile { key: 30 })
        .expect("custom content");
    let expected = inspector.observe().expect("custom frame");

    let mut application = NativeApplication::from_inspector(inspector, false);

    assert_eq!(
        application.inspector.observe().expect("native frame"),
        expected
    );
    application
        .insert_tile()
        .expect("insert after supplied content");
    assert_eq!(
        application
            .inspector
            .observe()
            .expect("updated frame")
            .keyed_keys(),
        [10, 20, 30, 40]
    );
}

#[test]
fn adapter_frames_preserve_the_inspector_pixels_and_dimensions() {
    let application = NativeApplication::new(false).expect("application initializes");
    let expected = application
        .inspector
        .reference_raster()
        .expect("inspector frame");
    let actual = application.frame().expect("native frame");
    assert_eq!(actual.size().width(), expected.viewport().width() as u32);
    assert_eq!(actual.size().height(), expected.viewport().height() as u32);
    assert_eq!(actual.bytes(), expected.bytes());
}

#[test]
fn adapter_space_events_keep_successive_keys_and_close_without_evidence() {
    let mut application = NativeApplication::new(false).expect("application initializes");
    application
        .event(WindowEvent::SpacePressed)
        .expect("space inserts");
    application
        .event(WindowEvent::SpacePressed)
        .expect("space inserts again");
    assert_eq!(
        application.inspector.observe().expect("frame").keyed_keys(),
        [10, 20, 30, 40]
    );
    application
        .event(WindowEvent::CloseRequested)
        .expect("normal close");
    assert!(application.output.is_none());
}

#[test]
fn owned_keyboard_and_ime_do_not_duplicate_legacy_inspector_actions() {
    let mut application = NativeApplication::new(false).unwrap();
    let initial = application.inspector.observe().unwrap();
    for event in [
        WindowEvent::KeyboardInput(KeyboardInput {
            key: Key::Space,
            state: KeyState::Pressed,
            modifiers: Modifiers::default(),
            repeat: false,
            text: Some(" ".into()),
            is_synthetic: false,
        }),
        WindowEvent::Ime(ImeEvent::Commit(" ".into())),
        WindowEvent::Focused(false),
        WindowEvent::ModifiersChanged(Modifiers::default()),
    ] {
        application.event(event).unwrap();
    }
    assert_eq!(application.inspector.observe().unwrap(), initial);
    application.event(WindowEvent::SpacePressed).unwrap();
    assert_eq!(
        application.inspector.observe().unwrap().keyed_keys(),
        [10, 20, 30]
    );
}
