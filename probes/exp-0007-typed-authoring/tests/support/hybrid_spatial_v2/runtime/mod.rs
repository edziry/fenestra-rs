mod oracle_bridge;
mod path;
mod projection;
mod queries;
mod receipt;
mod state;
mod types;

use fenestra_ui_ir::prototype::{
    ConstructionProgram, InputPolicy, PropertyId, PropertyValue, SchemaManifest, SpatialProgramV2,
    SpatialValidationLimitsV2, StructuralRegionId, StyleProgram, StyleValidationLimits,
    ValidatedConstruction, ValidatedSpatialProgramV2, ValidationLimits, validate_construction,
    validate_schema, validate_spatial, validate_style,
};
use fenestra_ui_runtime::prototype::{
    CommittedRuntimeSnapshot, RuntimeCapacity, RuntimeSpatialErrorV2, RuntimeSpatialIrErrorKindV2,
    TransactionErrorKind, UiRuntime, UiTransaction,
};
use fenestra_ui_spatial::prototype::{REGISTERED_SPATIAL_LIMITS_V2, SpatialViewportV2};

use projection::normalize_projection;
use queries::{normalize_queries, normalize_raster};
use receipt::{initial_receipt, normalize_receipt};
use state::normalize_state;
pub use types::AuthoredSpatialLaneLog;
use types::{NoopChecks, NormalizedFailure, NormalizedObservation};

type RawProgramsV2 = (
    SchemaManifest,
    ConstructionProgram,
    StyleProgram,
    SpatialProgramV2,
);

const IR_LIMITS: ValidationLimits = ValidationLimits::new(1, 8, 7, 1, 6, 19, 2, 4, 8);
const STYLE_LIMITS: StyleValidationLimits = StyleValidationLimits::new(3);
const SPATIAL_LIMITS: SpatialValidationLimitsV2 =
    SpatialValidationLimitsV2::new([7, 5, 3, 3, 4, 4, 3, 1, 5, 3, 3, 1, 16]);
const CAPACITY: RuntimeCapacity = RuntimeCapacity::new(4, 4, 12, 2, 96, 3);

pub fn run_authored_spatial_lane(programs: RawProgramsV2) -> AuthoredSpatialLaneLog {
    let program = validate(programs);
    let construction = program.style().construction().clone();
    let authored_factor_span = authored_factor_span(&program);
    let mut runtime = UiRuntime::new_spatial_ir(
        program,
        SpatialViewportV2::new(192, 128),
        REGISTERED_SPATIAL_LIMITS_V2,
        CAPACITY,
    )
    .expect("the independently validated authored lane should initialize");
    let mut observations = vec![normalize_observation(
        &construction,
        &runtime.committed(),
        initial_receipt(),
    )];

    record_commit(&mut runtime, &construction, &mut observations, |_, tx| {
        tx.resize_spatial(SpatialViewportV2::new(224, 160))
            .expect("the registered viewport resize should stage");
    });
    record_commit(
        &mut runtime,
        &construction,
        &mut observations,
        |before, tx| {
            tx.set_property(
                before.root(),
                PropertyId::new(0),
                PropertyValue::ScalarI32(176),
            )
            .expect("the root span update should stage");
        },
    );
    record_commit(
        &mut runtime,
        &construction,
        &mut observations,
        |before, tx| {
            tx.set_property(
                before.root(),
                PropertyId::new(4),
                PropertyValue::Rgba8([80, 40, 24, 255]),
            )
            .expect("the root tone update should stage");
        },
    );
    record_commit(
        &mut runtime,
        &construction,
        &mut observations,
        |before, tx| {
            tx.set_property(
                before.root(),
                PropertyId::new(7),
                PropertyValue::InputPolicy(InputPolicy::Accept),
            )
            .expect("the root policy update should stage");
        },
    );
    record_commit(
        &mut runtime,
        &construction,
        &mut observations,
        |before, tx| {
            tx.insert_keyed(tile_fragment(before), 30, 1)
                .expect("the keyed insertion should stage");
        },
    );
    record_commit(
        &mut runtime,
        &construction,
        &mut observations,
        |before, tx| {
            tx.move_keyed(tile_fragment(before), 30, 2)
                .expect("the keyed move should stage");
        },
    );
    record_commit(
        &mut runtime,
        &construction,
        &mut observations,
        |before, tx| {
            tx.update_keyed(
                tile_fragment(before),
                30,
                PropertyId::new(1),
                PropertyValue::ScalarI32(14),
            )
            .expect("the keyed property update should stage");
        },
    );
    record_commit(
        &mut runtime,
        &construction,
        &mut observations,
        |before, tx| {
            tx.remove_keyed(tile_fragment(before), 20)
                .expect("the keyed removal should stage");
        },
    );

    let final_snapshot = runtime.committed();
    let final_keys = final_snapshot
        .keyed_members(tile_fragment(&final_snapshot))
        .expect("the tile fragment should remain live")
        .map(|(key, _)| key)
        .collect();
    let noop = verify_noops(&mut runtime);
    let failure = verify_singular_rollback(
        &mut runtime,
        &construction,
        &observations,
        authored_factor_span,
    );
    AuthoredSpatialLaneLog {
        observations,
        final_keys,
        noop,
        failure,
    }
}

