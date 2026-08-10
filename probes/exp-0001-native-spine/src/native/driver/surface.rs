use fenestra_ui_runtime::prototype::{CallbackFinish, SchedulerAction, SchedulerTick};

use super::super::surface::{
    NativeSurfaceChangeV1, NativeSurfaceObservationV1, NativeSurfaceTupleV1,
};
use super::super::trace::{
    NativeFailureCauseV1, NativeObservationV1, NativeOutcomeV1, NativeTraceStageV1,
    NativeTraceStepV1,
};
use super::super::types::NativePhysicalExtentV1;
use super::state::{NativeDriverV1, PendingControlV1, map_contract_error};
use super::types::{NativeDriverActionV1, PresenterPortV1};

impl<P: PresenterPortV1> NativeDriverV1<P> {
    pub(crate) fn observe_surface(
        &mut self,
        physical: NativePhysicalExtentV1,
        scale: f64,
        tick: SchedulerTick,
    ) -> Result<NativeSurfaceChangeV1, NativeFailureCauseV1> {
        let mut draft = self.surface;
        let change = match draft.observe(physical, scale) {
            Ok(change) => change,
            Err(error) => return self.record_surface_error(error, physical, scale, tick),
        };
        self.reserve_trace(tick, 1, 0)?;
        if change == NativeSurfaceChangeV1::NativeOnly {
            let cause = NativeFailureCauseV1::SurfaceRepaintUnavailable;
            let mut step = NativeTraceStepV1::new(
                NativeTraceStageV1::Platform,
                NativeObservationV1::Surface,
                NativeOutcomeV1::Failed(cause),
            );
            step.surface = draft.pending_tuple();
            self.record_surface_draft(tick, step, &draft)?;
            self.surface = draft;
            return Err(cause);
        }
        let outcome = if change == NativeSurfaceChangeV1::NoChange {
            NativeOutcomeV1::Coalesced
        } else {
            NativeOutcomeV1::Observed
        };
        let mut step = NativeTraceStepV1::new(
            NativeTraceStageV1::Platform,
            NativeObservationV1::Surface,
            outcome,
        );
        step.surface = draft.pending_tuple().or(draft.accepted_tuple());
        self.record_surface_draft(tick, step, &draft)?;
        self.surface = draft;
        Ok(change)
    }

    pub(crate) fn drain_scheduler(
        &mut self,
        tick: SchedulerTick,
    ) -> Result<NativeDriverActionV1, NativeFailureCauseV1> {
        if self.scheduler.state() != fenestra_ui_runtime::prototype::SchedulerState::Running
            || self.pending_control.is_some()
        {
            return self.drain_without_surface(tick);
        }
        if let Some(candidate) = self.surface.pending_tuple() {
            return self.publish_pending_surface(candidate, tick);
        }
        self.drain_without_surface(tick)
    }

