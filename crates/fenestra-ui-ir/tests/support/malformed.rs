use fenestra_ui_ir::prototype::{
    ChildSlot, ComponentTypeId, ConstructionFormatVersion, ConstructionProgram, InvalidationClass,
    InvalidationSet, IrValidationErrorKind, PropertyId, PropertyValue, SchemaFormatVersion,
    SchemaManifest, SchemaNamespace, SourceSpan, StructuralRegionId, TemplateNodeId,
    ValidationLimits, ValueType,
};

use super::{
    COMPONENT, PROPERTY, REGION, REPEAT, ROOT, SCHEMA_NAMESPACE, SCHEMA_REVISION, TEST_LIMITS,
    basic_region, component, initial_property, key, manifest_with, node, program_with, property,
    region, region_with_invalidation, repeat, root, scalar_property, span,
};

#[derive(Clone, Copy, Debug)]
pub enum Fault {
    UnsupportedSchemaFormat,
    UnsupportedConstructionFormat,
    SchemaIdentityMismatch,
    InvalidSourceSpan,
    DuplicateComponent,
    DuplicateProperty,
    PropertyDefaultTypeMismatch,
    EmptyPropertyInvalidation,
    InvalidPropertyInvalidation,
    InvalidPropertySurface,
    DuplicateNode,
    DuplicateInitialProperty,
    UnknownInitialProperty,
    InitialPropertyTypeMismatch,
    DuplicateRegion,
    MissingComponent,
    MissingStaticChild,
    MissingRegion,
    MissingRegionOwner,
    RegionOwnerMismatch,
    DuplicateRegionPlacement,
    MissingRegionTemplate,
    UnplacedRegion,
    DuplicateNodeOwner,
    InvalidRootCount,
    OwnershipCycle,
    DuplicateRegionKey,
    InvalidRegionInvalidation,
    InvalidRegionSurface,
    LimitComponents,
    LimitProperties,
    LimitTemplates,
    LimitRegions,
    LimitChildSlots,
    LimitInitialProperties,
    LimitInitialKeys,
    LimitTemplateDepth,
    LimitInitialInstances,
    TwoDuplicateComponents,
}

type Malformed = (
    SchemaManifest,
    ConstructionProgram,
    ValidationLimits,
    IrValidationErrorKind,
    SourceSpan,
);

