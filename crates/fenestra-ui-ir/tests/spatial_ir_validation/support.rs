#![allow(dead_code)]

use super::*;

pub const NS: SchemaNamespace = SchemaNamespace::new(41);
pub const REV: SchemaRevision = SchemaRevision::new(7);
pub const COMPONENT: ComponentTypeId = ComponentTypeId::new(3);
pub const OTHER_COMPONENT: ComponentTypeId = ComponentTypeId::new(4);
pub const SCALAR: PropertyId = PropertyId::new(10);
pub const COLOR: PropertyId = PropertyId::new(11);
pub const POLICY: PropertyId = PropertyId::new(12);
pub const ROOT: TemplateNodeId = TemplateNodeId::new(20);
pub const STATIC_A: TemplateNodeId = TemplateNodeId::new(21);
pub const STATIC_B: TemplateNodeId = TemplateNodeId::new(22);
pub const STATIC_C: TemplateNodeId = TemplateNodeId::new(23);
pub const OUTER: TemplateNodeId = TemplateNodeId::new(24);
pub const OUTER_STATIC: TemplateNodeId = TemplateNodeId::new(25);
pub const INNER: TemplateNodeId = TemplateNodeId::new(26);
pub const OTHER: TemplateNodeId = TemplateNodeId::new(27);
pub const OUTER_REGION: StructuralRegionId = StructuralRegionId::new(30);
pub const INNER_REGION: StructuralRegionId = StructuralRegionId::new(31);
pub const OTHER_REGION: StructuralRegionId = StructuralRegionId::new(32);
pub const MAX_FIXED: i64 = 140_737_488_289_792;

pub fn span(index: u32) -> SourceSpan {
    SourceSpan::bytes(SourceId::new(9), index * 10, index * 10 + 5)
}

pub fn invalid_span(index: u32) -> SourceSpan {
    SourceSpan::bytes(SourceId::new(9), index * 10 + 5, index * 10)
}

fn property(
    id: PropertyId,
    value_type: ValueType,
    default: PropertyValue,
    invalidation: InvalidationClass,
    source: SourceSpan,
) -> PropertySchema {
    PropertySchema::new(
        id,
        value_type,
        default,
        InvalidationSet::from_class(invalidation),
        source,
    )
}

fn template(id: TemplateNodeId, children: Vec<ChildSlot>, source: SourceSpan) -> TemplateNode {
    template_for(id, COMPONENT, children, source)
}

fn template_for(
    id: TemplateNodeId,
    component: ComponentTypeId,
    children: Vec<ChildSlot>,
    source: SourceSpan,
) -> TemplateNode {
    TemplateNode::new(id, component, Vec::new(), children, source)
}

fn region(
    id: StructuralRegionId,
    owner: TemplateNodeId,
    body: TemplateNodeId,
    source: SourceSpan,
) -> StructuralRegion {
    let initial_keys = if id == OUTER_REGION {
        vec![
            InitialKey::new(1, span(33)),
            InitialKey::new(2, span(34)),
            InitialKey::new(3, span(35)),
        ]
    } else {
        Vec::new()
    };
    StructuralRegion::new(
        id,
        owner,
        body,
        initial_keys,
        InvalidationSet::from_class(InvalidationClass::Structure),
        source,
    )
}

