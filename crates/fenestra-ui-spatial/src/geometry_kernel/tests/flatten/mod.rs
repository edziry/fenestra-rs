use super::*;

mod curves;
mod limits;
mod representation;

const FLATTENED_TOTAL_MAXIMUM: usize =
    REGISTERED_SPATIAL_LIMITS_V2.limit(SpatialLimitKindV2::FlattenedSegmentsTotal);
const FLATTENED_PER_PATH_MAXIMUM: usize =
    REGISTERED_SPATIAL_LIMITS_V2.limit(SpatialLimitKindV2::FlattenedSegmentsPerPath);
const DEPTH_16_NONFLAT_HEIGHT: i64 = 257 * 4_294_967_296;

fn validated(verbs: &[SpatialPathVerbV2]) -> ValidatedPathK1<'_> {
    expect_valid(validate_path_k1(PATH_INDEX, verbs, 0, PATH_SUBPATH_MAXIMUM))
}

fn flatten(
    verbs: &[SpatialPathVerbV2],
    accepted_total: usize,
    maximum_per_path: usize,
    maximum_total: usize,
) -> Result<FlattenedPathK2, GeometryK2Error> {
    flatten_path_k2(
        PATH_INDEX,
        validated(verbs),
        accepted_total,
        maximum_per_path,
        maximum_total,
    )
}

fn expect_flattened(result: Result<FlattenedPathK2, GeometryK2Error>) -> FlattenedPathK2 {
    match result {
        Ok(flattened) => flattened,
        Err(error) => panic!("expected K2 flattening success, got {error:?}"),
    }
}

fn expect_k2_error(
    result: Result<FlattenedPathK2, GeometryK2Error>,
    kind: GeometryK2ErrorKind,
    source_verb: u32,
) {
    let error = match result {
        Ok(_) => panic!("expected K2 flattening failure"),
        Err(error) => error,
    };
    assert_eq!(error.kind(), kind);
    assert_eq!(
        error.location(),
        path_location(source_verb, GeometryK1Field::Kind)
    );
    assert_eq!(error.observed(), None);
    assert_eq!(error.maximum(), None);
}

fn expect_k2_limit(
    result: Result<FlattenedPathK2, GeometryK2Error>,
    limit: GeometryK2LimitKind,
    source_verb: u32,
    observed: usize,
    maximum: usize,
) {
    let error = match result {
        Ok(_) => panic!("expected K2 flattening limit failure"),
        Err(error) => error,
    };
    assert_eq!(error.kind(), GeometryK2ErrorKind::LimitExceeded(limit));
    assert_eq!(
        error.location(),
        path_location(source_verb, GeometryK1Field::Kind)
    );
    assert_eq!(error.observed(), Some(observed as u128));
    assert_eq!(error.maximum(), Some(maximum as u128));
}

fn assert_points(flattened: &FlattenedPathK2, expected: &[SpatialPointV2]) {
    assert_eq!(flattened.points(), expected);
}

fn assert_subpath(
    flattened: &FlattenedPathK2,
    ordinal: usize,
    point_start: usize,
    point_length: usize,
    explicitly_closed: bool,
) {
    let subpath = &flattened.subpaths()[ordinal];
    assert_eq!(subpath.point_start(), point_start);
    assert_eq!(subpath.point_length(), point_length);
    assert_eq!(subpath.is_explicitly_closed(), explicitly_closed);
}
