use super::super::super::error::{
    HeadlessArtifactDecodeErrorKindV1, HeadlessArtifactDecodeErrorV1,
};
use crate::headless::artifact::record::scheduler::{
    ActionRecordV1, AdmissionRecordV1, InputOutcomeRecordV1, InputRecordV1, InputResultRecordV1,
    StepRecordV1,
};

pub(super) fn validate_common_fields_v1(
    step: StepRecordV1,
    frame: Option<u64>,
    control: Option<u64>,
    line: u32,
) -> Result<(), HeadlessArtifactDecodeErrorV1> {
    if common_fields(step) == (frame, control) {
        Ok(())
    } else {
        Err(HeadlessArtifactDecodeErrorV1::at(
            HeadlessArtifactDecodeErrorKindV1::MalformedRecord,
            line,
        ))
    }
}

const fn common_fields(step: StepRecordV1) -> (Option<u64>, Option<u64>) {
    let frame = match step {
        StepRecordV1::Input {
            input: InputRecordV1::AcceptFrame(frame) | InputRecordV1::RejectFrame(frame),
            ..
        }
        | StepRecordV1::Action(ActionRecordV1::OfferFrame(frame)) => Some(frame),
        _ => None,
    };
    let control = match step {
        StepRecordV1::Input {
            outcome:
                InputOutcomeRecordV1::Accepted(InputResultRecordV1::Control(
                    AdmissionRecordV1::Accepted(control)
                    | AdmissionRecordV1::AlreadyAccepted(control),
                )),
            ..
        }
        | StepRecordV1::Action(ActionRecordV1::StopRenderer(control)) => Some(control),
        _ => None,
    };
    (frame, control)
}