pub fn style() -> ValidatedStyleProgram {
    let manifest = SchemaManifest::new(
        SUPPORTED_SCHEMA_FORMAT,
        NS,
        REV,
        vec![
            ComponentSchema::new(
                COMPONENT,
                vec![
                    property(
                        SCALAR,
                        ValueType::ScalarI32,
                        PropertyValue::ScalarI32(4),
                        InvalidationClass::Layout,
                        span(2),
                    ),
                    property(
                        COLOR,
                        ValueType::Rgba8,
                        PropertyValue::Rgba8([1, 2, 3, 255]),
                        InvalidationClass::Paint,
                        span(3),
                    ),
                    property(
                        POLICY,
                        ValueType::InputPolicy,
                        PropertyValue::InputPolicy(InputPolicy::Accept),
                        InvalidationClass::HitTest,
                        span(4),
                    ),
                ],
                span(1),
            ),
            ComponentSchema::new(
                OTHER_COMPONENT,
                vec![property(
                    SCALAR,
                    ValueType::Rgba8,
                    PropertyValue::Rgba8([9, 8, 7, 255]),
                    InvalidationClass::Paint,
                    span(5),
                )],
                span(6),
            ),
        ],
        span(0),
    );
    let root_children = vec![
        ChildSlot::static_node(STATIC_A, span(11)),
        ChildSlot::static_node(STATIC_B, span(12)),
        ChildSlot::static_node(STATIC_C, span(13)),
        ChildSlot::region(OUTER_REGION, span(14)),
        ChildSlot::region(OTHER_REGION, span(15)),
    ];
    let outer_children = vec![
        ChildSlot::static_node(OUTER_STATIC, span(16)),
        ChildSlot::region(INNER_REGION, span(17)),
    ];
    let construction = ConstructionProgram::new(
        SUPPORTED_CONSTRUCTION_FORMAT,
        NS,
        REV,
        vec![
            template(ROOT, root_children, span(10)),
            template(STATIC_A, Vec::new(), span(20)),
            template(STATIC_B, Vec::new(), span(21)),
            template_for(STATIC_C, OTHER_COMPONENT, Vec::new(), span(22)),
            template(OUTER, outer_children, span(23)),
            template(OUTER_STATIC, Vec::new(), span(24)),
            template(INNER, Vec::new(), span(25)),
            template(OTHER, Vec::new(), span(26)),
        ],
        vec![
            region(OUTER_REGION, ROOT, OUTER, span(30)),
            region(INNER_REGION, OUTER, INNER, span(31)),
            region(OTHER_REGION, ROOT, OTHER, span(32)),
        ],
        span(9),
    );
    let limits = ValidationLimits::new(8, 8, 8, 8, 16, 0, 8, 8, 64);
    let schema = validate_schema(manifest, limits).expect("schema fixture should validate");
    let construction =
        validate_construction(&schema, construction, limits).expect("construction fixture");
    validate_style(
        &construction,
        StyleProgram::new(SUPPORTED_STYLE_FORMAT, NS, REV, Vec::new(), span(40)),
        StyleValidationLimits::new(0),
    )
    .expect("style fixture should validate")
}

pub const fn field<T>(value: T, source: SourceSpan) -> SpatialFieldV2<T> {
    SpatialFieldV2::new(value, source)
}

pub const fn lit_i(value: i32, index: u32) -> SpatialFieldV2<SpatialBindingV2<i32>> {
    field(SpatialBindingV2::Literal(value), span_const(index))
}

pub const fn lit_f(value: i64, index: u32) -> SpatialFieldV2<SpatialBindingV2<i64>> {
    field(SpatialBindingV2::Literal(value), span_const(index))
}

const fn span_const(index: u32) -> SourceSpan {
    SourceSpan::bytes(SourceId::new(9), index * 10, index * 10 + 5)
}

pub fn point(x: i64, y: i64, index: u32) -> SpatialPointRecipeV2 {
    SpatialPointRecipeV2::new(lit_f(x, index), lit_f(y, index + 1))
}

pub fn transform(index: u32) -> SpatialTransformRecipeV2 {
    SpatialTransformRecipeV2::new(
        lit_f(65_536, index),
        lit_f(0, index + 1),
        lit_f(0, index + 2),
        lit_f(65_536, index + 3),
        lit_f(0, index + 4),
        lit_f(0, index + 5),
        point(0, 0, index + 6),
    )
}

pub fn placement(index: u32) -> SpatialPlacementRecipeV2 {
    let dimension =
        SpatialDimensionRecipeV2::new(lit_i(0, index), lit_i(10, index + 1), lit_i(20, index + 2));
    SpatialPlacementRecipeV2::Layout(SpatialLayoutPlacementRecipeV2::new(
        dimension,
        dimension,
        transform(index + 3),
    ))
}

pub fn free_placement(target: SpatialAnchorTargetRecipeV2, index: u32) -> SpatialPlacementRecipeV2 {
    SpatialPlacementRecipeV2::Free(SpatialFreePlacementRecipeV2::new(
        lit_i(10, index),
        lit_i(10, index + 1),
        [
            SpatialAnchorComponentV2::Start,
            SpatialAnchorComponentV2::Start,
        ],
        target,
        [SpatialAnchorComponentV2::End, SpatialAnchorComponentV2::End],
        point(0, 0, index + 2),
        transform(index + 4),
    ))
}

pub fn container(index: u32) -> SpatialContainerRecipeV2 {
    SpatialContainerRecipeV2::new(
        SpatialAxisV2::Row,
        SpatialPaddingRecipeV2::new(
            lit_i(0, index),
            lit_i(0, index + 1),
            lit_i(0, index + 2),
            lit_i(0, index + 3),
        ),
        lit_i(0, index + 4),
    )
}

