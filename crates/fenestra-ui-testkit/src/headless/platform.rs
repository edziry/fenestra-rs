use fenestra_ui_runtime::prototype::{
    CommittedRuntimeSnapshot, HeadlessPoint, HeadlessSurface, NodeId, SchedulerError,
    SchedulerErrorKind, SchedulerTick, TransactionError, UiScheduler,
};

use crate::observe::observe_snapshot_indexed_v1;
use crate::scheduler::{
    FakeCallbackDepthV1, FakeCallbackReportV1, FakeCallbackScriptV1, FakePlatformErrorKindV1,
    FakePlatformErrorV1, FakePlatformV1,
};
use crate::semantic::PathSegmentV1;

use super::fixture::HeadlessFixtureV1;
use super::oracle::ORACLE_LIMITS;

mod types;

pub use types::{
    HeadlessCallbackReportV1, HeadlessPlatformErrorKindV1, HeadlessPlatformErrorV1,
    HeadlessPointerCaptureV1, HeadlessPointerMutationV1, HeadlessPointerScriptV1,
    HeadlessPointerTargetV1,
};

impl FakePlatformV1 {
    /// Captures one committed hit-test result without retaining its snapshot.
    pub fn capture_headless_pointer(
        &self,
        scheduler: &UiScheduler,
        fixture: &HeadlessFixtureV1,
        point: HeadlessPoint,
    ) -> Result<HeadlessPointerCaptureV1, HeadlessPlatformErrorV1> {
        capture_pointer(fixture, &scheduler.committed(), point)
    }

    /// Queries one committed pointer target and runs its bounded callback script.
    pub fn run_headless_pointer_callback(
        &mut self,
        scheduler: &mut UiScheduler,
        fixture: &HeadlessFixtureV1,
        script: HeadlessPointerScriptV1,
        tick: SchedulerTick,
    ) -> Result<HeadlessCallbackReportV1, HeadlessPlatformErrorV1> {
        ensure_callback_order(self)?;
        let capture = self.capture_headless_pointer(scheduler, fixture, script.point)?;
        let mutation = match (capture.node, script.mutation) {
            (Some(node), Some(mutation)) => Some(mutation.with_node(node)),
            _ => None,
        };
        let callback = self
            .run_callback(
                scheduler,
                FakeCallbackScriptV1::new(script.depth, mutation, false),
                tick,
            )
            .map_err(headless_fake_error)?;
        Ok(HeadlessCallbackReportV1::new(callback, capture.target))
    }

    /// Runs a mutation against an earlier opaque pointer capability.
    pub fn run_headless_captured_callback(
        &mut self,
        scheduler: &mut UiScheduler,
        capture: &HeadlessPointerCaptureV1,
        depth: FakeCallbackDepthV1,
        mutation: HeadlessPointerMutationV1,
        tick: SchedulerTick,
    ) -> Result<HeadlessCallbackReportV1, HeadlessPlatformErrorV1> {
        ensure_callback_order(self)?;
        let mutation = capture.node.map(|node| mutation.with_node(node));
        let callback = self
            .run_callback(
                scheduler,
                FakeCallbackScriptV1::new(depth, mutation, false),
                tick,
            )
            .map_err(headless_fake_error)?;
        Ok(HeadlessCallbackReportV1::new(callback, capture.target))
    }

    /// Stages one outer callback resize for deferred scheduler publication.
    pub fn run_headless_resize_callback(
        &mut self,
        scheduler: &mut UiScheduler,
        surface: HeadlessSurface,
        tick: SchedulerTick,
    ) -> Result<HeadlessCallbackReportV1, HeadlessPlatformErrorV1> {
        ensure_callback_order(self)?;
        let entry = scheduler.committed();
        let generation = entry.generation();
        let mut scope = scheduler
            .begin_callback(tick)
            .map_err(headless_scheduler_error)?;
        let shares_entry_snapshot = scope.committed().shares_state_with(&entry);
        scope
            .transaction()
            .resize_headless(surface)
            .map_err(headless_transaction_error)?;
        let finish = scope.finish().map_err(headless_scheduler_error)?;
        let callback =
            FakeCallbackReportV1::from_parts(generation, 1, shares_entry_snapshot, finish);
        Ok(HeadlessCallbackReportV1::new(
            callback,
            HeadlessPointerTargetV1::None,
        ))
    }
}

