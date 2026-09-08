use super::*;

fn pixel(frame: &Raster, x: usize, y: usize) -> [u8; 4] {
    let offset = (y * frame.size().width() as usize + x) * 4;
    frame.bytes()[offset..offset + 4].try_into().unwrap()
}

#[test]
fn warm_raster_tracks_background_and_viewport_changes_without_mutating_old_frames() {
    let engine = Rc::new(EngineState::default());
    let mut app = application(&engine, StateStyle::new(), 128);
    app.set_background("choice", BASE).unwrap();
    let initial = app.raster().unwrap();
    let initial_bytes = initial.bytes().to_vec();
    assert_eq!(app.raster().unwrap(), initial);

    app.set_background("choice", CHECKED).unwrap();
    let recolored = app.raster().unwrap();
    assert_eq!(pixel(&recolored, 0, 0), CHECKED.to_rgba8());
    assert_eq!(pixel(&recolored, 3, 3), BASE.to_rgba8());
    assert_ne!(recolored, initial);

    app.resize(Size::new(40, 30)).unwrap();
    let resized = app.raster().unwrap();
    assert_eq!(resized.size(), app.size());
    assert_eq!(resized.bytes().len(), 40 * 30 * 4);
    assert_eq!(pixel(&resized, 0, 0), CHECKED.to_rgba8());
    assert_eq!(pixel(&resized, 39, 29), [0; 4]);
    assert_eq!(recolored.size(), Size::new(32, 24));
    assert_eq!(initial.bytes(), initial_bytes);
    assert_eq!(pixel(&initial, 0, 0), BASE.to_rgba8());
}

#[test]
fn warm_raster_tracks_focus_checked_colors_and_replacement_state_styles() {
    const FOCUS: Color = Color::rgba8(40, 220, 160, 255);
    let engine = Rc::new(EngineState::default());
    let mut app = application(&engine, StateStyle::new().checked_color(CHECKED), 128);
    app.set_state_style(
        "choice",
        StateStyle::new()
            .checked_background(DISABLED)
            .focus_color(FOCUS),
    )
    .unwrap();
    let idle = app.raster().unwrap();

    app.focus(Some("choice")).unwrap();
    let focused = app.raster().unwrap();
    assert_eq!(pixel(&focused, 0, 0), FOCUS.to_rgba8());
    assert_eq!(pixel(&idle, 0, 0), [0; 4]);

    app.set_checked("choice", true).unwrap();
    let checked = app.raster().unwrap();
    assert_eq!(pixel(&checked, 3, 3), CHECKED.to_rgba8());
    assert_eq!(pixel(&checked, 10, 5), DISABLED.to_rgba8());
    assert_eq!(pixel(&focused, 3, 3), BASE.to_rgba8());

    app.set_state_style(
        "choice",
        StateStyle::new()
            .checked_background(BASE)
            .focus_color(CHECKED),
    )
    .unwrap();
    let restyled = app.raster().unwrap();
    assert_eq!(pixel(&restyled, 0, 0), CHECKED.to_rgba8());
    assert_eq!(pixel(&restyled, 10, 5), BASE.to_rgba8());
    assert_eq!(pixel(&checked, 0, 0), FOCUS.to_rgba8());
    assert_eq!(pixel(&checked, 10, 5), DISABLED.to_rgba8());
}

#[test]
fn warm_raster_tracks_text_content_and_preserves_the_previous_text_extent() {
    let engine = Rc::new(EngineState::default());
    let mut app = application(&engine, StateStyle::new(), 128);
    let initial = app.raster().unwrap();
    assert_eq!(app.bounds("label").unwrap().width(), 3);
    assert_eq!(pixel(&initial, 7, 3), [0; 4]);

    app.set_text("label", "abcde").unwrap();
    let updated = app.raster().unwrap();
    assert_eq!(app.bounds("label").unwrap().width(), 5);
    assert_eq!(pixel(&updated, 7, 3), BASE.to_rgba8());
    assert_eq!(pixel(&initial, 7, 3), [0; 4]);

    let generation = app.generation();
    let calls = engine.calls();
    app.set_text("label", "abcde").unwrap();
    assert_eq!(app.raster().unwrap(), updated);
    assert_eq!(app.generation(), generation);
    assert_eq!(engine.calls(), calls);
}

#[test]
fn warm_raster_survives_noops_and_rejected_text_style_until_successful_retry() {
    let engine = Rc::new(EngineState::default());
    let mut app = application(&engine, StateStyle::new(), 128);
    let initial = app.raster().unwrap();
    let initial_bytes = initial.bytes().to_vec();
    let generation = app.generation();
    let calls = engine.calls();
    app.set_style("choice", app.style("choice").unwrap())
        .unwrap();
    app.set_state_style("choice", app.state_style("choice").unwrap())
        .unwrap();
    app.resize(app.size()).unwrap();
    app.focus(None).unwrap();
    app.set_checked("choice", false).unwrap();
    assert_eq!(app.generation(), generation);
    assert_eq!(app.raster().unwrap(), initial);
    assert_eq!(engine.calls(), calls);

    let next_style = app.text_style("label").unwrap().color(CHECKED);
    engine.reject.set(Some(CHECKED));
    assert_eq!(
        app.set_text_style("label", next_style),
        Err(rejected_color())
    );
    assert_eq!(engine.layouts.get(), calls.1 + 1);
    assert_eq!(app.generation(), generation);
    assert_eq!(app.text_style("label").unwrap().color_value(), BASE);
    assert_eq!(app.raster().unwrap(), initial);

    engine.reject.set(None);
    app.set_text_style("label", next_style).unwrap();
    let accepted = app.raster().unwrap();
    assert_eq!(app.generation(), generation + 1);
    assert_eq!(pixel(&accepted, 3, 3), CHECKED.to_rgba8());
    assert_ne!(accepted, initial);
    assert_eq!(initial.bytes(), initial_bytes);
    assert_eq!(pixel(&initial, 3, 3), BASE.to_rgba8());
}
