use fenestra_ui_ir::prototype::SpatialFillRuleV2;

use super::{
    ParsedColorBindingFieldV2, ParsedCoverageV2, ParsedFixedBindingFieldV2,
    ParsedInputBindingFieldV2, ParsedNameFieldV2, ParsedPointV2, ParsedU8FieldV2, ParsedU16FieldV2,
    ParsedU32FieldV2,
};

#[derive(Clone)]
pub(crate) struct ParsedBrushV2 {
    pub(crate) name: Box<str>,
    pub(crate) symbol: ParsedNameFieldV2,
    pub(crate) content: ParsedBrushContentV2,
    pub(crate) anchor: u32,
}

#[derive(Clone)]
pub(crate) enum ParsedBrushContentV2 {
    Solid(ParsedColorBindingFieldV2),
    LinearGradient {
        start: ParsedPointV2,
        end: ParsedPointV2,
        stops: Vec<ParsedGradientStopV2>,
    },
}

#[derive(Clone)]
pub(crate) struct ParsedGradientStopV2 {
    pub(crate) offset: ParsedU16FieldV2,
    pub(crate) color: ParsedColorBindingFieldV2,
    pub(crate) anchor: u32,
}

#[derive(Clone)]
pub(crate) struct ParsedClipAddressV2 {
    pub(crate) owner: ParsedNameFieldV2,
    pub(crate) clip: ParsedNameFieldV2,
}

#[derive(Clone)]
pub(crate) struct ParsedClipV2 {
    pub(crate) name: Box<str>,
    pub(crate) symbol: ParsedNameFieldV2,
    pub(crate) parent: Option<ParsedClipAddressV2>,
    pub(crate) shape: ParsedNameFieldV2,
    pub(crate) fill_rule: SpatialFillRuleV2,
    pub(crate) anchor: u32,
}

#[derive(Clone)]
pub(crate) enum ParsedPaintKindV2 {
    Coverage {
        coverage: ParsedCoverageV2,
        brush: ParsedNameFieldV2,
        opacity: ParsedU8FieldV2,
        clip: Option<ParsedClipAddressV2>,
    },
    Image {
        image: ParsedNameFieldV2,
        source: [ParsedU32FieldV2; 4],
        destination: ParsedPointV2,
        destination_width: ParsedFixedBindingFieldV2,
        destination_height: ParsedFixedBindingFieldV2,
        opacity: ParsedU8FieldV2,
        clip: Option<ParsedClipAddressV2>,
    },
}

#[derive(Clone)]
pub(crate) struct ParsedPaintV2 {
    pub(crate) kind: ParsedPaintKindV2,
    pub(crate) anchor: u32,
}

#[derive(Clone)]
pub(crate) struct ParsedHitV2 {
    pub(crate) coverage: ParsedCoverageV2,
    pub(crate) clip: Option<ParsedClipAddressV2>,
    pub(crate) input: ParsedInputBindingFieldV2,
    pub(crate) anchor: u32,
}

#[derive(Clone)]
pub(crate) struct ParsedSemanticV2 {
    pub(crate) shape: ParsedNameFieldV2,
    pub(crate) fill_rule: SpatialFillRuleV2,
    pub(crate) clip: Option<ParsedClipAddressV2>,
    pub(crate) anchor: u32,
}
