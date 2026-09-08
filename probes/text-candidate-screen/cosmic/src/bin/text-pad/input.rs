use fenestra_text_screen_common::grapheme_boundaries;
use fenestra_text_screen_cosmic::editor;
use fenestra_ui::native::{ImeEvent, Key, KeyState, KeyboardInput, Modifiers, WindowEvent};
use fenestra_ui::{Error, Selection};

use super::{FONT_SIZE, LINE_HEIGHT, Preedit, TextPad};

impl TextPad {
    pub(super) fn handle_event(&mut self, event: WindowEvent) -> Result<(), Error> {
        match event {
            WindowEvent::PointerMoved { x, y } => self.pointer = (x, y),
            WindowEvent::PointerPressed => self.pointer_pressed()?,
            WindowEvent::ModifiersChanged(value) => self.modifiers = value,
            WindowEvent::Focused(focused) => {
                self.window_focused = focused;
                if !focused {
                    self.preedit = None;
                    self.composing = false;
                    self.pointer = (-1, -1);
                    self.modifiers = Modifiers::default();
                }
                self.dirty();
            }
            WindowEvent::KeyboardInput(input) => self.keyboard(input),
            WindowEvent::Ime(event) => self.ime(event),
            WindowEvent::SpacePressed | WindowEvent::CloseRequested => {}
        }
        Ok(())
    }

    fn pointer_pressed(&mut self) -> Result<(), Error> {
        let (x, y) = self.pointer;
        self.editor_focused = matches!(self.app.hit_test(x, y), Some("editor" | "editor_frame"));
        self.preedit = None;
        self.composing = false;
        self.message = None;
        if self.focused() {
            let bounds = self.app.bounds("editor")?;
            let hit = editor::hit(
                &self.text,
                bounds.width(),
                FONT_SIZE,
                LINE_HEIGHT,
                (i64::from(x) - bounds.x()).max(0) as f32,
                (i64::from(y) - bounds.y()).max(0) as f32,
            );
            let selection = if self.modifiers.shift {
                Selection::new(self.text.selection().anchor(), hit)
            } else {
                Selection::caret(hit)
            };
            if let Err(error) = self.text.set_selection(selection) {
                self.message = Some(error.to_string());
            }
        }
        self.dirty();
        Ok(())
    }

    fn keyboard(&mut self, input: KeyboardInput) {
        if !self.focused() || input.state != KeyState::Pressed || input.is_synthetic {
            return;
        }
        self.modifiers = input.modifiers;
        if self.composing {
            if input.key == Key::Escape {
                self.preedit = None;
                self.composing = false;
                self.dirty();
            }
            return;
        }
        let shortcut = input.modifiers.control && !input.modifiers.alt;
        if shortcut || input.modifiers.super_key {
            if shortcut
                && !input.modifiers.super_key
                && matches!(&input.key, Key::Character(value) if value.eq_ignore_ascii_case("a"))
            {
                self.text.select_all();
                self.message = None;
                self.dirty();
            }
            return;
        }
        let extend = input.modifiers.shift;
        self.message = None;
        match input.key {
            Key::ArrowLeft => self.text.move_left(extend),
            Key::ArrowRight => self.text.move_right(extend),
            Key::Home => self.text.move_to_start(extend),
            Key::End => self.text.move_to_end(extend),
            Key::Backspace => self.text.backspace(),
            Key::Delete => self.text.delete_forward(),
            Key::Enter => self.insert("\n"),
            _ => {
                if let Some(text) = input.text.as_deref().filter(|text| !text.is_empty()) {
                    if text
                        .chars()
                        .any(|c| c.is_control() && c != '\n' && c != '\t')
                    {
                        self.message = Some("Control text ignored".into());
                    } else {
                        self.insert(text);
                    }
                }
            }
        }
        self.dirty();
    }

    fn insert(&mut self, text: &str) {
        if let Err(error) = self.text.replace_selection(text) {
            self.message = Some(error.to_string());
        }
    }

    fn ime(&mut self, event: ImeEvent) {
        if !self.focused() {
            self.preedit = None;
            self.composing = false;
            return;
        }
        self.message = None;
        match event {
            ImeEvent::Enabled | ImeEvent::Disabled => {
                self.preedit = None;
                self.composing = false;
            }
            ImeEvent::Commit(text) => {
                self.preedit = None;
                self.composing = false;
                if !text.is_empty() {
                    self.insert(&text);
                }
            }
            ImeEvent::Preedit { text, cursor } => {
                self.preedit = None;
                self.composing = !text.is_empty();
                if !text.is_empty() {
                    self.update_preedit(&text, cursor);
                }
            }
        }
        self.dirty();
    }

    fn update_preedit(&mut self, text: &str, cursor: Option<(usize, usize)>) {
        let start = self.text.selection().range().start;
        let mut display = self.text.clone();
        if let Err(error) = display.replace_selection(text) {
            self.message = Some(error.to_string());
            return;
        }
        let boundaries = grapheme_boundaries(display.text());
        let floor = |offset| {
            boundaries
                .iter()
                .copied()
                .take_while(|&i| i <= offset)
                .last()
                .unwrap_or(0)
        };
        let ceil = |offset| {
            boundaries
                .iter()
                .copied()
                .find(|&i| i >= offset)
                .unwrap_or(display.text().len())
        };
        let marked = Selection::new(floor(start), ceil(start + text.len()));
        let selected = cursor.and_then(|(anchor, focus)| {
            if anchor <= text.len()
                && focus <= text.len()
                && text.is_char_boundary(anchor)
                && text.is_char_boundary(focus)
            {
                Some(Selection::new(ceil(start + anchor), ceil(start + focus)))
            } else {
                None
            }
        });
        if let Some(selection) = selected {
            // Both positions have been snapped to complete display graphemes.
            if let Err(error) = display.set_selection(selection) {
                self.message = Some(error.to_string());
                return;
            }
        }
        self.preedit = Some(Preedit {
            display,
            marked,
            caret_visible: cursor.is_some(),
        });
    }
}
