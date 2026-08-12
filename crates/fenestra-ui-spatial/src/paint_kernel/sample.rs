use super::model::PreparedGradientP2;

use crate::brush::SpatialRgba8V2;
use crate::model::SpatialPointV2;
use crate::numeric::round_ratio_v2;

pub(super) fn sample_gradient_p3(
    gradient: &PreparedGradientP2,
    point: SpatialPointV2,
) -> SpatialRgba8V2 {
    sample_gradient_stops_p3(
        gradient.start(),
        gradient.end(),
        gradient.stop_count(),
        |index| {
            let stop = gradient.stop(index);
            (stop.offset(), stop.color())
        },
        point,
    )
}

pub(crate) fn sample_gradient_fields_p3(
    start: SpatialPointV2,
    end: SpatialPointV2,
    stops: &[(u16, SpatialRgba8V2)],
    point: SpatialPointV2,
) -> SpatialRgba8V2 {
    sample_gradient_stops_p3(start, end, stops.len(), |index| stops[index], point)
}

fn sample_gradient_stops_p3(
    start: SpatialPointV2,
    end: SpatialPointV2,
    stop_count: usize,
    stop: impl Fn(usize) -> (u16, SpatialRgba8V2),
    point: SpatialPointV2,
) -> SpatialRgba8V2 {
    let delta_x = i128::from(end.x().raw()) - i128::from(start.x().raw());
    let delta_y = i128::from(end.y().raw()) - i128::from(start.y().raw());
    let relative_x = i128::from(point.x().raw()) - i128::from(start.x().raw());
    let relative_y = i128::from(point.y().raw()) - i128::from(start.y().raw());
    let denominator = delta_x * delta_x + delta_y * delta_y;
    let numerator = (relative_x * delta_x + relative_y * delta_y) * i128::from(u16::MAX);
    let rounded =
        round_ratio_v2(numerator, denominator).expect("a prepared gradient has distinct endpoints");
    let parameter = u16::try_from(rounded.clamp(0, i128::from(u16::MAX)))
        .expect("the gradient parameter is clamped to u16");

    let mut lower_index = 0;
    for index in 1..stop_count {
        if stop(index).0 > parameter {
            break;
        }
        lower_index = index;
    }

    let lower = stop(lower_index);
    let upper_index = lower_index + 1;
    if upper_index == stop_count {
        return lower.1;
    }
    let upper = stop(upper_index);
    let local = parameter - lower.0;
    let span = upper.0 - lower.0;
    let lower_color = lower.1;
    let upper_color = upper.1;
    SpatialRgba8V2::new(
        interpolate_channel(lower_color.r(), upper_color.r(), local, span),
        interpolate_channel(lower_color.g(), upper_color.g(), local, span),
        interpolate_channel(lower_color.b(), upper_color.b(), local, span),
        interpolate_channel(lower_color.a(), upper_color.a(), local, span),
    )
}

fn interpolate_channel(lower: u8, upper: u8, local: u16, span: u16) -> u8 {
    let difference = i128::from(upper) - i128::from(lower);
    let adjustment = round_ratio_v2(difference * i128::from(local), i128::from(span))
        .expect("prepared gradient stops have increasing interpolation offsets");
    u8::try_from(i128::from(lower) + adjustment)
        .expect("interpolation between byte endpoints remains a byte")
}
