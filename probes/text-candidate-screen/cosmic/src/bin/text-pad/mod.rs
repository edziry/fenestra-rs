use std::cell::{Cell, RefCell};

use fenestra_ui::native::{Modifiers, WindowContent, WindowEvent};
use fenestra_ui::{Application, Error, Raster, Selection, Size, TextBuffer};

mod input;
mod layout;
mod paint;
#[cfg(test)]
mod tests;

pub const INITIAL_SIZE: Size = Size::new(820, 580);
pub const BYTE_LIMIT: usize = 16_384;
pub const SAMPLE: &str = concat!(
    "Write something in your own words.\n\n",
    "Cafe\u{301} / Bonjour / Hello\n",
    "\u{39a}\u{3b1}\u{3bb}\u{3b7}\u{3bc}\u{3ad}\u{3c1}\u{3b1}\n",
    "\u{645}\u{631}\u{62d}\u{628}\u{627} \u{628}\u{627}\u{644}\u{639}\u{627}\u{644}\u{645}\n",
    "\u{5e9}\u{5dc}\u{5d5}\u{5dd} \u{5e2}\u{5d5}\u{5dc}\u{5dd}"
);
const FONT_SIZE: f32 = 20.0;
const LINE_HEIGHT: f32 = 28.0;

struct Preedit {
    display: TextBuffer,
    marked: Selection,
    caret_visible: bool,
}

pub struct TextPad {
    app: Application,
    text: TextBuffer,
    preedit: Option<Preedit>,
    composing: bool,
    editor_focused: bool,
    window_focused: bool,
    pointer: (i32, i32),
    modifiers: Modifiers,
    message: Option<String>,
    background: RefCell<Option<Raster>>,
    cached: RefCell<Option<Raster>>,
    render_count: Cell<usize>,
    background_render_count: Cell<usize>,
    presentations: usize,
}

impl TextPad {
    pub fn new(text: TextBuffer, size: Size) -> Result<Self, Error> {
        Ok(Self {
            app: layout::application(size)?,
            text,
            preedit: None,
            composing: false,
            editor_focused: true,
            window_focused: true,
            pointer: (-1, -1),
            modifiers: Modifiers::default(),
            message: None,
            background: RefCell::new(None),
            cached: RefCell::new(None),
            render_count: Cell::new(0),
            background_render_count: Cell::new(0),
            presentations: 0,
        })
    }

    fn display_text(&self) -> &TextBuffer {
        self.preedit
            .as_ref()
            .map_or(&self.text, |value| &value.display)
    }

    fn focused(&self) -> bool {
        self.editor_focused && self.window_focused
    }

    fn dirty(&self) {
        self.cached.borrow_mut().take();
    }

    fn status_text(&self) -> String {
        if let Some(message) = &self.message {
            return message.clone();
        }
        let state = if self.composing {
            "Composing (provisional)"
        } else if self.focused() {
            "Editing"
        } else {
            "Click the text to edit"
        };
        format!(
            "{state}  |  {} / {} bytes  |  {} selected",
            self.text.text().len(),
            self.text.max_bytes(),
            self.text.selection().range().len()
        )
    }

    pub fn summary(&self, raster: &Raster) -> String {
        let checksum = raster.bytes().iter().fold(0_u64, |sum, byte| {
            sum.wrapping_mul(131).wrapping_add(u64::from(*byte))
        });
        format!(
            "text-pad-v1 viewport={}x{} content_bytes={} selection_bytes={} rgba_bytes={} checksum={checksum:016x} presentations={}",
            raster.size().width(),
            raster.size().height(),
            self.text.text().len(),
            self.text.selection().range().len(),
            raster.bytes().len(),
            self.presentations
        )
    }

    /// Runs only fixed fixture operations; the summary never includes text.
    pub fn exercise(&mut self) -> Result<(), Error> {
        use fenestra_ui::native::{ImeEvent, Key, KeyState, KeyboardInput};
        for repeat in [false, true] {
            self.event(WindowEvent::KeyboardInput(KeyboardInput {
                key: Key::Character("!".into()),
                state: KeyState::Pressed,
                modifiers: Modifiers::default(),
                repeat,
                text: Some("!".into()),
                is_synthetic: false,
            }))?;
        }
        self.event(WindowEvent::Ime(ImeEvent::Preedit {
            text: "e\u{301}".into(),
            cursor: Some((3, 3)),
        }))?;
        self.event(WindowEvent::Ime(ImeEvent::Commit("e\u{301}".into())))?;
        self.event(WindowEvent::KeyboardInput(KeyboardInput {
            key: Key::ArrowLeft,
            state: KeyState::Pressed,
            modifiers: Modifiers {
                shift: true,
                ..Modifiers::default()
            },
            repeat: false,
            text: None,
            is_synthetic: false,
        }))
    }
}

impl WindowContent for TextPad {
    type Error = Error;

    fn resize(&mut self, width: u32, height: u32) -> Result<(), Error> {
        let size = Size::new(width, height);
        if self.app.size() != size {
            let app = layout::application(size)?;
            self.app = app;
            self.background.borrow_mut().take();
            self.dirty();
        }
        Ok(())
    }

    fn event(&mut self, event: WindowEvent) -> Result<(), Error> {
        self.handle_event(event)
    }

    fn frame(&self) -> Result<Raster, Error> {
        if let Some(frame) = self.cached.borrow().as_ref() {
            return Ok(frame.clone());
        }
        let frame = paint::frame(self)?;
        self.render_count.set(self.render_count.get() + 1);
        *self.cached.borrow_mut() = Some(frame.clone());
        Ok(frame)
    }

    fn presented(&mut self) -> Result<(), Error> {
        self.presentations += 1;
        Ok(())
    }
}
