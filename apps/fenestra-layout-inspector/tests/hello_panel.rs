use fenestra_layout_inspector::{InspectorAction, InspectorRaster, LayoutInspector};

const BACKGROUND: [u8; 4] = [24, 32, 48, 255];
const BLUE: [u8; 4] = [48, 128, 192, 255];
const SELECTED: [u8; 4] = [255, 192, 32, 255];

fn hello_panel() -> LayoutInspector {
    LayoutInspector::from_programs(include!(concat!(env!("OUT_DIR"), "/hello_panel_fen_v2.rs")))
        .expect("the minimal authored panel must initialize")
}

#[test]
fn authored_panel_places_two_visible_cards_with_independent_hits() {
    let mut inspector = hello_panel();
    let frame = inspector.observe().expect("the frame must be observable");
    assert_eq!(frame.node_count(), 3);
    assert_eq!(frame.paint_count(), 3);
    assert_eq!(frame.hit_count(), 2);
    assert_eq!(frame.semantic_count(), 2);
    assert_eq!(frame.keyed_keys(), [10, 20]);

    let raster = inspector
        .reference_raster()
        .expect("the panel must rasterize");
    for (x, y, expected) in [
        (0, 0, [0, 0, 0, 0]),
        (8, 8, BACKGROUND),
        (19, 20, BACKGROUND),
        (20, 20, BLUE),
        (75, 83, BLUE),
        (76, 20, BACKGROUND),
        (84, 20, BLUE),
        (139, 83, BLUE),
        (140, 20, BACKGROUND),
        (168, 8, [0, 0, 0, 0]),
    ] {
        assert_eq!(pixel(&raster, x, y), expected, "pixel at ({x}, {y})");
    }

    inspector
        .dispatch(InspectorAction::PointerMove { x: 20, y: 20 })
        .expect("the first card must be hittable");
    let first = inspector
        .hovered()
        .expect("the first card must be targeted");
    inspector
        .dispatch(InspectorAction::PointerMove { x: 84, y: 20 })
        .expect("the second card must be hittable");
    let second = inspector
        .hovered()
        .expect("the second card must be targeted");
    assert_ne!(first, second);
    inspector
        .dispatch(InspectorAction::PointerMove { x: 76, y: 20 })
        .expect("the inter-card gap must be queryable");
    assert!(inspector.hovered().is_none());
}

#[test]
fn selection_changes_one_card_and_insertion_paints_another_after_resize() {
    let mut inspector = hello_panel();
    inspector
        .dispatch(InspectorAction::PointerMove { x: 20, y: 20 })
        .expect("the first card must be hittable");
    inspector
        .dispatch(InspectorAction::PointerPress)
        .expect("selection must publish the tone change");
    let selected_node = inspector.selected().expect("a card must be selected");
    let selected = inspector
        .reference_raster()
        .expect("selection must rasterize");
    assert_eq!(pixel(&selected, 20, 20), SELECTED);
    assert_eq!(pixel(&selected, 84, 20), BLUE);
    assert_eq!(pixel(&selected, 148, 20), BACKGROUND);

    inspector
        .dispatch(InspectorAction::InsertTile { key: 30 })
        .expect("insertion must create a third card");
    inspector
        .dispatch(InspectorAction::Resize {
            width: 224,
            height: 160,
        })
        .expect("resize must expose the complete third card");
    let frame = inspector
        .observe()
        .expect("the final frame must be observable");
    assert_eq!(frame.generation(), 3);
    assert_eq!(frame.node_count(), 4);
    assert_eq!(frame.keyed_keys(), [10, 20, 30]);
    assert_eq!(frame.paint_count(), 4);
    assert_eq!(frame.hit_count(), 3);
    assert_eq!(frame.semantic_count(), 3);
    assert_eq!(frame.raster_bytes(), 224 * 160 * 4);
    assert_eq!(inspector.selected(), Some(selected_node));

    let raster = inspector
        .reference_raster()
        .expect("the updated panel must rasterize");
    assert_eq!(pixel(&raster, 20, 20), SELECTED);
    assert_eq!(pixel(&raster, 84, 20), BLUE);
    assert_eq!(pixel(&raster, 148, 20), BLUE);
    assert_eq!(pixel(&raster, 203, 83), BLUE);
    assert_eq!(pixel(&raster, 204, 20), [0, 0, 0, 0]);
}

fn pixel(raster: &InspectorRaster, x: usize, y: usize) -> [u8; 4] {
    let width = usize::try_from(raster.viewport().width()).expect("width must be positive");
    let start = (y * width + x) * 4;
    raster.bytes()[start..start + 4]
        .try_into()
        .expect("one pixel must have four channels")
}
