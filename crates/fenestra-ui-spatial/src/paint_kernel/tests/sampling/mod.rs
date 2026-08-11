use super::*;

mod interpolation;
mod invariants;
mod padding_duplicates;
mod parameter;

fn sample_gradient(
    stops: &[SpatialGradientStopV2],
    start: SpatialPointV2,
    end: SpatialPointV2,
    query: SpatialPointV2,
) -> SpatialRgba8V2 {
    let proof = match prepare_gradient_p2(
        BRUSH_INDEX,
        STOP_START,
        stops.len() as u32,
        start,
        end,
        stops,
        stops.len(),
    ) {
        Ok(proof) => proof,
        Err(_) => panic!("sampling fixture must prepare"),
    };
    sample_gradient_p3(&proof, query)
}

fn sample_at_parameter(stops: &[SpatialGradientStopV2], parameter: u16) -> SpatialRgba8V2 {
    sample_gradient(
        stops,
        point(0, 0),
        point(i64::from(u16::MAX), 0),
        point(i64::from(parameter), 0),
    )
}

fn opaque(r: u8, g: u8, b: u8) -> SpatialRgba8V2 {
    color(r, g, b, u8::MAX)
}
