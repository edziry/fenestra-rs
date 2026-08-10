use fenestra_ui_ir::prototype::{
    ChildSlot, ComponentSchema, ComponentTypeId, ConstructionProgram, InitialKey, InitialProperty,
    InputPolicy, InvalidationClass, InvalidationSet, PropertyId, PropertySchema, PropertyValue,
    SUPPORTED_CONSTRUCTION_FORMAT, SUPPORTED_SCHEMA_FORMAT, SUPPORTED_STYLE_FORMAT, SchemaManifest,
    SchemaNamespace, SchemaRevision, SourceSpan, StructuralRegion, StructuralRegionId,
    StyleAssignment, StyleProgram, StyleValidationLimits, TemplateNode, TemplateNodeId,
    ValidatedStyleProgram, ValidationLimits, ValueType, validate_construction, validate_schema,
    validate_style,
};
use fenestra_ui_runtime::prototype::{
    HeadlessProjectionCapacity, HeadlessProjectionSpec, HeadlessSurface, RuntimeCapacity,
};

use crate::error::{HarnessError, HarnessErrorKind};
use crate::fixture::HarnessLimitsV1;

const COMPONENT: ComponentTypeId = ComponentTypeId::new(0);
const WIDTH: PropertyId = PropertyId::new(0);
const HEIGHT: PropertyId = PropertyId::new(1);
const COLOR: PropertyId = PropertyId::new(2);
const VISIBLE: PropertyId = PropertyId::new(3);
const INPUT: PropertyId = PropertyId::new(4);
const ROOT: TemplateNodeId = TemplateNodeId::new(0);
const CONTAINER: TemplateNodeId = TemplateNodeId::new(1);
const CONTROL: TemplateNodeId = TemplateNodeId::new(2);
const ITEM: TemplateNodeId = TemplateNodeId::new(3);
const ITEMS: StructuralRegionId = StructuralRegionId::new(0);
const NAMESPACE: SchemaNamespace = SchemaNamespace::new(8_001);
const REVISION: SchemaRevision = SchemaRevision::new(1);
const IR_LIMITS: ValidationLimits = ValidationLimits::new(1, 5, 4, 1, 3, 12, 2, 3, 5);

/// Validated fixed inputs for the synthetic headless V1 oracle.
#[derive(Clone)]
pub struct HeadlessFixtureV1 {
    style: ValidatedStyleProgram,
    spec: HeadlessProjectionSpec,
    surface: HeadlessSurface,
    runtime_capacity: RuntimeCapacity,
}

impl HeadlessFixtureV1 {
    /// Builds and validates the registered headless fixture.
    pub fn build() -> Result<Self, HarnessError> {
        let schema = validate_schema(schema(), IR_LIMITS).map_err(|_| fixture_error())?;
        let construction = validate_construction(&schema, construction(), IR_LIMITS)
            .map_err(|_| fixture_error())?;
        let style = validate_style(&construction, style(), StyleValidationLimits::new(2))
            .map_err(|_| fixture_error())?;
        Ok(Self {
            style,
            spec: HeadlessProjectionSpec::new(
                WIDTH,
                HEIGHT,
                COLOR,
                VISIBLE,
                INPUT,
                CONTROL,
                1,
                HeadlessProjectionCapacity::new(8, 8, 1, 8, 8),
            ),
            surface: HeadlessSurface::new(120, 90),
            runtime_capacity: RuntimeCapacity::new(8, 8, 8, 2, 40, 3),
        })
    }

    /// Returns the exact validated style and retained construction.
    #[must_use]
    pub const fn style(&self) -> &ValidatedStyleProgram {
        &self.style
    }

    /// Returns the registered projection role specification.
    #[must_use]
    pub const fn spec(&self) -> HeadlessProjectionSpec {
        self.spec
    }

    /// Returns the initial logical surface.
    #[must_use]
    pub const fn surface(&self) -> HeadlessSurface {
        self.surface
    }

    /// Returns the registered runtime bounds.
    #[must_use]
    pub const fn runtime_capacity(&self) -> RuntimeCapacity {
        self.runtime_capacity
    }

    /// Returns the bounds used by the complete headless snapshot observer.
    #[must_use]
    pub const fn harness_limits(&self) -> HarnessLimitsV1 {
        super::oracle::ORACLE_LIMITS
    }
}

