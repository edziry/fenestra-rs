use fenestra_ui_spatial::prototype::{SpatialNodeKeyV2, SpatialPointV2, SpatialScalarV2};

use crate::*;

#[test]
fn typed_keys_and_path_ranges_round_trip_raw_values() {
    let path_key = SpatialPathKeyV2::new(11);
    let shape_key = SpatialShapeKeyV2::new(22);
    let clip_key = SpatialClipKeyV2::new(u32::MAX);
    assert_eq!(path_key.get(), 11);
    assert_eq!(shape_key.get(), 22);
    assert_eq!(clip_key.get(), u32::MAX);

    let path = SpatialPathV2::new(path_key, u32::MAX - 1, u32::MAX);
    assert_eq!(path.key(), path_key);
    assert_eq!(path.verb_start(), u32::MAX - 1);
    assert_eq!(path.verb_length(), u32::MAX);
}

#[test]
fn every_path_verb_payload_round_trips_distinct_points() {
    let verbs = [
        SpatialPathVerbV2::MoveTo { to: point(1, 2) },
        SpatialPathVerbV2::LineTo { to: point(3, 4) },
        SpatialPathVerbV2::QuadraticTo {
            control: point(5, 6),
            to: point(7, 8),
        },
        SpatialPathVerbV2::CubicTo {
            control1: point(9, 10),
            control2: point(11, 12),
            to: point(13, 14),
        },
        SpatialPathVerbV2::Close,
    ];

    match verbs[0] {
        SpatialPathVerbV2::MoveTo { to } => assert_eq!(to, point(1, 2)),
        _ => panic!("expected move"),
    }
    match verbs[1] {
        SpatialPathVerbV2::LineTo { to } => assert_eq!(to, point(3, 4)),
        _ => panic!("expected line"),
    }
    match verbs[2] {
        SpatialPathVerbV2::QuadraticTo { control, to } => {
            assert_eq!(control, point(5, 6));
            assert_eq!(to, point(7, 8));
        }
        _ => panic!("expected quadratic"),
    }
    match verbs[3] {
        SpatialPathVerbV2::CubicTo {
            control1,
            control2,
            to,
        } => {
            assert_eq!(control1, point(9, 10));
            assert_eq!(control2, point(11, 12));
            assert_eq!(to, point(13, 14));
        }
        _ => panic!("expected cubic"),
    }
    match verbs[4] {
        SpatialPathVerbV2::Close => {}
        _ => panic!("expected close"),
    }
}

#[test]
fn every_shape_payload_and_record_round_trip_without_validation() {
    let geometries = [
        SpatialShapeGeometryV2::Rect {
            origin: point(15, 16),
            width: scalar(i64::MIN),
            height: scalar(i64::MAX),
        },
        SpatialShapeGeometryV2::Circle {
            center: point(17, 18),
            radius: scalar(-19),
        },
        SpatialShapeGeometryV2::Polygon {
            point_start: u32::MAX - 2,
            point_length: u32::MAX - 1,
        },
        SpatialShapeGeometryV2::Path {
            path: SpatialPathKeyV2::new(u32::MAX),
        },
    ];

    match geometries[0] {
        SpatialShapeGeometryV2::Rect {
            origin,
            width,
            height,
        } => {
            assert_eq!(origin, point(15, 16));
            assert_eq!(width.raw(), i64::MIN);
            assert_eq!(height.raw(), i64::MAX);
        }
        _ => panic!("expected rect"),
    }
    match geometries[1] {
        SpatialShapeGeometryV2::Circle { center, radius } => {
            assert_eq!(center, point(17, 18));
            assert_eq!(radius.raw(), -19);
        }
        _ => panic!("expected circle"),
    }
    match geometries[2] {
        SpatialShapeGeometryV2::Polygon {
            point_start,
            point_length,
        } => {
            assert_eq!(point_start, u32::MAX - 2);
            assert_eq!(point_length, u32::MAX - 1);
        }
        _ => panic!("expected polygon"),
    }
    match geometries[3] {
        SpatialShapeGeometryV2::Path { path } => assert_eq!(path.get(), u32::MAX),
        _ => panic!("expected path shape"),
    }

    for (index, geometry) in geometries.into_iter().enumerate() {
        let key = SpatialShapeKeyV2::new(31 + index as u32);
        let owner = if index == 0 {
            SpatialNodeKeyV2::new(0)
        } else {
            SpatialNodeKeyV2::new(35 + index as u32)
        };
        let shape = SpatialShapeV2::new(key, owner, geometry);
        assert_eq!(shape.key(), key);
        assert_eq!(shape.owner(), owner);
        assert_eq!(shape.geometry(), geometry);
    }
}

