use fenestra_ui_ir::prototype::{
    ChildSlot, ComponentSchema, ComponentTypeId, ConstructionProgram, InvalidationClass,
    InvalidationSet, PropertyId, PropertySchema, PropertyValue, SUPPORTED_CONSTRUCTION_FORMAT,
    SUPPORTED_SCHEMA_FORMAT, SUPPORTED_STYLE_FORMAT, SchemaManifest, SchemaNamespace,
    SchemaRevision, SourceSpan, StructuralRegion, StyleProgram, StyleValidationLimits,
    TemplateNode, TemplateNodeId, ValidatedStyleProgram, ValidationLimits, validate_construction,
    validate_schema, validate_style,
};
use fenestra_ui_runtime::prototype::{
    HeadlessProjectionCapacity, HeadlessProjectionSpec, HeadlessSurface,
};

use super::headless::{
    COLOR, COMPONENT, CONTROL, HEIGHT, INPUT, ITEMS, ROOT, SEMANTIC_LABEL, SURFACE_HEIGHT,
    SURFACE_WIDTH, VISIBLE, WIDTH,
};

pub const MISSING_PROPERTY: PropertyId = PropertyId::new(u32::MAX);
pub const MISSING_TEMPLATE: TemplateNodeId = TemplateNodeId::new(u32::MAX);

const TARGET_COMPONENT: ComponentTypeId = ComponentTypeId::new(1);
const NAMESPACE: SchemaNamespace = SchemaNamespace::new(8002);
const REVISION: SchemaRevision = SchemaRevision::new(1);
const LIMITS: ValidationLimits = ValidationLimits::new(2, 10, 2, 1, 1, 0, 0, 2, 2);

#[must_use]
pub const fn projection_capacity(computed_styles: usize) -> HeadlessProjectionCapacity {
    HeadlessProjectionCapacity::new(computed_styles, 8, 1, 8, 8)
}

#[must_use]
pub const fn surface() -> HeadlessSurface {
    HeadlessSurface::new(SURFACE_WIDTH, SURFACE_HEIGHT)
}

#[derive(Clone, Copy)]
pub struct HeadlessSpecBuilder {
    width: PropertyId,
    height: PropertyId,
    color: PropertyId,
    visible: PropertyId,
    input: PropertyId,
    semantic_template: TemplateNodeId,
    capacity: HeadlessProjectionCapacity,
}

impl HeadlessSpecBuilder {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            width: WIDTH,
            height: HEIGHT,
            color: COLOR,
            visible: VISIBLE,
            input: INPUT,
            semantic_template: CONTROL,
            capacity: projection_capacity(8),
        }
    }

    #[must_use]
    pub const fn with_width(mut self, width: PropertyId) -> Self {
        self.width = width;
        self
    }

    #[must_use]
    pub const fn with_height(mut self, height: PropertyId) -> Self {
        self.height = height;
        self
    }

    #[must_use]
    pub const fn with_color(mut self, color: PropertyId) -> Self {
        self.color = color;
        self
    }

    #[must_use]
    pub const fn with_visible(mut self, visible: PropertyId) -> Self {
        self.visible = visible;
        self
    }

    #[must_use]
    pub const fn with_input(mut self, input: PropertyId) -> Self {
        self.input = input;
        self
    }

    #[must_use]
    pub const fn with_semantic_template(mut self, semantic_template: TemplateNodeId) -> Self {
        self.semantic_template = semantic_template;
        self
    }

    #[must_use]
    pub const fn with_computed_capacity(mut self, computed_styles: usize) -> Self {
        self.capacity = projection_capacity(computed_styles);
        self
    }

    #[must_use]
    pub const fn with_capacity(mut self, capacity: HeadlessProjectionCapacity) -> Self {
        self.capacity = capacity;
        self
    }

    #[must_use]
    pub const fn build(self) -> HeadlessProjectionSpec {
        HeadlessProjectionSpec::new(
            self.width,
            self.height,
            self.color,
            self.visible,
            self.input,
            self.semantic_template,
            SEMANTIC_LABEL,
            self.capacity,
        )
    }
}

