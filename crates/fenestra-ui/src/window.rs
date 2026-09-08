use crate::native::{self, NativeError, WindowContent, WindowEvent, WindowOptions};
use crate::{Application, Error, Raster, Size};

/// Application events delivered by the optional native window host.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Event {
    /// A pointer moved to physical viewport coordinates.
    PointerMoved {
        /// Horizontal coordinate.
        x: i32,
        /// Vertical coordinate.
        y: i32,
    },
    /// The left mouse button was pressed over the current committed scene.
    Click {
        /// The topmost input-enabled element name, if any.
        target: Option<String>,
    },
    /// Space was pressed, excluding keyboard auto-repeat.
    SpacePressed,
    /// A valid nonzero viewport size was committed.
    Resized {
        /// The current pixel size.
        size: Size,
    },
    /// The user requested that the native window close.
    CloseRequested,
}

impl Application {
    /// Runs this application in a native window and returns its final state.
    ///
    /// The handler owns application behavior and may capture ordinary Rust
    /// state. Returning an error stops the window and preserves that error.
    pub fn run(
        self,
        options: WindowOptions,
        handler: impl FnMut(&mut Application, Event) -> Result<(), Error>,
    ) -> Result<Self, NativeError<Error>> {
        let mut content = ApplicationWindow {
            app: self,
            handler,
            pointer: None,
        };
        native::run(&mut content, options)?;
        Ok(content.app)
    }
}

struct ApplicationWindow<F> {
    app: Application,
    handler: F,
    pointer: Option<(i32, i32)>,
}

impl<F> WindowContent for ApplicationWindow<F>
where
    F: FnMut(&mut Application, Event) -> Result<(), Error>,
{
    type Error = Error;

    fn resize(&mut self, width: u32, height: u32) -> Result<(), Error> {
        let size = Size::new(width, height);
        self.app.resize(size)?;
        (self.handler)(&mut self.app, Event::Resized { size })
    }

    fn event(&mut self, event: WindowEvent) -> Result<(), Error> {
        let event = match event {
            WindowEvent::PointerMoved { x, y } => {
                self.pointer = Some((x, y));
                Event::PointerMoved { x, y }
            }
            WindowEvent::PointerPressed => Event::Click {
                target: self
                    .pointer
                    .and_then(|(x, y)| self.app.hit_test(x, y))
                    .map(str::to_owned),
            },
            WindowEvent::SpacePressed => Event::SpacePressed,
            WindowEvent::CloseRequested => Event::CloseRequested,
        };
        (self.handler)(&mut self.app, event)
    }

    fn frame(&self) -> Result<Raster, Error> {
        self.app.raster()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Color, Element, Style, View};

    #[test]
    fn native_clicks_resolve_against_current_layout_and_mutate_named_content() {
        let view = View::new(
            "test",
            Element::row("root")
                .child(Element::rect("card").style(Style::new().width(20).height(20).input(true))),
        );
        let app = Application::new(view, Size::new(80, 80)).unwrap();
        let mut content = ApplicationWindow {
            app,
            pointer: None,
            handler: |app: &mut Application, event| {
                if let Event::Click { target: Some(name) } = event {
                    app.set_background(&name, Color::rgba8(255, 0, 0, 255))?;
                }
                Ok(())
            },
        };
        content.event(WindowEvent::PointerPressed).unwrap();
        assert_eq!(content.app.generation(), 0);
        content
            .event(WindowEvent::PointerMoved { x: 30, y: 4 })
            .unwrap();
        content.app.set_size("card", 40, 20).unwrap();
        content.event(WindowEvent::PointerPressed).unwrap();
        let raster = content.frame().unwrap();
        let start = (4 * 80 + 30) * 4;
        assert_eq!(&raster.bytes()[start..start + 4], &[255, 0, 0, 255]);
    }
}