fn schema() -> SchemaManifest {
    let span = SourceSpan::synthetic();
    SchemaManifest::new(
        SUPPORTED_SCHEMA_FORMAT,
        NAMESPACE,
        REVISION,
        vec![ComponentSchema::new(
            COMPONENT,
            vec![
                property(
                    WIDTH,
                    ValueType::ScalarI32,
                    PropertyValue::ScalarI32(40),
                    dimension_invalidation(),
                ),
                property(
                    HEIGHT,
                    ValueType::ScalarI32,
                    PropertyValue::ScalarI32(10),
                    dimension_invalidation(),
                ),
                property(
                    COLOR,
                    ValueType::Rgba8,
                    PropertyValue::Rgba8([32, 32, 32, 255]),
                    InvalidationSet::from_class(InvalidationClass::Paint),
                ),
                property(
                    VISIBLE,
                    ValueType::Bool,
                    PropertyValue::Bool(true),
                    visibility_invalidation(),
                ),
                property(
                    INPUT,
                    ValueType::InputPolicy,
                    PropertyValue::InputPolicy(InputPolicy::Ignore),
                    InvalidationSet::from_class(InvalidationClass::HitTest),
                ),
            ],
            span,
        )],
        span,
    )
}

fn construction() -> ConstructionProgram {
    let span = SourceSpan::synthetic();
    ConstructionProgram::new(
        SUPPORTED_CONSTRUCTION_FORMAT,
        NAMESPACE,
        REVISION,
        vec![
            template(
                ROOT,
                vec![
                    initial(WIDTH, PropertyValue::ScalarI32(100)),
                    initial(HEIGHT, PropertyValue::ScalarI32(80)),
                    initial(COLOR, PropertyValue::Rgba8([1, 1, 1, 255])),
                ],
                vec![ChildSlot::static_node(CONTAINER, span)],
            ),
            template(
                CONTAINER,
                vec![
                    initial(WIDTH, PropertyValue::ScalarI32(80)),
                    initial(HEIGHT, PropertyValue::ScalarI32(50)),
                    initial(COLOR, PropertyValue::Rgba8([2, 2, 2, 255])),
                ],
                vec![
                    ChildSlot::static_node(CONTROL, span),
                    ChildSlot::region(ITEMS, span),
                ],
            ),
            template(
                CONTROL,
                vec![
                    initial(WIDTH, PropertyValue::ScalarI32(30)),
                    initial(COLOR, PropertyValue::Rgba8([3, 3, 3, 255])),
                    initial(INPUT, PropertyValue::InputPolicy(InputPolicy::Accept)),
                ],
                Vec::new(),
            ),
            template(
                ITEM,
                vec![
                    initial(HEIGHT, PropertyValue::ScalarI32(12)),
                    initial(COLOR, PropertyValue::Rgba8([4, 4, 4, 255])),
                    initial(INPUT, PropertyValue::InputPolicy(InputPolicy::Accept)),
                ],
                Vec::new(),
            ),
        ],
        vec![StructuralRegion::new(
            ITEMS,
            CONTAINER,
            ITEM,
            vec![InitialKey::new(10, span), InitialKey::new(20, span)],
            region_invalidation(),
            span,
        )],
        span,
    )
}

fn style() -> StyleProgram {
    let span = SourceSpan::synthetic();
    StyleProgram::new(
        SUPPORTED_STYLE_FORMAT,
        NAMESPACE,
        REVISION,
        vec![
            StyleAssignment::new(
                CONTROL,
                COLOR,
                PropertyValue::Rgba8([10, 20, 30, 255]),
                span,
            ),
            StyleAssignment::new(ITEM, COLOR, PropertyValue::Rgba8([80, 90, 100, 255]), span),
        ],
        span,
    )
}

fn property(
    id: PropertyId,
    value_type: ValueType,
    default: PropertyValue,
    invalidation: InvalidationSet,
) -> PropertySchema {
    PropertySchema::new(
        id,
        value_type,
        default,
        invalidation,
        SourceSpan::synthetic(),
    )
}

fn template(
    id: TemplateNodeId,
    properties: Vec<InitialProperty>,
    children: Vec<ChildSlot>,
) -> TemplateNode {
    TemplateNode::new(id, COMPONENT, properties, children, SourceSpan::synthetic())
}

fn initial(id: PropertyId, value: PropertyValue) -> InitialProperty {
    InitialProperty::new(id, value, SourceSpan::synthetic())
}

fn dimension_invalidation() -> InvalidationSet {
    invalidation(&[
        InvalidationClass::Layout,
        InvalidationClass::Semantics,
        InvalidationClass::HitTest,
        InvalidationClass::Paint,
        InvalidationClass::Composition,
    ])
}

fn visibility_invalidation() -> InvalidationSet {
    invalidation(&[
        InvalidationClass::Semantics,
        InvalidationClass::HitTest,
        InvalidationClass::Paint,
    ])
}

fn region_invalidation() -> InvalidationSet {
    invalidation(&[
        InvalidationClass::Structure,
        InvalidationClass::Layout,
        InvalidationClass::Semantics,
        InvalidationClass::HitTest,
        InvalidationClass::Paint,
        InvalidationClass::Composition,
    ])
}

fn invalidation(classes: &[InvalidationClass]) -> InvalidationSet {
    classes.iter().fold(InvalidationSet::NONE, |set, class| {
        set.union(InvalidationSet::from_class(*class))
    })
}

fn fixture_error() -> HarnessError {
    HarnessError::new(HarnessErrorKind::FixtureValidation)
}
