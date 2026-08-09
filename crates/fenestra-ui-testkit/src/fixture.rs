use fenestra_ui_ir::prototype::{
    ChildSlot, ComponentSchema, ComponentTypeId, ConstructionProgram, InitialKey, InitialProperty,
    InputPolicy, InvalidationClass, InvalidationSet, PropertyId, PropertySchema, PropertyValue,
    SUPPORTED_CONSTRUCTION_FORMAT, SUPPORTED_SCHEMA_FORMAT, SchemaManifest, SchemaNamespace,
    SchemaRevision, SourceSpan, StructuralRegion, StructuralRegionId, TemplateNode, TemplateNodeId,
    ValidatedConstruction, ValidationLimits, ValueType, validate_construction, validate_schema,
};
use fenestra_ui_runtime::prototype::RuntimeCapacity;

use crate::error::{HarnessError, HarnessErrorKind};

pub(crate) const SCHEMA_NAMESPACE: SchemaNamespace = SchemaNamespace::new(5_001);
pub(crate) const SCHEMA_REVISION: SchemaRevision = SchemaRevision::new(1);
const PAINT: InvalidationSet = InvalidationSet::from_class(InvalidationClass::Paint);
const HIT_TEST: InvalidationSet = InvalidationSet::from_class(InvalidationClass::HitTest);

/// Fixed resource ceilings for the registered runtime-oracle harness.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HarnessLimitsV1 {
    pub(crate) transactions: usize,
    pub(crate) operations_per_transaction: usize,
    pub(crate) operations: usize,
    pub(crate) live_memberships: usize,
    pub(crate) path_depth: usize,
    pub(crate) normalized_nodes: usize,
    pub(crate) normalized_fragments: usize,
    pub(crate) normalized_properties: usize,
    pub(crate) applicable_actions: usize,
    pub(crate) trace_bytes: usize,
}

impl HarnessLimitsV1 {
    const REGISTERED: Self = Self {
        transactions: 64,
        operations_per_transaction: 4,
        operations: 256,
        live_memberships: 12,
        path_depth: 8,
        normalized_nodes: 256,
        normalized_fragments: 128,
        normalized_properties: 1_024,
        applicable_actions: 16_384,
        trace_bytes: 262_144,
    };

    /// Returns the transaction-count ceiling.
    #[must_use]
    pub const fn transactions(self) -> usize {
        self.transactions
    }

    /// Returns the operation ceiling for one transaction.
    #[must_use]
    pub const fn operations_per_transaction(self) -> usize {
        self.operations_per_transaction
    }

    /// Returns the operation ceiling for one generated case.
    #[must_use]
    pub const fn operations(self) -> usize {
        self.operations
    }

    /// Returns the complete desired-state keyed-membership ceiling.
    #[must_use]
    pub const fn live_memberships(self) -> usize {
        self.live_memberships
    }

    /// Returns the semantic node-path depth ceiling.
    #[must_use]
    pub const fn path_depth(self) -> usize {
        self.path_depth
    }

    /// Returns the normalized-node ceiling.
    #[must_use]
    pub const fn normalized_nodes(self) -> usize {
        self.normalized_nodes
    }

    /// Returns the normalized-fragment ceiling.
    #[must_use]
    pub const fn normalized_fragments(self) -> usize {
        self.normalized_fragments
    }

    /// Returns the normalized-property-slot ceiling.
    #[must_use]
    pub const fn normalized_properties(self) -> usize {
        self.normalized_properties
    }

    /// Returns the applicable-action ceiling for one generator choice.
    #[must_use]
    pub const fn applicable_actions(self) -> usize {
        self.applicable_actions
    }

    /// Returns the transient logical-trace byte ceiling.
    #[must_use]
    pub const fn trace_bytes(self) -> usize {
        self.trace_bytes
    }
}

/// Fixed capacity used when replaying the registered runtime fixture.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReplayConfigV1 {
    operations: usize,
    structural_changes: usize,
    live_nodes: usize,
    live_fragments: usize,
    live_property_slots: usize,
    retained_generations: usize,
}

