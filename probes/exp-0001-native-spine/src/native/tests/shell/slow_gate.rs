use std::marker::PhantomData;
use std::num::NonZeroU32;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, SyncSender, TryRecvError, sync_channel};
use std::thread;
use std::time::Duration;

use fenestra_ui_runtime::prototype::{SchedulerState, SchedulerTick};

use super::super::super::driver::{NativeDriverActionV1, NativeDriverV1, NativeRedrawResultV1};
use super::super::super::shell::presenter::{
    NativePresenterBackendErrorV1, NativePresenterBufferPortV1, NativePresenterSurfacePortV1,
    NativeSoftbufferPresenterV1,
};
use super::super::super::trace::{
    NativeInputSourceV1, NativeObservationV1, NativeOutcomeV1, NativeTraceStageV1,
};
use super::super::super::types::NativePhysicalExtentV1;

struct SlowSurface {
    entered: SyncSender<()>,
    release: Receiver<()>,
    retained: Arc<AtomicUsize>,
}

struct SlowBuffer<'a> {
    entered: SyncSender<()>,
    release: &'a Receiver<()>,
    retained: Arc<AtomicUsize>,
    _surface: PhantomData<&'a mut SlowSurface>,
}

struct SlowEvidence {
    all_frame_events_same_tick: bool,
    all_lanes_bounded: bool,
    offered_visual: usize,
    accepted_presenter_pending: usize,
    presenter_pending_after: usize,
    stopped: bool,
}

impl NativePresenterSurfacePortV1 for SlowSurface {
    type Buffer<'a>
        = SlowBuffer<'a>
    where
        Self: 'a;

    fn resize(
        &mut self,
        _width: NonZeroU32,
        _height: NonZeroU32,
    ) -> Result<(), NativePresenterBackendErrorV1> {
        Ok(())
    }

    fn acquire(&mut self) -> Result<Self::Buffer<'_>, NativePresenterBackendErrorV1> {
        assert_eq!(self.retained.fetch_add(1, Ordering::SeqCst), 0);
        Ok(SlowBuffer {
            entered: self.entered.clone(),
            release: &self.release,
            retained: Arc::clone(&self.retained),
            _surface: PhantomData,
        })
    }
}

impl NativePresenterBufferPortV1 for SlowBuffer<'_> {
    fn copy_pixels(&mut self, pixels: &[u32]) -> Result<(), NativePresenterBackendErrorV1> {
        if pixels.is_empty() {
            return Err(NativePresenterBackendErrorV1::OperationFailed);
        }
        Ok(())
    }

    fn pre_present_notify(&mut self) -> Result<(), NativePresenterBackendErrorV1> {
        self.entered
            .send(())
            .map_err(|_| NativePresenterBackendErrorV1::OperationFailed)?;
        self.release
            .recv_timeout(Duration::from_secs(1))
            .map_err(|_| NativePresenterBackendErrorV1::OperationFailed)
    }

    fn present(self) -> Result<(), NativePresenterBackendErrorV1> {
        Ok(())
    }
}

impl Drop for SlowBuffer<'_> {
    fn drop(&mut self) {
        assert_eq!(self.retained.fetch_sub(1, Ordering::SeqCst), 1);
    }
}

#[test]
fn slow_gate_retains_one_offer_and_cpu_frame_with_zero_scheduler_residence() {
    let (entered_tx, entered_rx) = sync_channel(1);
    let (release_tx, release_rx) = sync_channel(1);
    let retained = Arc::new(AtomicUsize::new(0));
    let thread_retained = Arc::clone(&retained);
    let worker = thread::spawn(move || {
        let presenter = NativeSoftbufferPresenterV1::from_surface_port_for_test(SlowSurface {
            entered: entered_tx,
            release: release_rx,
            retained: thread_retained,
        });
        let mut driver = NativeDriverV1::new(presenter).expect("slow driver should initialize");
        driver
            .observe_surface(
                NativePhysicalExtentV1::new(2, 2),
                1.0,
                SchedulerTick::new(0),
            )
            .expect("slow surface should stage");
        driver
            .drain_scheduler(SchedulerTick::new(1))
            .expect("slow surface should publish");
        let result = driver
            .redraw_requested(SchedulerTick::new(2))
            .expect("released slow presenter should complete");
        assert!(matches!(result, NativeRedrawResultV1::Presented { .. }));
        let frame_events = driver
            .trace()
            .events()
            .iter()
            .filter(|event| {
                event.stage() == NativeTraceStageV1::Scheduler
                    && event.observation() == NativeObservationV1::Frame
            })
            .collect::<Vec<_>>();
        let evidence = SlowEvidence {
            all_frame_events_same_tick: frame_events.iter().all(|event| event.tick().get() == 2),
            all_lanes_bounded: frame_events
                .iter()
                .all(|event| event.visual().items() <= 1 && event.in_flight().items() <= 1),
            offered_visual: frame_events
                .iter()
                .find(|event| event.outcome() == NativeOutcomeV1::Offered)
                .expect("slow gate should retain one scheduler offer")
                .visual()
                .items(),
            accepted_presenter_pending: frame_events
                .iter()
                .find(|event| event.outcome() == NativeOutcomeV1::Accepted)
                .expect("release should accept the retained offer")
                .pending()
                .presenter(),
            presenter_pending_after: driver.presenter_pending_count(),
            stopped: false,
        };
        driver
            .close_requested(NativeInputSourceV1::Scripted, SchedulerTick::new(3))
            .expect("next input should enter only after release");
        assert!(matches!(
            driver
                .drain_scheduler(SchedulerTick::new(4))
                .expect("shutdown should drain"),
            NativeDriverActionV1::StopRenderer { .. }
        ));
        SlowEvidence {
            stopped: driver.scheduler_state() == SchedulerState::Stopped,
            ..evidence
        }
    });

    entered_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("presenter should reach the deterministic gate");
    assert_eq!(retained.load(Ordering::SeqCst), 1);
    assert_eq!(entered_rx.try_recv(), Err(TryRecvError::Empty));
    release_tx.send(()).expect("gate should release once");

    let evidence = worker.join().expect("slow presenter worker should join");
    assert_eq!(retained.load(Ordering::SeqCst), 0);
    assert!(evidence.all_frame_events_same_tick);
    assert!(evidence.all_lanes_bounded);
    assert_eq!(evidence.offered_visual, 1);
    assert_eq!(evidence.accepted_presenter_pending, 1);
    assert_eq!(evidence.presenter_pending_after, 0);
    assert!(evidence.stopped);
}
