use super::super::super::error::{
    HeadlessArtifactDecodeErrorKindV1, HeadlessArtifactDecodeErrorV1,
};
use super::super::scan::ScannedArtifactV1;
use super::super::state::LayoutV1;
use crate::headless::artifact::model::HeadlessArtifactV1;
use crate::headless::artifact::record::QueueRecordV1;
use crate::headless::artifact::record::scheduler::LaneRecordV1;

pub(super) fn validate_observations_v1(
    artifact: &HeadlessArtifactV1,
    scanned: &ScannedArtifactV1<'_>,
    layout: LayoutV1,
) -> Result<(), HeadlessArtifactDecodeErrorV1> {
    let scheduler = artifact.capacities.scheduler;
    let renderer = artifact.capacities.renderer;
    for (index, event) in artifact.scheduler_events.iter().enumerate() {
        let valid = valid_lane(event.deferred, event.tick, limits(&scheduler, 0), 80)
            && valid_lane(event.controls, event.tick, limits(&scheduler, 3), 32)
            && valid_lane(event.visual, event.tick, limits(&scheduler, 6), 40)
            && valid_lane(event.in_flight, event.tick, limits(&scheduler, 9), 40)
            && valid_lane(
                LaneRecordV1 {
                    items: event.renderer.items,
                    bytes: event.renderer.bytes,
                    residence: event.renderer.residence,
                },
                event.tick,
                QueueLimitsV1 {
                    items: renderer[0],
                    bytes: renderer[1],
                    residence: renderer[2],
                },
                96,
            )
            && (event.renderer.items == 0 || event.renderer.last.is_some());
        if !valid {
            return Err(invalid(
                scanned.lines()[layout.scheduler.records_start + index].number,
            ));
        }
    }
    for (index, event) in artifact.headless_events.iter().enumerate() {
        let valid = valid_queue(event.deferred, limits(&scheduler, 0), 80)
            && valid_queue(event.controls, limits(&scheduler, 3), 32)
            && valid_queue(event.visual, limits(&scheduler, 6), 40)
            && valid_queue(event.in_flight, limits(&scheduler, 9), 40)
            && valid_queue(
                event.renderer,
                QueueLimitsV1 {
                    items: renderer[0],
                    bytes: renderer[1],
                    residence: renderer[2],
                },
                96,
            );
        if !valid {
            return Err(invalid(
                scanned.lines()[layout.headless.records_start + index].number,
            ));
        }
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct QueueLimitsV1 {
    items: usize,
    bytes: usize,
    residence: usize,
}

fn limits(values: &[usize; 12], start: usize) -> QueueLimitsV1 {
    QueueLimitsV1 {
        items: values[start],
        bytes: values[start + 1],
        residence: values[start + 2],
    }
}

fn valid_queue(value: QueueRecordV1, limits: QueueLimitsV1, weight: usize) -> bool {
    value.items <= limits.items
        && value.bytes <= limits.bytes
        && value.items.checked_mul(weight) == Some(value.bytes)
}

fn valid_lane(value: LaneRecordV1, tick: u64, limits: QueueLimitsV1, weight: usize) -> bool {
    if !valid_queue(
        QueueRecordV1 {
            items: value.items,
            bytes: value.bytes,
        },
        limits,
        weight,
    ) {
        return false;
    }
    match (value.items, value.residence) {
        (0, None) => true,
        (0, Some(_)) | (_, None) => false,
        (_, Some(residence)) => {
            residence <= tick
                && usize::try_from(residence).is_ok_and(|value| value <= limits.residence)
        }
    }
}

fn invalid(line: u32) -> HeadlessArtifactDecodeErrorV1 {
    HeadlessArtifactDecodeErrorV1::at(HeadlessArtifactDecodeErrorKindV1::InvalidReference, line)
}