impl ReplayConfigV1 {
    const REGISTERED: Self = Self {
        operations: 4,
        structural_changes: 64,
        live_nodes: 256,
        live_fragments: 128,
        live_property_slots: 1_024,
        retained_generations: 1,
    };

    /// Converts the replay values to the runtime's capacity type.
    #[must_use]
    pub const fn runtime_capacity(self) -> RuntimeCapacity {
        RuntimeCapacity::new(
            self.operations,
            self.structural_changes,
            self.live_nodes,
            self.live_fragments,
            self.live_property_slots,
            self.retained_generations,
        )
    }
}

/// Validated synthetic construction and fixed bounds for runtime-oracle V1.
#[derive(Clone, Debug)]
pub struct RuntimeOracleFixtureV1 {
    construction: ValidatedConstruction,
}

impl RuntimeOracleFixtureV1 {
    /// Builds and validates the registered synthetic fixture.
    pub fn build() -> Result<Self, HarnessError> {
        let limits = validation_limits();
        let schema = validate_schema(schema_manifest(), limits)
            .map_err(|_| HarnessError::new(HarnessErrorKind::FixtureValidation))?;
        let construction = validate_construction(&schema, construction_program(), limits)
            .map_err(|_| HarnessError::new(HarnessErrorKind::FixtureValidation))?;
        Ok(Self { construction })
    }

    /// Returns the validated construction shared by all V1 oracle runs.
    #[must_use]
    pub const fn construction(&self) -> &ValidatedConstruction {
        &self.construction
    }

    /// Returns the fixed harness ceilings for this fixture revision.
    #[must_use]
    pub const fn harness_limits(&self) -> HarnessLimitsV1 {
        HarnessLimitsV1::REGISTERED
    }

    /// Returns the fixed runtime replay capacity for this fixture revision.
    #[must_use]
    pub const fn replay_config(&self) -> ReplayConfigV1 {
        ReplayConfigV1::REGISTERED
    }
}

const fn validation_limits() -> ValidationLimits {
    ValidationLimits::new(6, 10, 6, 3, 5, 1, 4, 3, 9)
}

fn schema_manifest() -> SchemaManifest {
    SchemaManifest::new(
        SUPPORTED_SCHEMA_FORMAT,
        SCHEMA_NAMESPACE,
        SCHEMA_REVISION,
        vec![
            component(
                0,
                vec![
                    property(
                        0,
                        ValueType::ScalarI32,
                        PropertyValue::ScalarI32(100),
                        layout_paint(),
                    ),
                    property(
                        1,
                        ValueType::Bool,
                        PropertyValue::Bool(true),
                        semantics_hit_paint(),
                    ),
                    property(
                        2,
                        ValueType::Rgba8,
                        PropertyValue::Rgba8([0, 0, 0, 255]),
                        PAINT,
                    ),
                    property(
                        3,
                        ValueType::InputPolicy,
                        PropertyValue::InputPolicy(InputPolicy::Accept),
                        HIT_TEST,
                    ),
                ],
            ),
            component(
                1,
                vec![property(
                    0,
                    ValueType::Bool,
                    PropertyValue::Bool(true),
                    semantics_paint(),
                )],
            ),
            component(
                2,
                vec![
                    property(
                        0,
                        ValueType::ScalarI32,
                        PropertyValue::ScalarI32(10),
                        intrinsic_layout_paint(),
                    ),
                    property(
                        1,
                        ValueType::Bool,
                        PropertyValue::Bool(true),
                        semantics_hit_paint(),
                    ),
                ],
            ),
            component(
                3,
                vec![property(
                    0,
                    ValueType::Rgba8,
                    PropertyValue::Rgba8([255; 4]),
                    PAINT,
                )],
            ),
            component(
                4,
                vec![property(
                    0,
                    ValueType::ScalarI32,
                    PropertyValue::ScalarI32(1),
                    intrinsic_layout_paint(),
                )],
            ),
            component(
                5,
                vec![property(
                    0,
                    ValueType::ScalarI32,
                    PropertyValue::ScalarI32(20),
                    intrinsic_layout_paint(),
                )],
            ),
        ],
        SourceSpan::synthetic(),
    )
}

