use std::cell::RefCell;
use std::marker::PhantomData;
use std::num::NonZeroU32;
use std::rc::Rc;
use std::sync::Arc;

use fenestra_ui_runtime::prototype::SchedulerTick;
use softbuffer::{Context, Surface};
use winit::event_loop::OwnedDisplayHandle;
use winit::window::Window;

use super::super::super::driver::NativeDriverV1;
use super::super::super::shell::presenter::{
    NativePresenterBackendErrorV1, NativePresenterBufferPortV1, NativePresenterSurfacePortV1,
    NativeSoftbufferPresenterV1, native_pre_present_notify_source_v1,
};
use super::super::super::trace::{
    NativeFailureCauseV1, NativeObservationV1, NativeOutcomeV1, NativeTraceStageV1,
};
use super::super::super::types::NativePhysicalExtentV1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Fault {
    None,
    Resize,
    Acquire,
    Copy,
    PrePresent,
    Present,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Phase {
    Resize(u32, u32),
    Acquire,
    Copy(usize),
    PrePresent,
    Present,
    DropBuffer,
}

#[derive(Default)]
struct State {
    phases: Vec<Phase>,
    copied_pixels: Vec<u32>,
}

struct FakeSurface {
    fault: Fault,
    state: Rc<RefCell<State>>,
}

struct FakeBuffer<'a> {
    fault: Fault,
    state: Rc<RefCell<State>>,
    _surface: PhantomData<&'a mut FakeSurface>,
}

impl NativePresenterSurfacePortV1 for FakeSurface {
    type Buffer<'a>
        = FakeBuffer<'a>
    where
        Self: 'a;

    fn resize(
        &mut self,
        width: NonZeroU32,
        height: NonZeroU32,
    ) -> Result<(), NativePresenterBackendErrorV1> {
        self.state
            .borrow_mut()
            .phases
            .push(Phase::Resize(width.get(), height.get()));
        fail_if(self.fault == Fault::Resize)
    }

    fn acquire(&mut self) -> Result<Self::Buffer<'_>, NativePresenterBackendErrorV1> {
        self.state.borrow_mut().phases.push(Phase::Acquire);
        fail_if(self.fault == Fault::Acquire)?;
        Ok(FakeBuffer {
            fault: self.fault,
            state: Rc::clone(&self.state),
            _surface: PhantomData,
        })
    }
}

impl NativePresenterBufferPortV1 for FakeBuffer<'_> {
    fn copy_pixels(&mut self, pixels: &[u32]) -> Result<(), NativePresenterBackendErrorV1> {
        let mut state = self.state.borrow_mut();
        state.phases.push(Phase::Copy(pixels.len()));
        if self.fault == Fault::Copy {
            return Err(NativePresenterBackendErrorV1::OperationFailed);
        }
        state.copied_pixels.clear();
        state.copied_pixels.extend_from_slice(pixels);
        Ok(())
    }

    fn pre_present_notify(&mut self) -> Result<(), NativePresenterBackendErrorV1> {
        self.state.borrow_mut().phases.push(Phase::PrePresent);
        fail_if(self.fault == Fault::PrePresent)
    }

    fn present(self) -> Result<(), NativePresenterBackendErrorV1> {
        self.state.borrow_mut().phases.push(Phase::Present);
        fail_if(self.fault == Fault::Present)
    }
}

impl Drop for FakeBuffer<'_> {
    fn drop(&mut self) {
        self.state.borrow_mut().phases.push(Phase::DropBuffer);
    }
}

#[test]
fn softbuffer_presenter_consumes_the_exact_owned_native_resource_types() {
    type OwnedContext = Context<OwnedDisplayHandle>;
    type OwnedSurface = Surface<OwnedDisplayHandle, Arc<Window>>;
    type OwnedConstructor =
        fn(OwnedContext, OwnedSurface, Arc<Window>) -> NativeSoftbufferPresenterV1;

    let constructor: OwnedConstructor = NativeSoftbufferPresenterV1::from_owned_parts;
    let notify: fn(&Window) = native_pre_present_notify_source_v1();

    let _ = constructor;
    assert!(std::ptr::fn_addr_eq(
        notify,
        Window::pre_present_notify as fn(&Window),
    ));
}

