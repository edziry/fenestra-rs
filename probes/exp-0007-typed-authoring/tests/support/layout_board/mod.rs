mod programs;

use fenestra_ui_authoring::prototype::{AnchorKindV1, AuthoringLimitsV1};
use fenestra_ui_exp_0007_typed_authoring::LAYOUT_BOARD_FEN_V1;
use fenestra_ui_ir::prototype::{SourceId, SourceSpan};

pub use programs::{expected_construction, expected_schema, expected_style};

pub const FIXTURE: &[u8] = LAYOUT_BOARD_FEN_V1;
pub const SOURCE: SourceId = SourceId::new(7);
pub const REGISTERED_LIMITS: AuthoringLimitsV1 =
    AuthoringLimitsV1::new(8_192, 1_024, 32, 8, 1, 5, 4, 1, 3, 12, 2, 2, 34, 32_768);
pub const EXPECTED_LOGICAL_CATALOG: &[u8] = &[b'@'; 34];

#[derive(Clone, Copy)]
pub struct ExpectedAnchor {
    pub kind: AnchorKindV1,
    pub label: &'static str,
    pub start: u32,
    pub end: u32,
}

pub const EXPECTED_ANCHORS: [ExpectedAnchor; 34] = [
    anchor(AnchorKindV1::Document, "format", 0, 6),
    anchor(AnchorKindV1::Schema, "schema", 10, 16),
    anchor(AnchorKindV1::Component, "fixture", 57, 64),
    anchor(AnchorKindV1::Property, "width", 84, 89),
    anchor(AnchorKindV1::Property, "height", 193, 199),
    anchor(AnchorKindV1::Property, "color", 303, 308),
    anchor(AnchorKindV1::Property, "visible", 385, 392),
    anchor(AnchorKindV1::Property, "input", 471, 476),
    anchor(AnchorKindV1::Construction, "construction", 540, 552),
    anchor(AnchorKindV1::Template, "root", 566, 570),
    anchor(AnchorKindV1::InitialProperty, "width", 594, 599),
    anchor(AnchorKindV1::InitialProperty, "height", 615, 621),
    anchor(AnchorKindV1::InitialProperty, "color", 636, 641),
    anchor(AnchorKindV1::StaticChild, "container", 684, 693),
    anchor(AnchorKindV1::Template, "container", 710, 719),
    anchor(AnchorKindV1::InitialProperty, "width", 743, 748),
    anchor(AnchorKindV1::InitialProperty, "height", 763, 769),
    anchor(AnchorKindV1::InitialProperty, "color", 784, 789),
    anchor(AnchorKindV1::StaticChild, "control", 832, 839),
    anchor(AnchorKindV1::RegionChild, "items", 858, 863),
    anchor(AnchorKindV1::Template, "control", 880, 887),
    anchor(AnchorKindV1::InitialProperty, "width", 911, 916),
    anchor(AnchorKindV1::InitialProperty, "color", 931, 936),
    anchor(AnchorKindV1::InitialProperty, "input", 968, 973),
    anchor(AnchorKindV1::Template, "item", 999, 1_003),
    anchor(AnchorKindV1::InitialProperty, "height", 1_027, 1_033),
    anchor(AnchorKindV1::InitialProperty, "color", 1_048, 1_053),
    anchor(AnchorKindV1::InitialProperty, "input", 1_085, 1_090),
    anchor(AnchorKindV1::Region, "items", 1_114, 1_119),
    anchor(AnchorKindV1::InitialKey, "10", 1_162, 1_164),
    anchor(AnchorKindV1::InitialKey, "20", 1_166, 1_168),
    anchor(AnchorKindV1::Style, "style", 1_250, 1_255),
    anchor(AnchorKindV1::StyleAssignment, "color", 1_272, 1_277),
    anchor(AnchorKindV1::StyleAssignment, "color", 1_315, 1_320),
];

pub const fn logical_span(ordinal: u32) -> SourceSpan {
    SourceSpan::bytes(SourceId::new(0), ordinal, ordinal + 1)
}

const fn anchor(kind: AnchorKindV1, label: &'static str, start: u32, end: u32) -> ExpectedAnchor {
    ExpectedAnchor {
        kind,
        label,
        start,
        end,
    }
}
