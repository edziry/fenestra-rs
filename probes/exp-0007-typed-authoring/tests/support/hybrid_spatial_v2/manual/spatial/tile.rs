use fenestra_ui_ir::prototype::{
    SpatialAxisV2, SpatialBrushContentV2, SpatialBrushDeclarationV2, SpatialClipDeclarationV2,
    SpatialCoverageRecipeV2, SpatialFillRuleV2, SpatialHitRecipeV2, SpatialNodeDeclarationV2,
    SpatialNodeParentV2, SpatialPaintRecipeV2, SpatialPlacementRecipeV2, SpatialSemanticRecipeV2,
    SpatialShapeDeclarationV2, SpatialShapeGeometryV2,
};

use super::super::value::{
    address, brush, clip, field, fixed_lit, fixed_prop, i32_lit, i32_prop, input_prop, node, point,
    rgba_prop, shape, span, template,
};
use super::layout::{container, layout_placement, transform};

pub(super) fn tile() -> SpatialNodeDeclarationV2 {
    SpatialNodeDeclarationV2::new(
        node(4, 276),
        template(4, 278),
        SpatialNodeParentV2::Node(node(2, 277)),
        SpatialPlacementRecipeV2::Layout(layout_placement(
            [i32_lit(0, 286), i32_prop(0, 287), i32_lit(32, 288)],
            [i32_lit(0, 289), i32_prop(1, 290), i32_lit(24, 291)],
            transform(
                [
                    fixed_lit(65_536, 293),
                    fixed_lit(0, 294),
                    fixed_lit(0, 295),
                    fixed_lit(65_536, 296),
                    fixed_prop(3, 297),
                    fixed_lit(-32_768, 298),
                ],
                point(fixed_lit(0, 299), fixed_lit(0, 300)),
            ),
        )),
        container(
            SpatialAxisV2::Column,
            [
                i32_lit(1, 280),
                i32_lit(1, 281),
                i32_prop(2, 282),
                i32_prop(2, 283),
            ],
            i32_lit(1, 284),
        ),
        vec![SpatialShapeDeclarationV2::new(
            shape(0, 302),
            SpatialShapeGeometryV2::Circle {
                center: point(fixed_lit(524_288, 303), fixed_lit(393_216, 304)),
                radius: fixed_lit(262_144, 305),
            },
            span(301),
        )],
        vec![SpatialBrushDeclarationV2::new(
            brush(0, 307),
            SpatialBrushContentV2::Solid {
                color: rgba_prop(4, 308),
            },
            span(306),
        )],
        vec![SpatialClipDeclarationV2::new(
            clip(0, 310),
            Some(address(0, 311, 0, 312)),
            shape(0, 313),
            SpatialFillRuleV2::NonZero,
            span(309),
        )],
        vec![SpatialPaintRecipeV2::CoveragePaint {
            coverage: SpatialCoverageRecipeV2::Fill {
                shape: shape(0, 315),
                rule: SpatialFillRuleV2::NonZero,
            },
            brush: brush(0, 316),
            opacity: field(128, 317),
            clip: Some(address(4, 318, 0, 319)),
            span: span(314),
        }],
        vec![SpatialHitRecipeV2::new(
            SpatialCoverageRecipeV2::RoundStroke {
                shape: shape(0, 321),
                width: fixed_lit(65_536, 322),
            },
            Some(address(0, 323, 0, 324)),
            input_prop(7, 325),
            span(320),
        )],
        vec![SpatialSemanticRecipeV2::new(
            shape(0, 327),
            SpatialFillRuleV2::EvenOdd,
            Some(address(4, 328, 0, 329)),
            span(326),
        )],
        span(275),
    )
}
