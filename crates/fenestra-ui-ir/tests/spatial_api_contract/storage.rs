use super::source::all_source;
use super::surface_support::{
    assert_enum_body, assert_private_tuple_struct, assert_struct_fields, assert_struct_private,
};

#[test]
fn leaf_and_layout_structs_have_exact_private_storage() {
    let source = all_source();
    assert_struct_fields(
        &source,
        "SpatialFieldV2",
        &[("value", "T"), ("span", "SourceSpan")],
    );
    assert_struct_fields(
        &source,
        "SpatialClipAddressV2",
        &[
            ("owner", "SpatialFieldV2<SpatialNodeSymbolV2>"),
            ("clip", "SpatialFieldV2<SpatialClipSymbolV2>"),
        ],
    );
    assert_struct_fields(
        &source,
        "SpatialPointRecipeV2",
        &[("x", fixed()), ("y", fixed())],
    );
    assert_struct_fields(
        &source,
        "SpatialPaddingRecipeV2",
        &[
            ("left", integer()),
            ("right", integer()),
            ("top", integer()),
            ("bottom", integer()),
        ],
    );
    assert_struct_fields(
        &source,
        "SpatialDimensionRecipeV2",
        &[
            ("minimum", integer()),
            ("preferred", integer()),
            ("maximum", integer()),
        ],
    );
    assert_struct_fields(
        &source,
        "SpatialTransformRecipeV2",
        &[
            ("a", fixed()),
            ("b", fixed()),
            ("c", fixed()),
            ("d", fixed()),
            ("tx", fixed()),
            ("ty", fixed()),
            ("origin", "SpatialPointRecipeV2"),
        ],
    );
    assert_struct_fields(
        &source,
        "SpatialViewportContainerV2",
        &[
            ("axis", "SpatialAxisV2"),
            ("left", "SpatialFieldV2<i32>"),
            ("right", "SpatialFieldV2<i32>"),
            ("top", "SpatialFieldV2<i32>"),
            ("bottom", "SpatialFieldV2<i32>"),
            ("gap", "SpatialFieldV2<i32>"),
            ("span", "SourceSpan"),
        ],
    );
    assert_struct_fields(
        &source,
        "SpatialContainerRecipeV2",
        &[
            ("axis", "SpatialAxisV2"),
            ("padding", "SpatialPaddingRecipeV2"),
            ("gap", integer()),
        ],
    );
    assert_struct_fields(
        &source,
        "SpatialLayoutPlacementRecipeV2",
        &[
            ("width", "SpatialDimensionRecipeV2"),
            ("height", "SpatialDimensionRecipeV2"),
            ("transform", "SpatialTransformRecipeV2"),
        ],
    );
    assert_struct_fields(
        &source,
        "SpatialFreePlacementRecipeV2",
        &[
            ("width", integer()),
            ("height", integer()),
            ("self_anchor", "[SpatialAnchorComponentV2;2]"),
            ("target", "SpatialAnchorTargetRecipeV2"),
            ("target_anchor", "[SpatialAnchorComponentV2;2]"),
            ("offset", "SpatialPointRecipeV2"),
            ("transform", "SpatialTransformRecipeV2"),
        ],
    );
}