#[test]
fn softbuffer_presenter_keeps_resize_buffer_copy_notify_accept_and_present_in_one_scope() {
    let (mut driver, state) = prepared_driver(Fault::None);

    let presented = driver
        .redraw_requested(SchedulerTick::new(2))
        .expect("fault-free presenter should complete");
    assert!(matches!(
        presented,
        super::super::super::driver::NativeRedrawResultV1::Presented { .. }
    ));
    let state = state.borrow();
    assert_eq!(
        state.phases,
        [
            Phase::Resize(2, 2),
            Phase::Acquire,
            Phase::Copy(4),
            Phase::PrePresent,
            Phase::Present,
            Phase::DropBuffer,
        ]
    );
    assert_eq!(state.copied_pixels.len(), 4);
    assert!(has_scheduler_outcome(&driver, NativeOutcomeV1::Accepted));
    assert!(!has_scheduler_outcome(&driver, NativeOutcomeV1::Rejected));
    let accepted = driver
        .trace()
        .events()
        .iter()
        .find(|event| {
            event.stage() == NativeTraceStageV1::Scheduler
                && event.observation() == NativeObservationV1::Frame
                && event.outcome() == NativeOutcomeV1::Accepted
        })
        .expect("acceptance should be recorded");
    let presented = driver
        .trace()
        .events()
        .iter()
        .find(|event| {
            event.stage() == NativeTraceStageV1::Renderer
                && event.observation() == NativeObservationV1::Present
                && event.outcome() == NativeOutcomeV1::Completed
        })
        .expect("presentation should be recorded");
    assert!(accepted.sequence() < presented.sequence());
}

#[test]
fn resize_acquire_copy_and_notify_fail_before_accept_with_closed_causes() {
    let cases = [
        (
            Fault::Resize,
            NativeFailureCauseV1::Presenter,
            vec![Phase::Resize(2, 2)],
        ),
        (
            Fault::Acquire,
            NativeFailureCauseV1::Presenter,
            vec![Phase::Resize(2, 2), Phase::Acquire],
        ),
        (
            Fault::Copy,
            NativeFailureCauseV1::Presenter,
            vec![
                Phase::Resize(2, 2),
                Phase::Acquire,
                Phase::Copy(4),
                Phase::DropBuffer,
            ],
        ),
        (
            Fault::PrePresent,
            NativeFailureCauseV1::PrePresent,
            vec![
                Phase::Resize(2, 2),
                Phase::Acquire,
                Phase::Copy(4),
                Phase::PrePresent,
                Phase::DropBuffer,
            ],
        ),
    ];

    for (fault, expected_cause, expected_phases) in cases {
        let (mut driver, state) = prepared_driver(fault);
        assert_eq!(
            driver
                .redraw_requested(SchedulerTick::new(2))
                .expect_err("preaccept fault should fail"),
            expected_cause
        );
        assert_eq!(state.borrow().phases, expected_phases);
        assert!(!has_scheduler_outcome(&driver, NativeOutcomeV1::Accepted));
        assert!(has_scheduler_outcome(&driver, NativeOutcomeV1::Rejected));
    }
}

#[test]
fn present_failure_is_postaccept_and_never_rejects_the_accepted_frame() {
    let (mut driver, state) = prepared_driver(Fault::Present);

    assert_eq!(
        driver
            .redraw_requested(SchedulerTick::new(2))
            .expect_err("postaccept present fault should fail"),
        NativeFailureCauseV1::Presenter
    );
    assert_eq!(
        state.borrow().phases,
        [
            Phase::Resize(2, 2),
            Phase::Acquire,
            Phase::Copy(4),
            Phase::PrePresent,
            Phase::Present,
            Phase::DropBuffer,
        ]
    );
    assert!(has_scheduler_outcome(&driver, NativeOutcomeV1::Accepted));
    assert!(!has_scheduler_outcome(&driver, NativeOutcomeV1::Rejected));
}

fn prepared_driver(
    fault: Fault,
) -> (
    NativeDriverV1<NativeSoftbufferPresenterV1<FakeSurface>>,
    Rc<RefCell<State>>,
) {
    let state = Rc::new(RefCell::new(State::default()));
    let surface = FakeSurface {
        fault,
        state: Rc::clone(&state),
    };
    let presenter = NativeSoftbufferPresenterV1::from_surface_port_for_test(surface);
    let mut driver = NativeDriverV1::new(presenter).expect("test driver should initialize");
    driver
        .observe_surface(
            NativePhysicalExtentV1::new(2, 2),
            1.0,
            SchedulerTick::new(0),
        )
        .expect("test surface should stage");
    driver
        .drain_scheduler(SchedulerTick::new(1))
        .expect("test surface should publish");
    (driver, state)
}

fn has_scheduler_outcome(
    driver: &NativeDriverV1<NativeSoftbufferPresenterV1<FakeSurface>>,
    outcome: NativeOutcomeV1,
) -> bool {
    driver.trace().events().iter().any(|event| {
        event.stage() == NativeTraceStageV1::Scheduler
            && event.observation() == NativeObservationV1::Frame
            && event.outcome() == outcome
    })
}

const fn fail_if(condition: bool) -> Result<(), NativePresenterBackendErrorV1> {
    if condition {
        Err(NativePresenterBackendErrorV1::OperationFailed)
    } else {
        Ok(())
    }
}
