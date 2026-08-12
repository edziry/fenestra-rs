use fenestra_ui_ir::prototype::SpatialAxisV2;

use super::{
    ParsedBrushV2, ParsedClipV2, ParsedContainerV2, ParsedFieldV2, ParsedHitV2, ParsedI32FieldV2,
    ParsedLiteralV2, ParsedNameFieldV2, ParsedPaintV2, ParsedPlacementV2, ParsedSemanticV2,
    ParsedShapeV2, ParsedTransformV2,
};

#[derive(Clone)]
pub(crate) struct ParsedSpatialV2 {
    pub(crate) format: u32,
    pub(crate) viewport: ParsedViewportV2,
    pub(crate) resources_anchor: u32,
    pub(crate) images: Vec<ParsedImageV2>,
    pub(crate) nodes: Vec<ParsedNodeV2>,
    pub(crate) field_count: usize,
    pub(crate) anchor: u32,
}

#[derive(Clone)]
pub(crate) struct ParsedViewportV2 {
    pub(crate) axis: SpatialAxisV2,
    pub(crate) left: ParsedI32FieldV2,
    pub(crate) right: ParsedI32FieldV2,
    pub(crate) top: ParsedI32FieldV2,
    pub(crate) bottom: ParsedI32FieldV2,
    pub(crate) gap: ParsedI32FieldV2,
    pub(crate) anchor: u32,
}

#[derive(Clone)]
pub(crate) struct ParsedImageV2 {
    pub(crate) name: Box<str>,
    pub(crate) symbol: ParsedNameFieldV2,
    pub(crate) width: ParsedFieldV2<ParsedLiteralV2<u32>>,
    pub(crate) height: ParsedFieldV2<ParsedLiteralV2<u32>>,
    pub(crate) stride: ParsedFieldV2<ParsedLiteralV2<u32>>,
    pub(crate) bytes: Vec<ParsedLiteralV2<u8>>,
    pub(crate) anchor: u32,
}

#[derive(Clone)]
pub(crate) struct ParsedNodeV2 {
    pub(crate) name: Box<str>,
    pub(crate) symbol: ParsedNameFieldV2,
    pub(crate) template: ParsedNameFieldV2,
    pub(crate) parent: Option<ParsedNameFieldV2>,
    pub(crate) container: ParsedContainerV2,
    pub(crate) placement: ParsedPlacementV2,
    pub(crate) transform: ParsedTransformV2,
    pub(crate) shapes: Vec<ParsedShapeV2>,
    pub(crate) brushes: Vec<ParsedBrushV2>,
    pub(crate) clips: Vec<ParsedClipV2>,
    pub(crate) paint: Vec<ParsedPaintV2>,
    pub(crate) hit: Vec<ParsedHitV2>,
    pub(crate) semantics: Vec<ParsedSemanticV2>,
    pub(crate) children: Vec<ParsedNodeV2>,
    pub(crate) anchor: u32,
}
