use std::cell::Cell;
use std::rc::Rc;

use fenestra_ui::{
    Application, Color, Dimension, Element, Error, Event, InputEvent, Key, KeyState, KeyboardInput,
    Limits, Modifiers, Raster, Size, StateStyle, Style, TextEngine, TextError, TextLayout,
    TextLimits, TextMeasureRequest, TextMetrics, TextRequest, TextStyle, View,
};

#[path = "control_atomicity/accessibility.rs"]
mod accessibility;
#[path = "control_atomicity/semantics.rs"]
mod semantics;

const BASE: Color = Color::rgba8(20, 40, 60, 255);
const CHECKED: Color = Color::rgba8(200, 80, 40, 255);
const DISABLED: Color = Color::rgba8(100, 100, 100, 255);

#[derive(Default)]
struct EngineState {
    reject: Cell<Option<Color>>,
    measures: Cell<usize>,
    layouts: Cell<usize>,
}

impl EngineState {
    fn calls(&self) -> (usize, usize) {
        (self.measures.get(), self.layouts.get())
    }
}

struct Engine(Rc<EngineState>);

fn metrics(text: &str) -> TextMetrics {
    TextMetrics::new(text.len() as f32, 4.0, 1, text.chars().count(), 0)
}

impl TextEngine for Engine {
    fn measure(&mut self, request: TextMeasureRequest<'_>) -> Result<TextMetrics, TextError> {
        self.0.measures.set(self.0.measures.get() + 1);
        Ok(metrics(request.text()))
    }

    fn layout(&mut self, request: TextRequest<'_>) -> Result<TextLayout, TextError> {
        self.0.layouts.set(self.0.layouts.get() + 1);
        let color = request.style().color_value();
        if self.0.reject.get() == Some(color) {
            return Err(TextError::Engine("rejected control color".into()));
        }
        let size = request.size();
        let bytes = color
            .to_rgba8()
            .repeat(size.width() as usize * size.height() as usize);
        TextLayout::new(Raster::new(size, bytes).unwrap(), metrics(request.text()))
    }
}

fn control(state_style: StateStyle) -> Element {
    Element::checkbox("choice", "Choice")
        .style(Style::new().width(24).height(14).padding(3))
        .child(
            Element::text("label", "abc")
                .style(
                    Style::new()
                        .width_mode(Dimension::Auto)
                        .height_mode(Dimension::Auto),
                )
                .text_style(TextStyle::new().color(BASE))
                .state_style(state_style),
        )
}

fn application(state: &Rc<EngineState>, state_style: StateStyle, max_bytes: usize) -> Application {
    Application::with_limits_and_text_engine(
        View::new("atomic", control(state_style)),
        Size::new(32, 24),
        Limits::default().text_limits(TextLimits::new(max_bytes, 1024, 128)),
        Engine(state.clone()),
    )
    .unwrap()
}

fn key(state: KeyState) -> InputEvent {
    InputEvent::KeyboardInput(KeyboardInput {
        key: Key::Space,
        state,
        modifiers: Modifiers::default(),
        repeat: false,
        text: None,
        is_synthetic: false,
    })
}

fn rejected_color() -> Error {
    Error::Text(TextError::Engine("rejected control color".into()))
}

#[test]
fn rejected_checkbox_release_retains_the_arm_and_authored_measurements_for_retry() {
    let state = Rc::new(EngineState::default());
    let mut app = application(&state, StateStyle::new().checked_color(CHECKED), 128);
    app.focus(Some("choice")).unwrap();
    app.dispatch_input(key(KeyState::Pressed)).unwrap();
    let before = app.control_snapshot("choice").unwrap();
    let bounds = app.bounds("label").unwrap();
    let frame = app.raster().unwrap();
    let measured = app.text_metrics("label").unwrap();
    let generation = app.generation();
    let accepted_calls = state.calls();
    assert!(before.state().pressed());
    state.reject.set(Some(CHECKED));
    for attempt in 1..=2 {
        assert_eq!(
            app.dispatch_input(key(KeyState::Released)),
            Err(rejected_color())
        );
        assert_eq!(app.control_snapshot("choice").unwrap(), before);
        assert_eq!(app.focused_control(), Some("choice"));
        assert_eq!(app.generation(), generation);
        assert_eq!(app.bounds("label").unwrap(), bounds);
        assert_eq!(app.text_metrics("label").unwrap(), measured);
        assert_eq!(app.raster().unwrap(), frame);
        assert_eq!(state.layouts.get(), accepted_calls.1 + attempt);
        assert_eq!(state.measures.get(), accepted_calls.0);
    }
    let failed_calls = state.calls();
    state.reject.set(None);
    let events = app.dispatch_input(key(KeyState::Released)).unwrap();
    assert_eq!(
        events
            .iter()
            .filter(|event| matches!(event, Event::CheckedChanged { .. }))
            .count(),
        1
    );
    assert!(events.contains(&Event::CheckedChanged {
        target: "choice".into(),
        checked: true
    }));
    let after = app.control_snapshot("choice").unwrap();
    assert_eq!(after.id(), before.id());
    assert_eq!(after.state().checked(), Some(true));
    assert!(after.state().focused());
    assert!(!after.state().pressed());
    assert_eq!(before.state().checked(), Some(false));
    assert!(before.state().pressed());
    assert_eq!(app.generation(), generation + 1);
    assert_eq!(app.bounds("label").unwrap(), bounds);
    assert_ne!(app.raster().unwrap(), frame);
    assert_eq!(app.text_style("label").unwrap().color_value(), BASE);
    assert_eq!(state.layouts.get(), failed_calls.1 + 1);
    assert_eq!(state.measures.get(), accepted_calls.0);
}

