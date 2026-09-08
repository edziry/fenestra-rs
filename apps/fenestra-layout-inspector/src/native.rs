use std::num::NonZeroU32;
use std::sync::Arc;

use softbuffer::{Context, Surface};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop, OwnedDisplayHandle};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use crate::evidence::{EvidenceError, EvidenceMilestone, LayoutInspectorEvidence};
use crate::{InspectorAction, InspectorErrorKind, LayoutInspector};

mod input;
#[cfg(test)]
mod tests;

type NativeContext = Context<OwnedDisplayHandle>;
type NativeSurface = Surface<OwnedDisplayHandle, Arc<Window>>;

/// Failures from the native application shell.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeInspectorError {
    /// The native event loop could not be created or completed.
    EventLoop,
    /// The native window could not be created.
    Window,
    /// The CPU presentation surface could not be created or updated.
    Presenter,
    /// The application core rejected an interaction or frame.
    Application(InspectorErrorKind),
    /// The bounded native evidence sequence rejected an observation.
    Evidence(EvidenceError),
}

/// Runs the interactive layout inspector until its window is closed.
pub fn run_native() -> Result<(), NativeInspectorError> {
    run_native_inner(false, false, None).map(|_| ())
}

/// Runs the supplied inspector until its native window is closed.
pub fn run_native_with_inspector(inspector: LayoutInspector) -> Result<(), NativeInspectorError> {
    run_native_inner(false, false, Some(inspector)).map(|_| ())
}

/// Runs one native presentation and exits through the event loop.
pub fn run_native_smoke() -> Result<(), NativeInspectorError> {
    run_native_inner(true, false, None).map(|_| ())
}

/// Presents the supplied inspector once and exits through the native event loop.
pub fn run_native_smoke_with_inspector(
    inspector: LayoutInspector,
) -> Result<(), NativeInspectorError> {
    run_native_inner(true, false, Some(inspector)).map(|_| ())
}

/// Runs the native inspector and returns its independently verified artifact.
pub fn run_native_artifact() -> Result<Vec<u8>, NativeInspectorError> {
    run_native_inner(false, true, None)?
        .ok_or(NativeInspectorError::Evidence(EvidenceError::Incomplete))
}

fn run_native_inner(
    auto_close: bool,
    record_evidence: bool,
    inspector: Option<LayoutInspector>,
) -> Result<Option<Vec<u8>>, NativeInspectorError> {
    let event_loop = EventLoop::new().map_err(|_| NativeInspectorError::EventLoop)?;
    let mut application = match inspector {
        Some(inspector) => {
            NativeApplication::from_inspector(inspector, auto_close, record_evidence)
        }
        None => NativeApplication::new(auto_close, record_evidence)?,
    };
    event_loop
        .run_app(&mut application)
        .map_err(|_| NativeInspectorError::EventLoop)?;
    if let Some(error) = application.failure {
        return Err(error);
    }
    Ok(application.output)
}

struct NativeApplication {
    inspector: LayoutInspector,
    auto_close: bool,
    presented: bool,
    drawable: bool,
    evidence: Option<LayoutInspectorEvidence>,
    window: Option<Arc<Window>>,
    _context: Option<NativeContext>,
    surface: Option<NativeSurface>,
    output: Option<Vec<u8>>,
    failure: Option<NativeInspectorError>,
}

impl NativeApplication {
    fn new(auto_close: bool, record_evidence: bool) -> Result<Self, NativeInspectorError> {
        let inspector = LayoutInspector::new().map_err(NativeInspectorError::Application)?;
        Ok(Self::from_inspector(inspector, auto_close, record_evidence))
    }

    fn from_inspector(inspector: LayoutInspector, auto_close: bool, record_evidence: bool) -> Self {
        Self {
            inspector,
            auto_close,
            presented: false,
            drawable: false,
            evidence: record_evidence.then(LayoutInspectorEvidence::new),
            window: None,
            _context: None,
            surface: None,
            output: None,
            failure: None,
        }
    }

