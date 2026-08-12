use fenestra_ui_runtime::prototype::{
    CompletionWatermark, FrameWork, SchedulerInput, SchedulerInputResult, SchedulerTick,
    SubmissionId, UiScheduler,
};

use super::port::SpatialPresenterPortV2;
use super::types::{
    SpatialPresentErrorKindV2, SpatialPresentErrorV2, SpatialPresentationOutcomeV2,
    SpatialPresentationReceiptV2, SpatialSurfaceTupleV2,
};

pub(crate) fn present_spatial_offer_v2<P: SpatialPresenterPortV2>(
    scheduler: &mut UiScheduler,
    work: &FrameWork,
    surface: SpatialSurfaceTupleV2,
    presenter: &mut P,
    tick: SchedulerTick,
) -> Result<SpatialPresentationOutcomeV2, SpatialPresentErrorV2> {
    if surface.physical().is_zero() {
        reject_offer(scheduler, work, tick)?;
        return Ok(SpatialPresentationOutcomeV2::Suspended);
    }
    let Some(frame) = work.paint_frame() else {
        return reject(scheduler, work, tick, SpatialPresentErrorKindV2::Invariant);
    };
    if frame.generation() != work.generation() {
        return reject(scheduler, work, tick, SpatialPresentErrorKindV2::Invariant);
    }
    if frame.spatial().viewport() != surface.logical() {
        return reject(
            scheduler,
            work,
            tick,
            SpatialPresentErrorKindV2::ViewportMismatch,
        );
    }
    let mut accepted = None;
    let result = presenter.present_offer(frame, surface, || {
        let result = scheduler
            .process_input(SchedulerInput::AcceptFrame(work.id()), tick)
            .map_err(|_| SpatialPresentErrorKindV2::Scheduler)?;
        let SchedulerInputResult::FrameAccepted(submission) = result else {
            return Err(SpatialPresentErrorKindV2::Invariant);
        };
        accepted = Some(submission);
        Ok(submission)
    });
    match (result, accepted) {
        (Ok(digest), Some(submission)) => {
            let result = scheduler
                .process_input(
                    SchedulerInput::Complete(CompletionWatermark::from_submission(submission)),
                    tick,
                )
                .map_err(|_| error(SpatialPresentErrorKindV2::Scheduler, Some(submission)))?;
            if !matches!(result, SchedulerInputResult::Control(_)) {
                return Err(error(
                    SpatialPresentErrorKindV2::Invariant,
                    Some(submission),
                ));
            }
            Ok(SpatialPresentationOutcomeV2::Completed(
                SpatialPresentationReceiptV2::new(work.generation(), digest),
            ))
        }
        (Ok(_), None) => reject(scheduler, work, tick, SpatialPresentErrorKindV2::Invariant),
        (Err(kind), Some(submission)) => {
            let result = scheduler
                .process_input(SchedulerInput::RendererLost(submission.epoch()), tick)
                .map_err(|_| error(SpatialPresentErrorKindV2::Scheduler, Some(submission)))?;
            if !matches!(result, SchedulerInputResult::Control(_)) {
                return Err(error(
                    SpatialPresentErrorKindV2::Invariant,
                    Some(submission),
                ));
            }
            Err(error(kind, Some(submission)))
        }
        (Err(kind), None) => reject(scheduler, work, tick, kind),
    }
}

fn reject_offer(
    scheduler: &mut UiScheduler,
    work: &FrameWork,
    tick: SchedulerTick,
) -> Result<(), SpatialPresentErrorV2> {
    let result = scheduler
        .process_input(SchedulerInput::RejectFrame(work.id()), tick)
        .map_err(|_| error(SpatialPresentErrorKindV2::Scheduler, None))?;
    if !matches!(result, SchedulerInputResult::FrameRejected(_)) {
        return Err(error(SpatialPresentErrorKindV2::Invariant, None));
    }
    Ok(())
}

fn reject(
    scheduler: &mut UiScheduler,
    work: &FrameWork,
    tick: SchedulerTick,
    kind: SpatialPresentErrorKindV2,
) -> Result<SpatialPresentationOutcomeV2, SpatialPresentErrorV2> {
    reject_offer(scheduler, work, tick)?;
    Err(error(kind, None))
}

const fn error(
    kind: SpatialPresentErrorKindV2,
    accepted: Option<SubmissionId>,
) -> SpatialPresentErrorV2 {
    SpatialPresentErrorV2::new(kind, accepted)
}
