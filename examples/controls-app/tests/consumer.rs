use fenestra_controls_app::{DemoState, Settings, application, exercise, key};
use fenestra_ui::{ControlRole, InputEvent, Key, KeyState, Raster, Size};

fn tap(app: &mut fenestra_ui::Application, state: &mut DemoState, value: Key) {
    state
        .input(app, key(value.clone(), KeyState::Pressed))
        .unwrap();
    state.input(app, key(value, KeyState::Released)).unwrap();
}

fn tab_to(app: &mut fenestra_ui::Application, state: &mut DemoState, target: &str) {
    for _ in 0..5 {
        if app.focused_control() == Some(target) {
            return;
        }
        tap(app, state, Key::Tab);
    }
    assert_eq!(app.focused_control(), Some(target));
}

fn pixel(raster: &Raster, x: i64, y: i64) -> [u8; 4] {
    let offset = (y as usize * raster.size().width() as usize + x as usize) * 4;
    raster.bytes()[offset..offset + 4].try_into().unwrap()
}

#[test]
fn preferences_can_be_changed_applied_and_reset_with_only_keyboard_input() {
    let mut app = application().unwrap();
    let mut state = DemoState::default();
    tap(&mut app, &mut state, Key::Tab);
    assert_eq!(app.focused_control(), Some("notifications"));
    tap(&mut app, &mut state, Key::Tab);
    assert_eq!(app.focused_control(), Some("compact"));
    state
        .input(&mut app, key(Key::Space, KeyState::Pressed))
        .unwrap();
    assert!(!Settings::read(&app).unwrap().compact);
    state
        .input(&mut app, key(Key::Space, KeyState::Released))
        .unwrap();
    assert!(Settings::read(&app).unwrap().compact);
    assert!(!app.control_snapshot("apply").unwrap().state().disabled());
    tap(&mut app, &mut state, Key::Tab);
    tap(&mut app, &mut state, Key::Tab);
    assert_eq!(app.focused_control(), Some("apply"));
    tap(&mut app, &mut state, Key::Enter);
    assert_eq!(state.apply_count(), 1);
    assert!(state.applied().compact);
    assert!(app.control_snapshot("apply").unwrap().state().disabled());
    tab_to(&mut app, &mut state, "reset");
    tap(&mut app, &mut state, Key::Space);
    assert_eq!(Settings::read(&app).unwrap(), Settings::default());
    assert!(!app.control_snapshot("apply").unwrap().state().disabled());
    tab_to(&mut app, &mut state, "apply");
    tap(&mut app, &mut state, Key::Enter);
    assert_eq!(state.applied(), Settings::default());
    assert_eq!(state.apply_count(), 2);
    assert!(app.control_snapshot("reset").unwrap().state().disabled());
}

#[test]
fn disabled_controls_are_skipped_and_pointer_press_never_applies_settings() {
    let mut app = application().unwrap();
    let mut state = DemoState::default();
    for expected in ["notifications", "compact", "autosave", "notifications"] {
        tap(&mut app, &mut state, Key::Tab);
        assert_eq!(app.focused_control(), Some(expected));
    }
    let apply = app.bounds("apply").unwrap();
    state
        .input(
            &mut app,
            InputEvent::PointerMoved {
                x: apply.x() as i32 + 4,
                y: apply.y() as i32 + 4,
            },
        )
        .unwrap();
    state.input(&mut app, InputEvent::PointerPressed).unwrap();
    state.input(&mut app, InputEvent::PointerReleased).unwrap();
    assert_eq!(state.apply_count(), 0);
    app.focus(Some("compact")).unwrap();
    tap(&mut app, &mut state, Key::Space);
    state.input(&mut app, InputEvent::PointerPressed).unwrap();
    assert_eq!(state.apply_count(), 0);
    state.input(&mut app, InputEvent::PointerLeft).unwrap();
    state.input(&mut app, InputEvent::PointerReleased).unwrap();
    assert_eq!(state.apply_count(), 0);
    state
        .input(
            &mut app,
            InputEvent::PointerMoved {
                x: apply.x() as i32 + 4,
                y: apply.y() as i32 + 4,
            },
        )
        .unwrap();
    state.input(&mut app, InputEvent::PointerPressed).unwrap();
    assert_eq!(state.apply_count(), 0);
    state.input(&mut app, InputEvent::PointerReleased).unwrap();
    assert_eq!(state.apply_count(), 1);
}

#[test]
fn snapshots_checked_pixels_and_responsive_bounds_share_committed_state() {
    let mut app = application().unwrap();
    let old = app.control_snapshot("compact").unwrap();
    let original_frame = app.raster().unwrap();
    let original_bytes = original_frame.bytes().to_vec();
    assert_eq!(old.role(), ControlRole::Checkbox);
    assert_eq!(old.label(), "Use compact spacing");
    assert_eq!(old.state().checked(), Some(false));
    assert_eq!(app.control_snapshots().unwrap().len(), 5);
    let mark = app.bounds("compact_indicator").unwrap();
    assert_eq!(
        pixel(&original_frame, mark.x(), mark.y()),
        [44, 56, 72, 255]
    );
    app.set_checked("compact", true).unwrap();
    let checked = app.raster().unwrap();
    assert_eq!(pixel(&checked, mark.x(), mark.y()), [48, 128, 192, 255]);
    let white_pixels = |frame: &Raster| {
        (mark.y()..mark.y() + i64::from(mark.height()))
            .flat_map(|y| (mark.x()..mark.x() + i64::from(mark.width())).map(move |x| (x, y)))
            .filter(|&(x, y)| {
                let [red, green, blue, _] = pixel(frame, x, y);
                red > 200 && green > 200 && blue > 200
            })
            .count()
    };
    assert_eq!(white_pixels(&original_frame), 0);
    assert!(white_pixels(&checked) > 0);
    app.resize(Size::new(420, 560)).unwrap();
    let current = app.control_snapshot("compact").unwrap();
    assert_eq!(current.id(), old.id());
    assert_eq!(current.state().checked(), Some(true));
    assert_eq!(current.bounds(), app.bounds("compact").unwrap());
    assert!(current.bounds().width() < old.bounds().width());
    assert_eq!(old.state().checked(), Some(false));
    assert_eq!(original_frame.bytes(), original_bytes);
    assert_eq!(original_frame.size(), Size::new(640, 520));
}

#[test]
fn headless_repeat_and_focus_cancellation_preserve_settings_deterministically() {
    let mut first = application().unwrap();
    let mut first_state = DemoState::default();
    let first_trace = exercise(&mut first, &mut first_state).unwrap();
    assert_eq!(first_trace.len(), 4);
    assert_eq!(Settings::read(&first).unwrap(), Settings::default());
    assert!(first_state.applied().compact);
    assert!(first_state.applied().notifications);
    assert_eq!(first_state.apply_count(), 1);
    assert!(!first.control_snapshot("apply").unwrap().state().disabled());
    assert_eq!(first.focused_control(), Some("apply"));
    assert_eq!(first.size(), Size::new(420, 560));
    let mut second = application().unwrap();
    let mut second_state = DemoState::default();
    assert_eq!(
        exercise(&mut second, &mut second_state).unwrap(),
        first_trace
    );
    assert_eq!(second.raster().unwrap(), first.raster().unwrap());
}