    fn initialize(&mut self, event_loop: &ActiveEventLoop) -> Result<(), NativeInspectorError> {
        if self.window.is_some() {
            return Ok(());
        }
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("Fenestra Layout Inspector")
                        .with_inner_size(LogicalSize::new(640_u32, 420_u32))
                        .with_transparent(false),
                )
                .map_err(|_| NativeInspectorError::Window)?,
        );
        let physical = window.inner_size();
        self.resize_application(physical.width, physical.height)?;
        let context = Context::new(event_loop.owned_display_handle())
            .map_err(|_| NativeInspectorError::Presenter)?;
        let surface = Surface::new(&context, Arc::clone(&window))
            .map_err(|_| NativeInspectorError::Presenter)?;
        self._context = Some(context);
        self.surface = Some(surface);
        self.window = Some(window);
        self.request_redraw();
        Ok(())
    }

    fn request_redraw(&self) {
        if let Some(window) = &self.window
            && self.drawable
        {
            window.request_redraw();
        }
    }

    fn redraw(&mut self) -> Result<(), NativeInspectorError> {
        if !self.drawable {
            return Ok(());
        }
        let raster = self
            .inspector
            .reference_raster()
            .map_err(NativeInspectorError::Application)?;
        let width = u32::try_from(raster.viewport().width())
            .map_err(|_| NativeInspectorError::Presenter)?;
        let height = u32::try_from(raster.viewport().height())
            .map_err(|_| NativeInspectorError::Presenter)?;
        let width = NonZeroU32::new(width).ok_or(NativeInspectorError::Presenter)?;
        let height = NonZeroU32::new(height).ok_or(NativeInspectorError::Presenter)?;
        let surface = self
            .surface
            .as_mut()
            .ok_or(NativeInspectorError::Presenter)?;
        surface
            .resize(width, height)
            .map_err(|_| NativeInspectorError::Presenter)?;
        let mut buffer = surface
            .buffer_mut()
            .map_err(|_| NativeInspectorError::Presenter)?;
        let mut pixels = raster.bytes().chunks_exact(4);
        if buffer.len() != pixels.len() {
            return Err(NativeInspectorError::Presenter);
        }
        for destination in buffer.iter_mut() {
            let source = pixels.next().ok_or(NativeInspectorError::Presenter)?;
            *destination =
                u32::from(source[0]) << 16 | u32::from(source[1]) << 8 | u32::from(source[2]);
        }
        let window = self.window.as_ref().ok_or(NativeInspectorError::Window)?;
        window.pre_present_notify();
        buffer
            .present()
            .map_err(|_| NativeInspectorError::Presenter)?;
        self.presented = true;
        self.record_presentation()?;
        Ok(())
    }

    fn record_presentation(&mut self) -> Result<(), NativeInspectorError> {
        let Some(next) = self
            .evidence
            .as_ref()
            .and_then(LayoutInspectorEvidence::next_required)
        else {
            return Ok(());
        };
        let frame = self
            .inspector
            .observe()
            .map_err(NativeInspectorError::Application)?;
        match next {
            EvidenceMilestone::InitialPresent => self
                .evidence
                .as_mut()
                .expect("evidence was checked above")
                .record_initial(&frame)
                .map_err(NativeInspectorError::Evidence),
            EvidenceMilestone::MutationPresent => self
                .evidence
                .as_mut()
                .expect("evidence was checked above")
                .record_mutation_present(&frame)
                .map_err(NativeInspectorError::Evidence),
            EvidenceMilestone::ResizePresent => self
                .evidence
                .as_mut()
                .expect("evidence was checked above")
                .record_resize_present(&frame)
                .map_err(NativeInspectorError::Evidence),
            _ => Ok(()),
        }
    }

    fn abort(&mut self, event_loop: &ActiveEventLoop, error: NativeInspectorError) {
        self.failure.get_or_insert(error);
        event_loop.exit();
    }
}

impl ApplicationHandler for NativeApplication {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Err(error) = self.initialize(event_loop) {
            self.abort(event_loop, error);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if self
            .window
            .as_ref()
            .is_none_or(|window| window.id() != window_id)
        {
            return;
        }
        let request_redraw = input::requests_redraw(&event);
        let result = match event {
            WindowEvent::RedrawRequested => self.redraw(),
            WindowEvent::CursorMoved { position, .. } => {
                (|| -> Result<(), NativeInspectorError> {
                    let x = i32::try_from(position.x as i64);
                    let y = i32::try_from(position.y as i64);
                    match (x, y) {
                        (Ok(x), Ok(y)) => {
                            self.inspector
                                .dispatch(InspectorAction::PointerMove { x, y })
                                .map_err(NativeInspectorError::Application)?;
                            if self.evidence.as_ref().is_some_and(|evidence| {
                                evidence.next_required() == Some(EvidenceMilestone::PointerMove)
                            }) {
                                let frame = self
                                    .inspector
                                    .observe()
                                    .map_err(NativeInspectorError::Application)?;
                                self.evidence
                                    .as_mut()
                                    .expect("evidence was checked above")
                                    .record_pointer_move(x, y, &frame)
                                    .map_err(NativeInspectorError::Evidence)?;
                            }
                            Ok(())
                        }
                        _ => Err(NativeInspectorError::Application(
                            InspectorErrorKind::Transaction,
                        )),
                    }
                })()
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => (|| -> Result<(), NativeInspectorError> {
                self.inspector
                    .dispatch(InspectorAction::PointerPress)
                    .map_err(NativeInspectorError::Application)?;
                if self.evidence.as_ref().is_some_and(|evidence| {
                    evidence.next_required() == Some(EvidenceMilestone::PointerPress)
                }) {
                    let frame = self
                        .inspector
                        .observe()
                        .map_err(NativeInspectorError::Application)?;
                    self.evidence
                        .as_mut()
                        .expect("evidence was checked above")
                        .record_pointer_press(&frame)
                        .map_err(NativeInspectorError::Evidence)?;
                }
                Ok(())
            })(),
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(KeyCode::Space),
                        state: ElementState::Pressed,
                        repeat: false,
                        ..
                    },
                ..
            } => self.insert_tile(),
            WindowEvent::Resized(size) => self.resize_window(size.width, size.height),
            WindowEvent::CloseRequested => (|| -> Result<(), NativeInspectorError> {
                if let Some(evidence) = &mut self.evidence {
                    evidence
                        .record_close()
                        .map_err(NativeInspectorError::Evidence)?;
                    self.output = Some(
                        std::mem::take(evidence)
                            .finish()
                            .map_err(NativeInspectorError::Evidence)?,
                    );
                }
                event_loop.exit();
                Ok(())
            })(),
            _ => Ok(()),
        };
        match result {
            Ok(()) if request_redraw => self.request_redraw(),
            Ok(()) => (),
            Err(error) => self.abort(event_loop, error),
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.auto_close && self.presented {
            event_loop.exit();
        }
    }
}
