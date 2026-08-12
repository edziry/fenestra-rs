use fenestra_ui_ir::prototype::SpatialFillRuleV2;

use super::{ParsedFixedBindingFieldV2, ParsedNameFieldV2, ParsedPointV2};

#[derive(Clone)]
pub(crate) struct ParsedShapeV2 {
    pub(crate) name: Box<str>,
    pub(crate) symbol: ParsedNameFieldV2,
    pub(crate) geometry: ParsedShapeGeometryV2,
    pub(crate) anchor: u32,
}

#[derive(Clone)]
pub(crate) enum ParsedShapeGeometryV2 {
    Rect {
        origin: ParsedPointV2,
        width: ParsedFixedBindingFieldV2,
        height: ParsedFixedBindingFieldV2,
    },
    Circle {
        center: ParsedPointV2,
        radius: ParsedFixedBindingFieldV2,
    },
    Polygon(Vec<ParsedPolygonPointV2>),
    Path(Vec<ParsedPathVerbV2>),
}

#[derive(Clone)]
pub(crate) struct ParsedPolygonPointV2 {
    pub(crate) point: ParsedPointV2,
    pub(crate) anchor: u32,
}

#[derive(Clone)]
pub(crate) enum ParsedPathVerbKindV2 {
    MoveTo(ParsedPointV2),
    LineTo(ParsedPointV2),
    QuadraticTo {
        control: ParsedPointV2,
        to: ParsedPointV2,
    },
    CubicTo {
        control1: ParsedPointV2,
        control2: ParsedPointV2,
        to: ParsedPointV2,
    },
    Close,
}

#[derive(Clone)]
pub(crate) struct ParsedPathVerbV2 {
    pub(crate) kind: ParsedPathVerbKindV2,
    pub(crate) anchor: u32,
}

#[derive(Clone)]
pub(crate) enum ParsedCoverageV2 {
    Fill {
        shape: ParsedNameFieldV2,
        rule: SpatialFillRuleV2,
    },
    RoundStroke {
        shape: ParsedNameFieldV2,
        width: ParsedFixedBindingFieldV2,
    },
}
