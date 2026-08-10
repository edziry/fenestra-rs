use fenestra_ui_authoring::prototype::{
    AnchorKindV1, AuthoringDiagnosticKindV1, AuthoringDiagnosticV1, AuthoringFrontendV1,
    AuthoringLimitKindV1, AuthoringLimitsV1, DiagnosticLocationV1, FenSourceV1, compile_fen_v1,
    compile_ui_v1,
};
use fenestra_ui_ir::prototype::{SourceId, SourceSpan};
use proc_macro2::{Delimiter, Group, Ident, Punct, Spacing, Span, TokenStream, TokenTree};

const SOURCE: SourceId = SourceId::new(94);
const DOCUMENT: &str = "format 1;
schema namespace 1 revision 1 {
  component c = 0 {
    property p = 0: scalar_i32 = 0 invalidates [layout];
  }
}
construction {
  template root = 0: c {
    child region rows;
  }
  template cell = 1: c {}
  region rows = 0 owner root repeat cell keys [] invalidates [structure];
}
style {}
";

#[test]
fn unsupported_and_adapter_limits_follow_the_registered_priority() {
    assert_ui_physical_error(
        punctuation('+'),
        limits(0, 0, 0, 0, 0),
        AuthoringDiagnosticKindV1::UnsupportedToken,
    );
    assert_ui_physical_error(
        identifier("wide"),
        limits(0, 0, 0, 0, 0),
        AuthoringDiagnosticKindV1::LimitExceeded(AuthoringLimitKindV1::Tokens),
    );
    assert_ui_physical_error(
        empty_group(Delimiter::Brace),
        limits(0, 0, 64, 0, 0),
        AuthoringDiagnosticKindV1::LimitExceeded(AuthoringLimitKindV1::Tokens),
    );
}

#[test]
fn group_delimiters_each_count_and_token_equality_is_inclusive() {
    assert_ui_physical_error(
        empty_group(Delimiter::Brace),
        limits(0, 1, 64, 1, 0),
        AuthoringDiagnosticKindV1::LimitExceeded(AuthoringLimitKindV1::Tokens),
    );
    assert_ui_physical_error(
        empty_group(Delimiter::Brace),
        limits(0, 2, 64, 1, 0),
        AuthoringDiagnosticKindV1::UnexpectedToken,
    );
}

#[test]
fn identifiers_and_root_zero_depth_use_inclusive_bounds() {
    assert_ui_physical_error(
        identifier("wide"),
        limits(0, 1, 3, 0, 0),
        AuthoringDiagnosticKindV1::LimitExceeded(AuthoringLimitKindV1::IdentifierBytes),
    );
    assert_ui_physical_error(
        identifier("wide"),
        limits(0, 1, 4, 0, 0),
        AuthoringDiagnosticKindV1::UnexpectedToken,
    );
    assert_ui_physical_error(
        empty_group(Delimiter::Brace),
        limits(0, 2, 64, 0, 0),
        AuthoringDiagnosticKindV1::LimitExceeded(AuthoringLimitKindV1::NestingDepth),
    );
}

#[test]
fn nested_group_depth_is_inclusive_from_root_depth_zero() {
    assert_ui_physical_error(
        nested_groups(),
        limits(0, 4, 64, 1, 0),
        AuthoringDiagnosticKindV1::LimitExceeded(AuthoringLimitKindV1::NestingDepth),
    );
    assert_ui_physical_error(
        nested_groups(),
        limits(0, 4, 64, 2, 0),
        AuthoringDiagnosticKindV1::UnexpectedToken,
    );
}

#[test]
fn fen_and_generated_byte_limits_do_not_apply_to_ui_compilation() {
    let compiled = compile_ui_v1(ui_tokens(DOCUMENT), limits(0, 256, 32, 8, 0))
        .expect("UI compilation should ignore FEN and generated Rust byte limits");
    assert_eq!(compiled.logical_source_catalog(), &[b'@'; 10]);
}

#[test]
fn empty_ui_stream_is_opaque_physical_unexpected_eof() {
    assert_ui_physical_error(
        TokenStream::new(),
        limits(0, 0, 0, 0, 0),
        AuthoringDiagnosticKindV1::UnexpectedEof,
    );
}

