use fenestra_ui_runtime::prototype::{
    CallbackFinish, ControlAdmission, FrameWork, HeadlessPoint, HeadlessSurface, SchedulerAction,
    SchedulerError, SchedulerInput, SchedulerInputResult, SubmissionId,
};

use crate::case::SemanticOperationV1;
use crate::headless::platform::{
    HeadlessCallbackReportV1, HeadlessPointerCaptureV1, HeadlessPointerMutationV1,
    HeadlessPointerScriptV1, HeadlessPointerTargetV1,
};
use crate::headless::renderer::headless_frame_resource_v1;
use crate::headless::trace::{
    HeadlessInputKindV1, HeadlessOutcomeV1, HeadlessTraceStageV1, HeadlessTraceStep,
};
use crate::scheduler::{
    FakeCallbackDepthV1, FakeControlDeliveryV1, FakeFrameReadyDeliveryV1, FakeRendererModeV1,
    FakeRendererOfferOutcomeV1, SchedulerTraceActionV1, SchedulerTraceCallbackOutcomeV1,
    SchedulerTraceCommitOutcomeV1, SchedulerTraceInputOutcomeV1, SchedulerTraceStepV1,
};

use super::state::{
    RunState, ensure, operation_trace, oracle_error, renderer_error, runtime_error, scheduler_error,
};
use super::types::HeadlessRunErrorV1;

pub(super) enum ActionAttempt {
    Ready(Option<SchedulerAction>),
    Failed(SchedulerError),
}

pub(super) const fn step(
    stage: HeadlessTraceStageV1,
    input: HeadlessInputKindV1,
    outcome: HeadlessOutcomeV1,
) -> HeadlessTraceStep {
    HeadlessTraceStep::new(stage, input, outcome)
}

impl RunState {
    pub(super) fn record_projection(
        &mut self,
        generation: fenestra_ui_runtime::prototype::RuntimeGeneration,
    ) -> Result<(), HeadlessRunErrorV1> {
        self.compare_projection(generation)?;
        let mut projection = step(
            HeadlessTraceStageV1::Projection,
            HeadlessInputKindV1::None,
            HeadlessOutcomeV1::Matched,
        );
        projection.records_publication = true;
        self.record_headless(projection)
    }

    pub(super) fn record_build(&mut self) -> Result<(), HeadlessRunErrorV1> {
        let generation = self.scheduler.committed().generation();
        let mut build = step(
            HeadlessTraceStageV1::Build,
            HeadlessInputKindV1::None,
            HeadlessOutcomeV1::Observed,
        );
        build.records_publication = true;
        self.record_headless(build)?;
        self.record_projection(generation)
    }

    pub(super) fn capture_pointer(
        &mut self,
        point: HeadlessPoint,
    ) -> Result<HeadlessPointerCaptureV1, HeadlessRunErrorV1> {
        let capture = self
            .platform
            .capture_headless_pointer(&self.scheduler, &self.fixture, point)
            .map_err(|_| scheduler_error())?;
        let mut event = step(
            HeadlessTraceStageV1::Input,
            HeadlessInputKindV1::Pointer,
            HeadlessOutcomeV1::Observed,
        );
        event.captured_generation = Some(capture.generation());
        event.target = capture.target();
        self.record_headless(event)?;
        Ok(capture)
    }

    pub(super) fn pointer_callback(
        &mut self,
        tick: u64,
        script: HeadlessPointerScriptV1,
    ) -> Result<HeadlessCallbackReportV1, HeadlessRunErrorV1> {
        self.advance_to(tick)?;
        let report = self
            .platform
            .run_headless_pointer_callback(
                &mut self.scheduler,
                &self.fixture,
                script,
                self.clock.now(),
            )
            .map_err(|_| scheduler_error())?;
        self.record_callback(
            report,
            FakeCallbackDepthV1::Nested,
            HeadlessInputKindV1::Pointer,
            report.captured_generation(),
            report.target(),
        )?;
        Ok(report)
    }

    pub(super) fn captured_callback(
        &mut self,
        tick: u64,
        capture: &HeadlessPointerCaptureV1,
        mutation: HeadlessPointerMutationV1,
    ) -> Result<HeadlessCallbackReportV1, HeadlessRunErrorV1> {
        self.advance_to(tick)?;
        let report = self
            .platform
            .run_headless_captured_callback(
                &mut self.scheduler,
                capture,
                FakeCallbackDepthV1::Nested,
                mutation,
                self.clock.now(),
            )
            .map_err(|_| scheduler_error())?;
        self.record_callback(
            report,
            FakeCallbackDepthV1::Nested,
            HeadlessInputKindV1::Pointer,
            capture.generation(),
            capture.target(),
        )?;
        Ok(report)
    }

