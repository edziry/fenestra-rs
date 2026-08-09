use fenestra_ui_ir::prototype::{PropertyId, PropertyValue};
use fenestra_ui_runtime::prototype::{
    CommittedRuntimeSnapshot, FragmentId, NodeId, QueueCapacity, RendererEpoch, RuntimeGeneration,
    SchedulerCapacity, SchedulerTick, UiRuntime, UiScheduler, UiTransaction,
};

use crate::case::SemanticOperationV1;
use crate::headless::HeadlessPointerTargetV1;
use crate::headless::fixture::HeadlessFixtureV1;
use crate::headless::oracle::{
    HeadlessOracleV1, ORACLE_LIMITS, compare_headless_projection_v1, observe_headless_projection_v1,
};
use crate::headless::trace::{
    HeadlessFailureCauseV1, HeadlessInputKindV1, HeadlessTraceCapacityV1, HeadlessTraceStep,
    HeadlessTraceV1,
};
use crate::identity::IdentityIndexV1;
use crate::observe::observe_snapshot_indexed_v1;
use crate::scheduler::{
    FakeClockDomainV1, FakeClockV1, FakePlatformV1, FakeRendererCapacityV1, FakeRendererV1,
    SchedulerTraceCapacityV1, SchedulerTraceStepV1, SchedulerTraceV1,
};
use crate::semantic::{FragmentPathV1, NodePathV1};

use super::types::{HeadlessResultV1, HeadlessRunErrorV1, HeadlessRunV1};

pub(super) const WIDTH: PropertyId = PropertyId::new(0);
pub(super) const HEIGHT: PropertyId = PropertyId::new(1);
pub(super) const COLOR: PropertyId = PropertyId::new(2);
pub(super) const VISIBLE: PropertyId = PropertyId::new(3);
pub(super) const SECOND_KEY: u64 = 20;
pub(super) const INSERTED_KEY: u64 = 30;
pub(super) type StableNodes = [(NodePathV1, NodeId); 4];

const DOMAIN: FakeClockDomainV1 = FakeClockDomainV1::new(8_001);
const SCHEDULER_TRACE_CAPACITY: SchedulerTraceCapacityV1 =
    SchedulerTraceCapacityV1::new(256, 24_576);
const HEADLESS_TRACE_CAPACITY: HeadlessTraceCapacityV1 = HeadlessTraceCapacityV1::new(128, 20_480);
pub(super) const RENDERER_CAPACITY: FakeRendererCapacityV1 = FakeRendererCapacityV1::new(2, 192, 8);

pub(super) struct RunState {
    pub(super) fixture: HeadlessFixtureV1,
    pub(super) oracle: HeadlessOracleV1,
    pub(super) scheduler: UiScheduler,
    pub(super) platform: FakePlatformV1,
    pub(super) renderer: FakeRendererV1,
    pub(super) clock: FakeClockV1,
    pub(super) headless_trace: HeadlessTraceV1,
    pub(super) scheduler_trace: SchedulerTraceV1,
}

impl RunState {
    pub(super) fn new() -> Result<Self, HeadlessRunErrorV1> {
        let fixture = HeadlessFixtureV1::build().map_err(|_| runtime_error())?;
        let oracle = HeadlessOracleV1::new(&fixture).map_err(|_| oracle_error())?;
        let runtime = UiRuntime::new_headless(
            fixture.style().clone(),
            fixture.spec(),
            fixture.surface(),
            fixture.runtime_capacity(),
        )
        .map_err(|_| runtime_error())?;
        let scheduler =
            UiScheduler::new(runtime, scheduler_capacity()).map_err(|_| scheduler_error())?;
        Ok(Self {
            fixture,
            oracle,
            scheduler,
            platform: FakePlatformV1::new(),
            renderer: FakeRendererV1::new(RendererEpoch::new(0), RENDERER_CAPACITY),
            clock: FakeClockV1::new(DOMAIN, SchedulerTick::new(0)),
            headless_trace: HeadlessTraceV1::new(DOMAIN, HEADLESS_TRACE_CAPACITY),
            scheduler_trace: SchedulerTraceV1::new(DOMAIN, SCHEDULER_TRACE_CAPACITY),
        })
    }

    pub(super) fn advance_to(&mut self, tick: u64) -> Result<(), HeadlessRunErrorV1> {
        let delta = tick
            .checked_sub(self.clock.now().get())
            .ok_or_else(trace_error)?;
        self.clock.advance(delta).map_err(|_| trace_error())?;
        Ok(())
    }

