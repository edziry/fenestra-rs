use super::{LayoutNode, resolve};
use crate::model::ElementKind::{self, Column, Rect, Row, Text};
use crate::{Dimension, Error, Size, Style, TextMetrics};
use Dimension::{Auto, Fill, Px};

mod distribution;
mod measurement;

fn style(width: Dimension, height: Dimension) -> Style {
    let mut style = Style::new();
    style.width = width;
    style.height = height;
    style
}

fn node<'a>(
    name: &'a str,
    kind: ElementKind,
    style: Style,
    children: &'a [usize],
) -> LayoutNode<'a> {
    LayoutNode {
        name,
        kind,
        style,
        children,
    }
}

fn no_measure(_: usize, _: Option<u32>) -> Result<TextMetrics, Error> {
    panic!("fixed geometry must not require text measurement")
}

#[test]
fn fixed_boxes_keep_overflow_and_never_measure_text() {
    let nodes = [
        node("root", Row, style(Px(10), Px(8)).padding(2).gap(3), &[1, 2]),
        node("first", Text, style(Px(20), Px(30)), &[]),
        node("second", Text, style(Px(40), Px(50)), &[]),
    ];
    assert_eq!(
        resolve(&nodes, Size::new(1, 1), no_measure).unwrap(),
        [Size::new(10, 8), Size::new(20, 30), Size::new(40, 50),]
    );
}

#[test]
fn responsive_rows_and_columns_distribute_both_axes() {
    let nodes = [
        node(
            "root",
            Column,
            style(Fill(1), Fill(1)).padding(2).gap(3),
            &[1, 2],
        ),
        node("header", Rect, style(Fill(1), Px(7)), &[]),
        node("body", Row, style(Fill(1), Fill(1)).gap(2), &[3, 4]),
        node("sidebar", Rect, style(Px(10), Fill(3)), &[]),
        node("content", Rect, style(Fill(2), Fill(1)), &[]),
    ];
    assert_eq!(
        resolve(&nodes, Size::new(40, 30), no_measure).unwrap(),
        [
            Size::new(40, 30),
            Size::new(36, 7),
            Size::new(36, 16),
            Size::new(10, 16),
            Size::new(24, 16),
        ]
    );
    assert_eq!(
        resolve(&nodes, Size::new(20, 15), no_measure).unwrap(),
        [
            Size::new(20, 15),
            Size::new(16, 7),
            Size::new(16, 1),
            Size::new(10, 1),
            Size::new(4, 1),
        ]
    );
}

#[test]
fn fill_reserves_minima_redistributes_caps_and_keeps_authored_ties() {
    let mut first = style(Fill(1), Px(1));
    first.min_width = 2;
    first.max_width = 3;
    let nodes = [
        node("root", Row, style(Fill(1), Px(1)), &[1, 2, 3]),
        node("first", Rect, first, &[]),
        node("second", Rect, style(Fill(1), Px(1)), &[]),
        node("third", Rect, style(Fill(1), Px(1)), &[]),
    ];
    let sizes = resolve(&nodes, Size::new(10, 1), no_measure).unwrap();
    assert_eq!(
        sizes[1..],
        [Size::new(3, 1), Size::new(4, 1), Size::new(3, 1)]
    );
}

#[test]
fn minimum_overflow_and_exhausted_maxima_do_not_resize_fixed_siblings() {
    let mut fill = style(Fill(1), Px(1));
    fill.min_width = 8;
    fill.max_width = 10;
    let nodes = [
        node("root", Row, style(Fill(1), Px(1)).gap(2), &[1, 2]),
        node("fixed", Rect, style(Px(6), Px(1)), &[]),
        node("fill", Rect, fill, &[]),
    ];
    for (width, expected) in [(10, 8), (40, 10)] {
        let sizes = resolve(&nodes, Size::new(width, 1), no_measure).unwrap();
        assert_eq!(sizes[1], Size::new(6, 1));
        assert_eq!(sizes[2], Size::new(expected, 1));
    }
}

#[test]
fn auto_hugs_overflow_and_padding_is_a_minimum_for_empty_containers() {
    let nodes = [
        node("root", Column, style(Auto, Auto).padding(2).gap(3), &[1, 4]),
        node("row", Row, style(Auto, Auto).padding(1).gap(2), &[2, 3]),
        node("a", Rect, style(Px(10), Px(5)), &[]),
        node("b", Rect, style(Px(20), Px(8)), &[]),
        node("empty", Column, style(Auto, Fill(1)).padding(3), &[]),
    ];
    assert_eq!(
        resolve(&nodes, Size::new(1, 1), no_measure).unwrap(),
        [
            Size::new(38, 23),
            Size::new(34, 10),
            Size::new(10, 5),
            Size::new(20, 8),
            Size::new(6, 6),
        ]
    );
}

#[test]
fn auto_and_fill_reject_maxima_that_cannot_contain_padding() {
    for dimension in [Auto, Fill(1)] {
        let mut style = style(dimension, Px(10)).padding(3);
        style.max_width = 5;
        let nodes = [node("padded", Row, style, &[])];
        assert!(matches!(
            resolve(&nodes, Size::new(10, 10), no_measure),
            Err(Error::InvalidElement { node, .. }) if node == "padded"
        ));
    }
    let nodes = [node("legacy", Row, style(Px(1), Px(1)).padding(2), &[])];
    assert_eq!(
        resolve(&nodes, Size::new(1, 1), no_measure).unwrap(),
        [Size::new(1, 1)]
    );
}

#[test]
fn default_maximum_clamps_wide_intrinsic_sums_without_resizing_fixed_children() {
    let nodes = [
        node("root", Row, style(Auto, Px(1)).gap(1), &[1, 2]),
        node("large", Rect, style(Px(i32::MAX), Px(1)), &[]),
        node("small", Rect, style(Px(1), Px(1)), &[]),
    ];
    assert_eq!(
        resolve(&nodes, Size::new(1, 1), no_measure).unwrap(),
        [
            Size::new(i32::MAX as u32, 1),
            Size::new(i32::MAX as u32, 1),
            Size::new(1, 1)
        ]
    );
}

#[test]
fn fixed_preferences_respect_explicit_minimum_and_maximum() {
    let mut fixed = style(Px(2), Px(20));
    fixed.min_width = 4;
    fixed.max_height = 8;
    let nodes = [node("fixed", Rect, fixed, &[])];
    assert_eq!(
        resolve(&nodes, Size::new(1, 1), no_measure).unwrap(),
        [Size::new(4, 8)]
    );
}
