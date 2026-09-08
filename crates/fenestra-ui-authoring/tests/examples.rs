use std::{fs, path::PathBuf};

use fenestra_ui_authoring::prototype::{
    FenSourceV2, REFERENCE_AUTHORING_LIMITS_V2, canonical_rust_v2, compile_fen_v2, compile_ui_v2,
};
use fenestra_ui_ir::prototype::SourceId;

#[test]
fn hello_panel_frontends_compile_to_the_same_small_program() {
    let examples = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples");
    let fen_source = fs::read(examples.join("hello-panel.fen"))
        .expect("the runnable hello-panel FEN example must exist");
    let ui_source = fs::read_to_string(examples.join("hello-panel.ui"))
        .expect("the equivalent ui! example must exist");
    let ui_body = ui_source
        .strip_prefix("ui! {\n")
        .and_then(|body| body.strip_suffix("}\n"))
        .expect("the Rust example must contain one ui! invocation");

    let fen = compile_fen_v2(
        FenSourceV2::new(SourceId::new(17), &fen_source),
        REFERENCE_AUTHORING_LIMITS_V2,
    )
    .expect("the FEN example must compile within reference authoring bounds");
    let ui = compile_ui_v2(
        ui_body.parse().expect("the ui! example must tokenize"),
        REFERENCE_AUTHORING_LIMITS_V2,
    )
    .expect("the ui! example must compile within reference authoring bounds");

    assert_eq!(fen.schema(), ui.schema());
    assert_eq!(fen.construction(), ui.construction());
    assert_eq!(fen.style(), ui.style());
    assert_eq!(fen.spatial(), ui.spatial());
    assert_eq!(fen.spatial().nodes().len(), 2);
    assert!(fen.spatial().images().is_empty());
    assert!(canonical_rust_v2(&fen, REFERENCE_AUTHORING_LIMITS_V2).is_ok());
}
