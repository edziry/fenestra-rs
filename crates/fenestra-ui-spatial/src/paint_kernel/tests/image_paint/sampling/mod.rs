use super::*;

macro_rules! with_final_image_paint {
    ($image:expr, $source:expr, $destination:expr, $opacity:expr, $paint:ident => $body:block) => {{
        with_p4_image!($image, p4_proof => {
            let preclip = expect_p5_success(prepare_image_paint_p5(
                PAINT_INDEX,
                &p4_proof,
                $source,
                $destination,
                $opacity,
            ));
            let $paint = expect_p5_success(
                finish_image_paint_bounds_after_item_phase_p5(preclip),
            );
            $body
        });
    }};
}

mod coverage;
mod mapping;
mod pixels;
mod widened;

fn image_from_pixels(key: u32, width: u32, height: u32, pixels: &[[u8; 4]]) -> SpatialImageV2 {
    assert_eq!(
        pixels.len(),
        usize::try_from(u64::from(width) * u64::from(height)).expect("fixture area fits usize")
    );
    let mut bytes = Vec::with_capacity(pixels.len() * 4);
    for pixel in pixels {
        bytes.extend_from_slice(pixel);
    }
    SpatialImageV2::new(
        SpatialImageKeyV2::new(key),
        width,
        height,
        width.checked_mul(4).expect("fixture stride fits u32"),
        bytes.into_boxed_slice(),
    )
}

fn id_image(key: u32, width: u32, height: u32) -> SpatialImageV2 {
    let count =
        usize::try_from(u64::from(width) * u64::from(height)).expect("fixture area fits usize");
    let pixels = (0..count)
        .map(|index| {
            [
                u8::try_from(index + 1).expect("fixture id fits u8"),
                0,
                0,
                255,
            ]
        })
        .collect::<Vec<_>>();
    image_from_pixels(key, width, height, &pixels)
}

fn sample_bytes(sample: Option<SpatialRgba8V2>) -> Option<[u8; 4]> {
    sample.map(|color| [color.r(), color.g(), color.b(), color.a()])
}
