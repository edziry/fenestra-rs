use crate::*;
use fenestra_ui_ir::prototype::{SourceId, SourceSpan};

pub(super) type Integer = SpatialFieldV2<SpatialBindingV2<i32>>;
pub(super) type Fixed = SpatialFieldV2<SpatialBindingV2<i64>>;

pub(super) fn span(value: u32) -> SourceSpan {
    SourceSpan::bytes(SourceId::new(7), value, value + 1)
}

pub(super) fn field<T>(value: T, at: u32) -> SpatialFieldV2<T> {
    SpatialFieldV2::new(value, span(at))
}

pub(super) fn integer(value: i32, at: u32) -> Integer {
    field(SpatialBindingV2::Literal(value), at)
}

pub(super) fn fixed(value: i64, at: u32) -> Fixed {
    field(SpatialBindingV2::Literal(value), at)
}

pub(super) fn point(x: i64, y: i64, at: u32) -> SpatialPointRecipeV2 {
    SpatialPointRecipeV2::new(fixed(x, at), fixed(y, at + 1))
}

pub(super) fn padding_from(value: i32, at: u32) -> SpatialPaddingRecipeV2 {
    SpatialPaddingRecipeV2::new(
        integer(value, at),
        integer(value + 1, at + 1),
        integer(value + 2, at + 2),
        integer(value + 3, at + 3),
    )
}

pub(super) fn dimension(
    minimum: i32,
    preferred: i32,
    maximum: i32,
    at: u32,
) -> SpatialDimensionRecipeV2 {
    SpatialDimensionRecipeV2::new(
        integer(minimum, at),
        integer(preferred, at + 1),
        integer(maximum, at + 2),
    )
}

pub(super) fn transform(value: i64, at: u32) -> SpatialTransformRecipeV2 {
    SpatialTransformRecipeV2::new(
        fixed(value, at),
        fixed(value + 1, at + 1),
        fixed(value + 2, at + 2),
        fixed(value + 3, at + 3),
        fixed(value + 4, at + 4),
        fixed(value + 5, at + 5),
        point(value + 6, value + 7, at + 6),
    )
}

pub(super) fn viewport(at: u32) -> SpatialViewportContainerV2 {
    SpatialViewportContainerV2::new(
        SpatialAxisV2::Column,
        field(51, at),
        field(52, at + 1),
        field(53, at + 2),
        field(54, at + 3),
        field(55, at + 4),
        span(at + 5),
    )
}

pub(super) fn container(at: u32) -> SpatialContainerRecipeV2 {
    SpatialContainerRecipeV2::new(
        SpatialAxisV2::Row,
        padding_from(61, at),
        integer(65, at + 4),
    )
}

pub(super) fn layout(at: u32) -> SpatialLayoutPlacementRecipeV2 {
    SpatialLayoutPlacementRecipeV2::new(
        dimension(71, 72, 73, at),
        dimension(74, 75, 76, at + 3),
        transform(77, at + 6),
    )
}

pub(super) fn free(at: u32) -> SpatialFreePlacementRecipeV2 {
    SpatialFreePlacementRecipeV2::new(
        integer(91, at),
        integer(92, at + 1),
        [
            SpatialAnchorComponentV2::Start,
            SpatialAnchorComponentV2::Center,
        ],
        SpatialAnchorTargetRecipeV2::Node(field(SpatialNodeSymbolV2::new(93), at + 2)),
        [
            SpatialAnchorComponentV2::End,
            SpatialAnchorComponentV2::Start,
        ],
        point(94, 95, at + 3),
        transform(96, at + 5),
    )
}

pub(super) fn rect() -> SpatialShapeGeometryV2 {
    SpatialShapeGeometryV2::Rect {
        origin: point(1, 2, 500),
        width: fixed(3, 502),
        height: fixed(4, 503),
    }
}

pub(super) fn circle() -> SpatialShapeGeometryV2 {
    SpatialShapeGeometryV2::Circle {
        center: point(5, 6, 504),
        radius: fixed(7, 506),
    }
}

pub(super) fn solid() -> SpatialBrushContentV2 {
    SpatialBrushContentV2::Solid {
        color: field(SpatialBindingV2::Literal([1, 2, 3, 4]), 507),
    }
}

pub(super) fn gradient() -> SpatialBrushContentV2 {
    SpatialBrushContentV2::LinearGradient {
        start: point(8, 9, 508),
        end: point(10, 11, 510),
        stops: Vec::new(),
    }
}

pub(super) fn fill() -> SpatialCoverageRecipeV2 {
    SpatialCoverageRecipeV2::Fill {
        shape: field(SpatialShapeSymbolV2::new(12), 512),
        rule: SpatialFillRuleV2::EvenOdd,
    }
}

pub(super) fn stroke() -> SpatialCoverageRecipeV2 {
    SpatialCoverageRecipeV2::RoundStroke {
        shape: field(SpatialShapeSymbolV2::new(13), 513),
        width: fixed(14, 514),
    }
}

pub(super) fn coverage_paint(at: u32) -> SpatialPaintRecipeV2 {
    SpatialPaintRecipeV2::CoveragePaint {
        coverage: fill(),
        brush: field(SpatialBrushSymbolV2::new(15), at),
        opacity: field(16, at + 1),
        clip: None,
        span: span(at + 5),
    }
}

pub(super) fn image_paint(at: u32) -> SpatialPaintRecipeV2 {
    SpatialPaintRecipeV2::ImagePaint {
        image: field(SpatialImageSymbolV2::new(17), at),
        source_x: field(18, at + 1),
        source_y: field(19, at + 2),
        source_width: field(20, at + 3),
        source_height: field(21, at + 4),
        destination_origin: point(22, 23, at + 5),
        destination_width: fixed(24, at + 7),
        destination_height: fixed(25, at + 8),
        opacity: field(26, at + 9),
        clip: None,
        span: span(at + 9),
    }
}
