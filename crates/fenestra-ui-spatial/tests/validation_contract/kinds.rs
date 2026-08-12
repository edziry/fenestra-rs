use crate::*;

#[test]
fn layout_error_inventory_has_exact_type_composition_and_order() {
    let actual: [SpatialLayoutErrorKindV2; 22] = SpatialLayoutErrorKindV2::ALL;
    let mut expected = Vec::with_capacity(22);
    expected.extend(
        LayoutEngineErrorKindV1::ALL
            .into_iter()
            .map(SpatialLayoutErrorKindV2::Engine),
    );
    expected.extend(
        LayoutOutputErrorKindV1::ALL
            .into_iter()
            .map(SpatialLayoutErrorKindV2::Output),
    );
    expected.extend(
        LayoutOutputFieldV1::ALL
            .into_iter()
            .map(SpatialLayoutErrorKindV2::SyntheticRootMismatch),
    );
    expected.push(SpatialLayoutErrorKindV2::BridgeInvariant);

    assert_eq!(actual.as_slice(), expected);
}

#[test]
fn output_error_inventory_has_exact_type_composition_and_order() {
    let actual: [SpatialOutputErrorKindV2; 10] = SpatialOutputErrorKindV2::ALL;
    assert_eq!(
        actual,
        [
            SpatialOutputErrorKindV2::RecordCountMismatch,
            SpatialOutputErrorKindV2::KeyMismatch,
            SpatialOutputErrorKindV2::ScalarOutOfDomain,
            SpatialOutputErrorKindV2::NegativeBaseExtent(SpatialExtentV2::Width),
            SpatialOutputErrorKindV2::NegativeBaseExtent(SpatialExtentV2::Height),
            SpatialOutputErrorKindV2::InvalidWorldDeterminant,
            SpatialOutputErrorKindV2::InvalidAabb,
            SpatialOutputErrorKindV2::InvalidClipChain,
            SpatialOutputErrorKindV2::InvalidProjectionOrder,
            SpatialOutputErrorKindV2::InvalidReference,
        ]
    );
}

#[test]
fn resolver_error_inventory_has_exact_type_composition_and_order() {
    let actual: [SpatialResolveErrorKindV2; 192] = SpatialResolveErrorKindV2::ALL;
    let mut expected = Vec::with_capacity(192);
    expected.extend(
        SpatialLimitKindV2::ALL
            .into_iter()
            .map(SpatialResolveErrorKindV2::LimitExceeded),
    );
    expected.extend(
        SpatialInputErrorKindV2::ALL
            .into_iter()
            .map(SpatialResolveErrorKindV2::Input),
    );
    expected.extend(
        SpatialContentErrorKindV2::ALL
            .into_iter()
            .map(SpatialResolveErrorKindV2::Content),
    );
    expected.extend(
        SpatialDependencyErrorKindV2::ALL
            .into_iter()
            .map(SpatialResolveErrorKindV2::Dependency),
    );
    expected.extend(
        SpatialTransformErrorKindV2::ALL
            .into_iter()
            .map(SpatialResolveErrorKindV2::Transform),
    );
    expected.extend(
        SpatialLayoutErrorKindV2::ALL
            .into_iter()
            .map(SpatialResolveErrorKindV2::Layout),
    );
    expected.extend(
        SpatialArithmeticOperationV2::ALL
            .into_iter()
            .map(SpatialResolveErrorKindV2::Arithmetic),
    );
    expected.extend(
        SpatialOutputErrorKindV2::ALL
            .into_iter()
            .map(SpatialResolveErrorKindV2::Output),
    );

    assert_eq!(actual.as_slice(), expected);
}

#[test]
fn payload_error_kinds_remain_exhaustively_matchable() {
    fn layout_tag(value: SpatialLayoutErrorKindV2) -> u8 {
        match value {
            SpatialLayoutErrorKindV2::Engine(_) => 0,
            SpatialLayoutErrorKindV2::Output(_) => 1,
            SpatialLayoutErrorKindV2::SyntheticRootMismatch(_) => 2,
            SpatialLayoutErrorKindV2::BridgeInvariant => 3,
        }
    }
    fn output_tag(value: SpatialOutputErrorKindV2) -> u8 {
        match value {
            SpatialOutputErrorKindV2::RecordCountMismatch => 0,
            SpatialOutputErrorKindV2::KeyMismatch => 1,
            SpatialOutputErrorKindV2::ScalarOutOfDomain => 2,
            SpatialOutputErrorKindV2::NegativeBaseExtent(_) => 3,
            SpatialOutputErrorKindV2::InvalidWorldDeterminant => 4,
            SpatialOutputErrorKindV2::InvalidAabb => 5,
            SpatialOutputErrorKindV2::InvalidClipChain => 6,
            SpatialOutputErrorKindV2::InvalidProjectionOrder => 7,
            SpatialOutputErrorKindV2::InvalidReference => 8,
        }
    }
    fn resolve_tag(value: SpatialResolveErrorKindV2) -> u8 {
        match value {
            SpatialResolveErrorKindV2::LimitExceeded(_) => 0,
            SpatialResolveErrorKindV2::Input(_) => 1,
            SpatialResolveErrorKindV2::Content(_) => 2,
            SpatialResolveErrorKindV2::Dependency(_) => 3,
            SpatialResolveErrorKindV2::Transform(_) => 4,
            SpatialResolveErrorKindV2::Layout(_) => 5,
            SpatialResolveErrorKindV2::Arithmetic(_) => 6,
            SpatialResolveErrorKindV2::Output(_) => 7,
        }
    }

    let _ = (
        layout_tag as fn(SpatialLayoutErrorKindV2) -> u8,
        output_tag as fn(SpatialOutputErrorKindV2) -> u8,
        resolve_tag as fn(SpatialResolveErrorKindV2) -> u8,
    );
}