    pub(super) fn resize_callback(
        &mut self,
        tick: u64,
        surface: HeadlessSurface,
    ) -> Result<HeadlessCallbackReportV1, HeadlessRunErrorV1> {
        self.advance_to(tick)?;
        let report = self
            .platform
            .run_headless_resize_callback(&mut self.scheduler, surface, self.clock.now())
            .map_err(|_| scheduler_error())?;
        self.record_callback(
            report,
            FakeCallbackDepthV1::Outer,
            HeadlessInputKindV1::Resize,
            report.captured_generation(),
            HeadlessPointerTargetV1::None,
        )?;
        Ok(report)
    }

    fn record_callback(
        &mut self,
        report: HeadlessCallbackReportV1,
        depth: FakeCallbackDepthV1,
        input: HeadlessInputKindV1,
        captured: fenestra_ui_runtime::prototype::RuntimeGeneration,
        target: HeadlessPointerTargetV1,
    ) -> Result<(), HeadlessRunErrorV1> {
        let expected_depth = match depth {
            FakeCallbackDepthV1::Outer => 1,
            FakeCallbackDepthV1::Nested => 2,
            FakeCallbackDepthV1::Grandchild => 3,
        };
        ensure(
            report.deepest_depth() == expected_depth
                && report.shares_entry_snapshot()
                && report.target() == target
                && report.finish()
                    == (CallbackFinish::Deferred {
                        operation_count: 1,
                        accounted_bytes: 80,
                    }),
            scheduler_error,
        )?;
        let scheduler = SchedulerTraceStepV1::Callback {
            depth,
            outcome: SchedulerTraceCallbackOutcomeV1::Finished(report.finish()),
        };
        let mut headless = step(
            HeadlessTraceStageV1::Callback,
            input,
            HeadlessOutcomeV1::Deferred,
        );
        headless.captured_generation = Some(captured);
        headless.target = target;
        self.record_both(scheduler, headless)
    }

    pub(super) fn commit_operation(
        &mut self,
        tick: u64,
        operation: &SemanticOperationV1,
        expected_generation: u64,
    ) -> Result<(), HeadlessRunErrorV1> {
        self.advance_to(tick)?;
        let snapshot = self.scheduler.committed();
        let mut transaction = self.scheduler.begin_transaction();
        self.stage_operation(&mut transaction, &snapshot, operation)?;
        self.oracle
            .apply_operation(operation)
            .map_err(|_| oracle_error())?;
        let committed = self
            .scheduler
            .commit(transaction, self.clock.now())
            .map_err(|_| runtime_error())?;
        ensure(
            committed.generation().get() == expected_generation && committed.mutation_count() == 1,
            runtime_error,
        )?;
        let (input, target) = operation_trace(operation);
        let mut headless = step(
            HeadlessTraceStageV1::Transaction,
            input,
            HeadlessOutcomeV1::Published,
        );
        headless.records_publication = true;
        headless.target = target;
        self.record_both(
            SchedulerTraceStepV1::Commit(SchedulerTraceCommitOutcomeV1::Published),
            headless,
        )?;
        self.record_projection(committed.generation())
    }

    pub(super) fn next_action(
        &mut self,
        tick: u64,
        mut headless: HeadlessTraceStep,
        publishes: bool,
    ) -> Result<ActionAttempt, HeadlessRunErrorV1> {
        self.advance_to(tick)?;
        let result = self.scheduler.next_action(self.clock.now());
        headless.records_publication = publishes && result.is_ok();
        if let Ok(Some(action)) = &result {
            match action {
                SchedulerAction::OfferFrame(frame) => headless.frame = Some(frame.id()),
                SchedulerAction::StopRenderer(control) => headless.control = Some(*control),
                SchedulerAction::RequestFrame => {}
            }
        }
        let scheduler = match &result {
            Ok(action) => {
                SchedulerTraceStepV1::Action(SchedulerTraceActionV1::from_action(action.as_ref()))
            }
            Err(error) => {
                SchedulerTraceStepV1::Action(SchedulerTraceActionV1::Rejected(error.kind()))
            }
        };
        self.record_both(scheduler, headless)?;
        Ok(match result {
            Ok(action) => ActionAttempt::Ready(action),
            Err(error) => ActionAttempt::Failed(error),
        })
    }

