use fenestra_ui_ir::prototype::{
    ConstructionProgram, SchemaManifest, StyleProgram, StyleValidationLimits,
    ValidatedStyleProgram, ValidationLimits, validate_construction, validate_schema,
    validate_style,
};
use fenestra_ui_runtime::prototype::UiRuntime;
use fenestra_ui_testkit::prototype::{
    HeadlessFixtureV1, HeadlessOracleV1, NormalizedHeadlessProjectionV1, NormalizedStateV1,
    SemanticOperationV1, observe_headless_projection_v1, observe_snapshot_v1,
};

use super::identity::{resolve_fragment, stage_operation};
use super::items_path;
use super::receipt::{NormalizedReceipt, normalize_receipt};

const IR_LIMITS: ValidationLimits = ValidationLimits::new(1, 5, 4, 1, 3, 12, 2, 3, 5);
const STYLE_LIMITS: StyleValidationLimits = StyleValidationLimits::new(2);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LaneLog {
    receipts: Vec<NormalizedReceipt>,
    states: Vec<NormalizedStateV1>,
    projections: Vec<NormalizedHeadlessProjectionV1>,
    final_keys: Vec<u64>,
}

impl LaneLog {
    pub fn receipts(&self) -> &[NormalizedReceipt] {
        &self.receipts
    }

    pub fn states(&self) -> &[NormalizedStateV1] {
        &self.states
    }

    pub fn projections(&self) -> &[NormalizedHeadlessProjectionV1] {
        &self.projections
    }

    pub fn final_keys(&self) -> &[u64] {
        &self.final_keys
    }

    #[allow(dead_code)]
    pub(super) const fn from_parts(
        receipts: Vec<NormalizedReceipt>,
        states: Vec<NormalizedStateV1>,
        projections: Vec<NormalizedHeadlessProjectionV1>,
        final_keys: Vec<u64>,
    ) -> Self {
        Self {
            receipts,
            states,
            projections,
            final_keys,
        }
    }
}

pub fn validate_programs(
    programs: (SchemaManifest, ConstructionProgram, StyleProgram),
) -> ValidatedStyleProgram {
    let schema = validate_schema(programs.0, IR_LIMITS)
        .expect("the generated schema should validate at registered limits");
    let construction = validate_construction(&schema, programs.1, IR_LIMITS)
        .expect("the generated construction should validate at registered limits");
    validate_style(&construction, programs.2, STYLE_LIMITS)
        .expect("the generated style should validate at registered limits")
}

pub fn run_lane(
    fixture: &HeadlessFixtureV1,
    style: ValidatedStyleProgram,
    operations: &[SemanticOperationV1],
) -> LaneLog {
    let construction = style.construction().clone();
    let mut runtime = UiRuntime::new_headless(
        style,
        fixture.spec(),
        fixture.surface(),
        fixture.runtime_capacity(),
    )
    .expect("the registered runtime lane should initialize");
    let initial = runtime.committed();
    let initial_state = observe_snapshot_v1(&construction, &initial, fixture.harness_limits())
        .expect("generation zero should normalize its logical state");
    let initial_projection = observe_headless_projection_v1(fixture, &initial)
        .expect("generation zero should normalize");
    assert_eq!(initial.generation().get(), 0);
    assert_eq!(initial_projection.generation(), initial.generation());
    let mut receipts = vec![NormalizedReceipt::new(
        0,
        Vec::new(),
        fenestra_ui_ir::prototype::InvalidationSet::NONE,
    )];
    let mut states = vec![initial_state];
    let mut projections = vec![initial_projection.projection().clone()];
    drop(initial);

    for operation in operations {
        let before = runtime.committed();
        let mut transaction = runtime.begin_transaction();
        stage_operation(&construction, &before, &mut transaction, operation)
            .expect("the registered semantic operation should stage");
        let receipt = runtime
            .commit(transaction)
            .expect("the registered semantic operation should publish");
        let after = runtime.committed();
        let normalized = normalize_receipt(&construction, &before, &after, &receipt)
            .expect("the registered receipt should normalize");
        let state = observe_snapshot_v1(&construction, &after, fixture.harness_limits())
            .expect("the committed logical state should normalize");
        let projection = observe_headless_projection_v1(fixture, &after)
            .expect("the committed headless projection should normalize");
        assert_eq!(projection.generation(), after.generation());
        receipts.push(normalized);
        states.push(state);
        projections.push(projection.projection().clone());
        drop(receipt);
        drop(before);
        drop(after);
    }

    let final_snapshot = runtime.committed();
    let fragment = resolve_fragment(&construction, &final_snapshot, &items_path())
        .expect("the final items fragment should remain live");
    let final_keys = final_snapshot
        .keyed_members(fragment)
        .expect("the final items fragment should be queryable")
        .map(|(key, _)| key)
        .collect();
    LaneLog {
        receipts,
        states,
        projections,
        final_keys,
    }
}

pub fn oracle_projection_log(
    fixture: &HeadlessFixtureV1,
    operations: &[SemanticOperationV1],
) -> Vec<NormalizedHeadlessProjectionV1> {
    let mut oracle = HeadlessOracleV1::new(fixture)
        .expect("the independent manual projection oracle should initialize");
    let mut projections = vec![
        oracle
            .rebuild()
            .expect("the generation-zero oracle should rebuild"),
    ];
    for operation in operations {
        oracle
            .apply_operation(operation)
            .expect("the registered operation should update the oracle");
        projections.push(
            oracle
                .rebuild()
                .expect("the updated projection oracle should rebuild"),
        );
    }
    projections
}