#[test]
fn semantic_faults_share_logical_identity_but_not_physical_coordinates() {
    let source = replace_once(
        DOCUMENT,
        "template root = 0: c {",
        "template root = 0: missing {",
    );
    let fen = compile_fen_v1(
        FenSourceV1::new(SOURCE, source.as_bytes()),
        generous_limits(),
    )
    .expect_err("the missing FEN component should fail");
    let ui = compile_ui_v1(ui_tokens(&source), generous_limits())
        .expect_err("the missing UI component should fail");

    assert_eq!(fen.frontend(), AuthoringFrontendV1::Fen);
    assert_eq!(ui.frontend(), AuthoringFrontendV1::UiMacro);
    assert_eq!(fen.kind(), AuthoringDiagnosticKindV1::UnknownComponentName);
    assert_eq!(ui.kind(), fen.kind());

    let (fen_logical, fen_kind, fen_origin) = anchored(&fen);
    let (ui_logical, ui_kind, ui_origin) = anchored(&ui);
    assert_eq!(fen_logical, SourceSpan::bytes(SourceId::new(0), 5, 6));
    assert_eq!(ui_logical, fen_logical);
    assert_eq!(fen_kind, AnchorKindV1::Template);
    assert_eq!(ui_kind, fen_kind);

    let expected = exact_range(&source, "missing");
    assert_eq!(fen_origin.source_id(), Some(SOURCE));
    assert_eq!(fen_origin.fen_byte_range(), Some(expected));
    assert_eq!(ui_origin.source_id(), None);
    assert_eq!(ui_origin.fen_byte_range(), None);
}

fn assert_ui_physical_error(
    tokens: TokenStream,
    limits: AuthoringLimitsV1,
    expected: AuthoringDiagnosticKindV1,
) {
    let error = compile_ui_v1(tokens, limits).expect_err("the UI input should fail");
    assert_eq!(error.frontend(), AuthoringFrontendV1::UiMacro);
    assert_eq!(error.kind(), expected);
    let DiagnosticLocationV1::Physical(origin) = error.location() else {
        panic!("adapter or pre-anchor failure should be Physical");
    };
    assert_eq!(origin.source_id(), None);
    assert_eq!(origin.fen_byte_range(), None);
}

fn anchored(
    error: &AuthoringDiagnosticV1,
) -> (
    SourceSpan,
    AnchorKindV1,
    &fenestra_ui_authoring::prototype::PhysicalOriginV1,
) {
    let DiagnosticLocationV1::Anchored {
        logical,
        anchor_kind,
        physical,
    } = error.location()
    else {
        panic!("semantic failure should be Anchored");
    };
    (*logical, *anchor_kind, physical)
}

fn identifier(value: &str) -> TokenStream {
    TokenStream::from(TokenTree::Ident(Ident::new(value, Span::call_site())))
}

fn punctuation(value: char) -> TokenStream {
    TokenStream::from(TokenTree::Punct(Punct::new(value, Spacing::Alone)))
}

fn empty_group(delimiter: Delimiter) -> TokenStream {
    TokenStream::from(TokenTree::Group(Group::new(delimiter, TokenStream::new())))
}

fn nested_groups() -> TokenStream {
    let inner = Group::new(Delimiter::Bracket, TokenStream::new());
    let outer = Group::new(Delimiter::Brace, TokenStream::from(TokenTree::Group(inner)));
    TokenStream::from(TokenTree::Group(outer))
}

fn ui_tokens(source: &str) -> TokenStream {
    source
        .parse()
        .unwrap_or_else(|error| panic!("test UI source should tokenize: {error}"))
}

fn replace_once(source: &str, before: &str, after: &str) -> String {
    assert_eq!(source.matches(before).count(), 1, "ambiguous `{before}`");
    source.replacen(before, after, 1)
}

fn exact_range(source: &str, spelling: &str) -> (u32, u32) {
    let start = source.find(spelling).expect("culprit should exist");
    let end = start + spelling.len();
    (
        u32::try_from(start).expect("test offset should fit"),
        u32::try_from(end).expect("test offset should fit"),
    )
}

const fn limits(
    fen_source_bytes: usize,
    tokens: usize,
    identifier_bytes: usize,
    nesting_depth: usize,
    generated_rust_bytes: usize,
) -> AuthoringLimitsV1 {
    AuthoringLimitsV1::new(
        fen_source_bytes,
        tokens,
        identifier_bytes,
        nesting_depth,
        8,
        8,
        8,
        8,
        8,
        8,
        8,
        8,
        64,
        generated_rust_bytes,
    )
}

const fn generous_limits() -> AuthoringLimitsV1 {
    limits(16_384, 4_096, 64, 16, 65_536)
}
