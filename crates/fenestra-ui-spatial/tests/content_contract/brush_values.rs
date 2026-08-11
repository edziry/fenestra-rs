use fenestra_ui_spatial::prototype::{SpatialPointV2, SpatialScalarV2};

use crate::*;

#[test]
fn content_keys_colors_and_stops_round_trip_exact_raw_values() {
    let brush_key = SpatialBrushKeyV2::new(u32::MAX);
    let image_key = SpatialImageKeyV2::new(u32::MAX - 1);
    assert_eq!(brush_key.get(), u32::MAX);
    assert_eq!(image_key.get(), u32::MAX - 1);

    let color = SpatialRgba8V2::new(1, 2, 3, 4);
    assert_eq!(color.r(), 1);
    assert_eq!(color.g(), 2);
    assert_eq!(color.b(), 3);
    assert_eq!(color.a(), 4);

    let stop = SpatialGradientStopV2::new(u16::MAX, color);
    assert_eq!(stop.offset(), u16::MAX);
    assert_eq!(stop.color(), color);
}

#[test]
fn every_brush_payload_and_record_round_trips_without_validation() {
    let solid = SpatialBrushContentV2::Solid {
        color: SpatialRgba8V2::new(255, 128, 64, 0),
    };
    let gradient = SpatialBrushContentV2::LinearGradient {
        stop_start: u32::MAX - 1,
        stop_length: u32::MAX,
        start: point(i64::MIN, i64::MAX),
        end: point(i64::MAX, i64::MIN),
    };

    match solid {
        SpatialBrushContentV2::Solid { color } => {
            assert_eq!(color, SpatialRgba8V2::new(255, 128, 64, 0));
        }
        _ => panic!("expected solid brush"),
    }
    match gradient {
        SpatialBrushContentV2::LinearGradient {
            stop_start,
            stop_length,
            start,
            end,
        } => {
            assert_eq!(stop_start, u32::MAX - 1);
            assert_eq!(stop_length, u32::MAX);
            assert_eq!(start, point(i64::MIN, i64::MAX));
            assert_eq!(end, point(i64::MAX, i64::MIN));
        }
        _ => panic!("expected linear gradient"),
    }

    for (index, content) in [solid, gradient].into_iter().enumerate() {
        let key = SpatialBrushKeyV2::new(index as u32);
        let brush = SpatialBrushV2::new(key, content);
        assert_eq!(brush.key(), key);
        assert_eq!(brush.content(), content);
    }

    let coincident = SpatialBrushContentV2::LinearGradient {
        stop_start: 0,
        stop_length: 1,
        start: point(9, 10),
        end: point(9, 10),
    };
    let brush = SpatialBrushV2::new(SpatialBrushKeyV2::new(2), coincident);
    assert_eq!(brush.content(), coincident);
}

fn point(x: i64, y: i64) -> SpatialPointV2 {
    SpatialPointV2::new(SpatialScalarV2::new(x), SpatialScalarV2::new(y))
}
