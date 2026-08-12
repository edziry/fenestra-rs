use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, Weak};

use fenestra_ui_ir::prototype::PropertyValue;
use fenestra_ui_runtime::prototype::{
    CompletionWatermark, ControlAdmission, QueueCapacity, SchedulerAction, SchedulerCapacity,
    SchedulerInput, SchedulerInputResult, SchedulerTick, UiRuntime, UiScheduler,
};
use fenestra_ui_spatial::prototype::{SpatialNodeKeyV2, SpatialOwnedInputV2, SpatialViewportV2};

use crate::spatial_support::engine::{EnginePlan, EngineSpy, EngineState};
use crate::spatial_support::input::{SourceIdentity, canonical_source};
use crate::spatial_support::{VIEWPORT, limits, styled_program};
use crate::support::headless::{FIRST_KEY, ITEMS, SECOND_KEY, WIDTH, runtime_capacity};
use crate::{RuntimeSpatialBuildViewV2, RuntimeSpatialInputV2, RuntimeSpatialProgramV2};

#[derive(Default)]
struct SubmissionProgramState {
    calls: AtomicUsize,
    sources: Mutex<Vec<Weak<SpatialOwnedInputV2>>>,
}

impl SubmissionProgramState {
    fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }

    fn source_weaks(&self) -> Vec<Weak<SpatialOwnedInputV2>> {
        self.sources
            .lock()
            .expect("submission sources should be available")
            .clone()
    }
}

struct SubmissionProgram {
    state: Arc<SubmissionProgramState>,
}

impl SubmissionProgram {
    fn new() -> (Self, Arc<SubmissionProgramState>) {
        let state = Arc::new(SubmissionProgramState::default());
        (
            Self {
                state: Arc::clone(&state),
            },
            state,
        )
    }
}

impl RuntimeSpatialProgramV2 for SubmissionProgram {
    fn build(
        &self,
        runtime: RuntimeSpatialBuildViewV2<'_>,
        viewport: SpatialViewportV2,
    ) -> RuntimeSpatialInputV2 {
        let root = runtime.root();
        let container = runtime.children(root).expect("root should be live")[0];
        let control = runtime
            .children(container)
            .expect("container should be live")[0];
        let items = runtime
            .fragment(container, ITEMS)
            .expect("items fragment should be live");
        let first = runtime
            .keyed_member(items, FIRST_KEY)
            .expect("first item should be live");
        let second = runtime
            .keyed_member(items, SECOND_KEY)
            .expect("second item should be live");
        let call = self.state.calls.fetch_add(1, Ordering::SeqCst) + 1;
        let source = canonical_source(viewport);
        self.state
            .sources
            .lock()
            .expect("submission sources should be available")
            .push(Arc::downgrade(&source));
        let mapping = if call == 2 {
            vec![control, first, second, container]
        } else {
            vec![second, container, first, control]
        };
        RuntimeSpatialInputV2::new(source, mapping.into_boxed_slice())
    }
}

fn capacity() -> SchedulerCapacity {
    SchedulerCapacity::new(
        QueueCapacity::new(1, 80, 8),
        QueueCapacity::new(4, 128, 8),
        QueueCapacity::new(1, 40, 8),
        QueueCapacity::new(2, 80, 8),
    )
}

fn assert_callbacks(program: &SubmissionProgramState, engine: &EngineState, expected: usize) {
    assert_eq!(program.calls(), expected);
    assert_eq!(engine.calls(), expected);
}

