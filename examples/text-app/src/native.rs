use fenestra_ui::native::{ImeEvent, Key, KeyState, KeyboardInput, WindowOptions};
use fenestra_ui::{Application, Error, Event, TextBuffer, TextError};

use crate::MAX_EDIT_BYTES;

pub fn run(app: Application, smoke: bool) -> Result<Application, Box<dyn std::error::Error>> {
    let mut editor = Editor {
        buffer: TextBuffer::new(app.text("content")?, MAX_EDIT_BYTES)?,
        composing: false,
    };
    let options = WindowOptions::new("Fenestra authored text")
        .size(640, 360)
        .ime_allowed(true)
        .smoke(smoke);
    Ok(app.run(options, move |app, event| editor.event(app, event))?)
}

struct Editor {
    buffer: TextBuffer,
    composing: bool,
}

impl Editor {
    fn event(&mut self, app: &mut Application, event: Event) -> Result<(), Error> {
        match event {
            Event::KeyboardInput(input) => self.keyboard(app, input),
            Event::Ime(ImeEvent::Preedit { text, .. }) => {
                self.composing = !text.is_empty();
                app.set_text(
                    "status",
                    if self.composing {
                        "Composing: provisional text is not committed."
                    } else {
                        "Composition cleared. Type to append at the end."
                    },
                )
            }
            Event::Ime(ImeEvent::Commit(text)) => {
                self.composing = false;
                self.insert(app, &text)
            }
            Event::Ime(ImeEvent::Disabled) | Event::Focused(false) => {
                self.composing = false;
                app.set_text(
                    "status",
                    "Composition discarded. Type to append at the end.",
                )
            }
            // SpacePressed is a compatibility notification. Inserting from it
            // as well as KeyboardInput would duplicate each Space press.
            _ => Ok(()),
        }
    }

    fn keyboard(&mut self, app: &mut Application, input: KeyboardInput) -> Result<(), Error> {
        if input.state != KeyState::Pressed || input.is_synthetic || self.composing {
            return Ok(());
        }
        match input.key {
            Key::Backspace => {
                let mut candidate = self.buffer.clone();
                candidate.backspace();
                self.commit(app, candidate)
            }
            Key::Enter => self.insert(app, "\n"),
            Key::Character(_) | Key::Space => {
                if let Some(text) = input.text
                    && !text.chars().any(char::is_control)
                {
                    self.insert(app, &text)?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn insert(&mut self, app: &mut Application, text: &str) -> Result<(), Error> {
        let mut candidate = self.buffer.clone();
        if candidate.replace_selection(text).is_err() {
            return app.set_text("status", "Input rejected: this demo limits edits to 4 KiB.");
        }
        self.commit(app, candidate)
    }

    fn commit(&mut self, app: &mut Application, candidate: TextBuffer) -> Result<(), Error> {
        match app.set_text("content", candidate.text()) {
            Ok(()) => {
                self.buffer = candidate;
                app.set_text(
                    "status",
                    "Text committed. Type to append; Backspace removes one grapheme.",
                )
            }
            Err(Error::Text(TextError::MissingGlyphs { .. })) => app.set_text(
                "status",
                "Input rejected: the bundled font does not cover a glyph.",
            ),
            Err(error) => Err(error),
        }
    }
}

#[cfg(test)]
mod tests;
