use fenestra_layout_inspector::{
    AUTHORED_FEN_V2, AUTHORED_UI_V2, DEFAULT_VIEWPORT, GENERATED_AUTHORING_RUST_V2,
    InspectorAction, LayoutInspector,
};

#[test]
fn authored_source_and_initial_frame_are_deterministic() {
    assert!(AUTHORED_FEN_V2.is_ascii());
    assert!(AUTHORED_FEN_V2.starts_with(b"format 2;"));
    assert!(
        AUTHORED_FEN_V2
            .windows(b"spatial format 2".len())
            .any(|window| { window == b"spatial format 2" })
    );
    assert!(GENERATED_AUTHORING_RUST_V2.contains("SpatialProgramV2"));

    let inspector = LayoutInspector::new().expect("the authored app should initialize");
    let frame = inspector
        .observe()
        .expect("the initial frame should be observable");

    assert_eq!(frame.generation(), 0);
    assert_eq!(frame.viewport(), DEFAULT_VIEWPORT);
    assert_eq!(frame.node_count(), 8);
    assert_eq!(frame.keyed_keys(), [10, 20]);
    assert_eq!(frame.image_count(), 1);
    assert_eq!(frame.paint_count(), 5);
    assert_eq!(frame.hit_count(), 5);
    assert_eq!(frame.semantic_count(), 4);
    assert_eq!(frame.raster_bytes(), 98_304);
    assert!(!frame.has_hover());
    assert!(!frame.has_selection());
}

#[test]
fn pointer_selection_keyed_content_and_resize_publish_new_frames() {
    let mut inspector = LayoutInspector::new().expect("the authored app should initialize");

    inspector
        .dispatch(InspectorAction::PointerMove { x: 4, y: 3 })
        .expect("pointer movement should be accepted");
    assert!(inspector.hovered().is_some());

    inspector
        .dispatch(InspectorAction::PointerPress)
        .expect("pointer press should commit selection");
    let selected = inspector.selected();
    let selected_frame = inspector.observe().expect("selection should be observable");
    assert_eq!(selected_frame.generation(), 1);
    assert_eq!(selected, inspector.hovered());
    assert!(selected_frame.has_selection());

    inspector
        .dispatch(InspectorAction::InsertTile { key: 30 })
        .expect("a new keyed tile should be committed");
    let keyed_frame = inspector
        .observe()
        .expect("keyed content should be observable");
    assert_eq!(keyed_frame.generation(), 2);
    assert_eq!(keyed_frame.node_count(), 9);
    assert_eq!(keyed_frame.keyed_keys(), [10, 20, 30]);

    inspector
        .dispatch(InspectorAction::Resize {
            width: 224,
            height: 160,
        })
        .expect("resize should be committed");
    let resized_frame = inspector.observe().expect("resize should be observable");
    assert_eq!(resized_frame.generation(), 3);
    assert_eq!(resized_frame.viewport().width(), 224);
    assert_eq!(resized_frame.viewport().height(), 160);
    assert_eq!(resized_frame.raster_bytes(), 143_360);
}

#[test]
fn pointer_press_without_a_hit_is_a_noop() {
    let mut inspector = LayoutInspector::new().expect("the authored app should initialize");

    inspector
        .dispatch(InspectorAction::PointerMove { x: 191, y: 127 })
        .expect("pointer movement should be accepted");
    assert!(inspector.hovered().is_none());
    inspector
        .dispatch(InspectorAction::PointerPress)
        .expect("a press outside the scene should be accepted");
    let frame = inspector
        .observe()
        .expect("the unchanged frame should be observable");
    assert_eq!(frame.generation(), 0);
    assert!(!frame.has_selection());
}

#[test]
fn configurable_programs_expose_authoring_and_selection_diagnostics() {
    let configurable = LayoutInspector::from_programs(LayoutInspector::default_programs())
        .expect("the default compiled programs should remain configurable");
    assert_eq!(
        configurable
            .observe()
            .expect("the configurable frame should be observable")
            .node_count(),
        8
    );
    let configurable_diagnostics = configurable
        .diagnostics()
        .expect("configurable diagnostics should be observable");
    assert_eq!(configurable_diagnostics.fen_source_bytes(), None);
    assert!(!configurable_diagnostics.frontends_equivalent());

    let mut inspector = LayoutInspector::new().expect("the default app should initialize");
    let initial = inspector
        .diagnostics()
        .expect("the initial diagnostics should be observable");

    assert_eq!(initial.fen_source_bytes(), Some(AUTHORED_FEN_V2.len()));
    assert_eq!(initial.ui_source_bytes(), Some(AUTHORED_UI_V2.len()));
    assert_eq!(
        initial.generated_rust_bytes(),
        Some(GENERATED_AUTHORING_RUST_V2.len())
    );
    assert!(initial.frontends_equivalent());
    assert_eq!(initial.selected_tone(), None);

    inspector
        .dispatch(InspectorAction::PointerMove { x: 4, y: 3 })
        .expect("pointer movement should be accepted");
    inspector
        .dispatch(InspectorAction::PointerPress)
        .expect("pointer press should commit selection");

    let selected = inspector
        .diagnostics()
        .expect("selection diagnostics should be observable");
    assert!(selected.selected_node().is_some());
    assert_eq!(selected.selected_tone(), Some([255, 192, 32, 255]));
}