#[test]
fn rejected_disabling_preserves_focused_pressed_checked_state_until_successful_retry() {
    let state = Rc::new(EngineState::default());
    let mut app = application(
        &state,
        StateStyle::new()
            .checked_color(CHECKED)
            .disabled_color(DISABLED),
        128,
    );
    app.set_checked("choice", true).unwrap();
    app.focus(Some("choice")).unwrap();
    app.dispatch_input(key(KeyState::Pressed)).unwrap();
    let before = app.control_snapshot("choice").unwrap();
    let bounds = app.bounds("label").unwrap();
    let frame = app.raster().unwrap();
    let generation = app.generation();
    let accepted_calls = state.calls();
    state.reject.set(Some(DISABLED));
    assert_eq!(app.set_disabled("choice", true), Err(rejected_color()));
    assert_eq!(app.control_snapshot("choice").unwrap(), before);
    assert_eq!(app.focused_control(), Some("choice"));
    assert_eq!(app.generation(), generation);
    assert_eq!(app.bounds("label").unwrap(), bounds);
    assert_eq!(app.raster().unwrap(), frame);
    assert_eq!(state.measures.get(), accepted_calls.0);
    assert_eq!(state.layouts.get(), accepted_calls.1 + 1);
    state.reject.set(None);
    app.set_disabled("choice", true).unwrap();
    let after = app.control_snapshot("choice").unwrap();
    assert_eq!(after.id(), before.id());
    assert!(after.state().disabled());
    assert!(!after.state().focused());
    assert!(!after.state().pressed());
    assert_eq!(after.state().checked(), Some(true));
    assert_eq!(app.focused_control(), None);
    assert_eq!(app.bounds("label").unwrap(), bounds);
    assert_eq!(app.generation(), generation + 1);
    assert_ne!(app.raster().unwrap(), frame);
    assert_eq!(state.measures.get(), accepted_calls.0);
    assert_eq!(state.layouts.get(), accepted_calls.1 + 2);
    assert!(before.state().focused() && before.state().pressed());
    assert!(!before.state().disabled());
    assert!(
        app.dispatch_input(key(KeyState::Released))
            .unwrap()
            .iter()
            .all(|event| !matches!(event, Event::CheckedChanged { .. }))
    );
    assert_eq!(
        app.control_snapshot("choice").unwrap().state().checked(),
        Some(true)
    );
}

#[test]
fn focus_and_background_only_state_changes_reuse_intrinsic_and_raster_caches() {
    let state = Rc::new(EngineState::default());
    let mut app = application(&state, StateStyle::new(), 128);
    app.set_state_style(
        "choice",
        StateStyle::new()
            .hover_background(Color::rgba8(30, 50, 80, 255))
            .pressed_background(Color::rgba8(80, 50, 30, 255))
            .checked_background(CHECKED)
            .disabled_background(DISABLED),
    )
    .unwrap();
    let baseline = state.calls();
    assert!(baseline.0 > 0 && baseline.1 > 0);
    let idle = app.raster().unwrap();
    app.focus(Some("choice")).unwrap();
    assert_ne!(app.raster().unwrap(), idle);
    app.dispatch_input(InputEvent::PointerMoved { x: 1, y: 1 })
        .unwrap();
    app.dispatch_input(key(KeyState::Pressed)).unwrap();
    app.dispatch_input(key(KeyState::Released)).unwrap();
    app.set_disabled("choice", true).unwrap();
    assert_eq!(state.calls(), baseline);
    assert_eq!(
        app.control_snapshot("choice").unwrap().state().checked(),
        Some(true)
    );
}