#[test]
fn accepted_submission_retains_its_exact_spatial_generation_until_completion_is_processed() {
    let (program, program_state) = SubmissionProgram::new();
    let (engine, engine_state) = EngineSpy::new(EnginePlan::Reference);
    let runtime = UiRuntime::new_spatial_with_layout_engine(
        styled_program(),
        Box::new(program),
        VIEWPORT,
        limits(),
        runtime_capacity().with_retained_generations(4),
        Box::new(engine),
    )
    .expect("spatial runtime should initialize");
    assert_callbacks(&program_state, &engine_state, 1);
    let mut scheduler = UiScheduler::new(runtime, capacity()).expect("scheduler should initialize");
    assert_callbacks(&program_state, &engine_state, 1);

    let initial = scheduler.committed();
    let root = initial.root();
    let container = initial.children(root).expect("root should be live")[0];
    let control = initial
        .children(container)
        .expect("container should be live")[0];
    let items = initial
        .fragment(container, ITEMS)
        .expect("items fragment should be live");
    let first_item = initial
        .keyed_member(items, FIRST_KEY)
        .expect("first item should be live");
    let second_item = initial
        .keyed_member(items, SECOND_KEY)
        .expect("second item should be live");
    drop(initial);
    let mut first = scheduler.begin_transaction();
    first
        .set_property(root, WIDTH, PropertyValue::ScalarI32(83))
        .expect("first property should stage");
    scheduler
        .commit(first, SchedulerTick::new(10))
        .expect("first property should publish");
    assert_callbacks(&program_state, &engine_state, 2);

    assert_eq!(
        scheduler
            .next_action(SchedulerTick::new(10))
            .expect("request tick should be monotonic"),
        Some(SchedulerAction::RequestFrame)
    );
    assert_callbacks(&program_state, &engine_state, 2);
    assert_eq!(
        scheduler
            .process_input(SchedulerInput::FrameReady, SchedulerTick::new(11))
            .expect("frame ready should be accepted"),
        SchedulerInputResult::FrameReady
    );
    assert_callbacks(&program_state, &engine_state, 2);
    let Some(SchedulerAction::OfferFrame(offer)) = scheduler
        .next_action(SchedulerTick::new(11))
        .expect("offer tick should be monotonic")
    else {
        panic!("first generation should be offered");
    };
    assert_callbacks(&program_state, &engine_state, 2);

    let source_weaks = program_state.source_weaks();
    let submitted_source = source_weaks[1].clone();
    let source_identity = SourceIdentity::capture(
        &submitted_source
            .upgrade()
            .expect("offered generation should retain its source"),
    );
    let offered_spatial = offer
        .snapshot()
        .spatial()
        .expect("offered spatial state should exist");
    for (key, node) in [
        (1, control),
        (2, first_item),
        (3, second_item),
        (4, container),
    ] {
        let key = SpatialNodeKeyV2::new(key);
        assert_eq!(offered_spatial.logical_node(key), Some(node));
        assert_eq!(offered_spatial.spatial_key(node), Some(key));
    }
    assert_eq!(offer.generation().get(), 1);
    let accepted = scheduler
        .process_input(
            SchedulerInput::AcceptFrame(offer.id()),
            SchedulerTick::new(12),
        )
        .expect("offer should be accepted");
    let SchedulerInputResult::FrameAccepted(submission) = accepted else {
        panic!("acceptance should return a submission identity");
    };
    assert_callbacks(&program_state, &engine_state, 2);
    drop(offer);

    let mut second = scheduler.begin_transaction();
    second
        .set_property(root, WIDTH, PropertyValue::ScalarI32(84))
        .expect("second property should stage");
    scheduler
        .commit(second, SchedulerTick::new(13))
        .expect("second property should publish");
    assert_callbacks(&program_state, &engine_state, 3);
    assert_eq!(scheduler.committed().generation().get(), 2);
    let current = scheduler.committed();
    let current_spatial = current
        .spatial()
        .expect("current spatial state should exist");
    for (key, node) in [
        (1, second_item),
        (2, container),
        (3, first_item),
        (4, control),
    ] {
        let key = SpatialNodeKeyV2::new(key);
        assert_eq!(current_spatial.logical_node(key), Some(node));
        assert_eq!(current_spatial.spatial_key(node), Some(key));
    }
    drop(current);
    assert_eq!(scheduler.stats().in_flight().items(), 1);
    let retained = submitted_source
        .upgrade()
        .expect("accepted submission should retain its exact source");
    source_identity.assert_source(&retained);
    drop(retained);

    let completion = scheduler
        .process_input(
            SchedulerInput::Complete(CompletionWatermark::from_submission(submission)),
            SchedulerTick::new(14),
        )
        .expect("completion should be admitted");
    assert!(matches!(
        completion,
        SchedulerInputResult::Control(ControlAdmission::Accepted(_))
    ));
    assert_callbacks(&program_state, &engine_state, 3);
    assert!(submitted_source.upgrade().is_some());
    assert_eq!(scheduler.stats().in_flight().items(), 1);

    assert_eq!(
        scheduler
            .next_action(SchedulerTick::new(14))
            .expect("completion should process at the same tick"),
        None
    );
    assert_callbacks(&program_state, &engine_state, 3);
    assert_eq!(scheduler.stats().in_flight().items(), 0);
    assert!(submitted_source.upgrade().is_none());

    assert_eq!(
        scheduler
            .next_action(SchedulerTick::new(14))
            .expect("newer request should remain pending"),
        Some(SchedulerAction::RequestFrame)
    );
    assert_callbacks(&program_state, &engine_state, 3);
}
