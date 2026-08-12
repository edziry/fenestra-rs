use fenestra_ui_authoring::prototype::{
    AnchorKindV2, AuthoringDiagnosticKindV2, AuthoringLimitKindV2, AuthoringLimitsV2,
    DiagnosticLocationV2, FenSourceV2, REFERENCE_AUTHORING_LIMITS_V2, compile_fen_v2,
};

use super::support::{
    ExpectedDiagnostic, FIXTURE, GENEROUS_LIMITS, LIMIT_VALUES, SOURCE, assert_anchor, limits_with,
};

const KINDS: [AuthoringLimitKindV2; 28] = [
    AuthoringLimitKindV2::FenSourceBytes,
    AuthoringLimitKindV2::Tokens,
    AuthoringLimitKindV2::IdentifierBytes,
    AuthoringLimitKindV2::NestingDepth,
    AuthoringLimitKindV2::Components,
    AuthoringLimitKindV2::Properties,
    AuthoringLimitKindV2::Templates,
    AuthoringLimitKindV2::Regions,
    AuthoringLimitKindV2::ChildSlots,
    AuthoringLimitKindV2::InitialProperties,
    AuthoringLimitKindV2::InitialKeys,
    AuthoringLimitKindV2::StyleAssignments,
    AuthoringLimitKindV2::Images,
    AuthoringLimitKindV2::ImageBytes,
    AuthoringLimitKindV2::SpatialNodes,
    AuthoringLimitKindV2::SpatialFields,
    AuthoringLimitKindV2::Shapes,
    AuthoringLimitKindV2::Paths,
    AuthoringLimitKindV2::PathVerbs,
    AuthoringLimitKindV2::PolygonPoints,
    AuthoringLimitKindV2::Brushes,
    AuthoringLimitKindV2::GradientStops,
    AuthoringLimitKindV2::Clips,
    AuthoringLimitKindV2::PaintItems,
    AuthoringLimitKindV2::HitItems,
    AuthoringLimitKindV2::SemanticItems,
    AuthoringLimitKindV2::SourceAnchors,
    AuthoringLimitKindV2::GeneratedRustBytes,
];

#[test]
fn limit_vocabulary_constructor_and_reference_values_are_exact() {
    assert_eq!(AuthoringLimitKindV2::ALL, KINDS);
    let limits = AuthoringLimitsV2::new(LIMIT_VALUES);
    for (index, kind) in KINDS.into_iter().enumerate() {
        assert_eq!(limits.limit(kind), LIMIT_VALUES[index]);
    }
    let mut expected = LIMIT_VALUES;
    expected[AuthoringLimitKindV2::GeneratedRustBytes as usize] = 107_789;
    for (index, kind) in KINDS.into_iter().enumerate() {
        assert_eq!(REFERENCE_AUTHORING_LIMITS_V2.limit(kind), expected[index]);
    }
}

#[test]
fn every_parser_limit_accepts_equality_and_rejects_one_under() {
    let observed = [
        7_714, 1_610, 15, 8, 1, 8, 7, 1, 6, 19, 2, 3, 1, 16, 7, 264, 5, 1, 5, 3, 3, 3, 3, 4, 4, 3,
        380,
    ];
    for (kind, count) in KINDS[..27].iter().copied().zip(observed) {
        compile_fen_v2(
            FenSourceV2::new(SOURCE, FIXTURE.as_bytes()),
            limits_with(kind, count),
        )
        .unwrap_or_else(|error| panic!("equality must pass for {kind:?}: {error:?}"));

        let error = compile_fen_v2(
            FenSourceV2::new(SOURCE, FIXTURE.as_bytes()),
            limits_with(kind, count - 1),
        )
        .expect_err("one under the observed count must fail");
        assert_eq!(
            error.kind(),
            AuthoringDiagnosticKindV2::LimitExceeded(kind),
            "kind {kind:?}"
        );
    }
}

#[test]
fn limit_priority_is_preflight_then_authored_record_order() {
    let zeroes = AuthoringLimitsV2::new([0; 28]);
    let bytes = compile_fen_v2(FenSourceV2::new(SOURCE, FIXTURE.as_bytes()), zeroes)
        .expect_err("source-byte preflight should win");
    assert_eq!(
        bytes.kind(),
        AuthoringDiagnosticKindV2::LimitExceeded(AuthoringLimitKindV2::FenSourceBytes)
    );
    assert_physical(&bytes);

    let tokens = compile_fen_v2(
        FenSourceV2::new(SOURCE, FIXTURE.as_bytes()),
        limits_with(AuthoringLimitKindV2::Tokens, 0),
    )
    .expect_err("token preflight should precede parsing");
    assert_eq!(
        tokens.kind(),
        AuthoringDiagnosticKindV2::LimitExceeded(AuthoringLimitKindV2::Tokens)
    );
    assert_physical(&tokens);

    let mut values = LIMIT_VALUES;
    values[AuthoringLimitKindV2::Properties as usize] = 0;
    values[AuthoringLimitKindV2::SpatialNodes as usize] = 0;
    let properties = compile_fen_v2(
        FenSourceV2::new(SOURCE, FIXTURE.as_bytes()),
        AuthoringLimitsV2::new(values),
    )
    .expect_err("the earlier property declaration should win");
    assert_eq!(
        properties.kind(),
        AuthoringDiagnosticKindV2::LimitExceeded(AuthoringLimitKindV2::Properties)
    );

    let image_bytes = compile_fen_v2(
        FenSourceV2::new(SOURCE, FIXTURE.as_bytes()),
        limits_with(AuthoringLimitKindV2::ImageBytes, 15),
    )
    .expect_err("the sixteenth image byte should cross the limit");
    assert_eq!(
        image_bytes.kind(),
        AuthoringDiagnosticKindV2::LimitExceeded(AuthoringLimitKindV2::ImageBytes)
    );
    assert_anchor(
        FIXTURE,
        &image_bytes,
        ExpectedDiagnostic {
            kind: image_bytes.kind(),
            anchor_kind: AnchorKindV2::Image,
            ordinal: 59,
            culprit: "0",
            occurrence: 33,
        },
    );
}

fn assert_physical(error: &fenestra_ui_authoring::prototype::AuthoringDiagnosticV2) {
    let DiagnosticLocationV2::Physical(origin) = error.location() else {
        panic!("pre-anchor limit failure should be physical");
    };
    assert_eq!(origin.source_id(), Some(SOURCE));
}

#[test]
fn generated_rust_limit_is_emitter_only() {
    compile_fen_v2(
        FenSourceV2::new(SOURCE, FIXTURE.as_bytes()),
        limits_with(AuthoringLimitKindV2::GeneratedRustBytes, 0),
    )
    .expect("compilation must ignore the generated-Rust limit");
    assert_eq!(
        GENEROUS_LIMITS.limit(AuthoringLimitKindV2::GeneratedRustBytes),
        usize::MAX
    );
}
