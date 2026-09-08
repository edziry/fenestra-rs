use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{DeviceId, ElementState, MouseButton, WindowEvent};

use super::{EvidenceMilestone, InspectorAction, NativeApplication, input::requests_redraw};
use crate::evidence::{EvidenceResult, verify_artifact};

#[test]
fn presenting_a_frame_does_not_schedule_another_frame() {
    assert!(!requests_redraw(&WindowEvent::RedrawRequested));
}

#[test]
fn unrelated_window_events_do_not_schedule_frames() {
    for event in [
        WindowEvent::Focused(true),
        WindowEvent::Occluded(false),
        WindowEvent::CloseRequested,
    ] {
        assert!(!requests_redraw(&event), "unexpected redraw for {event:?}");
    }
}

#[test]
fn pointer_interactions_schedule_frames() {
    assert!(requests_redraw(&WindowEvent::CursorMoved {
        device_id: DeviceId::dummy(),
        position: PhysicalPosition::new(4.0, 3.0),
    }));
    for (state, button, redraw) in [
        (ElementState::Pressed, MouseButton::Left, true),
        (ElementState::Released, MouseButton::Left, false),
        (ElementState::Pressed, MouseButton::Right, false),
    ] {
        assert_eq!(
            requests_redraw(&WindowEvent::MouseInput {
                device_id: DeviceId::dummy(),
                state,
                button,
            }),
            redraw
        );
    }
}

#[test]
fn repeated_insertions_preserve_unique_keys() {
    let mut application = NativeApplication::new(false, false).expect("application initializes");
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
fn minimized_window_skips_presentation_until_restored() {
    let mut application = NativeApplication::new(false, false).expect("application initializes");
    application
        .resize_application(192, 128)
        .expect("initial size");
    for size in [(0, 128), (192, 0), (0, 0)] {
        application
            .resize_application(size.0, size.1)
            .expect("minimize");
        application
            .redraw()
            .expect("zero size does not access the presenter");
        assert!(!application.presented);
        assert!(!requests_redraw(&WindowEvent::Resized(PhysicalSize::new(
            size.0, size.1
        ))));
    }
    application.resize_application(224, 160).expect("restore");
    assert!(application.drawable);
    let frame = application.inspector.observe().expect("restored frame");
    assert_eq!(frame.viewport().width(), 224);
    assert_eq!(frame.viewport().height(), 160);
    assert!(requests_redraw(&WindowEvent::Resized(PhysicalSize::new(
        224, 160
    ))));
}

#[test]
fn native_insertion_and_resize_preserve_evidence_sequence() {
    let mut application = NativeApplication::new(false, true).expect("application initializes");
    application
        .record_presentation()
        .expect("initial presentation");
    application
        .inspector
        .dispatch(InspectorAction::PointerMove { x: 4, y: 3 })
        .expect("pointer move");
    let moved = application.inspector.observe().expect("hover frame");
    application
        .evidence
        .as_mut()
        .expect("evidence enabled")
        .record_pointer_move(4, 3, &moved)
        .expect("pointer move evidence");
    application
        .inspector
        .dispatch(InspectorAction::PointerPress)
        .expect("pointer press");
    let selected = application.inspector.observe().expect("selected frame");
    application
        .evidence
        .as_mut()
        .expect("evidence enabled")
        .record_pointer_press(&selected)
        .expect("pointer press evidence");

    application.insert_tile().expect("native insertion");
    assert_eq!(
        application
            .inspector
            .observe()
            .expect("inserted frame")
            .keyed_keys(),
        [10, 20, 30]
    );
    application
        .record_presentation()
        .expect("mutation presentation");
    application
        .resize_window(0, 0)
        .expect("minimize during evidence");
    application
        .redraw()
        .expect("skip presentation while minimized");
    assert_eq!(
        application
            .evidence
            .as_ref()
            .expect("evidence enabled")
            .next_required(),
        Some(EvidenceMilestone::Resize)
    );
    application
        .resize_window(224, 160)
        .expect("restore during evidence");
    application
        .record_presentation()
        .expect("resize presentation");
    let mut evidence = application.evidence.take().expect("evidence enabled");
    evidence.record_close().expect("close evidence");
    let bytes = evidence.finish().expect("complete evidence");
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

    let mut application = NativeApplication::from_inspector(inspector, false, false);

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
