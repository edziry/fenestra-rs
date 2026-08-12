use std::cell::RefCell;
use std::marker::PhantomData;
use std::num::NonZeroU32;
use std::rc::Rc;

use fenestra_ui_runtime::prototype::{RuntimePaintFrameV2, SubmissionId};
use fenestra_ui_spatial::prototype::SpatialViewportV2;

use super::super::super::spatial_v2::{
    SpatialPhysicalExtentV2, SpatialPresentErrorKindV2, SpatialPresentationLimitsV2,
    SpatialPresenterBackendErrorV2, SpatialPresenterBufferPortV2, SpatialPresenterPortV2,
    SpatialPresenterSurfacePortV2, SpatialReferencePresenterV2, SpatialSurfaceTupleV2,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum BackendFault {
    None,
    Resize,
    Acquire,
    Copy,
    Notify,
    Present,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum BackendPhase {
    Resize(u32, u32),
    Acquire,
    Copy(usize),
    Notify,
    Present,
}

#[derive(Default)]
pub(super) struct BackendState {
    pub(super) phases: Vec<BackendPhase>,
    pub(super) pixels: Vec<u32>,
}

pub(super) struct FakeSurface {
    fault: BackendFault,
    state: Rc<RefCell<BackendState>>,
}

pub(super) struct FakeBuffer<'a> {
    fault: BackendFault,
    state: Rc<RefCell<BackendState>>,
    _surface: PhantomData<&'a mut FakeSurface>,
}

impl SpatialPresenterSurfacePortV2 for FakeSurface {
    type Buffer<'a>
        = FakeBuffer<'a>
    where
        Self: 'a;

    fn resize(
        &mut self,
        width: NonZeroU32,
        height: NonZeroU32,
    ) -> Result<(), SpatialPresenterBackendErrorV2> {
        self.state
            .borrow_mut()
            .phases
            .push(BackendPhase::Resize(width.get(), height.get()));
        backend_result(self.fault == BackendFault::Resize)
    }

    fn acquire(&mut self) -> Result<Self::Buffer<'_>, SpatialPresenterBackendErrorV2> {
        self.state.borrow_mut().phases.push(BackendPhase::Acquire);
        backend_result(self.fault == BackendFault::Acquire)?;
        Ok(FakeBuffer {
            fault: self.fault,
            state: Rc::clone(&self.state),
            _surface: PhantomData,
        })
    }
}

impl SpatialPresenterBufferPortV2 for FakeBuffer<'_> {
    fn copy_pixels(&mut self, pixels: &[u32]) -> Result<(), SpatialPresenterBackendErrorV2> {
        let mut state = self.state.borrow_mut();
        state.phases.push(BackendPhase::Copy(pixels.len()));
        if self.fault == BackendFault::Copy {
            return Err(SpatialPresenterBackendErrorV2::OperationFailed);
        }
        state.pixels.clear();
        state.pixels.extend_from_slice(pixels);
        Ok(())
    }

    fn pre_present_notify(&mut self) -> Result<(), SpatialPresenterBackendErrorV2> {
        self.state.borrow_mut().phases.push(BackendPhase::Notify);
        backend_result(self.fault == BackendFault::Notify)
    }

    fn present(self) -> Result<(), SpatialPresenterBackendErrorV2> {
        self.state.borrow_mut().phases.push(BackendPhase::Present);
        backend_result(self.fault == BackendFault::Present)
    }
}

pub(super) fn reference_presenter(
    fault: BackendFault,
    limits: SpatialPresentationLimitsV2,
) -> (
    SpatialReferencePresenterV2<FakeSurface>,
    Rc<RefCell<BackendState>>,
) {
    let state = Rc::new(RefCell::new(BackendState::default()));
    let presenter = SpatialReferencePresenterV2::from_surface_port_for_test(
        FakeSurface {
            fault,
            state: Rc::clone(&state),
        },
        limits,
    );
    (presenter, state)
}

pub(super) const fn limits() -> SpatialPresentationLimitsV2 {
    SpatialPresentationLimitsV2::new(4, 8, 2, 16, 64)
}

pub(super) const fn surface(
    width: u32,
    height: u32,
    logical: SpatialViewportV2,
) -> SpatialSurfaceTupleV2 {
    SpatialSurfaceTupleV2::new(SpatialPhysicalExtentV2::new(width, height), logical)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ProtocolPhase {
    Raster,
    Stage,
    Resize,
    Acquire,
    Copy,
    Notify,
    Accept,
    Present,
}

#[derive(Clone, Copy)]
pub(super) enum PortPlan {
    Success,
    FailBefore(ProtocolPhase, SpatialPresentErrorKindV2),
    FailPresent,
}

pub(super) struct RecordingPort {
    plan: PortPlan,
    phases: Vec<ProtocolPhase>,
    calls: usize,
    accept_calls: usize,
    last_successful_digest: Option<u64>,
}

impl RecordingPort {
    pub(super) const fn new(plan: PortPlan) -> Self {
        Self {
            plan,
            phases: Vec::new(),
            calls: 0,
            accept_calls: 0,
            last_successful_digest: None,
        }
    }

    pub(super) fn phases(&self) -> &[ProtocolPhase] {
        &self.phases
    }

    pub(super) const fn calls(&self) -> usize {
        self.calls
    }

    pub(super) const fn accept_calls(&self) -> usize {
        self.accept_calls
    }

    pub(super) const fn last_successful_digest(&self) -> Option<u64> {
        self.last_successful_digest
    }

    pub(super) fn set_plan_for_test(&mut self, plan: PortPlan) {
        self.plan = plan;
    }
}

impl SpatialPresenterPortV2 for RecordingPort {
    fn present_offer<A>(
        &mut self,
        frame: RuntimePaintFrameV2<'_>,
        _surface: SpatialSurfaceTupleV2,
        accept_once: A,
    ) -> Result<u64, SpatialPresentErrorKindV2>
    where
        A: FnOnce() -> Result<SubmissionId, SpatialPresentErrorKindV2>,
    {
        self.calls += 1;
        let ordered = [
            ProtocolPhase::Raster,
            ProtocolPhase::Stage,
            ProtocolPhase::Resize,
            ProtocolPhase::Acquire,
            ProtocolPhase::Copy,
            ProtocolPhase::Notify,
        ];
        for phase in ordered {
            self.phases.push(phase);
            if let PortPlan::FailBefore(expected, error) = self.plan
                && phase == expected
            {
                return Err(error);
            }
        }
        let _submission = accept_once()?;
        self.accept_calls += 1;
        self.phases.push(ProtocolPhase::Accept);
        self.phases.push(ProtocolPhase::Present);
        if matches!(self.plan, PortPlan::FailPresent) {
            return Err(SpatialPresentErrorKindV2::Presenter);
        }
        let digest = 0x5a00_0000_u64 | frame.generation().get();
        self.last_successful_digest = Some(digest);
        Ok(digest)
    }
}

const fn backend_result(failed: bool) -> Result<(), SpatialPresenterBackendErrorV2> {
    if failed {
        Err(SpatialPresenterBackendErrorV2::OperationFailed)
    } else {
        Ok(())
    }
}
