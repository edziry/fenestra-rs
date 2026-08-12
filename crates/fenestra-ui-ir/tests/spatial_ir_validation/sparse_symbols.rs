use super::*;
use support::*;

#[test]
fn maximum_symbols_are_sparse_values_not_dense_storage_sizes() {
    let style = style();
    let maximum = u32::MAX;
    let shape_symbol = SpatialShapeSymbolV2::new(maximum);
    let brush_symbol = SpatialBrushSymbolV2::new(maximum);
    let owner_symbol = SpatialNodeSymbolV2::new(maximum);
    let clip_symbol = SpatialClipSymbolV2::new(maximum);
    let image_symbol = SpatialImageSymbolV2::new(maximum);
    let terminal = SpatialClipAddressV2::new(
        field(owner_symbol, span(2100)),
        field(clip_symbol, span(2101)),
    );
    let declaration = node_with(
        maximum,
        ROOT,
        SpatialNodeParentV2::Viewport,
        placement(2110),
        vec![SpatialShapeDeclarationV2::new(
            field(shape_symbol, span(2120)),
            SpatialShapeGeometryV2::Circle {
                center: point(0, 0, 2121),
                radius: lit_f(1, 2123),
            },
            span(2124),
        )],
        vec![SpatialBrushDeclarationV2::new(
            field(brush_symbol, span(2130)),
            SpatialBrushContentV2::Solid {
                color: field(SpatialBindingV2::Literal([0, 0, 0, 255]), span(2131)),
            },
            span(2132),
        )],
        vec![SpatialClipDeclarationV2::new(
            field(clip_symbol, span(2140)),
            None,
            field(shape_symbol, span(2141)),
            SpatialFillRuleV2::NonZero,
            span(2142),
        )],
        vec![
            SpatialPaintRecipeV2::CoveragePaint {
                coverage: SpatialCoverageRecipeV2::Fill {
                    shape: field(shape_symbol, span(2150)),
                    rule: SpatialFillRuleV2::NonZero,
                },
                brush: field(brush_symbol, span(2151)),
                opacity: field(255, span(2152)),
                clip: Some(terminal),
                span: span(2153),
            },
            SpatialPaintRecipeV2::ImagePaint {
                image: field(image_symbol, span(2160)),
                source_x: field(0, span(2161)),
                source_y: field(0, span(2162)),
                source_width: field(1, span(2163)),
                source_height: field(1, span(2164)),
                destination_origin: point(0, 0, 2165),
                destination_width: lit_f(1, 2167),
                destination_height: lit_f(1, 2168),
                opacity: field(255, span(2169)),
                clip: Some(terminal),
                span: span(2170),
            },
        ],
        Vec::new(),
        Vec::new(),
        2109,
    );
    let image = SpatialImageDeclarationV2::new(
        field(image_symbol, span(2180)),
        field(1, span(2181)),
        field(1, span(2182)),
        field(4, span(2183)),
        vec![0, 0, 0, 0].into_boxed_slice(),
        span(2184),
    );
    let input = program_with(
        SUPPORTED_SPATIAL_FORMAT,
        NS,
        REV,
        viewport(2190),
        vec![declaration],
        vec![image],
        span(2189),
    );

    let validated = validate(&style, input).expect("sparse maximum symbols should validate");
    assert_eq!(
        validated.node(owner_symbol).unwrap().symbol().value().get(),
        maximum
    );
}
