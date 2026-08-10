use fenestra_ui_authoring::prototype::{
    AnchorKindV1, AuthoringDiagnosticKindV1, AuthoringDiagnosticV1, AuthoringFrontendV1,
    AuthoringLimitKindV1, AuthoringLimitsV1, CompiledAuthoringV1, DiagnosticLocationV1,
    FenSourceV1, PhysicalOriginV1, compile_fen_v1, compile_ui_v1, emit_tokens_v1,
};
use fenestra_ui_ir::prototype::{
    ConstructionProgram, SchemaManifest, SourceId, SourceSpan, StyleProgram,
};
use proc_macro2::{TokenStream, TokenTree};

const SOURCE: SourceId = SourceId::new(95);
const DOCUMENT: &str = "format 1;
schema namespace 9 revision 1 {
  component panel = 7 {
    property visible = 0: bool = true invalidates [paint];
    property width = 1: scalar_i32 = -2 invalidates [layout, semantics];
    property color = 2: rgba8 = rgba8(1, 2, 3, 255) invalidates [paint];
    property input = 3: input_policy = ignore invalidates [hit_test];
  }
}
construction {
  template root = 10: panel {
    set visible = false;
    set width = 3;
    set color = rgba8(4, 5, 6, 255);
    set input = accept;
    child template leaf;
    child region rows;
  }
  template leaf = 11: panel {}
  template cell = 12: panel {}
  region rows = 5 owner root repeat cell keys [42]
    invalidates [structure, layout];
}
style {
  set leaf.color = rgba8(7, 8, 9, 255);
}
";
const GOLDEN: &str = include_str!("fixtures/minimal_emitted_v1.rs");

#[test]
fn emitted_tokens_match_the_complete_golden_across_frontends_and_runs() {
    let (fen, ui) = compile_both();
    let expected_programs = expected_programs();
    assert_eq!(fen.schema(), &expected_programs.0);
    assert_eq!(fen.construction(), &expected_programs.1);
    assert_eq!(fen.style(), &expected_programs.2);
    assert_eq!(ui.schema(), &expected_programs.0);
    assert_eq!(ui.construction(), &expected_programs.1);
    assert_eq!(ui.style(), &expected_programs.2);

    let expected = golden_tokens().to_string();
    let outputs = [
        emit(&fen, generous_limits()).to_string(),
        emit(&fen, generous_limits()).to_string(),
        emit(&ui, generous_limits()).to_string(),
        emit(&ui, generous_limits()).to_string(),
    ];
    for output in outputs {
        assert_eq!(output, expected);
    }
    assert_closed_target_surface(&golden_tokens());
}

#[test]
fn generated_rust_byte_limit_is_inclusive_and_preserves_frontend_origin() {
    let (fen, ui) = compile_both();
    let baseline = emit(&fen, generous_limits());
    let measured = baseline
        .to_string()
        .len()
        .checked_add(1)
        .expect("golden spelling plus LF should fit");
    assert!(measured > 0);

    for (frontend, compiled) in [
        (AuthoringFrontendV1::Fen, &fen),
        (AuthoringFrontendV1::UiMacro, &ui),
    ] {
        let exact = emit_tokens_v1(compiled, limits(measured))
            .expect("the exact spelling-plus-LF byte limit should be inclusive");
        assert_eq!(exact.to_string(), baseline.to_string());

        let error = emit_tokens_v1(compiled, limits(measured - 1))
            .expect_err("one byte under the measured output should fail");
        assert_generated_limit(error, frontend);
    }
}

fn assert_generated_limit(error: AuthoringDiagnosticV1, frontend: AuthoringFrontendV1) {
    assert_eq!(error.frontend(), frontend);
    assert_eq!(
        error.kind(),
        AuthoringDiagnosticKindV1::LimitExceeded(AuthoringLimitKindV1::GeneratedRustBytes)
    );
    let DiagnosticLocationV1::Anchored {
        logical,
        anchor_kind,
        physical,
    } = error.location()
    else {
        panic!("generated output limit should be anchored to the document");
    };
    assert_eq!(*logical, SourceSpan::bytes(SourceId::new(0), 0, 1));
    assert_eq!(*anchor_kind, AnchorKindV1::Document);
    assert_frontend_origin(physical, frontend);
}

