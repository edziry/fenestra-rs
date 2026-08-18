use fenestra_ui_runtime::prototype::{
    CompletionWatermark, FrameWork, RuntimePaintFrameV2, SchedulerInput, SchedulerInputResult,
    SchedulerTick, SubmissionId, UiScheduler,
};

/// Physical native surface extent for one presentation attempt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GpuSurfaceExtentV1 {
    width: u32,
    height: u32,
}

impl GpuSurfaceExtentV1 {
    /// Creates one physical surface extent.
    #[must_use]
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// Returns the physical width in pixels.
    #[must_use]
    pub const fn width(self) -> u32 {
        self.width
    }

    /// Returns the physical height in pixels.
    #[must_use]
    pub const fn height(self) -> u32 {
        self.height
    }

    const fn is_zero(self) -> bool {
        self.width == 0 || self.height == 0
    }
}

/// Closed failures at the probe GPU presentation boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GpuPresentErrorKindV1 {
    /// The offered runtime frame or surface tuple was invalid.
    Viewport,
    /// Reference-raster or Vello scene preparation failed.
    Raster,
    /// Surface texture acquisition failed before acceptance.
    Acquire,
    /// Vello renderer creation or execution failed.
    Renderer,
    /// Native presentation failed after acceptance.
    Present,
    /// GPU completion did not arrive within the bounded wait.
    Timeout,
    /// The GPU device or surface reported terminal loss.
    Surface,
    /// The GPU reported an out-of-memory condition.
    OutOfMemory,
    /// The scheduler rejected an otherwise valid transition.
    Scheduler,
    /// The adapter violated the accept-once presentation contract.
    Invariant,
}

/// Redacted failure retaining whether a scheduler submission was accepted.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GpuPresentErrorV1 {
    kind: GpuPresentErrorKindV1,
    accepted_submission: Option<SubmissionId>,
}

impl GpuPresentErrorV1 {
    /// Returns the closed failure kind.
    #[must_use]
    pub const fn kind(self) -> GpuPresentErrorKindV1 {
        self.kind
    }

    /// Returns the accepted submission for a postaccept failure.
    #[must_use]
    pub const fn accepted_submission(self) -> Option<SubmissionId> {
        self.accepted_submission
    }
}

/// Successful private GPU work returned only after presentation and completion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GpuPortReceiptV1 {
    raster_digest: u64,
}

impl GpuPortReceiptV1 {
    /// Creates a completed-port receipt for one source-raster digest.
    #[must_use]
    pub const fn new(raster_digest: u64) -> Self {
        Self { raster_digest }
    }
}

/// Probe-only port around private candidate GPU and surface resources.
pub trait GpuPresentPortV1 {
    /// Presents one frame and calls `accept_once` immediately before GPU execution.
    fn present<A>(
        &mut self,
        frame: RuntimePaintFrameV2<'_>,
        surface: GpuSurfaceExtentV1,
        accept_once: A,
    ) -> Result<GpuPortReceiptV1, GpuPresentErrorKindV1>
    where
        A: FnOnce() -> Result<SubmissionId, GpuPresentErrorKindV1>;
}

/// Correlated scheduler and GPU identities from one completed presentation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GpuPresentationReceiptV1 {
    generation: u64,
    frame: u64,
    submission: u64,
    raster_digest: u64,
}

impl GpuPresentationReceiptV1 {
    /// Returns the completed runtime generation.
    #[must_use]
    pub const fn generation(self) -> u64 {
        self.generation
    }

    /// Returns the accepted scheduler frame identity.
    #[must_use]
    pub const fn frame(self) -> u64 {
        self.frame
    }

    /// Returns the completed scheduler submission token.
    #[must_use]
    pub const fn submission(self) -> u64 {
        self.submission
    }

    /// Returns the source reference-raster digest.
    #[must_use]
    pub const fn raster_digest(self) -> u64 {
        self.raster_digest
    }
}

/// Result of one nonzero presentation attempt or absent surface.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GpuPresentationOutcomeV1 {
    /// A zero physical extent was rejected without accepting the frame.
    Suspended,
    /// GPU work, presentation, and completion all succeeded.
    Completed(GpuPresentationReceiptV1),
}

