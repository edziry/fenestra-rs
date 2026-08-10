use fenestra_ui_authoring::prototype::{
    AnchorKindV1, AuthoringDiagnosticKindV1, AuthoringDiagnosticV1, AuthoringFrontendV1,
    AuthoringLimitsV1, DiagnosticLocationV1, FenSourceV1, compile_fen_v1, compile_ui_v1,
};
use fenestra_ui_ir::prototype::{SourceId, SourceSpan};
use proc_macro2::{Delimiter, Group, TokenStream, TokenTree};

const SOURCE: SourceId = SourceId::new(93);
const BASE: &str = "format 1;
schema namespace 77 revision 1 {
  component boxy = 0 {
    property scalar = 0: scalar_i32 = 0 invalidates [layout];
  }
}
construction {
  template root = 0: boxy {
    child region rows;
  }
  template cell = 1: boxy {}
  region rows = 0 owner root repeat cell keys [10] invalidates [structure];
}
style {}
";

#[test]
fn unsupported_lexical_forms_match_across_fen_and_ui_frontends() {
    let cases = [
        ("raw identifier", "r#boxy"),
        ("suffixed decimal", "1u32"),
        ("hex integer", "0xff"),
        ("underscored integer", "1_0"),
        ("float", "1.0"),
        ("string", "\"x\""),
        ("character", "'x'"),
        ("unicode identifier", "\u{03b1}\u{03b2}"),
        ("unsupported punctuation", "+"),
    ];
    let mut mismatches = Vec::new();

    for (name, spelling) in cases {
        let source = replace_component_name(spelling);
        if let Some(mismatch) = unsupported_mismatch(&source, spelling, AuthoringFrontendV1::Fen) {
            mismatches.push(format!("{name}/FEN: {mismatch}"));
        }
        if let Some(mismatch) =
            unsupported_mismatch(&source, spelling, AuthoringFrontendV1::UiMacro)
        {
            mismatches.push(format!("{name}/UI: {mismatch}"));
        }
    }

    assert!(
        mismatches.is_empty(),
        "unsupported-token mismatches:\n{}",
        mismatches.join("\n")
    );
}

#[test]
fn delimiter_none_is_an_unsupported_opaque_ui_token() {
    let group = Group::new(Delimiter::None, TokenStream::new());
    let error = compile_ui_v1(TokenStream::from(TokenTree::Group(group)), limits())
        .expect_err("a delimiter-none group must be rejected");

    assert_eq!(error.frontend(), AuthoringFrontendV1::UiMacro);
    assert_eq!(error.kind(), AuthoringDiagnosticKindV1::UnsupportedToken);
    assert_opaque_physical(&error);
}

#[test]
fn leading_zero_decimal_is_anchored_invalid_literal_in_both_frontends() {
    let source = replace_once(BASE, "format 1;", "format 01;");
    let expected_range = exact_range(&source, "01");
    let cases = [
        (AuthoringFrontendV1::Fen, fen_error(&source)),
        (AuthoringFrontendV1::UiMacro, ui_error(&source)),
    ];

    for (frontend, error) in cases {
        assert_eq!(error.frontend(), frontend);
        assert_eq!(error.kind(), AuthoringDiagnosticKindV1::InvalidLiteral);
        let DiagnosticLocationV1::Anchored {
            logical,
            anchor_kind,
            physical,
        } = error.location()
        else {
            panic!("{frontend:?} leading zero should be anchored");
        };
        assert_eq!(*logical, SourceSpan::bytes(SourceId::new(0), 0, 1));
        assert_eq!(*anchor_kind, AnchorKindV1::Document);
        match frontend {
            AuthoringFrontendV1::Fen => {
                assert_eq!(physical.source_id(), Some(SOURCE));
                assert_eq!(physical.fen_byte_range(), Some(expected_range));
            }
            AuthoringFrontendV1::UiMacro => {
                assert_eq!(physical.source_id(), None);
                assert_eq!(physical.fen_byte_range(), None);
            }
        }
    }
}

#[test]
fn canonical_zero_and_nonzero_decimals_compile_in_both_frontends() {
    for spelling in ["0", "1", "10"] {
        let replacement = format!("scalar_i32 = {spelling} invalidates");
        let source = replace_once(BASE, "scalar_i32 = 0 invalidates", &replacement);
        let fen = compile_fen_v1(FenSourceV1::new(SOURCE, source.as_bytes()), limits())
            .unwrap_or_else(|error| panic!("FEN decimal {spelling} failed: {error}"));
        let ui = compile_ui_v1(ui_tokens(&source), limits())
            .unwrap_or_else(|error| panic!("UI decimal {spelling} failed: {error}"));
        assert_eq!(fen.schema(), ui.schema());
        assert_eq!(fen.construction(), ui.construction());
        assert_eq!(fen.style(), ui.style());
    }
}

