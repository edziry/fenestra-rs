use super::super::super::error::{
    HeadlessArtifactDecodeErrorKindV1, HeadlessArtifactDecodeErrorV1,
};
use super::super::scan::ScannedArtifactV1;
use super::super::state::LayoutV1;
use crate::headless::artifact::model::HeadlessArtifactV1;
use crate::headless::artifact::record::scheduler::{
    ActionRecordV1, AdmissionRecordV1, InputOutcomeRecordV1, InputRecordV1, InputResultRecordV1,
    SchedulerEventRecordV1, StepRecordV1, SubmissionRecordV1,
};

pub(super) fn validate_scheduler_references_v1(
    artifact: &HeadlessArtifactV1,
    scanned: &ScannedArtifactV1<'_>,
    layout: LayoutV1,
) -> Result<(), HeadlessArtifactDecodeErrorV1> {
    let mut state = SchedulerReferencesV1::default();
    for (index, event) in artifact.scheduler_events.iter().enumerate() {
        let line = scanned.lines()[layout.scheduler.records_start + index].number;
        state.apply(event).map_err(|()| invalid(line))?;
        validate_renderer(event, &state).map_err(|()| invalid(line))?;
    }
    Ok(())
}

#[derive(Default)]
struct SchedulerReferencesV1 {
    next_frame: u64,
    offered: Option<u64>,
    submission_count: usize,
    next_control: u64,
    last_control: Option<u64>,
}

impl SchedulerReferencesV1 {
    fn apply(&mut self, event: &SchedulerEventRecordV1) -> Result<(), ()> {
        match event.step {
            StepRecordV1::Action(ActionRecordV1::OfferFrame(frame)) => {
                if event.frame != Some(frame) || frame != self.next_frame {
                    return Err(());
                }
                self.next_frame = self.next_frame.checked_add(1).ok_or(())?;
                self.offered = Some(frame);
            }
            StepRecordV1::Action(ActionRecordV1::StopRenderer(control)) => {
                if event.control != Some(control) || Some(control) != self.last_control {
                    return Err(());
                }
            }
            StepRecordV1::Input { input, outcome } => self.input(event, input, outcome)?,
            StepRecordV1::Callback { .. } | StepRecordV1::Commit(_) | StepRecordV1::Action(_) => {}
        }
        Ok(())
    }

    fn input(
        &mut self,
        event: &SchedulerEventRecordV1,
        input: InputRecordV1,
        outcome: InputOutcomeRecordV1,
    ) -> Result<(), ()> {
        match (input, outcome) {
            (
                InputRecordV1::AcceptFrame(frame),
                InputOutcomeRecordV1::Accepted(InputResultRecordV1::FrameAccepted { epoch, token }),
            ) => {
                if event.frame != Some(frame) || self.offered != Some(frame) {
                    return Err(());
                }
                let expected = u64::try_from(self.submission_count).map_err(|_| ())?;
                if epoch != 0 || token != expected {
                    return Err(());
                }
                self.submission_count = self.submission_count.checked_add(1).ok_or(())?;
                self.offered = None;
            }
            (
                InputRecordV1::RejectFrame(frame),
                InputOutcomeRecordV1::Accepted(InputResultRecordV1::FrameRejected(rejected)),
            ) => {
                if frame != rejected || event.frame != Some(frame) || self.offered != Some(frame) {
                    return Err(());
                }
                self.offered = None;
            }
            (
                InputRecordV1::Complete { epoch, token },
                InputOutcomeRecordV1::Accepted(InputResultRecordV1::Control(admission)),
            ) => {
                self.require_submission(SubmissionRecordV1 { epoch, token })?;
                self.admit(event, admission)?;
            }
            (
                InputRecordV1::RendererLost(epoch),
                InputOutcomeRecordV1::Accepted(InputResultRecordV1::Control(admission)),
            ) => {
                if self.submission_count != 0 && epoch != 0 {
                    return Err(());
                }
                self.admit(event, admission)?;
            }
            (
                InputRecordV1::RequestShutdown,
                InputOutcomeRecordV1::Accepted(InputResultRecordV1::Control(admission)),
            ) => self.admit(event, admission)?,
            (InputRecordV1::FrameReady, _) => {}
            _ => {}
        }
        Ok(())
    }

    fn admit(
        &mut self,
        event: &SchedulerEventRecordV1,
        admission: AdmissionRecordV1,
    ) -> Result<(), ()> {
        match admission {
            AdmissionRecordV1::Accepted(control) => {
                if event.control != Some(control) || control != self.next_control {
                    return Err(());
                }
                self.next_control = self.next_control.checked_add(1).ok_or(())?;
                self.last_control = Some(control);
            }
            AdmissionRecordV1::AlreadyAccepted(control) => {
                if event.control != Some(control) || Some(control) != self.last_control {
                    return Err(());
                }
            }
        }
        Ok(())
    }

    fn require_submission(&self, value: SubmissionRecordV1) -> Result<(), ()> {
        let token = usize::try_from(value.token).map_err(|_| ())?;
        if value.epoch == 0 && token < self.submission_count {
            Ok(())
        } else {
            Err(())
        }
    }
}

fn validate_renderer(
    event: &SchedulerEventRecordV1,
    state: &SchedulerReferencesV1,
) -> Result<(), ()> {
    if let Some(last) = event.renderer.last {
        state.require_submission(last)?;
    }
    if let Some(completed) = event.renderer.completed {
        state.require_submission(completed)?;
        let Some(last) = event.renderer.last else {
            return Err(());
        };
        if (completed.epoch, completed.token) > (last.epoch, last.token) {
            return Err(());
        }
    }
    Ok(())
}

fn invalid(line: u32) -> HeadlessArtifactDecodeErrorV1 {
    HeadlessArtifactDecodeErrorV1::at(HeadlessArtifactDecodeErrorKindV1::InvalidReference, line)
}
