mod construction;
mod program;

use fenestra_ui_ir::prototype::{
    InputPolicy, PropertyId, PropertyValue, SourceId, SourceSpan, StructuralRegionId,
    TemplateNodeId, ValidatedSpatialProgramV2,
};
use fenestra_ui_runtime::prototype::{
    CommittedRuntimeSnapshot, FragmentId, NodeId, RuntimeCapacity,
};
use fenestra_ui_spatial::prototype::{SpatialLimitKindV2, SpatialLimitsV2, SpatialViewportV2};

pub use program::SpatialSpans;

pub const WIDTH: PropertyId = PropertyId::new(0);
pub const COLOR: PropertyId = PropertyId::new(1);
pub const POLICY: PropertyId = PropertyId::new(2);

pub const ROOT: TemplateNodeId = TemplateNodeId::new(0);
pub const OUTER: TemplateNodeId = TemplateNodeId::new(1);
pub const INNER: TemplateNodeId = TemplateNodeId::new(2);
pub const EMPTY: TemplateNodeId = TemplateNodeId::new(3);
pub const LEAF: TemplateNodeId = TemplateNodeId::new(4);
pub const NODE_ANCHOR: TemplateNodeId = TemplateNodeId::new(5);
pub const VIEW_ANCHOR: TemplateNodeId = TemplateNodeId::new(6);

pub const OUTER_REGION: StructuralRegionId = StructuralRegionId::new(0);
pub const INNER_REGION: StructuralRegionId = StructuralRegionId::new(1);
pub const EMPTY_REGION: StructuralRegionId = StructuralRegionId::new(2);

pub const FIRST_KEY: u64 = 10;
pub const SECOND_KEY: u64 = 20;
pub const INNER_KEY: u64 = 30;
pub const INSERTED_KEY: u64 = 15;

pub const STYLED_WIDTH: i32 = 12;
pub const STYLED_COLOR: [u8; 4] = [20, 40, 60, 255];
pub const IMAGE_COLOR: [u8; 4] = [180, 10, 20, 255];
pub const VIEWPORT: SpatialViewportV2 = SpatialViewportV2::new(40, 30);

pub struct Fixture {
    pub program: ValidatedSpatialProgramV2,
    pub spans: SpatialSpans,
}

pub fn fixture() -> Fixture {
    fixture_with_outer_values(
        PropertyValue::ScalarI32(STYLED_WIDTH),
        PropertyValue::Rgba8(STYLED_COLOR),
        PropertyValue::InputPolicy(InputPolicy::Accept),
    )
}

pub fn fixture_with_width(width: i32) -> Fixture {
    fixture_with_outer_values(
        PropertyValue::ScalarI32(width),
        PropertyValue::Rgba8(STYLED_COLOR),
        PropertyValue::InputPolicy(InputPolicy::Accept),
    )
}

fn fixture_with_outer_values(
    width: PropertyValue,
    color: PropertyValue,
    policy: PropertyValue,
) -> Fixture {
    let style = construction::style(width, color, policy);
    let (raw, spans) = program::program();
    let program = fenestra_ui_ir::prototype::validate_spatial(
        &style,
        raw,
        fenestra_ui_ir::prototype::SpatialValidationLimitsV2::new([64; 13]),
    )
    .expect("runtime spatial IR fixture should validate");
    Fixture { program, spans }
}

pub const fn span(index: u32) -> SourceSpan {
    SourceSpan::bytes(SourceId::new(77), index * 8, index * 8 + 5)
}

pub fn capacity() -> RuntimeCapacity {
    RuntimeCapacity::new(32, 64, 32, 16, 128, 8)
}

pub fn limits() -> SpatialLimitsV2 {
    SpatialLimitsV2::new([usize::MAX; 30])
}

pub fn limit(kind: SpatialLimitKindV2, maximum: usize) -> SpatialLimitsV2 {
    let mut values = [usize::MAX; 30];
    let index = SpatialLimitKindV2::ALL
        .iter()
        .position(|candidate| *candidate == kind)
        .expect("registered limit kind should resolve");
    values[index] = maximum;
    SpatialLimitsV2::new(values)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LogicalNodes {
    pub root: NodeId,
    pub outer_fragment: FragmentId,
    pub first_outer: NodeId,
    pub first_inner: NodeId,
    pub second_outer: NodeId,
    pub second_inner: NodeId,
}

impl LogicalNodes {
    pub fn capture(committed: &CommittedRuntimeSnapshot) -> Self {
        let root = committed.root();
        let outer_fragment = committed
            .fragment(root, OUTER_REGION)
            .expect("outer region should be live");
        let empty_fragment = committed
            .fragment(root, EMPTY_REGION)
            .expect("empty region should retain a fragment");
        assert_eq!(
            committed
                .keyed_members(empty_fragment)
                .expect("empty fragment should be live")
                .count(),
            0
        );
        let first_outer = committed
            .keyed_member(outer_fragment, FIRST_KEY)
            .expect("first outer member should be live");
        let second_outer = committed
            .keyed_member(outer_fragment, SECOND_KEY)
            .expect("second outer member should be live");
        let first_inner = inner(committed, first_outer);
        let second_inner = inner(committed, second_outer);
        Self {
            root,
            outer_fragment,
            first_outer,
            first_inner,
            second_outer,
            second_inner,
        }
    }
}

pub fn inner(committed: &CommittedRuntimeSnapshot, outer: NodeId) -> NodeId {
    let fragment = committed
        .fragment(outer, INNER_REGION)
        .expect("nested region should be live");
    committed
        .keyed_member(fragment, INNER_KEY)
        .expect("nested member should be live")
}