pub fn viewport(index: u32) -> SpatialViewportContainerV2 {
    SpatialViewportContainerV2::new(
        SpatialAxisV2::Column,
        field(0, span(index)),
        field(0, span(index + 1)),
        field(0, span(index + 2)),
        field(0, span(index + 3)),
        field(0, span(index + 4)),
        span(index + 5),
    )
}

pub fn shape(symbol: u32, index: u32) -> SpatialShapeDeclarationV2 {
    SpatialShapeDeclarationV2::new(
        field(SpatialShapeSymbolV2::new(symbol), span(index)),
        SpatialShapeGeometryV2::Rect {
            origin: point(0, 0, index + 1),
            width: lit_f(65_536, index + 3),
            height: lit_f(65_536, index + 4),
        },
        span(index + 5),
    )
}

pub fn brush(symbol: u32, index: u32) -> SpatialBrushDeclarationV2 {
    SpatialBrushDeclarationV2::new(
        field(SpatialBrushSymbolV2::new(symbol), span(index)),
        SpatialBrushContentV2::Solid {
            color: field(SpatialBindingV2::Literal([0, 0, 0, 255]), span(index + 1)),
        },
        span(index + 2),
    )
}

pub fn coverage(shape: u32, index: u32) -> SpatialCoverageRecipeV2 {
    SpatialCoverageRecipeV2::Fill {
        shape: field(SpatialShapeSymbolV2::new(shape), span(index)),
        rule: SpatialFillRuleV2::NonZero,
    }
}

pub fn paint(shape: u32, brush: u32, index: u32) -> SpatialPaintRecipeV2 {
    SpatialPaintRecipeV2::CoveragePaint {
        coverage: coverage(shape, index),
        brush: field(SpatialBrushSymbolV2::new(brush), span(index + 1)),
        opacity: field(255, span(index + 2)),
        clip: None,
        span: span(index + 3),
    }
}

pub fn node_with(
    symbol: u32,
    target: TemplateNodeId,
    parent: SpatialNodeParentV2,
    placement: SpatialPlacementRecipeV2,
    shapes: Vec<SpatialShapeDeclarationV2>,
    brushes: Vec<SpatialBrushDeclarationV2>,
    clips: Vec<SpatialClipDeclarationV2>,
    paints: Vec<SpatialPaintRecipeV2>,
    hits: Vec<SpatialHitRecipeV2>,
    semantics: Vec<SpatialSemanticRecipeV2>,
    index: u32,
) -> SpatialNodeDeclarationV2 {
    SpatialNodeDeclarationV2::new(
        field(SpatialNodeSymbolV2::new(symbol), span(index)),
        field(target, span(index + 1)),
        parent,
        placement,
        container(index + 2),
        shapes,
        brushes,
        clips,
        paints,
        hits,
        semantics,
        span(index + 8),
    )
}

pub fn node(
    symbol: u32,
    target: TemplateNodeId,
    parent: SpatialNodeParentV2,
    index: u32,
) -> SpatialNodeDeclarationV2 {
    node_with(
        symbol,
        target,
        parent,
        placement(index + 10),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        index,
    )
}

pub fn parent(symbol: u32, index: u32) -> SpatialNodeParentV2 {
    SpatialNodeParentV2::Node(field(SpatialNodeSymbolV2::new(symbol), span(index)))
}

pub fn program_with(
    format: SpatialFormatVersion,
    namespace: SchemaNamespace,
    revision: SchemaRevision,
    viewport: SpatialViewportContainerV2,
    nodes: Vec<SpatialNodeDeclarationV2>,
    images: Vec<SpatialImageDeclarationV2>,
    source: SourceSpan,
) -> SpatialProgramV2 {
    SpatialProgramV2::new(format, namespace, revision, viewport, nodes, images, source)
}

pub fn program(nodes: Vec<SpatialNodeDeclarationV2>) -> SpatialProgramV2 {
    program_with(
        SUPPORTED_SPATIAL_FORMAT,
        NS,
        REV,
        viewport(50),
        nodes,
        Vec::new(),
        span(49),
    )
}

pub const fn generous_limits() -> SpatialValidationLimitsV2 {
    SpatialValidationLimitsV2::new([64; 13])
}

pub fn validate(
    style: &ValidatedStyleProgram,
    program: SpatialProgramV2,
) -> Result<ValidatedSpatialProgramV2, IrValidationError> {
    validate_spatial(style, program, generous_limits())
}

pub fn assert_error(
    style: &ValidatedStyleProgram,
    program: SpatialProgramV2,
    expected: IrValidationErrorKind,
    expected_span: SourceSpan,
) {
    let error = validate(style, program).expect_err("spatial program should fail");
    assert_eq!(error.kind(), expected);
    assert_eq!(error.span(), expected_span);
}