fn validate(programs: RawProgramsV2) -> ValidatedSpatialProgramV2 {
    let schema = validate_schema(programs.0, IR_LIMITS)
        .expect("the authored schema should validate independently");
    let construction = validate_construction(&schema, programs.1, IR_LIMITS)
        .expect("the authored construction should validate independently");
    let style = validate_style(&construction, programs.2, STYLE_LIMITS)
        .expect("the authored style should validate independently");
    validate_spatial(&style, programs.3, SPATIAL_LIMITS)
        .expect("the authored spatial program should validate independently")
}

fn record_commit(
    runtime: &mut UiRuntime,
    construction: &ValidatedConstruction,
    observations: &mut Vec<NormalizedObservation>,
    stage: impl FnOnce(&CommittedRuntimeSnapshot, &mut UiTransaction),
) {
    let before = runtime.committed();
    let mut transaction = runtime.begin_transaction();
    stage(&before, &mut transaction);
    let receipt = runtime
        .commit(transaction)
        .expect("the registered authored operation should commit");
    let after = runtime.committed();
    assert_eq!(after.generation(), receipt.generation());
    let receipt = normalize_receipt(construction, &before, &receipt, &after);
    observations.push(normalize_observation(construction, &after, receipt));
}

fn normalize_observation(
    construction: &ValidatedConstruction,
    snapshot: &CommittedRuntimeSnapshot,
    receipt: types::NormalizedReceipt,
) -> NormalizedObservation {
    let spatial = snapshot
        .spatial()
        .expect("the authored runtime should publish spatial state");
    NormalizedObservation {
        generation: snapshot.generation().get(),
        viewport: spatial.snapshot().viewport(),
        receipt,
        state: normalize_state(construction, snapshot),
        projection: normalize_projection(construction, snapshot),
        hit_queries: normalize_queries(construction, snapshot),
        raster: normalize_raster(snapshot),
    }
}

fn floating_node(snapshot: &CommittedRuntimeSnapshot) -> fenestra_ui_runtime::prototype::NodeId {
    let stack = snapshot
        .children(snapshot.root())
        .and_then(|children| children.first())
        .copied()
        .expect("the root should retain its stack child");
    snapshot
        .children(stack)
        .and_then(|children| children.first())
        .copied()
        .expect("the stack should retain its floating child")
}

fn tile_fragment(
    snapshot: &CommittedRuntimeSnapshot,
) -> fenestra_ui_runtime::prototype::FragmentId {
    snapshot
        .fragment(floating_node(snapshot), StructuralRegionId::new(0))
        .expect("the floating node should retain its tile fragment")
}

