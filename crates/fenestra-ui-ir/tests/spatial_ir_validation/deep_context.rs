use super::*;
use support::{COMPONENT, NS, REV, node, span, viewport};

#[test]
fn deep_static_construction_context_is_derived_iteratively() {
    let depth = 16_384usize;
    let limits = ValidationLimits::new(1, 0, depth, 0, depth - 1, 0, 0, depth, depth);
    let manifest = SchemaManifest::new(
        SUPPORTED_SCHEMA_FORMAT,
        NS,
        REV,
        vec![ComponentSchema::new(COMPONENT, Vec::new(), span(1))],
        span(0),
    );
    let mut templates = Vec::with_capacity(depth);
    for index in 0..depth {
        let children = if index + 1 < depth {
            vec![ChildSlot::static_node(
                TemplateNodeId::new(index as u32 + 1),
                span(10 + index as u32),
            )]
        } else {
            Vec::new()
        };
        templates.push(TemplateNode::new(
            TemplateNodeId::new(index as u32),
            COMPONENT,
            Vec::new(),
            children,
            span(20_000 + index as u32),
        ));
    }
    let schema = validate_schema(manifest, limits).expect("deep schema should validate");
    let construction = validate_construction(
        &schema,
        ConstructionProgram::new(
            SUPPORTED_CONSTRUCTION_FORMAT,
            NS,
            REV,
            templates,
            Vec::new(),
            span(2),
        ),
        limits,
    )
    .expect("deep construction should validate");
    let style = validate_style(
        &construction,
        StyleProgram::new(SUPPORTED_STYLE_FORMAT, NS, REV, Vec::new(), span(3)),
        StyleValidationLimits::new(0),
    )
    .expect("empty style should validate");

    let validated = validate_spatial(
        &style,
        SpatialProgramV2::new(
            SUPPORTED_SPATIAL_FORMAT,
            NS,
            REV,
            viewport(4),
            vec![node(
                0,
                TemplateNodeId::new(depth as u32 - 1),
                SpatialNodeParentV2::Viewport,
                40_000,
            )],
            Vec::new(),
            span(3),
        ),
        SpatialValidationLimitsV2::new([1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
    )
    .expect("deep context derivation must not recurse");

    assert_eq!(
        validated.region_signature(SpatialNodeSymbolV2::new(0)),
        Some([].as_slice())
    );
}

#[test]
fn empty_spatial_program_does_not_walk_the_construction_tree() {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/validation/spatial/context.rs"),
    )
    .expect("spatial context source");
    let requested = source
        .find("if requested.is_empty()")
        .expect("empty requested-template guard");
    let pending = source
        .find("let mut pending")
        .expect("construction traversal stack");
    assert!(requested < pending);
}