    fn publish_pending_surface(
        &mut self,
        candidate: NativeSurfaceTupleV1,
        tick: SchedulerTick,
    ) -> Result<NativeDriverActionV1, NativeFailureCauseV1> {
        self.reserve_trace(tick, 5, 3)?;
        let expected = self.expected_projection(candidate.logical_surface())?;
        let mut surface_draft = self.surface;
        surface_draft
            .promote_pending(candidate)
            .map_err(map_contract_error)?;
        let captured = self.scheduler.committed().generation();
        let finish = {
            let mut callback = self
                .scheduler
                .begin_callback(tick)
                .map_err(|_| NativeFailureCauseV1::Scheduler)?;
            callback
                .transaction()
                .resize_headless(candidate.logical_surface())
                .map_err(|_| NativeFailureCauseV1::Runtime)?;
            callback
                .finish()
                .map_err(|_| NativeFailureCauseV1::Scheduler)?
        };
        if finish
            != (CallbackFinish::Deferred {
                operation_count: 1,
                accounted_bytes: 80,
            })
        {
            return Err(NativeFailureCauseV1::Invariant);
        }
        let mut deferred = NativeTraceStepV1::new(
            NativeTraceStageV1::Scheduler,
            NativeObservationV1::Surface,
            NativeOutcomeV1::Deferred,
        );
        deferred.captured_generation = Some(captured);
        deferred.surface = Some(candidate);
        self.record_scheduler(tick, deferred)?;

        let action = self
            .scheduler
            .next_action(tick)
            .map_err(|_| NativeFailureCauseV1::Scheduler)?;
        let generation = self.scheduler.committed().generation();
        let published = generation != captured;
        if let Some(SchedulerAction::OfferFrame(work)) = action.as_ref() {
            self.surface = surface_draft;
            self.redraw_armed = false;
            let frame = work.id();
            let mut offered = NativeTraceStepV1::new(
                NativeTraceStageV1::Scheduler,
                NativeObservationV1::Frame,
                NativeOutcomeV1::Offered,
            );
            offered.surface = Some(candidate);
            offered.frame = Some(frame.get());
            if published {
                offered.published_generation = Some(generation);
            }
            self.record_scheduler(tick, offered)?;
            self.reject_offer(tick, candidate, frame)?;
            self.record_oracle_result(&expected, candidate, generation, published, tick)?;
            return Err(NativeFailureCauseV1::Invariant);
        }
        if published && !matches!(action, Some(SchedulerAction::RequestFrame) | None) {
            return Err(NativeFailureCauseV1::Invariant);
        }
        if !published && action.is_some() {
            return Err(NativeFailureCauseV1::Invariant);
        }
        self.surface = surface_draft;
        let suspended = self.surface.accepted_is_suspended();
        self.redraw_armed = published && !suspended
            || !suspended && matches!(action, Some(SchedulerAction::RequestFrame));

        let mut publication = NativeTraceStepV1::new(
            NativeTraceStageV1::Scheduler,
            NativeObservationV1::Surface,
            if published {
                NativeOutcomeV1::Published
            } else {
                NativeOutcomeV1::Coalesced
            },
        );
        if published {
            publication.published_generation = Some(generation);
        }
        publication.surface = Some(candidate);
        self.record_scheduler(tick, publication)?;
        self.record_oracle_result(&expected, candidate, generation, published, tick)?;

        if suspended {
            return Ok(NativeDriverActionV1::Suspended {
                generation,
                surface_generation: candidate.generation(),
            });
        }
        if published {
            return Ok(NativeDriverActionV1::RequestFrame {
                generation,
                surface_generation: candidate.generation(),
            });
        }
        Ok(NativeDriverActionV1::Idle)
    }

    fn drain_without_surface(
        &mut self,
        tick: SchedulerTick,
    ) -> Result<NativeDriverActionV1, NativeFailureCauseV1> {
        self.reserve_trace(tick, 3, 2)?;
        let action = self
            .scheduler
            .next_action(tick)
            .map_err(|_| NativeFailureCauseV1::Scheduler)?;
        let generation = self.scheduler.committed().generation();
        match action {
            Some(SchedulerAction::RequestFrame) => {
                let surface = self
                    .surface
                    .accepted_tuple()
                    .ok_or(NativeFailureCauseV1::Invariant)?;
                self.redraw_armed = !self.surface.accepted_is_suspended();
                let mut step = NativeTraceStepV1::new(
                    NativeTraceStageV1::Scheduler,
                    NativeObservationV1::Frame,
                    NativeOutcomeV1::Armed,
                );
                step.surface = Some(surface);
                self.record_scheduler(tick, step)?;
                if self.surface.accepted_is_suspended() {
                    Ok(NativeDriverActionV1::Suspended {
                        generation,
                        surface_generation: surface.generation(),
                    })
                } else {
                    Ok(NativeDriverActionV1::RequestFrame {
                        generation,
                        surface_generation: surface.generation(),
                    })
                }
            }
            Some(SchedulerAction::StopRenderer(control)) => {
                self.surface.discard_pending();
                self.redraw_armed = false;
                let outcome = if self.scheduler.state()
                    == fenestra_ui_runtime::prototype::SchedulerState::Stopped
                {
                    NativeOutcomeV1::Stopped
                } else {
                    NativeOutcomeV1::Accepted
                };
                let mut step = NativeTraceStepV1::new(
                    NativeTraceStageV1::Scheduler,
                    NativeObservationV1::Shutdown,
                    outcome,
                );
                step.control = Some(control.get());
                self.record_scheduler(tick, step)?;
                Ok(NativeDriverActionV1::StopRenderer { control })
            }
            Some(SchedulerAction::OfferFrame(work)) => {
                let surface = self
                    .surface
                    .accepted_tuple()
                    .ok_or(NativeFailureCauseV1::Invariant)?;
                let frame = work.id();
                let mut offered = NativeTraceStepV1::new(
                    NativeTraceStageV1::Scheduler,
                    NativeObservationV1::Frame,
                    NativeOutcomeV1::Offered,
                );
                offered.surface = Some(surface);
                offered.frame = Some(frame.get());
                self.record_scheduler(tick, offered)?;
                self.reject_before_accept(tick, surface, frame, NativeFailureCauseV1::Invariant)
                    .map(|_| NativeDriverActionV1::Idle)
            }
            None => {
                let (observation, outcome, surface) = match self.pending_control.take() {
                    Some(PendingControlV1::Completion {
                        submission,
                        control,
                    }) => {
                        let mut step = NativeTraceStepV1::new(
                            NativeTraceStageV1::Scheduler,
                            NativeObservationV1::Completion,
                            NativeOutcomeV1::Completed,
                        );
                        step.surface = self.surface.accepted_tuple();
                        step.submission = Some(super::super::trace::NativeTraceSubmissionV1::new(
                            submission.epoch().get(),
                            submission.token(),
                        ));
                        step.control = Some(control.get());
                        self.record_scheduler(tick, step)?;
                        return Ok(NativeDriverActionV1::Idle);
                    }
                    Some(PendingControlV1::Loss {
                        frame,
                        submission,
                        control,
                    }) => {
                        let mut step = NativeTraceStepV1::new(
                            NativeTraceStageV1::Scheduler,
                            NativeObservationV1::Present,
                            NativeOutcomeV1::Failed(NativeFailureCauseV1::Presenter),
                        );
                        step.surface = self.surface.accepted_tuple();
                        step.frame = Some(frame.get());
                        step.submission = Some(super::super::trace::NativeTraceSubmissionV1::new(
                            submission.epoch().get(),
                            submission.token(),
                        ));
                        step.control = Some(control.get());
                        self.record_scheduler(tick, step)?;
                        return Ok(NativeDriverActionV1::Idle);
                    }
                    None => (NativeObservationV1::Surface, NativeOutcomeV1::Ignored, None),
                };
                let mut step =
                    NativeTraceStepV1::new(NativeTraceStageV1::Scheduler, observation, outcome);
                step.surface = surface;
                self.record_scheduler(tick, step)?;
                Ok(NativeDriverActionV1::Idle)
            }
        }
    }

