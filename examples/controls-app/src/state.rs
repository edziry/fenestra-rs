use fenestra_ui::{Application, Error, Event, InputEvent};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Settings {
    pub notifications: bool,
    pub compact: bool,
    pub autosave: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            notifications: true,
            compact: false,
            autosave: true,
        }
    }
}

impl Settings {
    pub fn read(app: &Application) -> Result<Self, Error> {
        let checked = |name| -> Result<bool, Error> {
            Ok(app
                .control_snapshot(name)?
                .state()
                .checked()
                .expect("authored checkbox"))
        };
        Ok(Self {
            notifications: checked("notifications")?,
            compact: checked("compact")?,
            autosave: checked("autosave")?,
        })
    }
}

#[derive(Default)]
pub struct DemoState {
    applied: Settings,
    apply_count: u32,
}

impl DemoState {
    pub fn applied(&self) -> Settings {
        self.applied
    }

    pub fn apply_count(&self) -> u32 {
        self.apply_count
    }

    pub fn input(&mut self, app: &mut Application, input: InputEvent) -> Result<Vec<Event>, Error> {
        let events = app.dispatch_input(input)?;
        for event in &events {
            self.event(app, event)?;
        }
        Ok(events)
    }

    pub fn event(&mut self, app: &mut Application, event: &Event) -> Result<(), Error> {
        match event {
            Event::CheckedChanged { .. } => {
                let message = if Settings::read(app)? == self.applied {
                    "Preferences are up to date."
                } else {
                    "Changes pending. Apply when ready."
                };
                self.refresh(app, message)
            }
            Event::Activated { target } if target == "apply" => {
                let applied = Settings::read(app)?;
                let count = self.apply_count.saturating_add(1);
                app.set_disabled("apply", true)?;
                app.set_disabled("reset", applied == Settings::default())?;
                app.set_text("readout", format!("Preferences applied ({count})."))?;
                self.applied = applied;
                self.apply_count = count;
                Ok(())
            }
            Event::Activated { target } if target == "reset" => {
                let defaults = Settings::default();
                app.set_checked("notifications", defaults.notifications)?;
                app.set_checked("compact", defaults.compact)?;
                app.set_checked("autosave", defaults.autosave)?;
                let message = if defaults == self.applied {
                    "Defaults restored. Preferences are up to date."
                } else {
                    "Defaults restored. Apply when ready."
                };
                self.refresh(app, message)
            }
            _ => Ok(()),
        }
    }

    fn refresh(&self, app: &mut Application, message: &str) -> Result<(), Error> {
        let current = Settings::read(app)?;
        app.set_disabled("apply", current == self.applied)?;
        app.set_disabled("reset", current == Settings::default())?;
        app.set_text("readout", message)
    }
}
