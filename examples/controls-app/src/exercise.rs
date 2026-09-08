use fenestra_ui::{Application, Error, InputEvent, Key, KeyState, KeyboardInput, Modifiers, Size};

use crate::{DemoState, Settings};

pub fn key(key: Key, state: KeyState) -> InputEvent {
    InputEvent::KeyboardInput(KeyboardInput {
        key,
        state,
        modifiers: Modifiers::default(),
        repeat: false,
        text: None,
        is_synthetic: false,
    })
}

pub fn exercise(app: &mut Application, state: &mut DemoState) -> Result<Vec<String>, Error> {
    let mut trace = vec![summary("initial", app, state)?];
    tap(app, state, Key::Tab)?;
    tap(app, state, Key::Tab)?;
    state.input(app, key(Key::Space, KeyState::Pressed))?;
    let mut repeat = key(Key::Space, KeyState::Pressed);
    if let InputEvent::KeyboardInput(ref mut input) = repeat {
        input.repeat = true;
    }
    state.input(app, repeat)?;
    state.input(app, key(Key::Space, KeyState::Released))?;
    trace.push(summary("changed", app, state)?);
    tap(app, state, Key::Tab)?;
    tap(app, state, Key::Tab)?;
    tap(app, state, Key::Enter)?;
    trace.push(summary("applied", app, state)?);
    app.focus(Some("notifications"))?;
    state.input(app, key(Key::Space, KeyState::Pressed))?;
    state.input(app, InputEvent::Focused(false))?;
    state.input(app, key(Key::Space, KeyState::Released))?;
    state.input(app, InputEvent::Focused(true))?;
    app.focus(Some("reset"))?;
    tap(app, state, Key::Space)?;
    app.resize(Size::new(420, 560))?;
    app.focus(Some("apply"))?;
    trace.push(summary("reset", app, state)?);
    Ok(trace)
}

fn tap(app: &mut Application, state: &mut DemoState, value: Key) -> Result<(), Error> {
    state.input(app, key(value.clone(), KeyState::Pressed))?;
    state.input(app, key(value, KeyState::Released))?;
    Ok(())
}

pub fn summary(stage: &str, app: &Application, state: &DemoState) -> Result<String, Error> {
    let current = Settings::read(app)?;
    let apply = app.control_snapshot("apply")?;
    Ok(format!(
        "{stage} viewport={}x{} focus={} notifications={} compact={} autosave={} apply_disabled={} applied_count={} controls={}",
        app.size().width(),
        app.size().height(),
        app.focused_control().unwrap_or("none"),
        current.notifications,
        current.compact,
        current.autosave,
        apply.state().disabled(),
        state.apply_count(),
        app.control_snapshots()?.len(),
    ))
}
