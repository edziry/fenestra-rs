#![allow(dead_code)]

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use fenestra_ui_exp_0008_layout_conformance::prototype::TaffyStackEngineV1;
use fenestra_ui_ir::prototype::{InvalidationSet, PropertyValue};
use fenestra_ui_layout::prototype::{
    LayoutEngineErrorV1, LayoutEngineV1, LayoutOutputV1, ReferenceStackEngineV1,
    ValidatedLayoutInputV1,
};
use fenestra_ui_runtime::prototype::{
    CommitReceipt, CommittedRuntimeSnapshot, HeadlessSurface, UiRuntime,
};
use fenestra_ui_testkit::prototype::{
    HeadlessFixtureV1, HeadlessOracleV1, NormalizedHeadlessProjectionV1, SemanticOperationV1,
    compare_headless_projection_v1, observe_headless_projection_v1,
};

use super::support::{
    DIMENSION_INVALIDATION, INSERTED_KEY, PAINT_INVALIDATION, REGION_INVALIDATION,
    RESIZE_INVALIDATION, SECOND_KEY, control_path, items_path, stage_operation,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct MilestoneV1 {
    receipt_generation: Option<u64>,
    projection_generation: u64,
    invalidation: InvalidationSet,
    mutation_count: usize,
    oracle_projection: NormalizedHeadlessProjectionV1,
    projection: NormalizedHeadlessProjectionV1,
}

impl MilestoneV1 {
    pub(super) const fn receipt_generation(&self) -> Option<u64> {
        self.receipt_generation
    }

    pub(super) const fn projection_generation(&self) -> u64 {
        self.projection_generation
    }

    pub(super) const fn invalidation(&self) -> InvalidationSet {
        self.invalidation
    }

    pub(super) const fn mutation_count(&self) -> usize {
        self.mutation_count
    }

    pub(super) const fn oracle_projection(&self) -> &NormalizedHeadlessProjectionV1 {
        &self.oracle_projection
    }

    pub(super) const fn projection(&self) -> &NormalizedHeadlessProjectionV1 {
        &self.projection
    }
}

pub(super) struct LaneRunV1 {
    transcript: Vec<MilestoneV1>,
    engine_calls: Vec<usize>,
}

impl LaneRunV1 {
    pub(super) fn transcript(&self) -> &[MilestoneV1] {
        &self.transcript
    }

    pub(super) fn engine_calls(&self) -> &[usize] {
        &self.engine_calls
    }
}

struct CountingEngine<E> {
    inner: E,
    calls: Arc<AtomicUsize>,
}

impl<E> CountingEngine<E> {
    fn new(inner: E, calls: Arc<AtomicUsize>) -> Self {
        Self { inner, calls }
    }
}

impl<E: LayoutEngineV1> LayoutEngineV1 for CountingEngine<E> {
    fn compute(
        &self,
        input: ValidatedLayoutInputV1<'_>,
    ) -> Result<LayoutOutputV1, LayoutEngineErrorV1> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.inner.compute(input)
    }
}

pub(super) fn run_reference_lane_v1() -> LaneRunV1 {
    run_lane_v1(Box::new(ReferenceStackEngineV1::new()), None)
}

pub(super) fn run_candidate_lane_v1() -> LaneRunV1 {
    let calls = Arc::new(AtomicUsize::new(0));
    let engine = CountingEngine::new(TaffyStackEngineV1::new(), Arc::clone(&calls));
    run_lane_v1(Box::new(engine), Some(calls))
}

