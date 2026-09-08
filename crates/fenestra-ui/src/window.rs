use crate::native::{self, NativeError, WindowContent, WindowEvent, WindowOptions};
use crate::{Application, Error, Event, Raster, Size};

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
            WindowEvent::KeyboardInput(input) => Event::KeyboardInput(input),
            WindowEvent::ModifiersChanged(modifiers) => Event::ModifiersChanged(modifiers),
            WindowEvent::Focused(focused) => {
                if !focused {
                    self.pointer = None;
                }
                Event::Focused(focused)
            }
            WindowEvent::Ime(ime) => Event::Ime(ime),
            WindowEvent::CloseRequested => Event::CloseRequested,
        };
        (self.handler)(&mut self.app, event)
    }

    fn frame(&self) -> Result<Raster, Error> {
        self.app.raster()
    }
}

#[cfg(test)]
mod tests;