    pub(super) fn frame_ready(&mut self, tick: u64) -> Result<(), HeadlessRunErrorV1> {
        self.advance_to(tick)?;
        let delivery = self
            .platform
            .frame_ready(&mut self.scheduler, self.clock.now())
            .map_err(|_| scheduler_error())?;
        ensure(
            delivery == FakeFrameReadyDeliveryV1::Accepted,
            scheduler_error,
        )?;
        let scheduler = SchedulerTraceStepV1::Input {
            input: SchedulerInput::FrameReady,
            outcome: SchedulerTraceInputOutcomeV1::Accepted(SchedulerInputResult::FrameReady),
        };
        let headless = step(
            HeadlessTraceStageV1::Input,
            HeadlessInputKindV1::FrameReady,
            HeadlessOutcomeV1::Accepted,
        );
        self.record_both(scheduler, headless)
    }

    pub(super) fn offer(
        &mut self,
        frame: FrameWork,
        mode: FakeRendererModeV1,
    ) -> Result<FakeRendererOfferOutcomeV1, HeadlessRunErrorV1> {
        let frame_id = frame.id();
        let resource = headless_frame_resource_v1(&frame).map_err(|_| renderer_error())?;
        let outcome = self
            .renderer
            .offer(
                &mut self.scheduler,
                frame,
                std::slice::from_ref(&resource),
                mode,
                self.clock.now(),
            )
            .map_err(|_| renderer_error())?;
        let (scheduler_input, scheduler_outcome, headless_input, headless_outcome, control) =
            offer_projection(frame_id, outcome)?;
        let scheduler = SchedulerTraceStepV1::Input {
            input: scheduler_input,
            outcome: scheduler_outcome,
        };
        let mut headless = step(
            HeadlessTraceStageV1::Renderer,
            headless_input,
            headless_outcome,
        );
        headless.frame = Some(frame_id);
        headless.control = control;
        self.record_both(scheduler, headless)?;
        Ok(outcome)
    }
}

fn offer_projection(
    frame: fenestra_ui_runtime::prototype::FrameId,
    outcome: FakeRendererOfferOutcomeV1,
) -> Result<
    (
        SchedulerInput,
        SchedulerTraceInputOutcomeV1,
        HeadlessInputKindV1,
        HeadlessOutcomeV1,
        Option<fenestra_ui_runtime::prototype::ControlSequence>,
    ),
    HeadlessRunErrorV1,
> {
    match outcome {
        FakeRendererOfferOutcomeV1::Accepted(submission) => Ok((
            SchedulerInput::AcceptFrame(frame),
            SchedulerTraceInputOutcomeV1::Accepted(SchedulerInputResult::FrameAccepted(submission)),
            HeadlessInputKindV1::None,
            HeadlessOutcomeV1::Accepted,
            None,
        )),
        FakeRendererOfferOutcomeV1::Rejected(rejected) => {
            ensure(rejected == frame, renderer_error)?;
            Ok((
                SchedulerInput::RejectFrame(frame),
                SchedulerTraceInputOutcomeV1::Accepted(SchedulerInputResult::FrameRejected(frame)),
                HeadlessInputKindV1::None,
                HeadlessOutcomeV1::Rejected,
                None,
            ))
        }
        FakeRendererOfferOutcomeV1::Loss(FakeControlDeliveryV1::Accepted(admission)) => {
            let control = admission_sequence(admission);
            Ok((
                SchedulerInput::RendererLost(fenestra_ui_runtime::prototype::RendererEpoch::new(0)),
                SchedulerTraceInputOutcomeV1::Accepted(SchedulerInputResult::Control(admission)),
                HeadlessInputKindV1::Loss,
                HeadlessOutcomeV1::Lost,
                Some(control),
            ))
        }
        FakeRendererOfferOutcomeV1::Loss(FakeControlDeliveryV1::Retained(_))
        | FakeRendererOfferOutcomeV1::Immediate { .. } => Err(renderer_error()),
    }
}

pub(super) const fn admission_sequence(
    admission: ControlAdmission,
) -> fenestra_ui_runtime::prototype::ControlSequence {
    match admission {
        ControlAdmission::Accepted(sequence) | ControlAdmission::AlreadyAccepted(sequence) => {
            sequence
        }
    }
}

pub(super) fn accepted_submission(
    outcome: FakeRendererOfferOutcomeV1,
) -> Result<SubmissionId, HeadlessRunErrorV1> {
    match outcome {
        FakeRendererOfferOutcomeV1::Accepted(submission) => Ok(submission),
        _ => Err(renderer_error()),
    }
}
