mod program;
mod spatial;
mod value;

use fenestra_ui_ir::prototype::{
    InputPolicy, PropertyId, PropertyValue, SpatialValidationLimitsV2, StructuralRegionId,
    StyleValidationLimits, ValidationLimits, validate_construction, validate_schema,
    validate_spatial, validate_style,
};
use fenestra_ui_runtime::prototype::{
    CommittedRuntimeSnapshot, FragmentId, RuntimeCapacity, RuntimeSpatialErrorV2,
    RuntimeSpatialIrErrorKindV2, TransactionErrorKind, UiRuntime, UiTransaction,
};
use fenestra_ui_spatial::prototype::{
    REGISTERED_SPATIAL_LIMITS_V2, SpatialResolveErrorKindV2, SpatialTransformErrorKindV2,
    SpatialViewportV2,
};

use crate::baseline::literal_types::LiteralObservationInputV2;
use crate::baseline::model::EvidenceBuildErrorV2;

const IR_LIMITS: ValidationLimits = ValidationLimits::new(1, 8, 3, 1, 2, 9, 2, 2, 4);
const STYLE_LIMITS: StyleValidationLimits = StyleValidationLimits::new(0);
const SPATIAL_LIMITS: SpatialValidationLimitsV2 =
    SpatialValidationLimitsV2::new([3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8]);
const CAPACITY: RuntimeCapacity = RuntimeCapacity::new(4, 4, 8, 2, 64, 3);

pub(crate) struct RuntimeRollbackProbeV2 {
    pub(crate) attempted_generation: u64,
    pub(crate) retained_generation: u64,
    pub(crate) before_digest: u64,
    pub(crate) after_digest: u64,
    pub(crate) before_allocation: usize,
    pub(crate) after_allocation: usize,
    pub(crate) before_state: Vec<u8>,
    pub(crate) after_state: Vec<u8>,
}

#[allow(clippy::result_large_err)]
pub(crate) fn verify_runtime_observations_v2(
    observations: &[LiteralObservationInputV2],
) -> Result<RuntimeRollbackProbeV2, EvidenceBuildErrorV2> {
    if observations.len() != 9 {
        return Err(error("runtime-observation-count"));
    }
    let raw = program::raw();
    let schema = validate_schema(raw.0, IR_LIMITS).map_err(|_| error("runtime-schema"))?;
    let construction =
        validate_construction(&schema, raw.1, IR_LIMITS).map_err(|_| error("runtime-tree"))?;
    let style =
        validate_style(&construction, raw.2, STYLE_LIMITS).map_err(|_| error("runtime-style"))?;
    let spatial =
        validate_spatial(&style, raw.3, SPATIAL_LIMITS).map_err(|_| error("runtime-spatial"))?;
    let mut runtime = UiRuntime::new_spatial_ir(
        spatial,
        viewport(observations[0].scene.viewport),
        REGISTERED_SPATIAL_LIMITS_V2,
        CAPACITY,
    )
    .map_err(|_| error("runtime-init"))?;
    verify_generation(&runtime, &observations[0])?;

    commit(&mut runtime, &observations[1], |_, transaction| {
        transaction.resize_spatial(viewport(observations[1].scene.viewport))
    })?;
    commit(&mut runtime, &observations[2], |before, transaction| {
        transaction.set_property(
            before.root(),
            PropertyId::new(0),
            PropertyValue::ScalarI32(176),
        )
    })?;
    commit(&mut runtime, &observations[3], |before, transaction| {
        transaction.set_property(
            before.root(),
            PropertyId::new(4),
            PropertyValue::Rgba8([80, 40, 24, 255]),
        )
    })?;
    commit(&mut runtime, &observations[4], |before, transaction| {
        transaction.set_property(
            before.root(),
            PropertyId::new(7),
            PropertyValue::InputPolicy(InputPolicy::Accept),
        )
    })?;
    commit(&mut runtime, &observations[5], |before, transaction| {
        transaction.insert_keyed(fragment(before), 30, 1)
    })?;
    commit(&mut runtime, &observations[6], |before, transaction| {
        transaction.move_keyed(fragment(before), 30, 2)
    })?;
    commit(&mut runtime, &observations[7], |before, transaction| {
        transaction.update_keyed(
            fragment(before),
            30,
            PropertyId::new(1),
            PropertyValue::ScalarI32(14),
        )
    })?;
    commit(&mut runtime, &observations[8], |before, transaction| {
        transaction.remove_keyed(fragment(before), 20)
    })?;
    rollback_probe(&mut runtime)
}