#[test]
fn coverage_and_clip_payloads_round_trip_raw_invalid_values() {
    let shape = SpatialShapeKeyV2::new(41);
    let fill = SpatialCoverageV2::Fill {
        shape,
        rule: SpatialFillRuleV2::EvenOdd,
    };
    let stroke = SpatialCoverageV2::RoundStroke {
        shape: SpatialShapeKeyV2::new(42),
        width: scalar(-43),
    };
    match fill {
        SpatialCoverageV2::Fill { shape, rule } => {
            assert_eq!(shape.get(), 41);
            assert_eq!(rule, SpatialFillRuleV2::EvenOdd);
        }
        _ => panic!("expected fill"),
    }
    match stroke {
        SpatialCoverageV2::RoundStroke { shape, width } => {
            assert_eq!(shape.get(), 42);
            assert_eq!(width.raw(), -43);
        }
        _ => panic!("expected stroke"),
    }

    let clip = SpatialClipV2::new(
        SpatialClipKeyV2::new(51),
        SpatialNodeKeyV2::new(0),
        Some(SpatialClipKeyV2::new(u32::MAX)),
        SpatialShapeKeyV2::new(52),
        SpatialFillRuleV2::NonZero,
    );
    assert_eq!(clip.key().get(), 51);
    assert_eq!(clip.owner().get(), 0);
    assert_eq!(clip.parent().map(SpatialClipKeyV2::get), Some(u32::MAX));
    assert_eq!(clip.shape().get(), 52);
    assert_eq!(clip.fill_rule(), SpatialFillRuleV2::NonZero);

    let independent = SpatialClipV2::new(
        SpatialClipKeyV2::new(53),
        SpatialNodeKeyV2::new(54),
        None,
        SpatialShapeKeyV2::new(55),
        SpatialFillRuleV2::EvenOdd,
    );
    assert_eq!(independent.key().get(), 53);
    assert_eq!(independent.owner().get(), 54);
    assert_eq!(independent.parent(), None);
    assert_eq!(independent.shape().get(), 55);
    assert_eq!(independent.fill_rule(), SpatialFillRuleV2::EvenOdd);
}

#[test]
fn borrowed_geometry_input_preserves_every_slice_and_table_order() {
    let points = [point(61, 62), point(63, 64)];
    let verbs = [SpatialPathVerbV2::MoveTo { to: point(65, 66) }];
    let paths = [SpatialPathV2::new(SpatialPathKeyV2::new(0), 0, 1)];
    let shapes = [SpatialShapeV2::new(
        SpatialShapeKeyV2::new(0),
        SpatialNodeKeyV2::new(1),
        SpatialShapeGeometryV2::Path {
            path: SpatialPathKeyV2::new(0),
        },
    )];
    let clips = [SpatialClipV2::new(
        SpatialClipKeyV2::new(0),
        SpatialNodeKeyV2::new(1),
        None,
        SpatialShapeKeyV2::new(0),
        SpatialFillRuleV2::NonZero,
    )];

    let input = SpatialGeometryInputV2::new(&points, &verbs, &paths, &shapes, &clips);
    assert_eq!(input.polygon_points().as_ptr(), points.as_ptr());
    assert_eq!(input.polygon_points().len(), points.len());
    assert_eq!(input.path_verbs().as_ptr(), verbs.as_ptr());
    assert_eq!(input.path_verbs().len(), verbs.len());
    assert_eq!(input.paths().as_ptr(), paths.as_ptr());
    assert_eq!(input.paths().len(), paths.len());
    assert_eq!(input.shapes().as_ptr(), shapes.as_ptr());
    assert_eq!(input.shapes().len(), shapes.len());
    assert_eq!(input.clips().as_ptr(), clips.as_ptr());
    assert_eq!(input.clips().len(), clips.len());
}

fn scalar(raw: i64) -> SpatialScalarV2 {
    SpatialScalarV2::new(raw)
}

fn point(x: i64, y: i64) -> SpatialPointV2 {
    SpatialPointV2::new(scalar(x), scalar(y))
}
