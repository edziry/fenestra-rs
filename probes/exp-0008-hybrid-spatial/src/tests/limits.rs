use fenestra_ui_spatial::prototype::{
    REGISTERED_REFERENCE_RASTER_LIMITS_V2, REGISTERED_SPATIAL_LIMITS_V2,
    ReferenceRasterLimitKindV2, SpatialLimitKindV2,
};

use crate::baseline::{
    ARTIFACT_LIMITS_V2, ArtifactErrorKindV2, ArtifactLimitKindV2, ArtifactSyntheticFaultV2,
    artifact_limit_probe_v2, encode_fault_fixture_v2, registered_spatial_limits_v2,
};

use super::support::expected::SPATIAL_LIMITS;

#[test]
fn baseline_freezes_the_exact_thirty_spatial_limits_in_validation_order() {
    assert_eq!(SpatialLimitKindV2::ALL.len(), 30);
    assert_eq!(registered_spatial_limits_v2(), SPATIAL_LIMITS);
    for (ordinal, kind) in SpatialLimitKindV2::ALL.into_iter().enumerate() {
        assert_eq!(
            REGISTERED_SPATIAL_LIMITS_V2.limit(kind),
            SPATIAL_LIMITS[ordinal]
        );
    }
    assert_eq!(
        REGISTERED_REFERENCE_RASTER_LIMITS_V2.limit(ReferenceRasterLimitKindV2::Pixels),
        4_194_304
    );
}

#[test]
fn artifact_limits_are_exact_inclusive_and_have_widened_evidence() {
    assert_eq!(ARTIFACT_LIMITS_V2.records, 4096);
    assert_eq!(ARTIFACT_LIMITS_V2.line_bytes, 1024);
    assert_eq!(ARTIFACT_LIMITS_V2.artifact_bytes, 1_048_576);

    for (kind, maximum) in [
        (ArtifactLimitKindV2::Records, 4096),
        (ArtifactLimitKindV2::LineBytes, 1024),
        (ArtifactLimitKindV2::ArtifactBytes, 1_048_576),
    ] {
        artifact_limit_probe_v2(kind, maximum).expect("equality is admitted");
        let error = artifact_limit_probe_v2(kind, maximum + 1).expect_err("one over fails");
        assert_eq!(error.kind, ArtifactErrorKindV2::LimitExceeded(kind));
        assert_eq!(error.observed, Some((maximum + 1) as u128));
        assert_eq!(error.maximum, Some(maximum as u128));
        assert!(
            error.artifact.is_none(),
            "encoder must return no partial artifact"
        );
    }
}

#[test]
fn encoder_failure_priority_is_closed_and_preflights_before_rendering() {
    use ArtifactSyntheticFaultV2 as F;
    let probes: [(&[F], ArtifactErrorKindV2); 5] = [
        (
            &[
                F::InvalidModel,
                F::Records,
                F::Grammar,
                F::LineBytes,
                F::ArtifactBytes,
            ],
            ArtifactErrorKindV2::InvalidModel,
        ),
        (
            &[F::Records, F::Grammar, F::LineBytes, F::ArtifactBytes],
            ArtifactErrorKindV2::LimitExceeded(ArtifactLimitKindV2::Records),
        ),
        (
            &[F::Grammar, F::LineBytes, F::ArtifactBytes],
            ArtifactErrorKindV2::InvalidGrammar,
        ),
        (
            &[F::LineBytes, F::ArtifactBytes],
            ArtifactErrorKindV2::LimitExceeded(ArtifactLimitKindV2::LineBytes),
        ),
        (
            &[F::ArtifactBytes],
            ArtifactErrorKindV2::LimitExceeded(ArtifactLimitKindV2::ArtifactBytes),
        ),
    ];
    for (faults, expected) in probes {
        let error = encode_fault_fixture_v2(faults).expect_err("synthetic fault");
        assert_eq!(error.kind, expected);
        assert!(error.artifact.is_none());
    }
}

#[test]
fn grammar_is_checked_before_line_and_accumulated_bytes_in_record_order() {
    use ArtifactSyntheticFaultV2 as F;
    let grammar_first = encode_fault_fixture_v2(&[F::GrammarAt(7), F::LineAt(2), F::ArtifactBytes])
        .expect_err("grammar priority is global record order after record preflight");
    assert_eq!(grammar_first.kind, ArtifactErrorKindV2::InvalidGrammar);
    assert_eq!(grammar_first.record, Some(7));

    let first_line =
        encode_fault_fixture_v2(&[F::LineAt(9), F::LineAt(2)]).expect_err("first oversized line");
    assert_eq!(
        first_line.kind,
        ArtifactErrorKindV2::LimitExceeded(ArtifactLimitKindV2::LineBytes)
    );
    assert_eq!(first_line.record, Some(2));
}