impl Default for HeadlessSpecBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy)]
enum TargetPlacement {
    Static,
    EmptyRegion,
}

pub struct HeterogeneousHeadlessBuilder {
    target_properties: Vec<(PropertyId, PropertyValue)>,
    placement: TargetPlacement,
}

impl HeterogeneousHeadlessBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self {
            target_properties: role_defaults(),
            placement: TargetPlacement::Static,
        }
    }

    #[must_use]
    pub fn without_target_property(mut self, property: PropertyId) -> Self {
        self.target_properties
            .retain(|(candidate, _)| *candidate != property);
        self
    }

    #[must_use]
    pub fn with_target_default(mut self, property: PropertyId, value: PropertyValue) -> Self {
        let (_, default) = self
            .target_properties
            .iter_mut()
            .find(|(candidate, _)| *candidate == property)
            .expect("target property should exist before replacement");
        *default = value;
        self
    }

    #[must_use]
    pub const fn target_under_empty_region(mut self) -> Self {
        self.placement = TargetPlacement::EmptyRegion;
        self
    }

    #[must_use]
    pub fn empty_style(self) -> ValidatedStyleProgram {
        let span = SourceSpan::synthetic();
        let schema = validate_schema(
            SchemaManifest::new(
                SUPPORTED_SCHEMA_FORMAT,
                NAMESPACE,
                REVISION,
                vec![
                    component(COMPONENT, role_defaults(), span),
                    component(TARGET_COMPONENT, self.target_properties, span),
                ],
                span,
            ),
            LIMITS,
        )
        .expect("heterogeneous headless schema should validate");
        let (children, regions) = match self.placement {
            TargetPlacement::Static => (vec![ChildSlot::static_node(CONTROL, span)], Vec::new()),
            TargetPlacement::EmptyRegion => (
                vec![ChildSlot::region(ITEMS, span)],
                vec![StructuralRegion::new(
                    ITEMS,
                    ROOT,
                    CONTROL,
                    Vec::new(),
                    InvalidationSet::from_class(InvalidationClass::Structure),
                    span,
                )],
            ),
        };
        let construction = validate_construction(
            &schema,
            ConstructionProgram::new(
                SUPPORTED_CONSTRUCTION_FORMAT,
                NAMESPACE,
                REVISION,
                vec![
                    TemplateNode::new(ROOT, COMPONENT, Vec::new(), children, span),
                    TemplateNode::new(CONTROL, TARGET_COMPONENT, Vec::new(), Vec::new(), span),
                ],
                regions,
                span,
            ),
            LIMITS,
        )
        .expect("heterogeneous headless construction should validate");
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
        .expect("heterogeneous empty style should validate")
    }
}

impl Default for HeterogeneousHeadlessBuilder {
    fn default() -> Self {
        Self::new()
    }
}

fn component(
    id: ComponentTypeId,
    properties: Vec<(PropertyId, PropertyValue)>,
    span: SourceSpan,
) -> ComponentSchema {
    ComponentSchema::new(
        id,
        properties
            .into_iter()
            .map(|(id, default)| {
                let value_type = default.value_type();
                PropertySchema::new(
                    id,
                    value_type,
                    default,
                    InvalidationSet::from_class(InvalidationClass::Paint),
                    span,
                )
            })
            .collect(),
        span,
    )
}

fn role_defaults() -> Vec<(PropertyId, PropertyValue)> {
    vec![
        (WIDTH, PropertyValue::ScalarI32(100)),
        (HEIGHT, PropertyValue::ScalarI32(80)),
        (COLOR, PropertyValue::Rgba8([1, 1, 1, 255])),
        (VISIBLE, PropertyValue::Bool(true)),
        (
            INPUT,
            PropertyValue::InputPolicy(fenestra_ui_ir::prototype::InputPolicy::Ignore),
        ),
    ]
}
