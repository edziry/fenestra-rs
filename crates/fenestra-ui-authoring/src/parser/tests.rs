use fenestra_ui_ir::prototype::SourceId;

use crate::diagnostic::AuthoringDiagnosticKindV1;
use crate::fen::parse_fen_document_v1;
use crate::limits::{AuthoringLimitKindV1, AuthoringLimitsV1};
use crate::vocabulary::{AnchorKindV1, AuthoringFrontendV1};

const DOCUMENT: &str = "format 1;
schema namespace 1 revision 1 {
  component c = 0 {
    property p = 0: scalar_i32 = -1 invalidates [paint];
  }
}
construction {
  template t = 0: c {
    set p = 1;
    child template t;
    child region r;
  }
  region r = 0 owner t repeat t keys [1] invalidates [structure];
}
style { set t.p = 2; }
";

#[test]
fn compact_document_uses_shared_record_order_and_labels() {
    let parsed = match parse_fen_document_v1(SourceId::new(9), DOCUMENT, generous_limits()) {
        Ok(parsed) => parsed,
        Err(error) => panic!("compact document failed: {error}"),
    };
    assert_eq!(parsed.frontend, AuthoringFrontendV1::Fen);
    let expected = [
        (AnchorKindV1::Document, "format"),
        (AnchorKindV1::Schema, "schema"),
        (AnchorKindV1::Component, "c"),
        (AnchorKindV1::Property, "p"),
        (AnchorKindV1::Construction, "construction"),
        (AnchorKindV1::Template, "t"),
        (AnchorKindV1::InitialProperty, "p"),
        (AnchorKindV1::StaticChild, "t"),
        (AnchorKindV1::RegionChild, "r"),
        (AnchorKindV1::Region, "r"),
        (AnchorKindV1::InitialKey, "1"),
        (AnchorKindV1::Style, "style"),
        (AnchorKindV1::StyleAssignment, "p"),
    ];
    assert_eq!(parsed.anchors.len(), expected.len());
    for (anchor, (kind, label)) in parsed.anchors.iter().zip(expected) {
        assert_eq!(anchor.kind, kind);
        assert_eq!(&*anchor.label, label);
        assert_eq!(anchor.physical.source_id(), Some(SourceId::new(9)));
        let (start, end) = anchor
            .physical
            .fen_byte_range()
            .expect("fen anchors should retain bytes");
        assert_eq!(
            &DOCUMENT.as_bytes()[start as usize..end as usize],
            label.as_bytes()
        );
    }
}

#[test]
fn every_parser_owned_limit_rejects_its_first_crossing() {
    let kinds = [
        AuthoringLimitKindV1::FenSourceBytes,
        AuthoringLimitKindV1::Tokens,
        AuthoringLimitKindV1::IdentifierBytes,
        AuthoringLimitKindV1::NestingDepth,
        AuthoringLimitKindV1::Components,
        AuthoringLimitKindV1::Properties,
        AuthoringLimitKindV1::Templates,
        AuthoringLimitKindV1::Regions,
        AuthoringLimitKindV1::ChildSlots,
        AuthoringLimitKindV1::InitialProperties,
        AuthoringLimitKindV1::InitialKeys,
        AuthoringLimitKindV1::StyleAssignments,
        AuthoringLimitKindV1::SourceAnchors,
    ];
    for kind in kinds {
        let result = parse_fen_document_v1(SourceId::new(9), DOCUMENT, limits_with_zero(kind));
        let error = match result {
            Ok(_) => panic!("zero {kind:?} limit unexpectedly accepted the document"),
            Err(error) => error,
        };
        assert_eq!(
            error.kind(),
            AuthoringDiagnosticKindV1::LimitExceeded(kind),
            "wrong crossing for {kind:?}"
        );
    }
}

fn generous_limits() -> AuthoringLimitsV1 {
    AuthoringLimitsV1::new(4_096, 1_024, 32, 8, 8, 8, 8, 8, 8, 8, 8, 8, 64, 4_096)
}

fn limits_with_zero(zero: AuthoringLimitKindV1) -> AuthoringLimitsV1 {
    let selected = |kind, normal| if kind == zero { 0 } else { normal };
    AuthoringLimitsV1::new(
        selected(AuthoringLimitKindV1::FenSourceBytes, 4_096),
        selected(AuthoringLimitKindV1::Tokens, 1_024),
        selected(AuthoringLimitKindV1::IdentifierBytes, 32),
        selected(AuthoringLimitKindV1::NestingDepth, 8),
        selected(AuthoringLimitKindV1::Components, 8),
        selected(AuthoringLimitKindV1::Properties, 8),
        selected(AuthoringLimitKindV1::Templates, 8),
        selected(AuthoringLimitKindV1::Regions, 8),
        selected(AuthoringLimitKindV1::ChildSlots, 8),
        selected(AuthoringLimitKindV1::InitialProperties, 8),
        selected(AuthoringLimitKindV1::InitialKeys, 8),
        selected(AuthoringLimitKindV1::StyleAssignments, 8),
        selected(AuthoringLimitKindV1::SourceAnchors, 64),
        4_096,
    )
}
