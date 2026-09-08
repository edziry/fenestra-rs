use std::error::Error;
use std::fmt;

use fenestra_ui::native::{NativeError, WindowContent, WindowEvent, WindowOptions, run};
use fenestra_ui::{Raster, Size};

use crate::evidence::{EvidenceError, EvidenceMilestone, LayoutInspectorEvidence};
use crate::{InspectorAction, InspectorErrorKind, LayoutInspector};

mod input;
#[cfg(test)]
mod tests;

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

impl fmt::Display for NativeInspectorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "native inspector failed: {self:?}")
    }
}

impl Error for NativeInspectorError {}

fn run_native_inner(
    auto_close: bool,
    record_evidence: bool,
    inspector: Option<LayoutInspector>,
) -> Result<Option<Vec<u8>>, NativeInspectorError> {
    let mut application = match inspector {
        Some(inspector) => NativeApplication::from_inspector(inspector, record_evidence),
        None => NativeApplication::new(record_evidence)?,
    };
    run(
        &mut application,
        WindowOptions::new("Fenestra Layout Inspector")
            .size(640, 420)
            .smoke(auto_close),
    )
    .map_err(|error| match error {
        NativeError::EventLoop => NativeInspectorError::EventLoop,
        NativeError::Window => NativeInspectorError::Window,
        NativeError::Presenter => NativeInspectorError::Presenter,
        NativeError::Application(error) => error,
    })?;
    Ok(application.output)
}

struct NativeApplication {
    inspector: LayoutInspector,
    evidence: Option<LayoutInspectorEvidence>,
    output: Option<Vec<u8>>,
}

impl NativeApplication {
    fn new(record_evidence: bool) -> Result<Self, NativeInspectorError> {
        let inspector = LayoutInspector::new().map_err(NativeInspectorError::Application)?;
        Ok(Self::from_inspector(inspector, record_evidence))
    }

    fn from_inspector(inspector: LayoutInspector, record_evidence: bool) -> Self {
        Self {
            inspector,
            evidence: record_evidence.then(LayoutInspectorEvidence::new),
            output: None,
        }
    }

    fn record_presentation(&mut self) -> Result<(), NativeInspectorError> {
        let Some(evidence) = &mut self.evidence else {
            return Ok(());
        };
        let frame = self
            .inspector
            .observe()
            .map_err(NativeInspectorError::Application)?;
        match evidence.next_required() {
            Some(EvidenceMilestone::InitialPresent) => evidence.record_initial(&frame),
            Some(EvidenceMilestone::MutationPresent) => evidence.record_mutation_present(&frame),
            Some(EvidenceMilestone::ResizePresent) => evidence.record_resize_present(&frame),
            _ => Ok(()),
        }
        .map_err(NativeInspectorError::Evidence)
    }
}

impl WindowContent for NativeApplication {
    type Error = NativeInspectorError;

    fn resize(&mut self, width: u32, height: u32) -> Result<(), Self::Error> {
        self.resize_window(width, height)
    }

    fn event(&mut self, event: WindowEvent) -> Result<(), Self::Error> {
        match event {
            WindowEvent::PointerMoved { x, y } => self.pointer_move(x, y),
            WindowEvent::PointerPressed => self.pointer_press(),
            WindowEvent::SpacePressed => self.insert_tile(),
            WindowEvent::CloseRequested => {
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
                Ok(())
            }
        }
    }

    fn frame(&self) -> Result<Raster, Self::Error> {
        let raster = self
            .inspector
            .reference_raster()
            .map_err(NativeInspectorError::Application)?;
        let width = u32::try_from(raster.viewport().width())
            .map_err(|_| NativeInspectorError::Presenter)?;
        let height = u32::try_from(raster.viewport().height())
            .map_err(|_| NativeInspectorError::Presenter)?;
        Raster::new(Size::new(width, height), raster.bytes().to_vec())
            .map_err(|_| NativeInspectorError::Presenter)
    }

    fn presented(&mut self) -> Result<(), Self::Error> {
        self.record_presentation()
    }
}