fn verify_noops(runtime: &mut UiRuntime) -> NoopChecks {
    let empty = commit_noop(runtime, |_, _| {});
    let same_value = commit_noop(runtime, |before, transaction| {
        let root = before.root();
        transaction
            .set_property(root, PropertyId::new(0), PropertyValue::ScalarI32(176))
            .expect("the same-value property write should stage");
    });
    let round_trip = commit_noop(runtime, |before, transaction| {
        let root = before.root();
        transaction
            .set_property(root, PropertyId::new(0), PropertyValue::ScalarI32(177))
            .expect("the first round-trip write should stage");
        transaction
            .set_property(root, PropertyId::new(0), PropertyValue::ScalarI32(176))
            .expect("the second round-trip write should stage");
    });
    NoopChecks::new(empty, same_value, round_trip)
}

fn commit_noop(
    runtime: &mut UiRuntime,
    stage: impl FnOnce(&CommittedRuntimeSnapshot, &mut UiTransaction),
) -> bool {
    let before = runtime.committed();
    let before_spatial = before
        .spatial()
        .expect("the stable state should retain spatial output")
        .snapshot();
    let mut transaction = runtime.begin_transaction();
    stage(&before, &mut transaction);
    let receipt = runtime
        .commit(transaction)
        .expect("the no-op transaction should commit");
    let after = runtime.committed();
    receipt.is_empty()
        && before.shares_state_with(&after)
        && std::ptr::eq(
            before_spatial,
            after
                .spatial()
                .expect("the no-op state should retain spatial output")
                .snapshot(),
        )
}

fn verify_singular_rollback(
    runtime: &mut UiRuntime,
    construction: &ValidatedConstruction,
    observations: &[NormalizedObservation],
    authored_factor_span: fenestra_ui_ir::prototype::SourceSpan,
) -> NormalizedFailure {
    let before = runtime.committed();
    let before_spatial = before
        .spatial()
        .expect("the stable state should retain spatial output")
        .snapshot();
    let mut transaction = runtime.begin_transaction();
    transaction
        .set_property(
            floating_node(&before),
            PropertyId::new(3),
            PropertyValue::ScalarI32(0),
        )
        .expect("the singular factor write should stage");
    let error = runtime
        .commit(transaction)
        .expect_err("the singular authored transform should reject");
    let TransactionErrorKind::Spatial(RuntimeSpatialErrorV2::Ir(ir)) = error.kind() else {
        panic!("the singular failure should use the runtime IR wrapper");
    };
    let RuntimeSpatialIrErrorKindV2::Resolve(resolve) = ir.kind() else {
        panic!("the singular failure should retain resolver evidence");
    };
    let after = runtime.committed();
    let receipt = observations
        .last()
        .expect("the successful log should not be empty")
        .receipt
        .clone();
    let after_observation = normalize_observation(construction, &after, receipt);
    NormalizedFailure::new(
        resolve.kind(),
        resolve.location(),
        ir.span(),
        error.operation_index(),
        before.shares_state_with(&after),
        std::ptr::eq(
            before_spatial,
            after
                .spatial()
                .expect("the rolled-back state should retain spatial output")
                .snapshot(),
        ),
        observations.last() == Some(&after_observation),
        authored_factor_span,
    )
}

fn authored_factor_span(
    program: &ValidatedSpatialProgramV2,
) -> fenestra_ui_ir::prototype::SourceSpan {
    let declaration = program
        .program()
        .nodes()
        .get(2)
        .expect("the fixture should retain its floating declaration");
    let fenestra_ui_ir::prototype::SpatialPlacementRecipeV2::Free(placement) =
        declaration.placement()
    else {
        panic!("the floating declaration should use free placement");
    };
    placement.transform().a().span()
}
