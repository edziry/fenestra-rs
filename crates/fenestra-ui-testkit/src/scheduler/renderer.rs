mod ledger;
mod types;

use fenestra_ui_runtime::prototype::{
    CompletionWatermark, FrameId, FrameWork, RendererEpoch, SchedulerError, SchedulerErrorKind,
    SchedulerInput, SchedulerInputResult, SchedulerLane, SchedulerTick, SubmissionId, UiScheduler,
};

use ledger::RetirementLedgerV1;
pub use types::{
    FakeControlDeliveryV1, FakeRendererCapacityV1, FakeRendererErrorKindV1, FakeRendererErrorV1,
    FakeRendererModeV1, FakeRendererOfferOutcomeV1, FakeRendererStatsV1, SyntheticResourceIdV1,
    SyntheticResourceUseV1,
};

#[derive(Clone, Copy, Eq, PartialEq)]
enum PendingControlV1 {
    Completion(CompletionWatermark),
    Loss(RendererEpoch),
}

/// Deterministic bounded renderer adapter for scheduler experiments.
pub struct FakeRendererV1 {
    epoch: RendererEpoch,
    capacity: FakeRendererCapacityV1,
    ledger: RetirementLedgerV1,
    last_accepted: Option<SubmissionId>,
    completed: Option<CompletionWatermark>,
    pending_control: Option<PendingControlV1>,
    residence_exceeded: bool,
    last_tick: Option<SchedulerTick>,
}

impl FakeRendererV1 {
    /// Creates an empty fake renderer in one explicit epoch.
    #[must_use]
    pub fn new(epoch: RendererEpoch, capacity: FakeRendererCapacityV1) -> Self {
        Self {
            epoch,
            capacity,
            ledger: RetirementLedgerV1::new(),
            last_accepted: None,
            completed: None,
            pending_control: None,
            residence_exceeded: false,
            last_tick: None,
        }
    }

    /// Processes one outstanding frame offer according to a closed script mode.
    pub fn offer(
        &mut self,
        scheduler: &mut UiScheduler,
        frame: FrameWork,
        resources: &[SyntheticResourceUseV1],
        mode: FakeRendererModeV1,
        tick: SchedulerTick,
    ) -> Result<FakeRendererOfferOutcomeV1, FakeRendererErrorV1> {
        self.observe_tick(tick)?;
        self.ensure_no_pending_control()?;
        match mode {
            FakeRendererModeV1::Fail => {
                self.reject_offer(scheduler, frame.id(), tick)?;
                Ok(FakeRendererOfferOutcomeV1::Rejected(frame.id()))
            }
            FakeRendererModeV1::Loss => {
                let delivery = self.deliver_loss(scheduler, tick)?;
                Ok(FakeRendererOfferOutcomeV1::Loss(delivery))
            }
            FakeRendererModeV1::Late | FakeRendererModeV1::Immediate => {
                if let Err(error) = self.ensure_offer_residence(tick) {
                    self.reject_offer(scheduler, frame.id(), tick)?;
                    return Err(error);
                }
                let mut projected = match self.ledger.project_offer(resources, tick, self.capacity)
                {
                    Ok(projected) => projected,
                    Err(error) => {
                        self.reject_offer(scheduler, frame.id(), tick)?;
                        return Err(error);
                    }
                };
                let result = scheduler
                    .process_input(SchedulerInput::AcceptFrame(frame.id()), tick)
                    .map_err(scheduler_error)?;
                let SchedulerInputResult::FrameAccepted(submission) = result else {
                    return Err(input_order_error());
                };
                projected.bind_submission(resources, submission);
                self.ledger = projected;
                self.last_accepted = Some(submission);

                if mode == FakeRendererModeV1::Immediate {
                    let completion = self.deliver_completion(
                        scheduler,
                        CompletionWatermark::from_submission(submission),
                        tick,
                    )?;
                    Ok(FakeRendererOfferOutcomeV1::Immediate {
                        submission,
                        completion,
                    })
                } else {
                    Ok(FakeRendererOfferOutcomeV1::Accepted(submission))
                }
            }
        }
    }

    /// Admits one ordered completion and releases its fake retirement prefix.
    pub fn complete(
        &mut self,
        scheduler: &mut UiScheduler,
        watermark: CompletionWatermark,
        tick: SchedulerTick,
    ) -> Result<FakeControlDeliveryV1, FakeRendererErrorV1> {
        self.observe_tick(tick)?;
        self.ensure_no_pending_control()?;
        self.deliver_completion(scheduler, watermark, tick)
    }

    /// Retries the single renderer control retained under runtime backpressure.
    pub fn retry_control(
        &mut self,
        scheduler: &mut UiScheduler,
        tick: SchedulerTick,
    ) -> Result<FakeControlDeliveryV1, FakeRendererErrorV1> {
        self.observe_tick(tick)?;
        match self.pending_control {
            Some(PendingControlV1::Completion(watermark)) => {
                self.deliver_completion(scheduler, watermark, tick)
            }
            Some(PendingControlV1::Loss(epoch)) => {
                self.deliver_loss_for_epoch(scheduler, epoch, tick)
            }
            None => Err(input_order_error()),
        }
    }

