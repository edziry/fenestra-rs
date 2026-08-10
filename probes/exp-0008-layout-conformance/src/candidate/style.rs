use core::marker::PhantomData;

use fenestra_ui_layout::prototype::{LayoutAxisV1, LayoutStyleV1, LayoutViewportV1};
use taffy::geometry::{Point, Rect, Size};
use taffy::prelude::{
    AlignContent, AlignItems, AlignSelf, AvailableSpace, BoxSizing, Dimension, Display,
    FlexDirection, FlexWrap, JustifyContent, LengthPercentage, LengthPercentageAuto, Position,
    Style, TaffyAuto, TaffyTree,
};
use taffy::style::{Direction, Overflow};

pub(crate) fn map_taffy_style_v1(style: LayoutStyleV1) -> Style {
    let width = style.width();
    let height = style.height();
    let padding = style.padding();
    let (flex_direction, gap) = match style.axis() {
        LayoutAxisV1::Row => (
            FlexDirection::Row,
            Size {
                width: LengthPercentage::length(style.gap() as f32),
                height: LengthPercentage::length(0.0),
            },
        ),
        LayoutAxisV1::Column => (
            FlexDirection::Column,
            Size {
                width: LengthPercentage::length(0.0),
                height: LengthPercentage::length(style.gap() as f32),
            },
        ),
    };

    Style {
        dummy: PhantomData,
        display: Display::Flex,
        item_is_table: false,
        item_is_replaced: false,
        box_sizing: BoxSizing::BorderBox,
        direction: Direction::Ltr,
        overflow: Point {
            x: Overflow::Visible,
            y: Overflow::Visible,
        },
        scrollbar_width: 0.0,
        position: Position::Relative,
        inset: Rect {
            left: LengthPercentageAuto::length(0.0),
            right: LengthPercentageAuto::length(0.0),
            top: LengthPercentageAuto::length(0.0),
            bottom: LengthPercentageAuto::length(0.0),
        },
        size: Size {
            width: Dimension::length(width.preferred() as f32),
            height: Dimension::length(height.preferred() as f32),
        },
        min_size: Size {
            width: Dimension::length(width.minimum() as f32),
            height: Dimension::length(height.minimum() as f32),
        },
        max_size: Size {
            width: Dimension::length(width.maximum() as f32),
            height: Dimension::length(height.maximum() as f32),
        },
        aspect_ratio: None,
        margin: Rect {
            left: LengthPercentageAuto::length(0.0),
            right: LengthPercentageAuto::length(0.0),
            top: LengthPercentageAuto::length(0.0),
            bottom: LengthPercentageAuto::length(0.0),
        },
        padding: Rect {
            left: LengthPercentage::length(padding.left() as f32),
            right: LengthPercentage::length(padding.right() as f32),
            top: LengthPercentage::length(padding.top() as f32),
            bottom: LengthPercentage::length(padding.bottom() as f32),
        },
        border: Rect {
            left: LengthPercentage::length(0.0),
            right: LengthPercentage::length(0.0),
            top: LengthPercentage::length(0.0),
            bottom: LengthPercentage::length(0.0),
        },
        align_items: Some(AlignItems::START),
        align_self: Some(AlignSelf::START),
        align_content: Some(AlignContent::START),
        justify_content: Some(JustifyContent::START),
        gap,
        flex_direction,
        flex_wrap: FlexWrap::NoWrap,
        flex_basis: Dimension::AUTO,
        flex_grow: 0.0,
        flex_shrink: 0.0,
    }
}

pub(crate) fn map_taffy_available_space_v1(viewport: LayoutViewportV1) -> Size<AvailableSpace> {
    Size {
        width: AvailableSpace::Definite(viewport.width() as f32),
        height: AvailableSpace::Definite(viewport.height() as f32),
    }
}

pub(crate) fn new_taffy_tree_v1() -> TaffyTree<()> {
    let mut tree = TaffyTree::with_capacity(32);
    tree.disable_rounding();
    tree
}
