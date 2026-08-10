use fenestra_ui_authoring::prototype::{
    AnchorKindV1, AuthoringDiagnosticKindV1, AuthoringDiagnosticV1, AuthoringFrontendV1,
    AuthoringLimitKindV1, AuthoringLimitsV1, CompiledAuthoringV1, DiagnosticLocationV1,
    FenSourceV1, GeneratedRustV1, PhysicalOriginV1, canonical_rust_v1, compile_fen_v1,
    compile_ui_v1, emit_tokens_v1,
};
use fenestra_ui_ir::prototype::{SourceId, SourceSpan};
use proc_macro2::TokenStream;

const SOURCE: SourceId = SourceId::new(96);
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
fn canonical_rust_is_exact_emitter_spelling_one_lf_and_deterministic() {
    let (fen, ui) = compile_both();
    let fen_tokens = emit_tokens_v1(&fen, generous_limits())
        .expect("the FEN program should emit with a generous bound");
    let ui_tokens = emit_tokens_v1(&ui, generous_limits())
        .expect("the UI program should emit with a generous bound");
    assert_eq!(ui_tokens.to_string(), fen_tokens.to_string());
    let expected = format!("{}\n", fen_tokens);

    let outputs = [
        canonical(&fen, generous_limits()),
        canonical(&fen, generous_limits()),
        canonical(&ui, generous_limits()),
        canonical(&ui, generous_limits()),
    ];
    for output in outputs {
        assert_eq!(output.as_str(), expected);
        assert!(output.as_str().is_ascii());
        assert!(!output.as_str().contains('\r'));
        let body = output
            .as_str()
            .strip_suffix('\n')
            .expect("canonical Rust should have one final LF");
        assert!(!body.ends_with('\n'));
    }
}

#[test]
fn canonical_rust_limit_is_inclusive_and_preserves_compiled_origin() {
    let (fen, ui) = compile_both();
    let measured = canonical(&fen, generous_limits()).as_str().len();
    assert!(measured > 0);

    for (frontend, compiled) in [
        (AuthoringFrontendV1::Fen, &fen),
        (AuthoringFrontendV1::UiMacro, &ui),
    ] {
        let exact = canonical_rust_v1(compiled, limits(measured))
            .expect("the exact canonical byte limit should be inclusive");
        assert_eq!(exact.as_str().len(), measured);

        let error = canonical_rust_v1(compiled, limits(measured - 1))
            .expect_err("one byte under canonical output should fail");
        assert_generated_limit(error, frontend);
    }
}

#[test]
fn generated_rust_debug_is_bounded_and_payload_free() {
    let (fen, _) = compile_both();
    let generated = canonical(&fen, generous_limits());
    let rendered = format!("{generated:?}");

    assert!(rendered.starts_with("GeneratedRustV1"));
    assert!(rendered.contains(&generated.as_str().len().to_string()));
    for forbidden in [
        "fenestra_ui_ir",
        "SchemaManifest",
        "panel",
        "/home/",
        "C:\\",
    ] {
        assert!(!rendered.contains(forbidden), "debug leaked {forbidden}");
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
        panic!("canonical output limit should be anchored to the document");
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

fn canonical(compiled: &CompiledAuthoringV1, limits: AuthoringLimitsV1) -> GeneratedRustV1 {
    canonical_rust_v1(compiled, limits).expect("generous canonical output bound should succeed")
}

fn compile_both() -> (CompiledAuthoringV1, CompiledAuthoringV1) {
    let fen = compile_fen_v1(
        FenSourceV1::new(SOURCE, DOCUMENT.as_bytes()),
        generous_limits(),
    )
    .expect("canonical FEN document should compile");
    let ui = compile_ui_v1(ui_tokens(DOCUMENT), generous_limits())
        .expect("canonical UI document should compile");
    (fen, ui)
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