fn capture_pointer(
    fixture: &HeadlessFixtureV1,
    snapshot: &CommittedRuntimeSnapshot,
    point: HeadlessPoint,
) -> Result<HeadlessPointerCaptureV1, HeadlessPlatformErrorV1> {
    let generation = snapshot.generation();
    let projection = snapshot
        .headless_projection()
        .ok_or_else(headless_unavailable_error)?;
    if projection.generation() != generation {
        return Err(identity_error());
    }
    let node = projection.hit_test(point);
    let target = node.map_or(Ok(HeadlessPointerTargetV1::None), |node| {
        pointer_target(fixture, snapshot, node)
    })?;
    Ok(HeadlessPointerCaptureV1 {
        generation,
        target,
        node,
    })
}

fn pointer_target(
    fixture: &HeadlessFixtureV1,
    snapshot: &CommittedRuntimeSnapshot,
    node: NodeId,
) -> Result<HeadlessPointerTargetV1, HeadlessPlatformErrorV1> {
    let indexed =
        observe_snapshot_indexed_v1(fixture.style().construction(), snapshot, ORACLE_LIMITS)
            .map_err(|_| identity_error())?;
    let path = indexed
        .identities()
        .node_path(node)
        .ok_or_else(identity_error)?;
    match path.segments() {
        [
            PathSegmentV1::Static { authored_slot: 0 },
            PathSegmentV1::Static { authored_slot: 0 },
        ] => Ok(HeadlessPointerTargetV1::StaticControl),
        [
            PathSegmentV1::Static { authored_slot: 0 },
            PathSegmentV1::Member {
                region_slot: 1,
                key,
            },
        ] => Ok(HeadlessPointerTargetV1::Key(*key)),
        _ => Err(identity_error()),
    }
}

fn ensure_callback_order(platform: &FakePlatformV1) -> Result<(), HeadlessPlatformErrorV1> {
    if platform.has_pending_frame_ready() {
        Err(HeadlessPlatformErrorV1::new(
            HeadlessPlatformErrorKindV1::Scheduler(SchedulerErrorKind::InputOutOfOrder),
            None,
        ))
    } else {
        Ok(())
    }
}

fn headless_fake_error(error: FakePlatformErrorV1) -> HeadlessPlatformErrorV1 {
    match error.kind() {
        FakePlatformErrorKindV1::Scheduler(kind) => HeadlessPlatformErrorV1::new(
            HeadlessPlatformErrorKindV1::Scheduler(kind),
            error.operation_index(),
        ),
    }
}

fn headless_scheduler_error(error: SchedulerError) -> HeadlessPlatformErrorV1 {
    HeadlessPlatformErrorV1::new(
        HeadlessPlatformErrorKindV1::Scheduler(error.kind()),
        error.operation_index(),
    )
}

fn headless_transaction_error(error: TransactionError) -> HeadlessPlatformErrorV1 {
    HeadlessPlatformErrorV1::new(
        HeadlessPlatformErrorKindV1::Scheduler(SchedulerErrorKind::Transaction(error.kind())),
        error.operation_index(),
    )
}

const fn headless_unavailable_error() -> HeadlessPlatformErrorV1 {
    HeadlessPlatformErrorV1::new(HeadlessPlatformErrorKindV1::HeadlessUnavailable, None)
}

const fn identity_error() -> HeadlessPlatformErrorV1 {
    HeadlessPlatformErrorV1::new(HeadlessPlatformErrorKindV1::IdentityMismatch, None)
}