    fn deliver_completion(
        &mut self,
        scheduler: &mut UiScheduler,
        watermark: CompletionWatermark,
        tick: SchedulerTick,
    ) -> Result<FakeControlDeliveryV1, FakeRendererErrorV1> {
        let projected = self.ledger.project_completion(watermark)?;
        let result = match scheduler.process_input(SchedulerInput::Complete(watermark), tick) {
            Ok(result) => result,
            Err(error) => {
                return self.retain_or_error(PendingControlV1::Completion(watermark), error);
            }
        };
        let SchedulerInputResult::Control(admission) = result else {
            return Err(input_order_error());
        };
        self.ledger = projected;
        self.completed = Some(watermark);
        self.pending_control = None;
        Ok(FakeControlDeliveryV1::Accepted(admission))
    }

    fn deliver_loss(
        &mut self,
        scheduler: &mut UiScheduler,
        tick: SchedulerTick,
    ) -> Result<FakeControlDeliveryV1, FakeRendererErrorV1> {
        self.deliver_loss_for_epoch(scheduler, self.epoch, tick)
    }

    fn deliver_loss_for_epoch(
        &mut self,
        scheduler: &mut UiScheduler,
        epoch: RendererEpoch,
        tick: SchedulerTick,
    ) -> Result<FakeControlDeliveryV1, FakeRendererErrorV1> {
        let result = match scheduler.process_input(SchedulerInput::RendererLost(epoch), tick) {
            Ok(result) => result,
            Err(error) => return self.retain_or_error(PendingControlV1::Loss(epoch), error),
        };
        let SchedulerInputResult::Control(admission) = result else {
            return Err(input_order_error());
        };
        self.pending_control = None;
        Ok(FakeControlDeliveryV1::Accepted(admission))
    }

    fn retain_or_error(
        &mut self,
        control: PendingControlV1,
        error: SchedulerError,
    ) -> Result<FakeControlDeliveryV1, FakeRendererErrorV1> {
        let kind = error.kind();
        if kind == SchedulerErrorKind::CapacityExceeded(SchedulerLane::Controls) {
            self.pending_control = Some(control);
            Ok(FakeControlDeliveryV1::Retained(kind))
        } else {
            Err(scheduler_error(error))
        }
    }

    /// Returns bounded fake retirement accounting.
    #[must_use]
    pub fn stats(&self) -> FakeRendererStatsV1 {
        FakeRendererStatsV1 {
            items: self.ledger.items(),
            accounted_bytes: self.ledger.accounted_bytes(),
            earliest_tick: self.ledger.earliest_tick(),
            latest_tick: self.ledger.latest_tick(),
            last_accepted: self.last_accepted,
            completed: self.completed,
            has_pending_control: self.pending_control.is_some(),
        }
    }

    fn ensure_no_pending_control(&self) -> Result<(), FakeRendererErrorV1> {
        if self.pending_control.is_some() {
            Err(input_order_error())
        } else {
            Ok(())
        }
    }

    fn observe_tick(&mut self, tick: SchedulerTick) -> Result<(), FakeRendererErrorV1> {
        if self.last_tick.is_some_and(|last| tick < last) {
            return Err(FakeRendererErrorV1::new(
                FakeRendererErrorKindV1::Scheduler(SchedulerErrorKind::TickRegression),
            ));
        }
        self.last_tick = Some(tick);
        Ok(())
    }

    fn ensure_offer_residence(&mut self, tick: SchedulerTick) -> Result<(), FakeRendererErrorV1> {
        if self.residence_exceeded {
            return Err(FakeRendererErrorV1::new(
                FakeRendererErrorKindV1::ResidenceExceeded,
            ));
        }
        let Some(earliest) = self.ledger.earliest_tick() else {
            return Ok(());
        };
        let age = tick.get().checked_sub(earliest.get()).ok_or_else(|| {
            FakeRendererErrorV1::new(FakeRendererErrorKindV1::Scheduler(
                SchedulerErrorKind::TickRegression,
            ))
        })?;
        if age > self.capacity.residence_ticks() {
            self.residence_exceeded = true;
            return Err(FakeRendererErrorV1::new(
                FakeRendererErrorKindV1::ResidenceExceeded,
            ));
        }
        Ok(())
    }

    fn reject_offer(
        &self,
        scheduler: &mut UiScheduler,
        frame: FrameId,
        tick: SchedulerTick,
    ) -> Result<(), FakeRendererErrorV1> {
        let result = scheduler
            .process_input(SchedulerInput::RejectFrame(frame), tick)
            .map_err(scheduler_error)?;
        if result == SchedulerInputResult::FrameRejected(frame) {
            Ok(())
        } else {
            Err(input_order_error())
        }
    }
}

fn scheduler_error(error: SchedulerError) -> FakeRendererErrorV1 {
    FakeRendererErrorV1::new(FakeRendererErrorKindV1::Scheduler(error.kind()))
}

fn input_order_error() -> FakeRendererErrorV1 {
    FakeRendererErrorV1::new(FakeRendererErrorKindV1::Scheduler(
        SchedulerErrorKind::InputOutOfOrder,
    ))
}
