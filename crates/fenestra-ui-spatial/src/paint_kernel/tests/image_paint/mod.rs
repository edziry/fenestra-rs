use super::*;

const PAINT_INDEX: u32 = 13;
const IMAGE_KEY: u32 = 7;

macro_rules! with_p4_image {
    ($image:expr, $proof:ident => $body:block) => {{
        let image_value = $image;
        let mut accepted_pixels = 0;
        let $proof = prepare_image_p4(&image_value, &mut accepted_pixels, usize::MAX, usize::MAX)
            .expect("fixture image must satisfy P4");
        assert_eq!(
            accepted_pixels,
            usize::try_from(u64::from(image_value.width()) * u64::from(image_value.height()))
                .expect("fixture area fits usize")
        );
        $body
    }};
}

mod binding;
mod bounds;
mod destination;
mod source;

fn raw_image(key: u32, width: u32, height: u32, pixel: [u8; 4]) -> SpatialImageV2 {
    let pixels = usize::try_from(u64::from(width) * u64::from(height))
        .expect("small fixture area fits usize");
    SpatialImageV2::new(
        SpatialImageKeyV2::new(key),
        width,
        height,
        width.checked_mul(4).expect("small fixture stride fits u32"),
        pixel.repeat(pixels).into_boxed_slice(),
    )
}

fn source(x: u32, y: u32, width: u32, height: u32) -> SpatialImageSourceRectV2 {
    SpatialImageSourceRectV2::new(x, y, width, height)
}

fn destination(x: i64, y: i64, width: i64, height: i64) -> SpatialImageDestinationRectV2 {
    SpatialImageDestinationRectV2::new(scalar(x), scalar(y), scalar(width), scalar(height))
}

fn valid_source() -> SpatialImageSourceRectV2 {
    source(1, 1, 3, 2)
}

fn valid_destination() -> SpatialImageDestinationRectV2 {
    destination(-17, 23, 31, 47)
}

fn expect_p5_error<T>(
    result: Result<T, PaintP5Error>,
    kind: PaintP5ErrorKind,
    field: PaintP5Field,
) {
    expect_p5_error_at(result, PAINT_INDEX, kind, field);
}

fn expect_p5_error_at<T>(
    result: Result<T, PaintP5Error>,
    index: u32,
    kind: PaintP5ErrorKind,
    field: PaintP5Field,
) {
    let error = match result {
        Ok(_) => panic!("expected P5 image-paint failure"),
        Err(error) => error,
    };
    assert_eq!(error.kind(), kind);
    assert_eq!(error.location(), PaintP5Location::Paint { index, field });
}

fn expect_p5_success<T>(result: Result<T, PaintP5Error>) -> T {
    match result {
        Ok(value) => value,
        Err(_) => panic!("expected P5 image-paint success"),
    }
}