/// Correlates one scheduler offer with exactly one private GPU presentation.
#[must_use = "GPU presentation failures must be handled"]
pub fn present_gpu_offer_v1<P: GpuPresentPortV1>(
    scheduler: &mut UiScheduler,
    work: &FrameWork,
    surface: GpuSurfaceExtentV1,
    presenter: &mut P,
    tick: SchedulerTick,
) -> Result<GpuPresentationOutcomeV1, GpuPresentErrorV1> {
    if surface.is_zero() {
        reject_offer(scheduler, work, tick)?;
        return Ok(GpuPresentationOutcomeV1::Suspended);
    }
    let frame = work
        .paint_frame()
        .ok_or_else(|| reject_with(scheduler, work, tick, GpuPresentErrorKindV1::Invariant))?;
    let viewport = frame.spatial().viewport();
    if u32::try_from(viewport.width()) != Ok(surface.width)
        || u32::try_from(viewport.height()) != Ok(surface.height)
        || frame.generation() != work.generation()
    {
        return Err(reject_with(
            scheduler,
            work,
            tick,
            GpuPresentErrorKindV1::Viewport,
        ));
    }
    let mut accepted = None;
    let result = presenter.present(frame, surface, || {
        let result = scheduler
            .process_input(SchedulerInput::AcceptFrame(work.id()), tick)
            .map_err(|_| GpuPresentErrorKindV1::Scheduler)?;
        let SchedulerInputResult::FrameAccepted(submission) = result else {
            return Err(GpuPresentErrorKindV1::Invariant);
        };
        accepted = Some(submission);
        Ok(submission)
    });
    match (result, accepted) {
        (Ok(port), Some(submission)) => {
            let result = scheduler
                .process_input(
                    SchedulerInput::Complete(CompletionWatermark::from_submission(submission)),
                    tick,
                )
                .map_err(|_| error(GpuPresentErrorKindV1::Scheduler, Some(submission)))?;
            if !matches!(result, SchedulerInputResult::Control(_)) {
                return Err(error(GpuPresentErrorKindV1::Invariant, Some(submission)));
            }
            Ok(GpuPresentationOutcomeV1::Completed(
                GpuPresentationReceiptV1 {
                    generation: work.generation().get(),
                    frame: work.id().get(),
                    submission: submission.token(),
                    raster_digest: port.raster_digest,
                },
            ))
        }
        (Ok(_), None) => Err(reject_with(
            scheduler,
            work,
            tick,
            GpuPresentErrorKindV1::Invariant,
        )),
        (Err(kind), Some(submission)) => {
            let result = scheduler
                .process_input(SchedulerInput::RendererLost(submission.epoch()), tick)
                .map_err(|_| error(GpuPresentErrorKindV1::Scheduler, Some(submission)))?;
            if !matches!(result, SchedulerInputResult::Control(_)) {
                return Err(error(GpuPresentErrorKindV1::Invariant, Some(submission)));
            }
            Err(error(kind, Some(submission)))
        }
        (Err(kind), None) => Err(reject_with(scheduler, work, tick, kind)),
    }
}

fn reject_offer(
    scheduler: &mut UiScheduler,
    work: &FrameWork,
    tick: SchedulerTick,
) -> Result<(), GpuPresentErrorV1> {
    let result = scheduler
        .process_input(SchedulerInput::RejectFrame(work.id()), tick)
        .map_err(|_| error(GpuPresentErrorKindV1::Scheduler, None))?;
    if result != SchedulerInputResult::FrameRejected(work.id()) {
        return Err(error(GpuPresentErrorKindV1::Invariant, None));
    }
    Ok(())
}

fn reject_with(
    scheduler: &mut UiScheduler,
    work: &FrameWork,
    tick: SchedulerTick,
    kind: GpuPresentErrorKindV1,
) -> GpuPresentErrorV1 {
    reject_offer(scheduler, work, tick)
        .err()
        .unwrap_or(error(kind, None))
}

const fn error(
    kind: GpuPresentErrorKindV1,
    accepted_submission: Option<SubmissionId>,
) -> GpuPresentErrorV1 {
    GpuPresentErrorV1 {
        kind,
        accepted_submission,
    }
}
