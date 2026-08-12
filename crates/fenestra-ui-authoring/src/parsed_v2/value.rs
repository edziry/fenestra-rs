use fenestra_ui_ir::prototype::{InputPolicy, SpatialAnchorComponentV2, SpatialAxisV2};

use crate::source_v2::PhysicalOriginV2;

use super::ParsedLiteralV2;

#[derive(Clone)]
pub(crate) struct ParsedFieldV2<T> {
    pub(crate) value: T,
    pub(crate) anchor: u32,
}

#[derive(Clone)]
pub(crate) enum ParsedBindingV2<T> {
    Literal(ParsedLiteralV2<T>),
    Property(Box<str>),
}

pub(crate) type ParsedI32FieldV2 = ParsedFieldV2<ParsedLiteralV2<i32>>;
pub(crate) type ParsedU8FieldV2 = ParsedFieldV2<ParsedLiteralV2<u8>>;
pub(crate) type ParsedU16FieldV2 = ParsedFieldV2<ParsedLiteralV2<u16>>;
pub(crate) type ParsedU32FieldV2 = ParsedFieldV2<ParsedLiteralV2<u32>>;
pub(crate) type ParsedI32BindingFieldV2 = ParsedFieldV2<ParsedBindingV2<i32>>;
pub(crate) type ParsedFixedBindingFieldV2 = ParsedFieldV2<ParsedBindingV2<i64>>;
pub(crate) type ParsedColorBindingFieldV2 = ParsedFieldV2<ParsedBindingV2<[u8; 4]>>;
pub(crate) type ParsedInputBindingFieldV2 = ParsedFieldV2<ParsedBindingV2<InputPolicy>>;
pub(crate) type ParsedNameFieldV2 = ParsedFieldV2<Box<str>>;

#[derive(Clone)]
pub(crate) struct ParsedPointV2 {
    pub(crate) x: ParsedFixedBindingFieldV2,
    pub(crate) y: ParsedFixedBindingFieldV2,
}

#[derive(Clone)]
pub(crate) struct ParsedPaddingV2 {
    pub(crate) left: ParsedI32BindingFieldV2,
    pub(crate) right: ParsedI32BindingFieldV2,
    pub(crate) top: ParsedI32BindingFieldV2,
    pub(crate) bottom: ParsedI32BindingFieldV2,
}

#[derive(Clone)]
pub(crate) struct ParsedDimensionV2 {
    pub(crate) minimum: ParsedI32BindingFieldV2,
    pub(crate) preferred: ParsedI32BindingFieldV2,
    pub(crate) maximum: ParsedI32BindingFieldV2,
}

#[derive(Clone, Copy)]
pub(crate) struct ParsedAnchorPairV2 {
    pub(crate) horizontal: SpatialAnchorComponentV2,
    pub(crate) vertical: SpatialAnchorComponentV2,
}

#[derive(Clone)]
pub(crate) enum ParsedAnchorTargetV2 {
    Viewport,
    Parent,
    Node(ParsedNameFieldV2),
}

#[derive(Clone)]
pub(crate) struct ParsedContainerV2 {
    pub(crate) axis: SpatialAxisV2,
    pub(crate) padding: ParsedPaddingV2,
    pub(crate) gap: ParsedI32BindingFieldV2,
    pub(crate) anchor: u32,
}

#[derive(Clone)]
pub(crate) enum ParsedPlacementV2 {
    Layout {
        width: ParsedDimensionV2,
        height: ParsedDimensionV2,
        anchor: u32,
    },
    Free {
        width: ParsedI32BindingFieldV2,
        height: ParsedI32BindingFieldV2,
        self_anchor: ParsedAnchorPairV2,
        target: ParsedAnchorTargetV2,
        target_anchor: ParsedAnchorPairV2,
        offset: ParsedPointV2,
        anchor: u32,
    },
}

impl ParsedPlacementV2 {
    pub(crate) const fn anchor(&self) -> u32 {
        match self {
            Self::Layout { anchor, .. } | Self::Free { anchor, .. } => *anchor,
        }
    }
}

#[derive(Clone)]
pub(crate) struct ParsedTransformV2 {
    pub(crate) coefficients: Option<[ParsedFixedBindingFieldV2; 6]>,
    pub(crate) origin: ParsedPointV2,
    pub(crate) invalid_turn: Option<PhysicalOriginV2>,
    pub(crate) anchor: u32,
}