    pub(super) fn record_headless(
        &mut self,
        step: HeadlessTraceStep,
    ) -> Result<(), HeadlessRunErrorV1> {
        self.headless_trace
            .record(&self.clock, step, &self.scheduler, &self.renderer)
            .map_err(|_| trace_error())
    }

    pub(super) fn record_both(
        &mut self,
        scheduler_step: SchedulerTraceStepV1,
        headless_step: HeadlessTraceStep,
    ) -> Result<(), HeadlessRunErrorV1> {
        self.scheduler_trace
            .record(&self.clock, scheduler_step, &self.scheduler, &self.renderer)
            .map_err(|_| trace_error())?;
        self.record_headless(headless_step)
    }

    pub(super) fn compare_projection(
        &mut self,
        generation: RuntimeGeneration,
    ) -> Result<(), HeadlessRunErrorV1> {
        let expected = self.oracle.rebuild().map_err(|_| oracle_error())?;
        let snapshot = self.scheduler.committed();
        let observed = observe_headless_projection_v1(&self.fixture, &snapshot)
            .map_err(|_| projection_error())?;
        ensure(observed.generation() == generation, projection_error)?;
        let mismatch = compare_headless_projection_v1(&expected, observed.projection())
            .map_err(|_| projection_error())?;
        ensure(mismatch.is_none(), projection_error)
    }

    pub(super) fn identities(
        &self,
        snapshot: &CommittedRuntimeSnapshot,
    ) -> Result<IdentityIndexV1, HeadlessRunErrorV1> {
        let observed = observe_snapshot_indexed_v1(
            self.fixture.style().construction(),
            snapshot,
            ORACLE_LIMITS,
        )
        .map_err(|_| projection_error())?;
        let mut identities = IdentityIndexV1::default();
        for (path, node) in observed.identities().nodes_in_authored_order() {
            ensure(identities.record_node(path.clone(), node), projection_error)?;
        }
        for (path, fragment) in observed.identities().fragments_in_authored_order() {
            ensure(
                identities.record_fragment(path.clone(), fragment),
                projection_error,
            )?;
        }
        Ok(identities)
    }

    pub(super) fn node(
        &self,
        snapshot: &CommittedRuntimeSnapshot,
        path: &NodePathV1,
    ) -> Result<NodeId, HeadlessRunErrorV1> {
        self.identities(snapshot)?
            .node(path)
            .ok_or_else(projection_error)
    }

    pub(super) fn ensure_nodes(
        &self,
        snapshot: &CommittedRuntimeSnapshot,
        expected: &[(NodePathV1, NodeId)],
    ) -> Result<(), HeadlessRunErrorV1> {
        let identities = self.identities(snapshot)?;
        for (path, node) in expected {
            ensure(identities.node(path) == Some(*node), projection_error)?;
        }
        Ok(())
    }

    pub(super) fn fragment(
        &self,
        snapshot: &CommittedRuntimeSnapshot,
        path: &FragmentPathV1,
    ) -> Result<FragmentId, HeadlessRunErrorV1> {
        self.identities(snapshot)?
            .fragment(path)
            .ok_or_else(projection_error)
    }

    pub(super) fn stage_operation(
        &self,
        transaction: &mut UiTransaction,
        snapshot: &CommittedRuntimeSnapshot,
        operation: &SemanticOperationV1,
    ) -> Result<(), HeadlessRunErrorV1> {
        match operation {
            SemanticOperationV1::SetProperty {
                node,
                property,
                value,
            } => transaction.set_property(self.node(snapshot, node)?, *property, value.clone()),
            SemanticOperationV1::InsertKeyed {
                fragment,
                key,
                final_index,
            } => transaction.insert_keyed(
                self.fragment(snapshot, fragment)?,
                *key,
                usize::try_from(*final_index).map_err(|_| runtime_error())?,
            ),
            SemanticOperationV1::MoveKeyed {
                fragment,
                key,
                final_index,
            } => transaction.move_keyed(
                self.fragment(snapshot, fragment)?,
                *key,
                usize::try_from(*final_index).map_err(|_| runtime_error())?,
            ),
            SemanticOperationV1::UpdateKeyed {
                fragment,
                key,
                property,
                value,
            } => transaction.update_keyed(
                self.fragment(snapshot, fragment)?,
                *key,
                *property,
                value.clone(),
            ),
            SemanticOperationV1::RemoveKeyed { fragment, key } => {
                transaction.remove_keyed(self.fragment(snapshot, fragment)?, *key)
            }
        }
        .map_err(|_| runtime_error())
    }

