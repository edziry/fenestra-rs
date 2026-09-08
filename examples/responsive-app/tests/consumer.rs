use fenestra_responsive_app::{DemoState, application, exercise};
use fenestra_ui::{Raster, Size};

fn pixel(raster: &Raster, x: i64, y: i64) -> [u8; 4] {
    let offset = (y as usize * raster.size().width() as usize + x as usize) * 4;
    raster.bytes()[offset..offset + 4].try_into().unwrap()
}

#[test]
fn narrowing_wraps_text_and_moves_named_hit_geometry_with_visible_cards() {
    let mut app = application().unwrap();
    assert_eq!(app.bounds("root").unwrap().width(), 820);
    assert_eq!(app.bounds("sidebar").unwrap().width(), 170);
    assert_eq!(app.bounds("main").unwrap().width(), 582);
    let paragraph = app.bounds("paragraph").unwrap();
    let lines = app.text_metrics("paragraph").unwrap().lines();
    let old_frame = app.raster().unwrap();
    let old_bytes = old_frame.bytes().to_vec();
    app.resize(Size::new(520, 520)).unwrap();
    assert_eq!(app.bounds("main").unwrap().width(), 282);
    assert_eq!(app.bounds("sidebar").unwrap().width(), 170);
    let narrow = app.bounds("paragraph").unwrap();
    assert!(narrow.height() > paragraph.height());
    assert!(app.text_metrics("paragraph").unwrap().lines() > lines);
    let actions = app.bounds("actions").unwrap();
    assert_eq!(actions.y(), narrow.y() + i64::from(narrow.height()) + 16);
    let card = app.bounds("highlight").unwrap();
    assert_eq!(
        app.hit_test(card.x() as i32 + 1, card.y() as i32 + 1),
        Some("highlight")
    );
    assert_eq!(
        pixel(&app.raster().unwrap(), card.x() + 1, card.y() + 1),
        [48, 128, 192, 255]
    );
    assert_eq!(old_frame.size(), Size::new(820, 520));
    assert_eq!(old_frame.bytes(), old_bytes);
}

#[test]
fn weighted_cards_honor_bounds_and_named_actions_update_text_and_color() {
    let mut app = application().unwrap();
    let mut state = DemoState::default();
    let original = app.bounds("paragraph").unwrap().height();
    state.activate(&mut app, "highlight").unwrap();
    let card = app.bounds("highlight").unwrap();
    assert_eq!(
        pixel(&app.raster().unwrap(), card.x() + 1, card.y() + 1),
        [208, 144, 48, 255]
    );
    assert_eq!(app.text("readout").unwrap(), "Highlight enabled.");
    state.activate(&mut app, "revise").unwrap();
    assert!(app.bounds("paragraph").unwrap().height() < original);
    app.resize(Size::new(960, 520)).unwrap();
    assert_eq!(app.bounds("main").unwrap().width(), 722);
    assert_eq!(app.bounds("highlight").unwrap().width(), 200);
    assert_eq!(app.bounds("revise").unwrap().width(), 400);
}

#[test]
fn headless_exercise_is_deterministic_across_independent_applications() {
    let mut first = application().unwrap();
    let first_trace = exercise(&mut first).unwrap();
    assert_eq!(first_trace.len(), 3);
    assert!(first_trace[0].contains("viewport=820x520"));
    assert!(first_trace[1].contains("viewport=520x520"));
    assert!(first_trace[2].contains("viewport=960x520"));
    let mut second = application().unwrap();
    assert_eq!(exercise(&mut second).unwrap(), first_trace);
    assert_eq!(first.raster().unwrap(), second.raster().unwrap());
}

#[test]
fn tiny_viewports_clip_overflow_without_rejecting_resizes() {
    let mut app = application().unwrap();
    app.resize(Size::new(1, 1)).unwrap();
    assert_eq!(app.bounds("root").unwrap().width(), 48);
    assert_eq!(app.bounds("root").unwrap().height(), 48);
    assert_eq!(app.bounds("sidebar").unwrap().width(), 170);
    assert_eq!(app.raster().unwrap().bytes().len(), 4);
    app.resize(Size::new(820, 520)).unwrap();
    assert_eq!(app.bounds("main").unwrap().width(), 582);
}
