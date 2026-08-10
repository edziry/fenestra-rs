use fenestra_ui_runtime::prototype::{
    CompletionWatermark, ControlAdmission, FrameWork, SchedulerAction, SchedulerInput,
    SchedulerInputResult, SchedulerTick,
};
use fenestra_ui_testkit::prototype::observe_headless_projection_v1;

use super::super::raster::build_cpu_frame_v1;
use super::super::trace::{
    NativeFailureCauseV1, NativeObservationV1, NativeOutcomeV1, NativeTraceStageV1,
    NativeTraceStepV1, NativeTraceSubmissionV1,
};
use super::super::types::{NativeFrameLimitsV1, NativeSceneRectangleV1};
use super::record::SchedulerSnapshotRecorderV1;
use super::state::{NativeDriverV1, PendingControlV1};
use super::types::{NativeRedrawResultV1, PresenterPortV1};

const FRAME_LIMITS: NativeFrameLimitsV1 =
    NativeFrameLimitsV1::new(4_096, 4_096, 16_777_216, 67_108_864);

impl<P: PresenterPortV1> NativeDriverV1<P> {
    pub(crate) fn redraw_requested(
        &mut self,
        tick: SchedulerTick,
    ) -> Result<NativeRedrawResultV1, NativeFailureCauseV1> {
        if !self.redraw_armed {
            self.reserve_trace(tick, 1, 0)?;
            let mut ignored = NativeTraceStepV1::new(
                NativeTraceStageV1::Platform,
                NativeObservationV1::Redraw,
                NativeOutcomeV1::Ignored,
            );
            ignored.surface = self.surface.accepted_tuple();
            self.record(tick, ignored)?;
            return Ok(NativeRedrawResultV1::Ignored);
        }
        self.reserve_trace(tick, 6, 5)?;
        let surface = self
            .surface
            .accepted_tuple()
            .ok_or(NativeFailureCauseV1::Invariant)?;
        self.redraw_armed = false;
        let SchedulerInputResult::FrameReady = self
            .scheduler
            .process_input(SchedulerInput::FrameReady, tick)
            .map_err(|_| NativeFailureCauseV1::Scheduler)?
        else {
            return Err(NativeFailureCauseV1::Invariant);
        };
        let mut ready = NativeTraceStepV1::new(
            NativeTraceStageV1::Scheduler,
            NativeObservationV1::Frame,
            NativeOutcomeV1::Armed,
        );
        ready.surface = Some(surface);
        self.record_scheduler(tick, ready)?;

        let Some(SchedulerAction::OfferFrame(work)) = self
            .scheduler
            .next_action(tick)
            .map_err(|_| NativeFailureCauseV1::Scheduler)?
        else {
            return Err(NativeFailureCauseV1::Invariant);
        };
        let frame = work.id();
        let mut offered = NativeTraceStepV1::new(
            NativeTraceStageV1::Scheduler,
            NativeObservationV1::Frame,
            NativeOutcomeV1::Offered,
        );
        offered.surface = Some(surface);
        offered.frame = Some(frame.get());
        self.record_scheduler(tick, offered)?;

        let staged = match self.build_frame(&work, surface) {
            Ok(staged) => staged,
            Err(cause) => return self.reject_before_accept(tick, surface, frame, cause),
        };
        let staging_digest = staged.digest();
        self.presenter_pending = true;
        let mut accepted_submission = None;
        let present_result = {
            let scheduler = &mut self.scheduler;
            let presenter = &mut self.presenter;
            let trace = &mut self.trace;
            let scheduler_turn = &mut self.scheduler_turn;
            let surface_state = &self.surface;
            let pointer_pending = self.pending_pointer.is_some();
            let redraw_armed = self.redraw_armed;
            presenter.present_offer(staged, || {
                let SchedulerInputResult::FrameAccepted(submission) = scheduler
                    .process_input(SchedulerInput::AcceptFrame(frame), tick)
                    .map_err(|_| NativeFailureCauseV1::Scheduler)?
                else {
                    return Err(NativeFailureCauseV1::Invariant);
                };
                accepted_submission = Some(submission);
                let trace_submission =
                    NativeTraceSubmissionV1::new(submission.epoch().get(), submission.token());
                let mut accepted = NativeTraceStepV1::new(
                    NativeTraceStageV1::Scheduler,
                    NativeObservationV1::Frame,
                    NativeOutcomeV1::Accepted,
                );
                accepted.surface = Some(surface);
                accepted.frame = Some(frame.get());
                accepted.submission = Some(trace_submission);
                accepted.staging_digest = Some(staging_digest);
                SchedulerSnapshotRecorderV1::new(
                    trace,
                    scheduler,
                    scheduler_turn,
                    super::super::trace::NativeTracePendingV1::new(
                        surface_state.pending_count(),
                        usize::from(pointer_pending),
                        1,
                    ),
                    redraw_armed,
                )
                .record(tick, accepted)?;
                Ok(submission)
            })
        };
        self.presenter_pending = false;
        let Some(submission) = accepted_submission else {
            let cause = present_result
                .err()
                .unwrap_or(NativeFailureCauseV1::Invariant);
            return self.reject_before_accept(tick, surface, frame, cause);
        };
        let trace_submission =
            NativeTraceSubmissionV1::new(submission.epoch().get(), submission.token());

        if let Err(cause) = present_result {
            let SchedulerInputResult::Control(admission) = self
                .scheduler
                .process_input(SchedulerInput::RendererLost(submission.epoch()), tick)
                .map_err(|_| NativeFailureCauseV1::Scheduler)?
            else {
                return Err(NativeFailureCauseV1::Invariant);
            };
            let control = admission_sequence(admission);
            self.surface.discard_pending();
            self.redraw_armed = false;
            let mut loss = NativeTraceStepV1::new(
                NativeTraceStageV1::Scheduler,
                NativeObservationV1::Present,
                NativeOutcomeV1::Accepted,
            );
            loss.surface = Some(surface);
            loss.frame = Some(frame.get());
            loss.submission = Some(trace_submission);
            loss.control = Some(control.get());
            self.record_scheduler(tick, loss)?;
            self.pending_control = Some(PendingControlV1::Loss {
                frame,
                submission,
                control,
            });
            self.retiring_submission = Some(submission);
            return Err(cause);
        }

        let mut presented = NativeTraceStepV1::new(
            NativeTraceStageV1::Renderer,
            NativeObservationV1::Present,
            NativeOutcomeV1::Completed,
        );
        presented.surface = Some(surface);
        presented.frame = Some(frame.get());
        presented.submission = Some(trace_submission);
        self.record(tick, presented)?;

        let SchedulerInputResult::Control(admission) = self
            .scheduler
            .process_input(
                SchedulerInput::Complete(CompletionWatermark::from_submission(submission)),
                tick,
            )
            .map_err(|_| NativeFailureCauseV1::Scheduler)?
        else {
            return Err(NativeFailureCauseV1::Invariant);
        };
        let control = admission_sequence(admission);
        let mut completed = NativeTraceStepV1::new(
            NativeTraceStageV1::Scheduler,
            NativeObservationV1::Completion,
            NativeOutcomeV1::Accepted,
        );
        completed.surface = Some(surface);
        completed.submission = Some(trace_submission);
        completed.control = Some(control.get());
        self.record_scheduler(tick, completed)?;
        self.pending_control = Some(PendingControlV1::Completion {
            submission,
            control,
        });
        if self
            .scheduler
            .next_action(tick)
            .map_err(|_| NativeFailureCauseV1::Scheduler)?
            .is_some()
        {
            return Err(NativeFailureCauseV1::Invariant);
        }
        let Some(PendingControlV1::Completion {
            submission: pending_submission,
            control: pending_control,
        }) = self.pending_control.take()
        else {
            return Err(NativeFailureCauseV1::Invariant);
        };
        if pending_submission != submission || pending_control != control {
            return Err(NativeFailureCauseV1::Invariant);
        }
        let mut retired = NativeTraceStepV1::new(
            NativeTraceStageV1::Scheduler,
            NativeObservationV1::Completion,
            NativeOutcomeV1::Completed,
        );
        retired.surface = Some(surface);
        retired.submission = Some(trace_submission);
        retired.control = Some(control.get());
        self.record_scheduler(tick, retired)?;
        Ok(NativeRedrawResultV1::Presented {
            frame,
            submission,
            completion_control: control,
        })
    }

