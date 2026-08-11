use super::*;

mod extents;
mod layout;
mod limits;
mod pixels;
mod success;

const IMAGE_INDEX: u32 = 7;
const IMAGE_EDGE_MAXIMUM: usize = REGISTERED_SPATIAL_LIMITS_V2.limit(SpatialLimitKindV2::ImageEdge);
const IMAGE_PIXELS_MAXIMUM: usize =
    REGISTERED_SPATIAL_LIMITS_V2.limit(SpatialLimitKindV2::ImagePixelsTotal);

fn image(width: u32, height: u32, stride: u32, bytes: Vec<u8>) -> SpatialImageV2 {
    SpatialImageV2::new(
        SpatialImageKeyV2::new(IMAGE_INDEX),
        width,
        height,
        stride,
        bytes.into_boxed_slice(),
    )
}

fn image_location(field: PaintP4Field) -> PaintP4Location {
    PaintP4Location::Image {
        index: IMAGE_INDEX,
        field,
    }
}

fn pixel_location(pixel: u128, channel: PaintP4Channel) -> PaintP4Location {
    PaintP4Location::ImagePixel {
        image: IMAGE_INDEX,
        pixel,
        channel,
    }
}

fn expect_p4_error<T>(
    result: Result<T, PaintP4Error>,
    kind: PaintP4ErrorKind,
    location: PaintP4Location,
) {
    let error = match result {
        Ok(_) => panic!("expected P4 image validation failure"),
        Err(error) => error,
    };
    assert_eq!(error.kind(), kind);
    assert_eq!(error.location(), location);
    assert_eq!(error.observed(), None);
    assert_eq!(error.maximum(), None);
}

fn expect_p4_limit<T>(
    result: Result<T, PaintP4Error>,
    limit: PaintP4LimitKind,
    location: PaintP4Location,
    observed: u128,
    maximum: u128,
) {
    let error = match result {
        Ok(_) => panic!("expected P4 image limit failure"),
        Err(error) => error,
    };
    assert_eq!(error.kind(), PaintP4ErrorKind::LimitExceeded(limit));
    assert_eq!(error.location(), location);
    assert_eq!(error.observed(), Some(observed));
    assert_eq!(error.maximum(), Some(maximum));
}

fn expect_semantic_rejection(
    image: &SpatialImageV2,
    accepted_pixels: usize,
    maximum_edge: usize,
    maximum_pixels: usize,
    kind: PaintP4ImageKind,
    location: PaintP4Location,
) {
    let mut accepted = accepted_pixels;
    expect_p4_error(
        prepare_image_p4(image, &mut accepted, maximum_edge, maximum_pixels),
        PaintP4ErrorKind::InvalidImage(kind),
        location,
    );
    assert_eq!(accepted, accepted_pixels);
}

fn expect_limit_rejection(
    image: &SpatialImageV2,
    accepted_pixels: usize,
    maximum_edge: usize,
    maximum_pixels: usize,
    limit: PaintP4LimitKind,
    location: PaintP4Location,
    observed: u128,
    maximum: u128,
) {
    let mut accepted = accepted_pixels;
    expect_p4_limit(
        prepare_image_p4(image, &mut accepted, maximum_edge, maximum_pixels),
        limit,
        location,
        observed,
        maximum,
    );
    assert_eq!(accepted, accepted_pixels);
}
