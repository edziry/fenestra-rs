use super::{apply_opacity_p1, normalize_straight_p1, source_over_p1};

use crate::brush::SpatialRgba8V2;

mod invariants;
mod normalize;
mod opacity;
mod source_over;

fn color(r: u8, g: u8, b: u8, a: u8) -> SpatialRgba8V2 {
    SpatialRgba8V2::new(r, g, b, a)
}

fn assert_color(actual: SpatialRgba8V2, expected: [u8; 4]) {
    assert_eq!([actual.r(), actual.g(), actual.b(), actual.a()], expected);
}

fn reference_scale(channel: u8, factor: u8) -> u8 {
    ((u16::from(channel) * u16::from(factor) + 127) / 255) as u8
}
