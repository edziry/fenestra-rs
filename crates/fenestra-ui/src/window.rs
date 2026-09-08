use crate::native::{self, NativeError, WindowContent, WindowEvent, WindowOptions};
use crate::{
    AccessibilityActionRequest, AccessibilityTree, Application, Error, Event, Raster, Size,
};

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
        let mut content = ApplicationWindow { app: self, handler };
        native::run(&mut content, options)?;
        Ok(content.app)
    }
}

struct ApplicationWindow<F> {
    app: Application,
    handler: F,
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
        for event in self.app.dispatch_input(event)? {
            (self.handler)(&mut self.app, event)?;
        }
        Ok(())
    }

    fn frame(&self) -> Result<Raster, Error> {
        self.app.raster()
    }

    fn accessibility(&self) -> Result<Option<AccessibilityTree>, Error> {
        self.app.accessibility_tree().map(Some)
    }

    fn accessibility_action(&mut self, request: AccessibilityActionRequest) -> Result<(), Error> {
        for event in self.app.dispatch_accessibility_action(request)? {
            (self.handler)(&mut self.app, event)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
