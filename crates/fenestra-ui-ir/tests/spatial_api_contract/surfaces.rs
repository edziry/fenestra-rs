use super::source::all_source;
use super::surface_support::{
    assert_method_surface, assert_payload_surface, assert_u32_symbol_surface,
};

#[test]
fn versions_and_symbols_have_exact_private_u32_surfaces() {
    let source = all_source();
    for type_name in [
        "SpatialFormatVersion",
        "SpatialNodeSymbolV2",
        "SpatialShapeSymbolV2",
        "SpatialBrushSymbolV2",
        "SpatialClipSymbolV2",
        "SpatialImageSymbolV2",
    ] {
        assert_u32_symbol_surface(&source, type_name);
    }
}

#[test]
fn every_record_and_payload_has_only_its_registered_methods() {
    let source = all_source();
    for (type_name, fields) in copy_surfaces() {
        let mut methods = vec!["new"];
        methods.extend(fields);
        assert_method_surface(&source, type_name, &methods, &methods);
    }
    for (type_name, fields, const_fields) in owned_surfaces() {
        let mut methods = vec!["new"];
        methods.extend(fields);
        assert_method_surface(&source, type_name, &methods, const_fields);
    }
    assert_payload_surface(&source, "SpatialBindingV2", &[]);
    for vocabulary in [
        "SpatialAxisV2",
        "SpatialAnchorComponentV2",
        "SpatialFillRuleV2",
    ] {
        assert!(super::surface_support::public_methods(&source, vocabulary).is_empty());
        assert_eq!(
            super::surface_support::public_constants(&source, vocabulary),
            super::surface_support::names(&["ALL"])
        );
    }
    for payload in [
        "SpatialNodeParentV2",
        "SpatialAnchorTargetRecipeV2",
        "SpatialPlacementRecipeV2",
        "SpatialShapeGeometryV2",
        "SpatialBrushContentV2",
        "SpatialCoverageRecipeV2",
    ] {
        assert_payload_surface(&source, payload, &[]);
    }
    assert_method_surface(&source, "SpatialPathVerbRecipeV2", &["span"], &["span"]);
    assert_method_surface(&source, "SpatialPaintRecipeV2", &["span"], &["span"]);
    assert_method_surface(&source, "SpatialValidationLimitsV2", &["new"], &["new"]);
    assert_method_surface(
        &source,
        "ValidatedSpatialProgramV2",
        &[
            "program",
            "style",
            "node",
            "node_for_template",
            "region_signature",
            "shares_domain_with",
        ],
        &[],
    );

    assert!(source.contains(concat!(
        "#[must_use = \"spatial IR validation errors must be handled\"]\n",
        "pub fn validate_spatial("
    )));
}

fn copy_surfaces() -> Vec<(&'static str, Vec<&'static str>)> {
    vec![
        ("SpatialFieldV2", vec!["value", "span"]),
        ("SpatialClipAddressV2", vec!["owner", "clip"]),
        ("SpatialPointRecipeV2", vec!["x", "y"]),
        (
            "SpatialPaddingRecipeV2",
            vec!["left", "right", "top", "bottom"],
        ),
        (
            "SpatialDimensionRecipeV2",
            vec!["minimum", "preferred", "maximum"],
        ),
        (
            "SpatialTransformRecipeV2",
            vec!["a", "b", "c", "d", "tx", "ty", "origin"],
        ),
        (
            "SpatialViewportContainerV2",
            vec!["axis", "left", "right", "top", "bottom", "gap", "span"],
        ),
        ("SpatialContainerRecipeV2", vec!["axis", "padding", "gap"]),
        (
            "SpatialLayoutPlacementRecipeV2",
            vec!["width", "height", "transform"],
        ),
        (
            "SpatialFreePlacementRecipeV2",
            vec![
                "width",
                "height",
                "self_anchor",
                "target",
                "target_anchor",
                "offset",
                "transform",
            ],
        ),
        ("SpatialPolygonPointV2", vec!["point", "span"]),
        ("SpatialGradientStopV2", vec!["offset", "color", "span"]),
        (
            "SpatialClipDeclarationV2",
            vec!["symbol", "parent", "shape", "fill_rule", "span"],
        ),
        (
            "SpatialHitRecipeV2",
            vec!["coverage", "clip", "input_policy", "span"],
        ),
        (
            "SpatialSemanticRecipeV2",
            vec!["shape", "fill_rule", "clip", "span"],
        ),
    ]
}

fn owned_surfaces() -> Vec<(&'static str, Vec<&'static str>, &'static [&'static str])> {
    vec![
        (
            "SpatialShapeDeclarationV2",
            vec!["symbol", "geometry", "span"],
            &["symbol", "geometry", "span"],
        ),
        (
            "SpatialBrushDeclarationV2",
            vec!["symbol", "content", "span"],
            &["symbol", "content", "span"],
        ),
        (
            "SpatialImageDeclarationV2",
            vec!["symbol", "width", "height", "stride", "bytes", "span"],
            &["symbol", "width", "height", "stride", "span"],
        ),
        (
            "SpatialNodeDeclarationV2",
            vec![
                "symbol",
                "template",
                "parent",
                "placement",
                "container",
                "shapes",
                "brushes",
                "clips",
                "paint_items",
                "hit_items",
                "semantic_items",
                "span",
            ],
            &[
                "symbol",
                "template",
                "parent",
                "placement",
                "container",
                "span",
            ],
        ),
        (
            "SpatialProgramV2",
            vec![
                "format",
                "schema_namespace",
                "schema_revision",
                "viewport_container",
                "nodes",
                "images",
                "span",
            ],
            &[
                "format",
                "schema_namespace",
                "schema_revision",
                "viewport_container",
                "span",
            ],
        ),
    ]
}