fn commit(
    runtime: &mut UiRuntime,
    expected: &LiteralObservationInputV2,
    stage: impl FnOnce(
        &CommittedRuntimeSnapshot,
        &mut UiTransaction,
    ) -> Result<(), fenestra_ui_runtime::prototype::TransactionError>,
) -> Result<(), EvidenceBuildErrorV2> {
    let before = runtime.committed();
    let mut transaction = runtime.begin_transaction();
    stage(&before, &mut transaction).map_err(|_| error("runtime-stage"))?;
    runtime
        .commit(transaction)
        .map_err(|_| error("runtime-commit"))?;
    verify_generation(runtime, expected)
}

fn verify_generation(
    runtime: &UiRuntime,
    expected: &LiteralObservationInputV2,
) -> Result<(), EvidenceBuildErrorV2> {
    let snapshot = runtime.committed();
    let spatial = snapshot
        .spatial()
        .ok_or_else(|| error("runtime-publication"))?;
    let viewport = spatial.snapshot().viewport();
    let matches = Some(snapshot.generation().get()) == expected.scene.receipt.generation
        && viewport.width() == expected.scene.viewport.0 as i32
        && viewport.height() == expected.scene.viewport.1 as i32;
    matches
        .then_some(())
        .ok_or_else(|| error("runtime-generation"))
}

fn rollback_probe(runtime: &mut UiRuntime) -> Result<RuntimeRollbackProbeV2, EvidenceBuildErrorV2> {
    let before = runtime.committed();
    let before_state = state_bytes(&before)?;
    let before_allocation = spatial_address(&before)?;
    let mut transaction = runtime.begin_transaction();
    transaction
        .set_property(
            before.root(),
            PropertyId::new(3),
            PropertyValue::ScalarI32(0),
        )
        .map_err(|_| error("rollback-stage"))?;
    let failure = runtime
        .commit(transaction)
        .err()
        .ok_or_else(|| error("rollback-accepted"))?;
    let TransactionErrorKind::Spatial(RuntimeSpatialErrorV2::Ir(ir)) = failure.kind() else {
        return Err(error("rollback-wrapper"));
    };
    let RuntimeSpatialIrErrorKindV2::Resolve(resolve) = ir.kind() else {
        return Err(error("rollback-resolve"));
    };
    if resolve.kind()
        != SpatialResolveErrorKindV2::Transform(SpatialTransformErrorKindV2::SingularTransform)
    {
        return Err(error("rollback-kind"));
    }
    let after = runtime.committed();
    let after_state = state_bytes(&after)?;
    let after_allocation = spatial_address(&after)?;
    if !before.shares_state_with(&after) || before_state != after_state {
        return Err(error("rollback-state"));
    }
    Ok(RuntimeRollbackProbeV2 {
        attempted_generation: before.generation().get() + 1,
        retained_generation: after.generation().get(),
        before_digest: digest(&before_state),
        after_digest: digest(&after_state),
        before_allocation,
        after_allocation,
        before_state,
        after_state,
    })
}

fn fragment(snapshot: &CommittedRuntimeSnapshot) -> FragmentId {
    snapshot
        .fragment(snapshot.root(), StructuralRegionId::new(0))
        .expect("validated runtime retains its registered fragment")
}

fn spatial_address(snapshot: &CommittedRuntimeSnapshot) -> Result<usize, EvidenceBuildErrorV2> {
    snapshot
        .spatial()
        .map(|spatial| spatial.snapshot() as *const _ as usize)
        .ok_or_else(|| error("rollback-allocation"))
}

fn state_bytes(snapshot: &CommittedRuntimeSnapshot) -> Result<Vec<u8>, EvidenceBuildErrorV2> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&snapshot.generation().get().to_le_bytes());
    bytes.extend_from_slice(&(snapshot.node_count() as u64).to_le_bytes());
    bytes.extend_from_slice(&(snapshot.fragment_count() as u64).to_le_bytes());
    bytes.extend_from_slice(&(snapshot.property_slot_count() as u64).to_le_bytes());
    let fragment = snapshot
        .fragment(snapshot.root(), StructuralRegionId::new(0))
        .ok_or_else(|| error("rollback-fragment"))?;
    for (key, _) in snapshot
        .keyed_members(fragment)
        .ok_or_else(|| error("rollback-members"))?
    {
        bytes.extend_from_slice(&key.to_le_bytes());
    }
    let spatial = snapshot
        .spatial()
        .ok_or_else(|| error("rollback-spatial"))?;
    let output = spatial.snapshot().output();
    bytes.extend_from_slice(&(output.geometry().len() as u64).to_le_bytes());
    for row in output.geometry() {
        bytes.extend_from_slice(&row.key().get().to_le_bytes());
        bytes.extend_from_slice(&row.world_determinant().to_le_bytes());
    }
    Ok(bytes)
}

fn viewport(value: (u32, u32)) -> SpatialViewportV2 {
    SpatialViewportV2::new(value.0 as i32, value.1 as i32)
}

fn digest(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

const fn error(location: &'static str) -> EvidenceBuildErrorV2 {
    EvidenceBuildErrorV2 { location }
}