fn assert_frontend_origin(origin: &PhysicalOriginV1, frontend: AuthoringFrontendV1) {
    match frontend {
        AuthoringFrontendV1::Fen => {
            assert_eq!(origin.source_id(), Some(SOURCE));
            assert_eq!(origin.fen_byte_range(), Some((0, 6)));
        }
        AuthoringFrontendV1::UiMacro => {
            assert_eq!(origin.source_id(), None);
            assert_eq!(origin.fen_byte_range(), None);
        }
    }
}

fn assert_closed_target_surface(tokens: &TokenStream) {
    let spelling = tokens.to_string();
    let compact = spelling
        .chars()
        .filter(|character| !character.is_ascii_whitespace())
        .collect::<String>();
    let crate_name = "fenestra_ui_ir";
    let occurrences = compact.match_indices(crate_name).collect::<Vec<_>>();
    assert!(!occurrences.is_empty(), "golden should construct IR values");
    for (start, _) in occurrences {
        assert_eq!(&compact[start - 2..start], "::");
        assert!(compact[start + crate_name.len()..].starts_with("::prototype::"));
    }

    let mut identifiers = Vec::new();
    collect_identifiers(tokens.clone(), &mut identifiers);
    for forbidden in ["fenestra_ui_authoring", "std", "core", "alloc"] {
        assert!(!identifiers.iter().any(|ident| ident == forbidden));
    }
    for authored_name in [
        "panel", "visible", "width", "color", "input", "root", "leaf", "cell", "rows",
    ] {
        assert!(!identifiers.iter().any(|ident| ident == authored_name));
    }
    assert!(!identifiers.iter().any(|ident| {
        ident.starts_with("Authoring")
            || ident.starts_with("Resolved")
            || ident.starts_with("SourceMap")
    }));
}

fn collect_identifiers(stream: TokenStream, output: &mut Vec<String>) {
    for tree in stream {
        match tree {
            TokenTree::Group(group) => collect_identifiers(group.stream(), output),
            TokenTree::Ident(ident) => output.push(ident.to_string()),
            TokenTree::Literal(_) | TokenTree::Punct(_) => {}
        }
    }
}

fn compile_both() -> (CompiledAuthoringV1, CompiledAuthoringV1) {
    let fen = compile_fen_v1(
        FenSourceV1::new(SOURCE, DOCUMENT.as_bytes()),
        generous_limits(),
    )
    .expect("minimal FEN document should compile");
    let ui = compile_ui_v1(ui_tokens(DOCUMENT), generous_limits())
        .expect("minimal UI document should compile");
    (fen, ui)
}

fn emit(compiled: &CompiledAuthoringV1, limits: AuthoringLimitsV1) -> TokenStream {
    emit_tokens_v1(compiled, limits).expect("generous emission limit should succeed")
}

fn golden_tokens() -> TokenStream {
    GOLDEN
        .parse()
        .unwrap_or_else(|error| panic!("emission golden should tokenize: {error}"))
}

fn expected_programs() -> (SchemaManifest, ConstructionProgram, StyleProgram) {
    include!("fixtures/minimal_emitted_v1.rs")
}

fn ui_tokens(source: &str) -> TokenStream {
    source
        .parse()
        .unwrap_or_else(|error| panic!("test UI source should tokenize: {error}"))
}

const fn generous_limits() -> AuthoringLimitsV1 {
    limits(65_536)
}

const fn limits(generated_rust_bytes: usize) -> AuthoringLimitsV1 {
    AuthoringLimitsV1::new(
        8_192,
        1_024,
        32,
        8,
        8,
        16,
        8,
        8,
        16,
        16,
        8,
        8,
        64,
        generated_rust_bytes,
    )
}
