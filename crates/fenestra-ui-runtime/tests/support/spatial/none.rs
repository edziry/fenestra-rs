use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use fenestra_ui_ir::prototype::{
    ComponentSchema, ComponentTypeId, ConstructionProgram, InvalidationClass, InvalidationSet,
    PropertyId, PropertySchema, PropertyValue, SUPPORTED_CONSTRUCTION_FORMAT,
    SUPPORTED_SCHEMA_FORMAT, SUPPORTED_STYLE_FORMAT, SchemaManifest, SchemaNamespace,
    SchemaRevision, SourceSpan, StyleProgram, StyleValidationLimits, TemplateNode, TemplateNodeId,
    ValidatedStyleProgram, ValidationLimits, ValueType, validate_construction, validate_schema,
    validate_style,
};
use fenestra_ui_spatial::prototype::SpatialViewportV2;

use crate::{RuntimeSpatialBuildViewV2, RuntimeSpatialInputV2, RuntimeSpatialProgramV2};

use super::input::layout_source;

pub const VALUE: PropertyId = PropertyId::new(0);

const COMPONENT: ComponentTypeId = ComponentTypeId::new(0);
const ROOT: TemplateNodeId = TemplateNodeId::new(0);
const NAMESPACE: SchemaNamespace = SchemaNamespace::new(9913);
const REVISION: SchemaRevision = SchemaRevision::new(1);
const IR_LIMITS: ValidationLimits = ValidationLimits::new(4, 4, 4, 4, 4, 4, 4, 4, 4);

pub fn style() -> ValidatedStyleProgram {
    let span = SourceSpan::synthetic();
    let schema = validate_schema(
        SchemaManifest::new(
            SUPPORTED_SCHEMA_FORMAT,
            NAMESPACE,
            REVISION,
            vec![ComponentSchema::new(
                COMPONENT,
                vec![PropertySchema::new(
                    VALUE,
                    ValueType::ScalarI32,
                    PropertyValue::ScalarI32(0),
                    InvalidationSet::from_class(InvalidationClass::StyleMatch),
                    span,
                )],
                span,
            )],
            span,
        ),
        IR_LIMITS,
    )
    .expect("style-match-only schema should validate");
    let construction = validate_construction(
        &schema,
        ConstructionProgram::new(
            SUPPORTED_CONSTRUCTION_FORMAT,
            NAMESPACE,
            REVISION,
            vec![TemplateNode::new(
                ROOT,
                COMPONENT,
                Vec::new(),
                Vec::new(),
                span,
            )],
            Vec::new(),
            span,
        ),
        IR_LIMITS,
    )
    .expect("style-match-only construction should validate");
    validate_style(
        &construction,
        StyleProgram::new(
            SUPPORTED_STYLE_FORMAT,
            NAMESPACE,
            REVISION,
            Vec::new(),
            span,
        ),
        StyleValidationLimits::new(0),
    )
    .expect("style-match-only style should validate")
}

#[derive(Default)]
pub struct RootOnlyState {
    calls: AtomicUsize,
    values: Mutex<Vec<i32>>,
}

impl RootOnlyState {
    pub fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }

    pub fn values(&self) -> Vec<i32> {
        self.values
            .lock()
            .expect("root-only facts should be available")
            .clone()
    }
}

pub struct RootOnlyProgram {
    state: Arc<RootOnlyState>,
}

impl RootOnlyProgram {
    pub fn new() -> (Self, Arc<RootOnlyState>) {
        let state = Arc::new(RootOnlyState::default());
        (
            Self {
                state: Arc::clone(&state),
            },
            state,
        )
    }
}

impl RuntimeSpatialProgramV2 for RootOnlyProgram {
    fn build(
        &self,
        runtime: RuntimeSpatialBuildViewV2<'_>,
        viewport: SpatialViewportV2,
    ) -> RuntimeSpatialInputV2 {
        let Some(PropertyValue::ScalarI32(value)) = runtime.property(runtime.root(), VALUE) else {
            panic!("root-only property should retain its type");
        };
        self.state.calls.fetch_add(1, Ordering::SeqCst);
        self.state
            .values
            .lock()
            .expect("root-only facts should be available")
            .push(*value);
        RuntimeSpatialInputV2::new(layout_source(viewport, &[]), Box::new([]))
    }
}