    pub(super) fn finish(self) -> Result<HeadlessRunV1, HeadlessRunErrorV1> {
        let snapshot = self.scheduler.committed();
        let observed = observe_headless_projection_v1(&self.fixture, &snapshot)
            .map_err(|_| projection_error())?;
        let expected = self.oracle.rebuild().map_err(|_| oracle_error())?;
        ensure(
            compare_headless_projection_v1(&expected, observed.projection())
                .map_err(|_| projection_error())?
                .is_none(),
            projection_error,
        )?;
        Ok(HeadlessRunV1 {
            result: HeadlessResultV1::Pass,
            final_generation: observed.generation(),
            final_projection: observed.projection().clone(),
            headless_trace: self.headless_trace,
            scheduler_trace: self.scheduler_trace,
        })
    }
}

pub(super) fn operation_trace(
    operation: &SemanticOperationV1,
) -> (HeadlessInputKindV1, HeadlessPointerTargetV1) {
    match operation {
        SemanticOperationV1::SetProperty { node, .. } if node == &control_path() => (
            HeadlessInputKindV1::Direct,
            HeadlessPointerTargetV1::StaticControl,
        ),
        SemanticOperationV1::SetProperty { .. } => {
            (HeadlessInputKindV1::Direct, HeadlessPointerTargetV1::None)
        }
        SemanticOperationV1::InsertKeyed { key, .. } => (
            HeadlessInputKindV1::Insert,
            HeadlessPointerTargetV1::Key(*key),
        ),
        SemanticOperationV1::MoveKeyed { key, .. } => (
            HeadlessInputKindV1::Move,
            HeadlessPointerTargetV1::Key(*key),
        ),
        SemanticOperationV1::UpdateKeyed { key, .. } => (
            HeadlessInputKindV1::Update,
            HeadlessPointerTargetV1::Key(*key),
        ),
        SemanticOperationV1::RemoveKeyed { key, .. } => (
            HeadlessInputKindV1::Remove,
            HeadlessPointerTargetV1::Key(*key),
        ),
    }
}

pub(super) const fn scheduler_capacity() -> SchedulerCapacity {
    SchedulerCapacity::new(
        QueueCapacity::new(1, 80, 8),
        QueueCapacity::new(4, 128, 8),
        QueueCapacity::new(1, 40, 8),
        QueueCapacity::new(2, 80, 8),
    )
}

pub(super) fn root_path() -> NodePathV1 {
    NodePathV1::root()
}
pub(super) fn container_path() -> NodePathV1 {
    root_path().static_child(0)
}
pub(super) fn control_path() -> NodePathV1 {
    container_path().static_child(0)
}
pub(super) fn item_path(key: u64) -> NodePathV1 {
    container_path().member(1, key)
}
pub(super) fn items_path() -> FragmentPathV1 {
    FragmentPathV1::new(container_path(), 1)
}
pub(super) const fn rgba(value: [u8; 4]) -> PropertyValue {
    PropertyValue::Rgba8(value)
}

pub(super) fn ensure(
    condition: bool,
    error: fn() -> HeadlessRunErrorV1,
) -> Result<(), HeadlessRunErrorV1> {
    if condition { Ok(()) } else { Err(error()) }
}

pub(super) const fn runtime_error() -> HeadlessRunErrorV1 {
    HeadlessRunErrorV1::new(HeadlessFailureCauseV1::Runtime)
}
pub(super) const fn projection_error() -> HeadlessRunErrorV1 {
    HeadlessRunErrorV1::new(HeadlessFailureCauseV1::Projection)
}
pub(super) const fn oracle_error() -> HeadlessRunErrorV1 {
    HeadlessRunErrorV1::new(HeadlessFailureCauseV1::Oracle)
}
pub(super) const fn scheduler_error() -> HeadlessRunErrorV1 {
    HeadlessRunErrorV1::new(HeadlessFailureCauseV1::Scheduler)
}
pub(super) const fn renderer_error() -> HeadlessRunErrorV1 {
    HeadlessRunErrorV1::new(HeadlessFailureCauseV1::Renderer)
}
pub(super) const fn trace_error() -> HeadlessRunErrorV1 {
    HeadlessRunErrorV1::new(HeadlessFailureCauseV1::Trace)
}
