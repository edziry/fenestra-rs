use super::image_error::{
    PaintP4Channel, PaintP4Error, PaintP4ErrorKind, PaintP4Field, PaintP4ImageKind,
    PaintP4LimitKind, PaintP4Location,
};
use super::image_model::ValidatedImageP4;

use crate::image::SpatialImageV2;

pub(crate) fn prepare_image_p4<'a>(
    image: &'a SpatialImageV2,
    accepted_pixels: &mut usize,
    maximum_edge: usize,
    maximum_pixels: usize,
) -> Result<ValidatedImageP4<'a>, PaintP4Error> {
    let index = image.key().get();
    if image.width() == 0 {
        return Err(image_error(
            index,
            PaintP4ImageKind::ZeroExtent,
            PaintP4Field::Width,
        ));
    }
    if image.height() == 0 {
        return Err(image_error(
            index,
            PaintP4ImageKind::ZeroExtent,
            PaintP4Field::Height,
        ));
    }

    validate_edge(index, PaintP4Field::Width, image.width(), maximum_edge)?;
    validate_edge(index, PaintP4Field::Height, image.height(), maximum_edge)?;

    let pixels = u128::from(image.width()) * u128::from(image.height());
    let candidate_pixels = *accepted_pixels as u128 + pixels;
    if candidate_pixels > maximum_pixels as u128 {
        return Err(PaintP4Error::limit(
            PaintP4LimitKind::ImagePixelsTotal,
            image_location(index, PaintP4Field::Pixel),
            candidate_pixels,
            maximum_pixels as u128,
        ));
    }

    let expected_stride = u128::from(image.width()) * 4;
    if u128::from(image.stride()) != expected_stride {
        return Err(image_error(
            index,
            PaintP4ImageKind::StrideMismatch,
            PaintP4Field::Stride,
        ));
    }

    let expected_length = u128::from(image.stride()) * u128::from(image.height());
    if image.bytes().len() as u128 != expected_length {
        return Err(image_error(
            index,
            PaintP4ImageKind::LengthMismatch,
            PaintP4Field::ByteLength,
        ));
    }

    validate_pixels(index, image.bytes())?;
    *accepted_pixels = usize::try_from(candidate_pixels)
        .expect("a pixel candidate within a usize maximum fits usize");
    Ok(ValidatedImageP4::new(image))
}

fn validate_edge(
    index: u32,
    field: PaintP4Field,
    observed: u32,
    maximum: usize,
) -> Result<(), PaintP4Error> {
    if observed as u128 > maximum as u128 {
        Err(PaintP4Error::limit(
            PaintP4LimitKind::ImageEdge,
            image_location(index, field),
            u128::from(observed),
            maximum as u128,
        ))
    } else {
        Ok(())
    }
}

fn validate_pixels(index: u32, bytes: &[u8]) -> Result<(), PaintP4Error> {
    for (pixel, rgba) in bytes.chunks_exact(4).enumerate() {
        let alpha = rgba[3];
        for (channel, value) in [
            (PaintP4Channel::R, rgba[0]),
            (PaintP4Channel::G, rgba[1]),
            (PaintP4Channel::B, rgba[2]),
        ] {
            if value > alpha {
                return Err(PaintP4Error::new(
                    PaintP4ErrorKind::InvalidImage(PaintP4ImageKind::InvalidPremultipliedPixel),
                    PaintP4Location::ImagePixel {
                        image: index,
                        pixel: pixel as u128,
                        channel,
                    },
                ));
            }
        }
    }
    Ok(())
}

fn image_error(index: u32, kind: PaintP4ImageKind, field: PaintP4Field) -> PaintP4Error {
    PaintP4Error::new(
        PaintP4ErrorKind::InvalidImage(kind),
        image_location(index, field),
    )
}

const fn image_location(index: u32, field: PaintP4Field) -> PaintP4Location {
    PaintP4Location::Image { index, field }
}