    pub(crate) fn renderer_stopped(
        &mut self,
        tick: SchedulerTick,
    ) -> Result<fenestra_ui_runtime::prototype::ControlSequence, NativeFailureCauseV1> {
        if self.scheduler.state() != fenestra_ui_runtime::prototype::SchedulerState::Draining
            || self.pending_control.is_some()
        {
            return Err(NativeFailureCauseV1::Invariant);
        }
        let submission = self
            .retiring_submission
            .ok_or(NativeFailureCauseV1::Invariant)?;
        self.reserve_trace(tick, 2, 2)?;
        let SchedulerInputResult::Control(admission) = self
            .scheduler
            .process_input(
                SchedulerInput::Complete(CompletionWatermark::from_submission(submission)),
                tick,
            )
            .map_err(|_| NativeFailureCauseV1::Scheduler)?
        else {
            return Err(NativeFailureCauseV1::Invariant);
        };
        let control = admission_sequence(admission);
        let trace_submission =
            NativeTraceSubmissionV1::new(submission.epoch().get(), submission.token());
        let mut accepted = NativeTraceStepV1::new(
            NativeTraceStageV1::Scheduler,
            NativeObservationV1::Completion,
            NativeOutcomeV1::Accepted,
        );
        accepted.surface = self.surface.accepted_tuple();
        accepted.submission = Some(trace_submission);
        accepted.control = Some(control.get());
        self.record_scheduler(tick, accepted)?;
        if self
            .scheduler
            .next_action(tick)
            .map_err(|_| NativeFailureCauseV1::Scheduler)?
            .is_some()
        {
            return Err(NativeFailureCauseV1::Invariant);
        }
        let mut completed = NativeTraceStepV1::new(
            NativeTraceStageV1::Scheduler,
            NativeObservationV1::Completion,
            NativeOutcomeV1::Completed,
        );
        completed.surface = self.surface.accepted_tuple();
        completed.submission = Some(trace_submission);
        completed.control = Some(control.get());
        self.record_scheduler(tick, completed)?;
        self.retiring_submission = None;
        Ok(control)
    }

