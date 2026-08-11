use super::*;

mod base;
mod errors;
mod projection;
mod stroke;

fn aabb(min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> SpatialAabbV2 {
    match SpatialAabbV2::from_edges(scalar(min_x), scalar(min_y), scalar(max_x), scalar(max_y)) {
        Some(bounds) => bounds,
        None => panic!("test bounds must be canonical and ordered"),
    }
}

fn rect(origin: SpatialPointV2, width: i64, height: i64) -> ValidatedRectK1 {
    expect_valid(validate_rect_k1(
        SHAPE_INDEX,
        origin,
        scalar(width),
        scalar(height),
    ))
}

fn circle(center: SpatialPointV2, radius: i64) -> ValidatedCircleK1 {
    expect_valid(validate_circle_k1(SHAPE_INDEX, center, scalar(radius)))
}

fn polygon(points: &[SpatialPointV2]) -> ValidatedPolygonK1<'_> {
    expect_valid(validate_polygon_k1(
        SHAPE_INDEX,
        points,
        POLYGON_POINT_MAXIMUM,
    ))
}

fn path(verbs: &[SpatialPathVerbV2]) -> ValidatedPathK1<'_> {
    expect_valid(validate_path_k1(PATH_INDEX, verbs, 0, PATH_SUBPATH_MAXIMUM))
}

fn stroke(source: GeometryK1StrokeSource, width: i64) -> ValidatedStrokeK1 {
    expect_valid(validate_stroke_k1(source, scalar(width)))
}

fn expect_derived(result: Result<DerivedLocalBoundsK3, GeometryK3Error>) -> DerivedLocalBoundsK3 {
    match result {
        Ok(derived) => derived,
        Err(error) => panic!("expected K3 bounds success, got {error:?}"),
    }
}

fn expect_k3_error<T>(
    result: Result<T, GeometryK3Error>,
    axis: SpatialAxisV2,
    location: GeometryK1Location,
) {
    let error = match result {
        Ok(_) => panic!("expected K3 bounds failure"),
        Err(error) => error,
    };
    assert_eq!(
        error.kind(),
        GeometryK3ErrorKind::LocalBoundsOutOfDomain(axis)
    );
    assert_eq!(error.location(), location);
}