pub fn malformed_fixture(fault: Fault) -> Malformed {
    let mut schema_format = fenestra_ui_ir::prototype::SUPPORTED_SCHEMA_FORMAT;
    let schema_namespace = SCHEMA_NAMESPACE;
    let schema_revision = SCHEMA_REVISION;
    let mut construction_format = fenestra_ui_ir::prototype::SUPPORTED_CONSTRUCTION_FORMAT;
    let mut construction_namespace = SCHEMA_NAMESPACE;
    let construction_revision = SCHEMA_REVISION;
    let mut components = vec![component(
        COMPONENT,
        vec![scalar_property(PROPERTY, span(2))],
        span(1),
    )];
    let mut nodes = vec![
        root(vec![ChildSlot::region(REGION, span(7))]),
        repeat(vec![initial_property(
            PROPERTY,
            PropertyValue::ScalarI32(10),
            span(9),
        )]),
    ];
    let mut regions = vec![basic_region(vec![key(7, span(10))])];
    let mut limits = TEST_LIMITS;

    let (kind, expected_span) = match fault {
        Fault::UnsupportedSchemaFormat => {
            schema_format = SchemaFormatVersion::new(2);
            (IrValidationErrorKind::UnsupportedSchemaFormat, span(0))
        }
        Fault::UnsupportedConstructionFormat => {
            construction_format = ConstructionFormatVersion::new(2);
            (
                IrValidationErrorKind::UnsupportedConstructionFormat,
                span(4),
            )
        }
        Fault::SchemaIdentityMismatch => {
            construction_namespace = SchemaNamespace::new(9);
            (IrValidationErrorKind::SchemaIdentityMismatch, span(4))
        }
        Fault::InvalidSourceSpan => {
            let invalid = SourceSpan::bytes(super::SourceId::new(0), 20, 10);
            components[0] = component(COMPONENT, vec![scalar_property(PROPERTY, span(2))], invalid);
            (IrValidationErrorKind::InvalidSourceSpan, invalid)
        }
        Fault::DuplicateComponent | Fault::TwoDuplicateComponents => {
            components.push(component(
                COMPONENT,
                vec![scalar_property(PROPERTY, span(12))],
                span(11),
            ));
            if matches!(fault, Fault::TwoDuplicateComponents) {
                components.push(component(
                    COMPONENT,
                    vec![scalar_property(PROPERTY, span(14))],
                    span(13),
                ));
            }
            (IrValidationErrorKind::DuplicateComponent, span(11))
        }
        Fault::DuplicateProperty => {
            components[0] = component(
                COMPONENT,
                vec![
                    scalar_property(PROPERTY, span(2)),
                    scalar_property(PROPERTY, span(11)),
                ],
                span(1),
            );
            (IrValidationErrorKind::DuplicateProperty, span(11))
        }
        Fault::PropertyDefaultTypeMismatch => {
            components[0] = component(
                COMPONENT,
                vec![property(
                    PROPERTY,
                    ValueType::Bool,
                    PropertyValue::ScalarI32(0),
                    InvalidationSet::from_class(InvalidationClass::Layout),
                    span(2),
                )],
                span(1),
            );
            (IrValidationErrorKind::PropertyDefaultTypeMismatch, span(2))
        }
        Fault::EmptyPropertyInvalidation => {
            components[0] = component(
                COMPONENT,
                vec![property(
                    PROPERTY,
                    ValueType::ScalarI32,
                    PropertyValue::ScalarI32(0),
                    InvalidationSet::NONE,
                    span(2),
                )],
                span(1),
            );
            (IrValidationErrorKind::EmptyPropertyInvalidation, span(2))
        }
        Fault::InvalidPropertyInvalidation => {
            components[0] = component(
                COMPONENT,
                vec![property(
                    PROPERTY,
                    ValueType::ScalarI32,
                    PropertyValue::ScalarI32(0),
                    InvalidationSet::from_class(InvalidationClass::Structure),
                    span(2),
                )],
                span(1),
            );
            (IrValidationErrorKind::InvalidPropertyInvalidation, span(2))
        }
        Fault::InvalidPropertySurface => {
            components[0] = component(
                COMPONENT,
                vec![property(
                    PROPERTY,
                    ValueType::ScalarI32,
                    PropertyValue::ScalarI32(0),
                    InvalidationSet::from_class(InvalidationClass::Surface),
                    span(2),
                )],
                span(1),
            );
            (IrValidationErrorKind::InvalidPropertyInvalidation, span(2))
        }
        Fault::DuplicateNode => {
            nodes.insert(1, node(ROOT, COMPONENT, Vec::new(), Vec::new(), span(11)));
            (IrValidationErrorKind::DuplicateNode, span(11))
        }
        Fault::DuplicateInitialProperty => {
            nodes[1] = repeat(vec![
                initial_property(PROPERTY, PropertyValue::ScalarI32(10), span(9)),
                initial_property(PROPERTY, PropertyValue::ScalarI32(20), span(11)),
            ]);
            (IrValidationErrorKind::DuplicateInitialProperty, span(11))
        }
        Fault::UnknownInitialProperty => {
            nodes[1] = repeat(vec![initial_property(
                PropertyId::new(9),
                PropertyValue::ScalarI32(10),
                span(11),
            )]);
            (IrValidationErrorKind::UnknownInitialProperty, span(11))
        }
        Fault::InitialPropertyTypeMismatch => {
            nodes[1] = repeat(vec![initial_property(
                PROPERTY,
                PropertyValue::Bool(true),
                span(11),
            )]);
            (IrValidationErrorKind::InitialPropertyTypeMismatch, span(11))
        }
        Fault::DuplicateRegion => {
            regions.push(region(REGION, ROOT, REPEAT, Vec::new(), span(11)));
            (IrValidationErrorKind::DuplicateRegion, span(11))
        }
        Fault::MissingComponent => {
            nodes[0] = node(
                ROOT,
                ComponentTypeId::new(9),
                Vec::new(),
                vec![ChildSlot::region(REGION, span(7))],
                span(5),
            );
            (IrValidationErrorKind::MissingComponent, span(5))
        }
        Fault::MissingStaticChild => {
            nodes[0] = root(vec![
                ChildSlot::static_node(TemplateNodeId::new(9), span(11)),
                ChildSlot::region(REGION, span(7)),
            ]);
            (IrValidationErrorKind::MissingStaticChild, span(11))
        }
        Fault::MissingRegion => {
            nodes[0] = root(vec![ChildSlot::region(
                StructuralRegionId::new(9),
                span(11),
            )]);
            regions.clear();
            (IrValidationErrorKind::MissingRegion, span(11))
        }
        Fault::MissingRegionOwner => {
            regions[0] = region(REGION, TemplateNodeId::new(9), REPEAT, Vec::new(), span(8));
            (IrValidationErrorKind::MissingRegionOwner, span(8))
        }
        Fault::RegionOwnerMismatch => {
            let owner = TemplateNodeId::new(1);
            let body = TemplateNodeId::new(2);
            nodes = vec![
                root(vec![
                    ChildSlot::static_node(owner, span(11)),
                    ChildSlot::region(REGION, span(12)),
                ]),
                node(owner, COMPONENT, Vec::new(), Vec::new(), span(6)),
                node(body, COMPONENT, Vec::new(), Vec::new(), span(13)),
            ];
            regions[0] = region(REGION, owner, body, Vec::new(), span(8));
            (IrValidationErrorKind::RegionOwnerMismatch, span(12))
        }
        Fault::DuplicateRegionPlacement => {
            nodes[0] = root(vec![
                ChildSlot::region(REGION, span(7)),
                ChildSlot::region(REGION, span(11)),
            ]);
            (IrValidationErrorKind::DuplicateRegionPlacement, span(11))
        }
        Fault::MissingRegionTemplate => {
            regions[0] = region(REGION, ROOT, TemplateNodeId::new(9), Vec::new(), span(8));
            (IrValidationErrorKind::MissingRegionTemplate, span(8))
        }
        Fault::UnplacedRegion => {
            nodes[0] = root(Vec::new());
            (IrValidationErrorKind::UnplacedRegion, span(8))
        }
        Fault::DuplicateNodeOwner => {
            nodes[0] = root(vec![
                ChildSlot::static_node(REPEAT, span(11)),
                ChildSlot::region(REGION, span(7)),
            ]);
            (IrValidationErrorKind::DuplicateNodeOwner, span(8))
        }
        Fault::InvalidRootCount => {
            nodes = vec![root(Vec::new()), repeat(Vec::new())];
            regions.clear();
            (IrValidationErrorKind::InvalidRootCount, span(4))
        }
        Fault::OwnershipCycle => {
            let first = TemplateNodeId::new(1);
            let second = TemplateNodeId::new(2);
            nodes = vec![
                root(Vec::new()),
                node(
                    first,
                    COMPONENT,
                    Vec::new(),
                    vec![ChildSlot::static_node(second, span(11))],
                    span(6),
                ),
                node(
                    second,
                    COMPONENT,
                    Vec::new(),
                    vec![ChildSlot::static_node(first, span(12))],
                    span(13),
                ),
            ];
            regions.clear();
            (IrValidationErrorKind::OwnershipCycle, span(12))
        }
        Fault::DuplicateRegionKey => {
            regions[0] = basic_region(vec![key(7, span(10)), key(7, span(11))]);
            (IrValidationErrorKind::DuplicateRegionKey, span(11))
        }
        Fault::InvalidRegionInvalidation => {
            regions[0] = region_with_invalidation(
                REGION,
                ROOT,
                REPEAT,
                Vec::new(),
                InvalidationSet::from_class(InvalidationClass::Paint),
                span(8),
            );
            (IrValidationErrorKind::InvalidRegionInvalidation, span(8))
        }
        Fault::InvalidRegionSurface => {
            regions[0] = region_with_invalidation(
                REGION,
                ROOT,
                REPEAT,
                Vec::new(),
                InvalidationSet::from_class(InvalidationClass::Structure)
                    .union(InvalidationSet::from_class(InvalidationClass::Surface)),
                span(8),
            );
            (IrValidationErrorKind::InvalidRegionInvalidation, span(8))
        }
        Fault::LimitComponents
        | Fault::LimitProperties
        | Fault::LimitTemplates
        | Fault::LimitRegions
        | Fault::LimitChildSlots
        | Fault::LimitInitialProperties
        | Fault::LimitInitialKeys
        | Fault::LimitTemplateDepth
        | Fault::LimitInitialInstances => {
            let (selected_limits, kind, source) =
                super::limits::limit_case(fault).expect("limit fault should be mapped");
            limits = selected_limits;
            (kind, source)
        }
    };

    let manifest = manifest_with(
        schema_format,
        schema_namespace,
        schema_revision,
        components,
        span(0),
    );
    let program = program_with(
        construction_format,
        construction_namespace,
        construction_revision,
        nodes,
        regions,
        span(4),
    );
    (manifest, program, limits, kind, expected_span)
}
