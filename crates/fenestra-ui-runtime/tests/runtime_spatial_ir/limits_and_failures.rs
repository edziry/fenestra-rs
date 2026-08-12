use fenestra_ui_layout::prototype::{
    LayoutConstraintFieldV1, LayoutEngineErrorKindV1, LayoutExtentV1,
};
use fenestra_ui_runtime::prototype::{
    RuntimeInitializationError, RuntimeInitializationErrorKind, TransactionErrorKind,
};
use fenestra_ui_spatial::prototype::{
    SpatialErrorLocationV2, SpatialExtentV2, SpatialInputErrorKindV2,
    SpatialLayoutDimensionErrorKindV2, SpatialLayoutErrorKindV2, SpatialLimitKindV2,
    SpatialNodeFieldV2, SpatialResolveErrorKindV2, SpatialViewportV2,
};

use crate::spatial_support::engine::{EnginePlan, EngineSpy};
use crate::support::spatial_ir::{
    LogicalNodes, VIEWPORT, WIDTH, capacity, fixture, fixture_with_width, limit, limits,
};
use crate::{RuntimeSpatialErrorV2, RuntimeSpatialIrErrorKindV2, new_ir, new_ir_with_engine};

#[test]
fn expanded_direct_limits_fail_before_layout_and_retain_widened_evidence() {
    let fixture = fixture();
    let (engine, engine_state) = EngineSpy::new(EnginePlan::Panic);
    let error = initialization_error(new_ir_with_engine(
        fixture.program,
        VIEWPORT,
        limit(SpatialLimitKindV2::Nodes, 4),
        capacity(),
        Box::new(engine),
    ));
    let ir = expect_ir_initialization(error);
    let RuntimeSpatialIrErrorKindV2::Resolve(resolve) = ir.kind() else {
        panic!("direct count should preserve the resolver diagnostic");
    };

    assert_eq!(engine_state.calls(), 0);
    assert_eq!(ir.span(), fixture.spans.program);
    assert_eq!(
        resolve.kind(),
        SpatialResolveErrorKindV2::LimitExceeded(SpatialLimitKindV2::Nodes)
    );
    assert_eq!(resolve.location(), SpatialErrorLocationV2::Input);
    assert_eq!(resolve.observed(), Some(5));
    assert_eq!(resolve.maximum(), Some(4));
}

#[test]
fn initialization_wraps_dynamic_field_failures_with_the_exact_binding_span() {
    let fixture = fixture_with_width(-1);
    let error = initialization_error(new_ir(fixture.program, VIEWPORT, limits(), capacity()));
    let ir = expect_ir_initialization(error);
    let RuntimeSpatialIrErrorKindV2::Resolve(resolve) = ir.kind() else {
        panic!("dynamic field rejection should preserve resolver detail");
    };

    assert_eq!(ir.span(), fixture.spans.outer_width_minimum);
    assert_eq!(
        resolve.kind(),
        SpatialResolveErrorKindV2::Input(SpatialInputErrorKindV2::InvalidLayoutDimensions(
            SpatialLayoutDimensionErrorKindV2::NegativeConstraint {
                extent: LayoutExtentV1::Width,
                field: LayoutConstraintFieldV1::Minimum,
            },
        ))
    );
    assert_eq!(
        resolve.location(),
        SpatialErrorLocationV2::NodeField {
            index: 1,
            field: SpatialNodeFieldV2::LayoutWidthMinimum,
        }
    );
}

#[test]
fn runtime_owned_viewport_failures_use_synthetic_source_attribution() {
    let fixture = fixture();
    let error = initialization_error(new_ir(
        fixture.program,
        SpatialViewportV2::new(-1, VIEWPORT.height()),
        limits(),
        capacity(),
    ));
    let ir = expect_ir_initialization(error);
    let RuntimeSpatialIrErrorKindV2::Resolve(resolve) = ir.kind() else {
        panic!("viewport rejection should preserve resolver detail");
    };

    assert_eq!(ir.span(), fenestra_ui_ir::prototype::SourceSpan::Synthetic);
    assert_eq!(
        resolve.kind(),
        SpatialResolveErrorKindV2::Input(SpatialInputErrorKindV2::NegativeViewport(
            SpatialExtentV2::Width,
        ))
    );
    assert_eq!(
        resolve.location(),
        SpatialErrorLocationV2::Viewport {
            extent: SpatialExtentV2::Width,
        }
    );
}

#[test]
fn late_layout_failure_rolls_back_with_no_operation_attribution() {
    let fixture = fixture();
    let expected_span = fixture.spans.program;
    let (engine, engine_state) = EngineSpy::new(EnginePlan::RejectOnCall(2));
    let mut runtime = new_ir_with_engine(
        fixture.program,
        VIEWPORT,
        limits(),
        capacity(),
        Box::new(engine),
    )
    .expect("first layout pass should initialize");
    let before = runtime.committed();
    let logical = LogicalNodes::capture(&before);
    let before_snapshot = before
        .spatial()
        .expect("spatial state should exist")
        .snapshot();
    let mut transaction = runtime.begin_transaction();
    transaction
        .set_property(
            logical.first_outer,
            WIDTH,
            fenestra_ui_ir::prototype::PropertyValue::ScalarI32(13),
        )
        .expect("effective update should stage");
    let error = runtime
        .commit(transaction)
        .expect_err("second layout pass should reject");
    let TransactionErrorKind::Spatial(RuntimeSpatialErrorV2::Ir(ir)) = error.kind() else {
        panic!("IR rebuild failure should use the additive wrapper");
    };
    let RuntimeSpatialIrErrorKindV2::Resolve(resolve) = ir.kind() else {
        panic!("layout rejection should preserve resolver detail");
    };

    assert_eq!(
        resolve.kind(),
        SpatialResolveErrorKindV2::Layout(SpatialLayoutErrorKindV2::Engine(
            LayoutEngineErrorKindV1::RejectedInput,
        ))
    );
    assert_eq!(
        resolve.location(),
        SpatialErrorLocationV2::Island { index: 0 }
    );
    assert_eq!(ir.span(), expected_span);
    assert_eq!(error.operation_index(), None);
    assert_eq!(engine_state.calls(), 2);
    assert!(before.shares_state_with(&runtime.committed()));
    assert!(std::ptr::eq(
        before_snapshot,
        runtime
            .committed()
            .spatial()
            .expect("prior spatial state should remain")
            .snapshot()
    ));
}

fn initialization_error(
    result: Result<fenestra_ui_runtime::prototype::UiRuntime, RuntimeInitializationError>,
) -> RuntimeInitializationError {
    match result {
        Ok(_) => panic!("fixture should fail initialization"),
        Err(error) => error,
    }
}

fn expect_ir_initialization(error: RuntimeInitializationError) -> crate::RuntimeSpatialIrErrorV2 {
    let RuntimeInitializationErrorKind::Spatial(RuntimeSpatialErrorV2::Ir(ir)) = error.kind()
    else {
        panic!("expected one wrapped runtime spatial IR error");
    };
    ir
}