fn run_lane_v1(engine: Box<dyn LayoutEngineV1>, counter: Option<Arc<AtomicUsize>>) -> LaneRunV1 {
    let fixture = HeadlessFixtureV1::build().expect("registered fixture must validate");
    let mut oracle = HeadlessOracleV1::new(&fixture).expect("registered oracle must initialize");
    let mut runtime = UiRuntime::new_headless_with_layout_engine(
        fixture.style().clone(),
        fixture.spec(),
        fixture.surface(),
        fixture.runtime_capacity(),
        engine,
    )
    .expect("registered runtime lane must initialize");
    let mut transcript = vec![observe_milestone_v1(
        &fixture,
        &oracle,
        &runtime.committed(),
        None,
        0,
        InvalidationSet::NONE,
    )];
    let mut engine_calls = vec![call_count(&counter)];

    let color = SemanticOperationV1::SetProperty {
        node: control_path(),
        property: fixture.spec().color(),
        value: PropertyValue::Rgba8([20, 30, 40, 255]),
    };
    transcript.push(commit_operation_v1(
        &fixture,
        &mut oracle,
        &mut runtime,
        &color,
        1,
        PAINT_INVALIDATION,
    ));
    engine_calls.push(call_count(&counter));

    for (operation, generation, invalidation) in [
        (
            SemanticOperationV1::InsertKeyed {
                fragment: items_path(),
                key: INSERTED_KEY,
                final_index: 1,
            },
            2,
            REGION_INVALIDATION,
        ),
        (
            SemanticOperationV1::MoveKeyed {
                fragment: items_path(),
                key: INSERTED_KEY,
                final_index: 2,
            },
            3,
            REGION_INVALIDATION,
        ),
        (
            SemanticOperationV1::UpdateKeyed {
                fragment: items_path(),
                key: INSERTED_KEY,
                property: fixture.spec().height(),
                value: PropertyValue::ScalarI32(14),
            },
            4,
            DIMENSION_INVALIDATION,
        ),
        (
            SemanticOperationV1::RemoveKeyed {
                fragment: items_path(),
                key: SECOND_KEY,
            },
            5,
            REGION_INVALIDATION,
        ),
    ] {
        transcript.push(commit_operation_v1(
            &fixture,
            &mut oracle,
            &mut runtime,
            &operation,
            generation,
            invalidation,
        ));
        engine_calls.push(call_count(&counter));
    }

    let resized = HeadlessSurface::new(90, 70);
    let before = runtime.committed();
    let mut transaction = runtime.begin_transaction();
    transaction
        .resize_headless(resized)
        .expect("registered resize must stage");
    let receipt = runtime
        .commit(transaction)
        .expect("registered resize must publish");
    drop(before);
    oracle
        .resize(resized)
        .expect("registered oracle must resize");
    transcript.push(observe_milestone_v1(
        &fixture,
        &oracle,
        &runtime.committed(),
        Some(&receipt),
        6,
        RESIZE_INVALIDATION,
    ));
    engine_calls.push(call_count(&counter));

    LaneRunV1 {
        transcript,
        engine_calls,
    }
}

fn commit_operation_v1(
    fixture: &HeadlessFixtureV1,
    oracle: &mut HeadlessOracleV1,
    runtime: &mut UiRuntime,
    operation: &SemanticOperationV1,
    generation: u64,
    invalidation: InvalidationSet,
) -> MilestoneV1 {
    let before = runtime.committed();
    let mut transaction = runtime.begin_transaction();
    stage_operation(&mut transaction, &before, operation);
    let receipt = runtime
        .commit(transaction)
        .expect("registered operation must publish");
    drop(before);
    oracle
        .apply_operation(operation)
        .expect("registered oracle operation must apply");
    observe_milestone_v1(
        fixture,
        oracle,
        &runtime.committed(),
        Some(&receipt),
        generation,
        invalidation,
    )
}

fn observe_milestone_v1(
    fixture: &HeadlessFixtureV1,
    oracle: &HeadlessOracleV1,
    snapshot: &CommittedRuntimeSnapshot,
    receipt: Option<&CommitReceipt>,
    generation: u64,
    invalidation: InvalidationSet,
) -> MilestoneV1 {
    let observed = observe_headless_projection_v1(fixture, snapshot)
        .expect("registered projection must normalize");
    let expected = oracle.rebuild().expect("registered oracle must rebuild");
    assert_eq!(snapshot.generation().get(), generation);
    assert_eq!(observed.generation(), snapshot.generation());
    assert_eq!(
        compare_headless_projection_v1(&expected, observed.projection())
            .expect("matching surfaces must compare"),
        None
    );

    let (receipt_generation, mutation_count) = receipt.map_or((None, 0), |receipt| {
        assert_eq!(receipt.generation(), snapshot.generation());
        assert_eq!(receipt.invalidation(), invalidation);
        (Some(receipt.generation().get()), receipt.mutations().len())
    });
    assert_eq!(mutation_count, usize::from(receipt.is_some()));
    MilestoneV1 {
        receipt_generation,
        projection_generation: generation,
        invalidation,
        mutation_count,
        oracle_projection: expected,
        projection: observed.projection().clone(),
    }
}

fn call_count(counter: &Option<Arc<AtomicUsize>>) -> usize {
    counter
        .as_ref()
        .map_or(0, |counter| counter.load(Ordering::SeqCst))
}