    fn record_oracle_result(
        &mut self,
        expected: &fenestra_ui_testkit::prototype::NormalizedHeadlessProjectionV1,
        surface: NativeSurfaceTupleV1,
        generation: fenestra_ui_runtime::prototype::RuntimeGeneration,
        published: bool,
        tick: SchedulerTick,
    ) -> Result<(), NativeFailureCauseV1> {
        if let Err(cause) = self.compare_projection(expected) {
            let mut failed = NativeTraceStepV1::new(
                NativeTraceStageV1::Oracle,
                NativeObservationV1::Surface,
                NativeOutcomeV1::Failed(cause),
            );
            failed.surface = Some(surface);
            self.record(tick, failed)?;
            return Err(cause);
        }
        let mut matched = NativeTraceStepV1::new(
            NativeTraceStageV1::Oracle,
            NativeObservationV1::Surface,
            NativeOutcomeV1::Matched,
        );
        if published {
            matched.published_generation = Some(generation);
        }
        matched.surface = Some(surface);
        self.record(tick, matched)
    }

    fn record_surface_error(
        &mut self,
        error: super::super::types::NativeContractErrorKindV1,
        physical: NativePhysicalExtentV1,
        scale: f64,
        tick: SchedulerTick,
    ) -> Result<NativeSurfaceChangeV1, NativeFailureCauseV1> {
        let cause = map_contract_error(error);
        self.reserve_trace(tick, 1, 0)?;
        let observation = if matches!(
            cause,
            NativeFailureCauseV1::InvalidScale | NativeFailureCauseV1::EnvironmentScaleChanged
        ) {
            NativeObservationV1::Scale
        } else {
            NativeObservationV1::Surface
        };
        let mut step = NativeTraceStepV1::new(
            NativeTraceStageV1::Platform,
            observation,
            NativeOutcomeV1::Failed(cause),
        );
        if cause == NativeFailureCauseV1::EnvironmentScaleChanged {
            step.surface_observation = Some(
                NativeSurfaceObservationV1::try_new(physical, scale).map_err(map_contract_error)?,
            );
        }
        self.record(tick, step)?;
        Err(cause)
    }
}
