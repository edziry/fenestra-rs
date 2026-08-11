// P1-P6 are staged before validated paint consumers.
#![allow(dead_code)]

use crate::brush::SpatialRgba8V2;

mod error;
mod gradient;
mod image_error;
mod image_model;
mod image_paint;
mod image_paint_error;
mod image_paint_model;
mod image_sample;
mod image_validation;
mod model;
mod sample;

pub(crate) use error::{
    PaintP2Error, PaintP2ErrorKind, PaintP2Field, PaintP2GradientKind, PaintP2LimitKind,
    PaintP2Location,
};
pub(crate) use gradient::{prepare_gradient_p2, prepare_solid_p2};
#[cfg(test)]
use image_error::{
    PaintP4Channel, PaintP4Error, PaintP4ErrorKind, PaintP4Field, PaintP4ImageKind,
    PaintP4LimitKind, PaintP4Location,
};
#[cfg(test)]
use image_paint::{finish_image_paint_bounds_after_item_phase_p5, prepare_image_paint_p5};
#[cfg(test)]
use image_paint_error::{
    PaintP5Error, PaintP5ErrorKind, PaintP5Field, PaintP5ImageKind, PaintP5Location,
};
#[cfg(test)]
use image_sample::sample_image_p6;
#[cfg(test)]
use image_validation::prepare_image_p4;
pub(crate) use model::PreparedGradientP2;
#[cfg(test)]
use sample::sample_gradient_p3;

fn scale_byte(channel: u8, factor: u8) -> u8 {
    ((u16::from(channel) * u16::from(factor) + 127) / 255) as u8
}

fn normalize_straight_p1(straight: SpatialRgba8V2) -> SpatialRgba8V2 {
    let alpha = straight.a();
    SpatialRgba8V2::new(
        scale_byte(straight.r(), alpha),
        scale_byte(straight.g(), alpha),
        scale_byte(straight.b(), alpha),
        alpha,
    )
}

fn apply_opacity_p1(premultiplied: SpatialRgba8V2, opacity: u8) -> SpatialRgba8V2 {
    SpatialRgba8V2::new(
        scale_byte(premultiplied.r(), opacity),
        scale_byte(premultiplied.g(), opacity),
        scale_byte(premultiplied.b(), opacity),
        scale_byte(premultiplied.a(), opacity),
    )
}

fn source_over_channel(source: u8, destination: u8, inverse_source_alpha: u8) -> u8 {
    let output = u16::from(source) + u16::from(scale_byte(destination, inverse_source_alpha));
    u8::try_from(output).expect("validated premultiplied SourceOver stays in byte range")
}

fn source_over_p1(source: SpatialRgba8V2, destination: SpatialRgba8V2) -> SpatialRgba8V2 {
    let inverse_source_alpha = u8::MAX - source.a();
    SpatialRgba8V2::new(
        source_over_channel(source.r(), destination.r(), inverse_source_alpha),
        source_over_channel(source.g(), destination.g(), inverse_source_alpha),
        source_over_channel(source.b(), destination.b(), inverse_source_alpha),
        source_over_channel(source.a(), destination.a(), inverse_source_alpha),
    )
}

#[cfg(test)]
mod tests;
