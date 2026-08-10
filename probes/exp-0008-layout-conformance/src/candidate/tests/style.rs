use fenestra_ui_layout::prototype::{
    LayoutAxisV1::{Column, Row},
    LayoutStyleV1,
};
use taffy::prelude::{
    AlignContent, AlignItems, AlignSelf, AvailableSpace, BoxSizing, Dimension, Display,
    FlexDirection, FlexWrap, JustifyContent, LengthPercentage, LengthPercentageAuto, Position,
    Size, Style, TaffyAuto,
};
use taffy::style::{Direction, Overflow};

use crate::candidate::{map_taffy_available_space_v1, map_taffy_style_v1, new_taffy_tree_v1};

use super::support::{dimension, padding};

#[test]
fn authored_dimensions_padding_and_row_gap_map_to_exact_lengths() {
    let mapped = map_taffy_style_v1(LayoutStyleV1::new(
        Row,
        dimension(10, 20, 30),
        dimension(40, 50, 60),
        padding(1, 2, 3, 4),
        5,
    ));

    assert_eq!(mapped.size.width, Dimension::length(20.0));
    assert_eq!(mapped.size.height, Dimension::length(50.0));
    assert_eq!(mapped.min_size.width, Dimension::length(10.0));
    assert_eq!(mapped.min_size.height, Dimension::length(40.0));
    assert_eq!(mapped.max_size.width, Dimension::length(30.0));
    assert_eq!(mapped.max_size.height, Dimension::length(60.0));
    assert_eq!(mapped.padding.left, LengthPercentage::length(1.0));
    assert_eq!(mapped.padding.right, LengthPercentage::length(2.0));
    assert_eq!(mapped.padding.top, LengthPercentage::length(3.0));
    assert_eq!(mapped.padding.bottom, LengthPercentage::length(4.0));
    assert_eq!(mapped.gap.width, LengthPercentage::length(5.0));
    assert_eq!(mapped.gap.height, LengthPercentage::length(0.0));
    assert_eq!(mapped.flex_direction, FlexDirection::Row);
}

#[test]
fn column_gap_maps_only_to_the_vertical_axis() {
    let mapped = map_taffy_style_v1(LayoutStyleV1::new(
        Column,
        dimension(0, 1, 2),
        dimension(0, 1, 2),
        padding(0, 0, 0, 0),
        7,
    ));

    assert_eq!(mapped.gap.width, LengthPercentage::length(0.0));
    assert_eq!(mapped.gap.height, LengthPercentage::length(7.0));
    assert_eq!(mapped.flex_direction, FlexDirection::Column);
}

#[test]
fn non_authored_taffy_fields_are_explicitly_neutralized() {
    let mapped = map_taffy_style_v1(LayoutStyleV1::new(
        Row,
        dimension(0, 1, 2),
        dimension(0, 1, 2),
        padding(0, 0, 0, 0),
        0,
    ));

    assert_eq!(mapped.display, Display::Flex);
    assert!(!mapped.item_is_table);
    assert!(!mapped.item_is_replaced);
    assert_eq!(mapped.box_sizing, BoxSizing::BorderBox);
    assert_eq!(mapped.direction, Direction::Ltr);
    assert_eq!(mapped.overflow.x, Overflow::Visible);
    assert_eq!(mapped.overflow.y, Overflow::Visible);
    assert_eq!(mapped.scrollbar_width, 0.0);
    assert_eq!(mapped.position, Position::Relative);
    assert_eq!(mapped.inset.left, LengthPercentageAuto::length(0.0));
    assert_eq!(mapped.inset.right, LengthPercentageAuto::length(0.0));
    assert_eq!(mapped.inset.top, LengthPercentageAuto::length(0.0));
    assert_eq!(mapped.inset.bottom, LengthPercentageAuto::length(0.0));
    assert_eq!(mapped.margin.left, LengthPercentageAuto::length(0.0));
    assert_eq!(mapped.margin.right, LengthPercentageAuto::length(0.0));
    assert_eq!(mapped.margin.top, LengthPercentageAuto::length(0.0));
    assert_eq!(mapped.margin.bottom, LengthPercentageAuto::length(0.0));
    assert_eq!(mapped.border.left, LengthPercentage::length(0.0));
    assert_eq!(mapped.border.right, LengthPercentage::length(0.0));
    assert_eq!(mapped.border.top, LengthPercentage::length(0.0));
    assert_eq!(mapped.border.bottom, LengthPercentage::length(0.0));
    assert_eq!(mapped.aspect_ratio, None);
    assert_eq!(mapped.align_items, Some(AlignItems::START));
    assert_eq!(mapped.align_self, Some(AlignSelf::START));
    assert_eq!(mapped.align_content, Some(AlignContent::START));
    assert_eq!(mapped.justify_content, Some(JustifyContent::START));
    assert_eq!(mapped.flex_wrap, FlexWrap::NoWrap);
    assert_eq!(mapped.flex_basis, Dimension::AUTO);
    assert_eq!(mapped.flex_grow, 0.0);
    assert_eq!(mapped.flex_shrink, 0.0);
}

#[test]
fn viewport_maps_to_definite_available_space() {
    assert_eq!(
        map_taffy_available_space_v1(super::support::viewport(17, 23)),
        Size {
            width: AvailableSpace::Definite(17.0),
            height: AvailableSpace::Definite(23.0),
        }
    );
}

#[test]
fn call_local_taffy_tree_has_pixel_rounding_disabled() {
    let mut tree = new_taffy_tree_v1();
    let child = tree
        .new_leaf(Style {
            size: Size {
                width: Dimension::length(1.25),
                height: Dimension::length(1.25),
            },
            ..Style::default()
        })
        .expect("synthetic child must build");
    let root = tree
        .new_with_children(
            Style {
                size: Size {
                    width: Dimension::length(10.5),
                    height: Dimension::length(10.5),
                },
                ..Style::default()
            },
            &[child],
        )
        .expect("synthetic root must build");

    tree.compute_layout(
        root,
        Size {
            width: AvailableSpace::Definite(10.5),
            height: AvailableSpace::Definite(10.5),
        },
    )
    .expect("synthetic tree must solve");

    assert_eq!(tree.layout(root).expect("root layout").size.width, 10.5);
    assert_eq!(tree.layout(child).expect("child layout").size.width, 1.25);
}
