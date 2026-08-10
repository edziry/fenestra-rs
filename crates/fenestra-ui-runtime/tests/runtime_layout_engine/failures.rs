use std::sync::Arc;

use fenestra_ui_ir::prototype::PropertyValue;
use fenestra_ui_layout::prototype::{
    LayoutArithmeticOperationV1, LayoutEngineErrorKindV1, LayoutExtentV1,
};
use fenestra_ui_runtime::prototype::{
    HeadlessProjectionErrorKind, RuntimeInitializationErrorKind, TransactionErrorKind,
};

use super::headless_projection::nodes;
use super::layout_support::{
    EngineFaultV1, ScriptedFaultEngineV1, SpyState, exact_runtime_with_engine,
    try_exact_runtime_with_engine,
};
use super::support::headless::WIDTH;
use super::support::headless_projection_state::capture_projection;

#[test]
fn every_initial_layout_failure_maps_to_the_closed_runtime_vocabulary() {
    for (fault, expected) in failure_cases() {
        let state = Arc::new(SpyState::default());
        let engine = ScriptedFaultEngineV1::new(Arc::clone(&state), 0, fault);
        let error = try_exact_runtime_with_engine(Box::new(engine))
            .err()
            .expect("scripted initialization should fail");

        assert_eq!(
            error.kind(),
            RuntimeInitializationErrorKind::Headless(expected),
            "initial runtime mapping for {fault:?}"
        );
        assert_eq!(state.calls(), 1, "initial fault call for {fault:?}");
    }
}

#[test]
fn every_layout_failure_maps_closed_and_rolls_back_the_exact_state() {
    for (fault, expected) in failure_cases() {
        let state = Arc::new(SpyState::default());
        let engine = ScriptedFaultEngineV1::new(Arc::clone(&state), 1, fault);
        let mut runtime = exact_runtime_with_engine(Box::new(engine));
        assert_eq!(state.calls(), 1, "initial call for {fault:?}");
        let before = runtime.committed();
        let fixture = nodes(&before);
        let retained_projection = capture_projection(&before);
        let retained_property = before.property(fixture.root, WIDTH).cloned();
        let mut transaction = runtime.begin_transaction();
        transaction
            .set_property(fixture.root, WIDTH, PropertyValue::ScalarI32(101))
            .expect("effective rebuild should stage");

        let error = runtime
            .commit(transaction)
            .expect_err("scripted rebuild should fail");
        let after = runtime.committed();

        assert_eq!(
            error.kind(),
            TransactionErrorKind::Headless(expected),
            "runtime mapping for {fault:?}"
        );
        assert_eq!(error.operation_index(), None, "layout fault location");
        assert_eq!(state.calls(), 2, "rebuild call for {fault:?}");
        assert!(before.shares_state_with(&after), "allocation for {fault:?}");
        assert_eq!(after.generation(), before.generation());
        assert_eq!(
            after.property(fixture.root, WIDTH),
            retained_property.as_ref()
        );
        assert_eq!(capture_projection(&after), retained_projection);
    }
}

fn failure_cases() -> [(EngineFaultV1, HeadlessProjectionErrorKind); 8] {
    [
        (
            EngineFaultV1::Engine(LayoutEngineErrorKindV1::RejectedInput),
            HeadlessProjectionErrorKind::InvariantViolation,
        ),
        (
            EngineFaultV1::Engine(LayoutEngineErrorKindV1::UnrepresentableOutput),
            HeadlessProjectionErrorKind::InvariantViolation,
        ),
        (
            EngineFaultV1::Engine(LayoutEngineErrorKindV1::InvariantViolation),
            HeadlessProjectionErrorKind::InvariantViolation,
        ),
        (
            EngineFaultV1::Engine(LayoutEngineErrorKindV1::ArithmeticExhausted {
                operation: LayoutArithmeticOperationV1::FarEdge,
                extent: LayoutExtentV1::Width,
            }),
            HeadlessProjectionErrorKind::ArithmeticExhausted,
        ),
        (
            EngineFaultV1::RecordCount,
            HeadlessProjectionErrorKind::InvariantViolation,
        ),
        (
            EngineFaultV1::Key,
            HeadlessProjectionErrorKind::InvariantViolation,
        ),
        (
            EngineFaultV1::Negative,
            HeadlessProjectionErrorKind::InvariantViolation,
        ),
        (
            EngineFaultV1::FarEdge,
            HeadlessProjectionErrorKind::InvariantViolation,
        ),
    ]
}