    fn build_frame(
        &self,
        work: &FrameWork,
        surface: super::super::surface::NativeSurfaceTupleV1,
    ) -> Result<super::super::raster::CpuFrameV1, NativeFailureCauseV1> {
        if work.generation() != work.snapshot().generation()
            || work.generation() != self.scheduler.committed().generation()
        {
            return Err(NativeFailureCauseV1::Invariant);
        }
        let observed = observe_headless_projection_v1(&self.fixture, work.snapshot())
            .map_err(|_| NativeFailureCauseV1::Oracle)?;
        if observed.generation() != work.generation()
            || observed.projection().surface() != surface.logical_surface()
        {
            return Err(NativeFailureCauseV1::Invariant);
        }
        let records = observed.projection().scene_rectangles();
        let mut scene = Vec::new();
        scene
            .try_reserve_exact(records.len())
            .map_err(|_| NativeFailureCauseV1::Storage)?;
        for record in records {
            scene.push(NativeSceneRectangleV1::new(
                record.rectangle(),
                record.color(),
            ));
        }
        build_cpu_frame_v1(work.generation(), surface, &scene, FRAME_LIMITS)
            .map_err(super::state::map_contract_error)?
            .ok_or(NativeFailureCauseV1::Invariant)
    }

    pub(super) fn reject_before_accept(
        &mut self,
        tick: SchedulerTick,
        surface: super::super::surface::NativeSurfaceTupleV1,
        frame: fenestra_ui_runtime::prototype::FrameId,
        cause: NativeFailureCauseV1,
    ) -> Result<NativeRedrawResultV1, NativeFailureCauseV1> {
        self.reject_offer(tick, surface, frame)?;
        Err(cause)
    }

    pub(super) fn reject_offer(
        &mut self,
        tick: SchedulerTick,
        surface: super::super::surface::NativeSurfaceTupleV1,
        frame: fenestra_ui_runtime::prototype::FrameId,
    ) -> Result<(), NativeFailureCauseV1> {
        self.presenter_pending = false;
        let SchedulerInputResult::FrameRejected(rejected) = self
            .scheduler
            .process_input(SchedulerInput::RejectFrame(frame), tick)
            .map_err(|_| NativeFailureCauseV1::Scheduler)?
        else {
            return Err(NativeFailureCauseV1::Invariant);
        };
        if rejected != frame {
            return Err(NativeFailureCauseV1::Invariant);
        }
        let mut scheduler = NativeTraceStepV1::new(
            NativeTraceStageV1::Scheduler,
            NativeObservationV1::Frame,
            NativeOutcomeV1::Rejected,
        );
        scheduler.surface = Some(surface);
        scheduler.frame = Some(frame.get());
        self.record_scheduler(tick, scheduler)?;
        let mut renderer = NativeTraceStepV1::new(
            NativeTraceStageV1::Renderer,
            NativeObservationV1::Frame,
            NativeOutcomeV1::Rejected,
        );
        renderer.surface = Some(surface);
        renderer.frame = Some(frame.get());
        self.record(tick, renderer)?;
        Ok(())
    }
}

const fn admission_sequence(
    admission: ControlAdmission,
) -> fenestra_ui_runtime::prototype::ControlSequence {
    match admission {
        ControlAdmission::Accepted(control) | ControlAdmission::AlreadyAccepted(control) => control,
    }
}
