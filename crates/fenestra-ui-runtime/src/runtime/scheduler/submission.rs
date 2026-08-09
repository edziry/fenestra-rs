use super::frame::{SubmittedFrame, VisualState};
use super::{
    CompletionWatermark, FrameId, FrameWork, SchedulerError, SchedulerErrorKind,
    SchedulerInputResult, SchedulerTick, SubmissionId, UiScheduler, VISUAL_ENVELOPE_BYTES,
};

impl UiScheduler {
    pub(super) fn frame_ready(&mut self) -> Result<SchedulerInputResult, SchedulerError> {
        let state = self.visual.take();
        match state {
            Some(VisualState::RequestOutstanding {
                work,
                action_pending: false,
            }) => {
                self.visual = Some(VisualState::PendingPublication(work));
                Ok(SchedulerInputResult::FrameReady)
            }
            state => {
                self.visual = state;
                Err(SchedulerError::new(
                    SchedulerErrorKind::InputOutOfOrder,
                    None,
                ))
            }
        }
    }

    pub(super) fn reject_frame(
        &mut self,
        frame: FrameId,
    ) -> Result<SchedulerInputResult, SchedulerError> {
        let offered = self.take_offer(frame)?;
        self.visual = Some(VisualState::PendingPublication(offered.into()));
        Ok(SchedulerInputResult::FrameRejected(frame))
    }

    pub(super) fn accept_frame(
        &mut self,
        frame: FrameId,
        tick: SchedulerTick,
    ) -> Result<SchedulerInputResult, SchedulerError> {
        self.ensure_offer(frame)?;
        let Some(in_flight_bytes) = self.next_in_flight_bytes()? else {
            return Err(SchedulerError::new(
                SchedulerErrorKind::ControlPending,
                None,
            ));
        };
        let token = self
            .next_submission_token
            .ok_or_else(|| SchedulerError::new(SchedulerErrorKind::ArithmeticExhausted, None))?;
        let submission = SubmissionId::new(self.renderer_epoch, token);
        let offered = self.take_offer(frame)?;
        self.in_flight.push_back(SubmittedFrame {
            submission,
            accepted_tick: tick,
            _snapshot: offered.snapshot().clone(),
        });
        self.in_flight_bytes = in_flight_bytes;
        self.last_accepted_token = Some(token);
        self.next_submission_token = token.checked_add(1);
        Ok(SchedulerInputResult::FrameAccepted(submission))
    }

    pub(super) fn validate_completion_watermark(
        &self,
        watermark: CompletionWatermark,
    ) -> Result<(), SchedulerError> {
        if watermark.epoch() != self.renderer_epoch {
            return Err(SchedulerError::new(
                SchedulerErrorKind::ForeignRendererEpoch,
                None,
            ));
        }
        if self
            .last_accepted_token
            .is_none_or(|accepted| watermark.token() > accepted)
        {
            return Err(SchedulerError::new(
                SchedulerErrorKind::CompletionBeyondAccepted,
                None,
            ));
        }
        if let Some(completed) = self.completed_token
            && watermark.token() < completed
        {
            return Err(SchedulerError::new(
                SchedulerErrorKind::CompletionRegression,
                None,
            ));
        }
        Ok(())
    }

    pub(super) fn apply_completion(
        &mut self,
        watermark: CompletionWatermark,
    ) -> Result<(), SchedulerError> {
        let released = self
            .in_flight
            .iter()
            .take_while(|frame| frame.submission.token() <= watermark.token())
            .count();
        let released_bytes = released
            .checked_mul(VISUAL_ENVELOPE_BYTES)
            .ok_or_else(|| SchedulerError::new(SchedulerErrorKind::ArithmeticExhausted, None))?;
        let remaining_bytes = self
            .in_flight_bytes
            .checked_sub(released_bytes)
            .ok_or_else(|| SchedulerError::new(SchedulerErrorKind::ArithmeticExhausted, None))?;
        self.in_flight.drain(..released);
        self.in_flight_bytes = remaining_bytes;
        self.completed_token = Some(watermark.token());
        Ok(())
    }

    fn ensure_offer(&self, frame: FrameId) -> Result<(), SchedulerError> {
        match self.visual.as_ref() {
            Some(VisualState::OfferAwaitingDisposition(offered)) if offered.id() == frame => Ok(()),
            Some(VisualState::OfferAwaitingDisposition(_)) => Err(SchedulerError::new(
                SchedulerErrorKind::FrameIdMismatch,
                None,
            )),
            _ => Err(SchedulerError::new(
                SchedulerErrorKind::InputOutOfOrder,
                None,
            )),
        }
    }

    fn take_offer(&mut self, frame: FrameId) -> Result<FrameWork, SchedulerError> {
        self.ensure_offer(frame)?;
        let Some(VisualState::OfferAwaitingDisposition(offered)) = self.visual.take() else {
            return Err(SchedulerError::new(
                SchedulerErrorKind::InputOutOfOrder,
                None,
            ));
        };
        Ok(offered)
    }

    pub(super) fn next_in_flight_bytes(&self) -> Result<Option<usize>, SchedulerError> {
        let items =
            self.in_flight.len().checked_add(1).ok_or_else(|| {
                SchedulerError::new(SchedulerErrorKind::ArithmeticExhausted, None)
            })?;
        let bytes = self
            .in_flight_bytes
            .checked_add(VISUAL_ENVELOPE_BYTES)
            .ok_or_else(|| SchedulerError::new(SchedulerErrorKind::ArithmeticExhausted, None))?;
        let capacity = self.capacity.in_flight();
        Ok((items <= capacity.max_items() && bytes <= capacity.max_bytes()).then_some(bytes))
    }

    pub(super) fn allocate_frame_id(&mut self) -> Result<FrameId, SchedulerError> {
        let value = self
            .next_frame_id
            .ok_or_else(|| SchedulerError::new(SchedulerErrorKind::ArithmeticExhausted, None))?;
        self.next_frame_id = value.checked_add(1);
        Ok(FrameId::new(value))
    }
}