#[test]
fn dot_after_decimal_matches_rust_token_boundaries() {
    for spelling in ["1..2", "1.foo"] {
        let source = replace_component_name(spelling);
        let fen = fen_error(&source);
        let ui = ui_error(&source);

        assert_eq!(fen.frontend(), AuthoringFrontendV1::Fen);
        assert_eq!(ui.frontend(), AuthoringFrontendV1::UiMacro);
        assert_eq!(fen.kind(), AuthoringDiagnosticKindV1::UnexpectedToken);
        assert_eq!(ui.kind(), AuthoringDiagnosticKindV1::UnexpectedToken);

        let DiagnosticLocationV1::Physical(fen_origin) = fen.location() else {
            panic!("FEN numeric boundary should fail before an anchor");
        };
        let DiagnosticLocationV1::Physical(ui_origin) = ui.location() else {
            panic!("UI numeric boundary should fail before an anchor");
        };
        let start = source
            .find(&format!("component {spelling}"))
            .expect("mutated component should exist")
            + "component ".len();
        let start = u32::try_from(start).expect("test offset should fit");
        assert_eq!(fen_origin.fen_byte_range(), Some((start, start + 1)));
        assert_eq!(ui_origin.source_id(), None);
        assert_eq!(ui_origin.fen_byte_range(), None);
    }
}

fn unsupported_mismatch(
    source: &str,
    spelling: &str,
    frontend: AuthoringFrontendV1,
) -> Option<String> {
    let error = match frontend {
        AuthoringFrontendV1::Fen => fen_error(source),
        AuthoringFrontendV1::UiMacro => ui_error(source),
    };
    if error.frontend() != frontend {
        return Some(format!("wrong frontend: {:?}", error.frontend()));
    }
    if error.kind() != AuthoringDiagnosticKindV1::UnsupportedToken {
        return Some(format!("wrong kind: {:?}", error.kind()));
    }
    let DiagnosticLocationV1::Physical(physical) = error.location() else {
        return Some("diagnostic was not Physical".into());
    };
    match frontend {
        AuthoringFrontendV1::Fen => {
            let expected = exact_range(source, spelling);
            if physical.source_id() != Some(SOURCE) || physical.fen_byte_range() != Some(expected) {
                return Some(format!(
                    "expected source {SOURCE:?} range {expected:?}, got source {:?} range {:?}",
                    physical.source_id(),
                    physical.fen_byte_range()
                ));
            }
        }
        AuthoringFrontendV1::UiMacro => {
            if physical.source_id().is_some() || physical.fen_byte_range().is_some() {
                return Some("opaque UI origin exposed FEN coordinates".into());
            }
        }
    }
    None
}

fn assert_opaque_physical(error: &AuthoringDiagnosticV1) {
    let DiagnosticLocationV1::Physical(physical) = error.location() else {
        panic!("unsupported UI token should be Physical");
    };
    assert_eq!(physical.source_id(), None);
    assert_eq!(physical.fen_byte_range(), None);
}

fn fen_error(source: &str) -> AuthoringDiagnosticV1 {
    compile_fen_v1(FenSourceV1::new(SOURCE, source.as_bytes()), limits())
        .expect_err("the FEN source should fail")
}

fn ui_error(source: &str) -> AuthoringDiagnosticV1 {
    compile_ui_v1(ui_tokens(source), limits()).expect_err("the UI tokens should fail")
}

fn ui_tokens(source: &str) -> TokenStream {
    source
        .parse()
        .unwrap_or_else(|error| panic!("test UI source should tokenize: {error}"))
}

fn replace_component_name(spelling: &str) -> String {
    replace_once(
        BASE,
        "component boxy = 0",
        &format!("component {spelling} = 0"),
    )
}

fn replace_once(source: &str, before: &str, after: &str) -> String {
    assert_eq!(source.matches(before).count(), 1, "ambiguous `{before}`");
    source.replacen(before, after, 1)
}

fn exact_range(source: &str, spelling: &str) -> (u32, u32) {
    let mut matches = source.match_indices(spelling);
    let start = matches
        .next()
        .unwrap_or_else(|| panic!("missing `{spelling}`"))
        .0;
    assert!(matches.next().is_none(), "ambiguous `{spelling}`");
    let end = start + spelling.len();
    (
        u32::try_from(start).expect("test offset should fit"),
        u32::try_from(end).expect("test offset should fit"),
    )
}

const fn limits() -> AuthoringLimitsV1 {
    AuthoringLimitsV1::new(
        16_384, 4_096, 64, 16, 32, 32, 32, 32, 64, 64, 64, 64, 128, 65_536,
    )
}