#[test]
fn content_and_program_structs_have_exact_private_storage() {
    let source = all_source();
    assert_struct_fields(
        &source,
        "SpatialPolygonPointV2",
        &[("point", "SpatialPointRecipeV2"), ("span", "SourceSpan")],
    );
    assert_struct_fields(
        &source,
        "SpatialShapeDeclarationV2",
        &[
            ("symbol", "SpatialFieldV2<SpatialShapeSymbolV2>"),
            ("geometry", "SpatialShapeGeometryV2"),
            ("span", "SourceSpan"),
        ],
    );
    assert_struct_fields(
        &source,
        "SpatialGradientStopV2",
        &[
            ("offset", "SpatialFieldV2<u16>"),
            ("color", color()),
            ("span", "SourceSpan"),
        ],
    );
    assert_struct_fields(
        &source,
        "SpatialBrushDeclarationV2",
        &[
            ("symbol", "SpatialFieldV2<SpatialBrushSymbolV2>"),
            ("content", "SpatialBrushContentV2"),
            ("span", "SourceSpan"),
        ],
    );
    assert_struct_fields(
        &source,
        "SpatialClipDeclarationV2",
        &[
            ("symbol", "SpatialFieldV2<SpatialClipSymbolV2>"),
            ("parent", "Option<SpatialClipAddressV2>"),
            ("shape", "SpatialFieldV2<SpatialShapeSymbolV2>"),
            ("fill_rule", "SpatialFillRuleV2"),
            ("span", "SourceSpan"),
        ],
    );
    assert_struct_fields(
        &source,
        "SpatialHitRecipeV2",
        &[
            ("coverage", "SpatialCoverageRecipeV2"),
            ("clip", "Option<SpatialClipAddressV2>"),
            ("input_policy", policy()),
            ("span", "SourceSpan"),
        ],
    );
    assert_struct_fields(
        &source,
        "SpatialSemanticRecipeV2",
        &[
            ("shape", "SpatialFieldV2<SpatialShapeSymbolV2>"),
            ("fill_rule", "SpatialFillRuleV2"),
            ("clip", "Option<SpatialClipAddressV2>"),
            ("span", "SourceSpan"),
        ],
    );
    assert_struct_fields(
        &source,
        "SpatialImageDeclarationV2",
        &[
            ("symbol", "SpatialFieldV2<SpatialImageSymbolV2>"),
            ("width", "SpatialFieldV2<u32>"),
            ("height", "SpatialFieldV2<u32>"),
            ("stride", "SpatialFieldV2<u32>"),
            ("bytes", "Box<[u8]>"),
            ("span", "SourceSpan"),
        ],
    );
    assert_node_and_program_fields(&source);
    assert_private_tuple_struct(&source, "SpatialValidationLimitsV2", "[usize; 13]");
    assert_struct_private(&source, "ValidatedSpatialProgramV2");
}

#[test]
fn payload_enums_have_exact_exhaustive_variant_storage() {
    let source = all_source();
    assert_enum_body(
        &source,
        "SpatialBindingV2",
        "Literal(T),Property(PropertyId),",
    );
    assert_enum_body(
        &source,
        "SpatialNodeParentV2",
        "Viewport,Node(SpatialFieldV2<SpatialNodeSymbolV2>),",
    );
    assert_enum_body(
        &source,
        "SpatialAnchorTargetRecipeV2",
        "Viewport,Parent,Node(SpatialFieldV2<SpatialNodeSymbolV2>),",
    );
    assert_enum_body(
        &source,
        "SpatialPlacementRecipeV2",
        "Layout(SpatialLayoutPlacementRecipeV2),Free(SpatialFreePlacementRecipeV2),",
    );
    assert_enum_body(
        &source,
        "SpatialPathVerbRecipeV2",
        concat!(
            "MoveTo{to:SpatialPointRecipeV2,span:SourceSpan},",
            "LineTo{to:SpatialPointRecipeV2,span:SourceSpan},",
            "QuadraticTo{control:SpatialPointRecipeV2,to:SpatialPointRecipeV2,span:SourceSpan},",
            "CubicTo{control1:SpatialPointRecipeV2,control2:SpatialPointRecipeV2,",
            "to:SpatialPointRecipeV2,span:SourceSpan},Close{span:SourceSpan},"
        ),
    );
    assert_enum_body(
        &source,
        "SpatialShapeGeometryV2",
        concat!(
            "Rect{origin:SpatialPointRecipeV2,width:",
            "SpatialFieldV2<SpatialBindingV2<i64>>,height:",
            "SpatialFieldV2<SpatialBindingV2<i64>>},",
            "Circle{center:SpatialPointRecipeV2,radius:",
            "SpatialFieldV2<SpatialBindingV2<i64>>},",
            "Polygon{points:Vec<SpatialPolygonPointV2>},",
            "Path{verbs:Vec<SpatialPathVerbRecipeV2>},"
        ),
    );
    assert_brush_coverage_and_paint_variants(&source);
}