fn construction_program() -> ConstructionProgram {
    let span = SourceSpan::synthetic();
    ConstructionProgram::new(
        SUPPORTED_CONSTRUCTION_FORMAT,
        SCHEMA_NAMESPACE,
        SCHEMA_REVISION,
        vec![
            TemplateNode::new(
                TemplateNodeId::new(0),
                ComponentTypeId::new(0),
                vec![InitialProperty::new(
                    PropertyId::new(0),
                    PropertyValue::ScalarI32(120),
                    span,
                )],
                vec![
                    ChildSlot::static_node(TemplateNodeId::new(1), span),
                    ChildSlot::region(StructuralRegionId::new(0), span),
                    ChildSlot::region(StructuralRegionId::new(2), span),
                ],
                span,
            ),
            template(1, 1, Vec::new()),
            template(
                2,
                2,
                vec![
                    ChildSlot::static_node(TemplateNodeId::new(3), span),
                    ChildSlot::region(StructuralRegionId::new(1), span),
                ],
            ),
            template(3, 3, Vec::new()),
            template(4, 4, Vec::new()),
            template(5, 5, Vec::new()),
        ],
        vec![
            region(0, 0, 2, &[7, 8], structure_layout_paint()),
            region(1, 2, 4, &[1], structure_layout_paint()),
            region(2, 0, 5, &[7], structure_paint()),
        ],
        span,
    )
}

fn component(id: u32, properties: Vec<PropertySchema>) -> ComponentSchema {
    ComponentSchema::new(
        ComponentTypeId::new(id),
        properties,
        SourceSpan::synthetic(),
    )
}

fn property(
    id: u32,
    value_type: ValueType,
    default: PropertyValue,
    invalidation: InvalidationSet,
) -> PropertySchema {
    PropertySchema::new(
        PropertyId::new(id),
        value_type,
        default,
        invalidation,
        SourceSpan::synthetic(),
    )
}

fn template(id: u32, component_id: u32, children: Vec<ChildSlot>) -> TemplateNode {
    TemplateNode::new(
        TemplateNodeId::new(id),
        ComponentTypeId::new(component_id),
        Vec::new(),
        children,
        SourceSpan::synthetic(),
    )
}

fn region(
    id: u32,
    owner: u32,
    repeat_body: u32,
    keys: &[u64],
    invalidation: InvalidationSet,
) -> StructuralRegion {
    StructuralRegion::new(
        StructuralRegionId::new(id),
        TemplateNodeId::new(owner),
        TemplateNodeId::new(repeat_body),
        keys.iter()
            .copied()
            .map(|key| InitialKey::new(key, SourceSpan::synthetic()))
            .collect(),
        invalidation,
        SourceSpan::synthetic(),
    )
}

const fn layout_paint() -> InvalidationSet {
    InvalidationSet::from_class(InvalidationClass::Layout).union(PAINT)
}

const fn semantics_paint() -> InvalidationSet {
    InvalidationSet::from_class(InvalidationClass::Semantics).union(PAINT)
}

const fn semantics_hit_paint() -> InvalidationSet {
    InvalidationSet::from_class(InvalidationClass::Semantics)
        .union(HIT_TEST)
        .union(PAINT)
}

const fn intrinsic_layout_paint() -> InvalidationSet {
    InvalidationSet::from_class(InvalidationClass::Intrinsic).union(layout_paint())
}

const fn structure_paint() -> InvalidationSet {
    InvalidationSet::from_class(InvalidationClass::Structure).union(PAINT)
}

const fn structure_layout_paint() -> InvalidationSet {
    InvalidationSet::from_class(InvalidationClass::Structure).union(layout_paint())
}
