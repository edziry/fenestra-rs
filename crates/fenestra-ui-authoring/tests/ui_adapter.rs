use fenestra_ui_authoring::prototype::{AuthoringLimitsV1, compile_ui_v1};
use proc_macro2::TokenStream;

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
fn ui_token_stream_compiles_through_the_public_authoring_seam() {
    let tokens = DOCUMENT
        .parse::<TokenStream>()
        .expect("the canonical UI document should tokenize");
    let compiled = compile_ui_v1(tokens, limits())
        .expect("the canonical UI document should compile through the shared pipeline");

    assert_eq!(compiled.logical_source_catalog(), &[b'@'; 10]);
    assert_eq!(compiled.source_map().entries().len(), 10);
    for entry in compiled.source_map().entries() {
        assert_eq!(entry.physical_origin().source_id(), None);
        assert_eq!(entry.physical_origin().fen_byte_range(), None);
    }
}

const fn limits() -> AuthoringLimitsV1 {
    AuthoringLimitsV1::new(4_096, 256, 32, 8, 1, 1, 2, 1, 1, 0, 0, 0, 10, 4_096)
}