fn assert_node_and_program_fields(source: &str) {
    assert_struct_fields(
        source,
        "SpatialNodeDeclarationV2",
        &[
            ("symbol", "SpatialFieldV2<SpatialNodeSymbolV2>"),
            ("template", "SpatialFieldV2<TemplateNodeId>"),
            ("parent", "SpatialNodeParentV2"),
            ("placement", "SpatialPlacementRecipeV2"),
            ("container", "SpatialContainerRecipeV2"),
            ("shapes", "Vec<SpatialShapeDeclarationV2>"),
            ("brushes", "Vec<SpatialBrushDeclarationV2>"),
            ("clips", "Vec<SpatialClipDeclarationV2>"),
            ("paint_items", "Vec<SpatialPaintRecipeV2>"),
            ("hit_items", "Vec<SpatialHitRecipeV2>"),
            ("semantic_items", "Vec<SpatialSemanticRecipeV2>"),
            ("span", "SourceSpan"),
        ],
    );
    assert_struct_fields(
        source,
        "SpatialProgramV2",
        &[
            ("format", "SpatialFormatVersion"),
            ("schema_namespace", "SchemaNamespace"),
            ("schema_revision", "SchemaRevision"),
            ("viewport_container", "SpatialViewportContainerV2"),
            ("nodes", "Vec<SpatialNodeDeclarationV2>"),
            ("images", "Vec<SpatialImageDeclarationV2>"),
            ("span", "SourceSpan"),
        ],
    );
}

fn assert_brush_coverage_and_paint_variants(source: &str) {
    assert_enum_body(
        source,
        "SpatialBrushContentV2",
        concat!(
            "Solid{color:SpatialFieldV2<SpatialBindingV2<[u8;4]>>},",
            "LinearGradient{start:SpatialPointRecipeV2,end:SpatialPointRecipeV2,",
            "stops:Vec<SpatialGradientStopV2>},"
        ),
    );
    assert_enum_body(
        source,
        "SpatialCoverageRecipeV2",
        concat!(
            "Fill{shape:SpatialFieldV2<SpatialShapeSymbolV2>,rule:SpatialFillRuleV2},",
            "RoundStroke{shape:SpatialFieldV2<SpatialShapeSymbolV2>,width:",
            "SpatialFieldV2<SpatialBindingV2<i64>>},"
        ),
    );
    assert_enum_body(
        source,
        "SpatialPaintRecipeV2",
        concat!(
            "CoveragePaint{coverage:SpatialCoverageRecipeV2,brush:",
            "SpatialFieldV2<SpatialBrushSymbolV2>,opacity:SpatialFieldV2<u8>,",
            "clip:Option<SpatialClipAddressV2>,span:SourceSpan},",
            "ImagePaint{image:SpatialFieldV2<SpatialImageSymbolV2>,",
            "source_x:SpatialFieldV2<u32>,source_y:SpatialFieldV2<u32>,",
            "source_width:SpatialFieldV2<u32>,source_height:SpatialFieldV2<u32>,",
            "destination_origin:SpatialPointRecipeV2,destination_width:",
            "SpatialFieldV2<SpatialBindingV2<i64>>,destination_height:",
            "SpatialFieldV2<SpatialBindingV2<i64>>,opacity:SpatialFieldV2<u8>,",
            "clip:Option<SpatialClipAddressV2>,span:SourceSpan},"
        ),
    );
}

const fn integer() -> &'static str {
    "SpatialFieldV2<SpatialBindingV2<i32>>"
}
const fn fixed() -> &'static str {
    "SpatialFieldV2<SpatialBindingV2<i64>>"
}
const fn color() -> &'static str {
    "SpatialFieldV2<SpatialBindingV2<[u8;4]>>"
}
const fn policy() -> &'static str {
    "SpatialFieldV2<SpatialBindingV2<InputPolicy>>"
}
