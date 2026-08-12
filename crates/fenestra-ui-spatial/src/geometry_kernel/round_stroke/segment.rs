use crate::model::{SpatialPointV2, SpatialScalarV2};
use crate::numeric::round_ratio_v2;

pub(super) fn segment_round_stroke_contains(
    start: SpatialPointV2,
    end: SpatialPointV2,
    width: i128,
    query: SpatialPointV2,
) -> bool {
    let start_x = i128::from(start.x().raw());
    let start_y = i128::from(start.y().raw());
    let delta_x = i128::from(end.x().raw()) - start_x;
    let delta_y = i128::from(end.y().raw()) - start_y;
    if delta_x == 0 && delta_y == 0 {
        return disk_contains(start_x, start_y, width, query);
    }

    let query_delta_x = i128::from(query.x().raw()) - start_x;
    let query_delta_y = i128::from(query.y().raw()) - start_y;
    let dot = query_delta_x * delta_x + query_delta_y * delta_y;
    let length_squared = delta_x * delta_x + delta_y * delta_y;
    let scale = i128::from(SpatialScalarV2::SCALE);
    let parameter = round_ratio_v2(dot * scale, length_squared)
        .expect("a nonzero K5 segment has positive squared length")
        .clamp(0, scale);
    let closest_x = start_x
        + round_ratio_v2(delta_x * parameter, scale).expect("the spatial format scale is positive");
    let closest_y = start_y
        + round_ratio_v2(delta_y * parameter, scale).expect("the spatial format scale is positive");
    disk_contains(closest_x, closest_y, width, query)
}

fn disk_contains(center_x: i128, center_y: i128, width: i128, query: SpatialPointV2) -> bool {
    let dx = i128::from(query.x().raw()) - center_x;
    let dy = i128::from(query.y().raw()) - center_y;
    4 * (dx * dx + dy * dy) <= width * width
}
