use std::panic::{AssertUnwindSafe, catch_unwind};

use fenestra_ui_ir::prototype::{
    ChildSlot, ComponentSchema, ComponentTypeId, ConstructionProgram, InitialKey,
    InvalidationClass, InvalidationSet, PropertyId, PropertySchema, PropertyValue,
    SUPPORTED_CONSTRUCTION_FORMAT, SUPPORTED_SCHEMA_FORMAT, SchemaManifest, SchemaNamespace,
    SchemaRevision, SourceSpan, StructuralRegion, StructuralRegionId, TemplateNode, TemplateNodeId,
    ValidationLimits, ValueType, validate_construction, validate_schema,
};

use super::capacity::RuntimeCapacity;
use super::error::{CapacityKind, TransactionErrorKind};
use super::transaction::{CommitTestHook, UiRuntime};

const COMPONENT: ComponentTypeId = ComponentTypeId::new(0);
const PROPERTY: PropertyId = PropertyId::new(0);
const ROOT_TEMPLATE: TemplateNodeId = TemplateNodeId::new(0);
const MEMBER_TEMPLATE: TemplateNodeId = TemplateNodeId::new(1);
const REGION: StructuralRegionId = StructuralRegionId::new(0);

fn construction() -> fenestra_ui_ir::prototype::ValidatedConstruction {
    let span = SourceSpan::synthetic();
    let namespace = SchemaNamespace::new(99);
    let revision = SchemaRevision::new(1);
    let manifest = SchemaManifest::new(
        SUPPORTED_SCHEMA_FORMAT,
        namespace,
        revision,
        vec![ComponentSchema::new(
            COMPONENT,
            vec![PropertySchema::new(
                PROPERTY,
                ValueType::ScalarI32,
                PropertyValue::ScalarI32(0),
                InvalidationSet::from_class(InvalidationClass::Layout),
                span,
            )],
            span,
        )],
        span,
    );
    let program = ConstructionProgram::new(
        SUPPORTED_CONSTRUCTION_FORMAT,
        namespace,
        revision,
        vec![
            TemplateNode::new(
                ROOT_TEMPLATE,
                COMPONENT,
                Vec::new(),
                vec![ChildSlot::region(REGION, span)],
                span,
            ),
            TemplateNode::new(MEMBER_TEMPLATE, COMPONENT, Vec::new(), Vec::new(), span),
        ],
        vec![StructuralRegion::new(
            REGION,
            ROOT_TEMPLATE,
            MEMBER_TEMPLATE,
            vec![InitialKey::new(7, span)],
            InvalidationSet::from_class(InvalidationClass::Structure),
            span,
        )],
        span,
    );
    let ir_limits = ValidationLimits::new(4, 4, 4, 4, 4, 4, 4, 4, 4);
    let schema = validate_schema(manifest, ir_limits).unwrap();
    validate_construction(&schema, program, ir_limits).unwrap()
}

fn runtime(retained_generations: usize) -> UiRuntime {
    UiRuntime::new(
        construction(),
        RuntimeCapacity::new(8, 8, 8, 8, 8, retained_generations),
    )
    .unwrap()
}

fn changed_transaction(runtime: &UiRuntime, value: i32) -> super::transaction::UiTransaction {
    let mut transaction = runtime.begin_transaction();
    transaction
        .set_property(
            runtime.committed().root(),
            PROPERTY,
            PropertyValue::ScalarI32(value),
        )
        .unwrap();
    transaction
}

#[test]
fn every_prepublication_unwind_preserves_the_exact_committed_state() {
    let hooks = [
        CommitTestHook::PanicAfterDraft,
        CommitTestHook::PanicAfterApply,
        CommitTestHook::PanicAfterValidation,
        CommitTestHook::PanicAfterPreparation,
    ];
    for hook in hooks {
        let mut runtime = runtime(4);
        let before = runtime.committed();
        let transaction = changed_transaction(&runtime, 10);

        let panic = catch_unwind(AssertUnwindSafe(|| {
            runtime.commit_with_test_hook(transaction, hook)
        }));

        assert!(panic.is_err());
        assert!(before.shares_state_with(&runtime.committed()));
        assert_eq!(runtime.committed().generation().get(), 0);
        assert_eq!(
            runtime.committed().property(before.root(), PROPERTY),
            Some(&PropertyValue::ScalarI32(0))
        );
    }
}

#[test]
fn every_injected_draft_corruption_is_rejected_without_publication() {
    let hooks = [
        CommitTestHook::CorruptPropertiesBeforeValidation,
        CommitTestHook::CorruptTreeBeforeValidation,
        CommitTestHook::CorruptFragmentBeforeValidation,
    ];
    for hook in hooks {
        let mut runtime = runtime(4);
        let before = runtime.committed();
        let transaction = changed_transaction(&runtime, 10);

        let error = runtime
            .commit_with_test_hook(transaction, hook)
            .unwrap_err();

        assert_eq!(error.kind(), TransactionErrorKind::InvariantViolation);
        assert_eq!(error.operation_index(), None);
        assert!(before.shares_state_with(&runtime.committed()));
        assert_eq!(runtime.committed().generation().get(), 0);
    }
}

#[test]
fn generation_exhaustion_is_checked_only_for_real_publication() {
    let mut runtime = runtime(4);
    runtime.set_generation_for_test(u64::MAX);
    let empty = runtime.begin_transaction();
    let receipt = runtime.commit(empty).unwrap();
    assert!(receipt.is_empty());
    assert_eq!(receipt.generation().get(), u64::MAX);

    let transaction = changed_transaction(&runtime, 10);
    let error = runtime.commit(transaction).unwrap_err();
    assert_eq!(error.kind(), TransactionErrorKind::GenerationExhausted);
    assert_eq!(error.operation_index(), None);
    assert_eq!(runtime.committed().generation().get(), u64::MAX);
}

#[test]
fn retained_generation_capacity_precedes_generation_exhaustion() {
    let mut runtime = runtime(0);
    runtime.set_generation_for_test(u64::MAX);
    let transaction = changed_transaction(&runtime, 10);

    let error = runtime.commit(transaction).unwrap_err();

    assert_eq!(
        error.kind(),
        TransactionErrorKind::CapacityExceeded(CapacityKind::RetainedGenerations)
    );
    assert_eq!(error.operation_index(), None);
}

mod spatial;
