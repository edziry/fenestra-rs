#![forbid(unsafe_code)]

//! Fixture boundary for the disposable EXP-0007 typed authoring probe.

use fenestra_ui_ir::prototype::{ConstructionProgram, SchemaManifest, StyleProgram};
use fenestra_ui_macros::ui;

/// Exact registered external authoring fixture.
pub const LAYOUT_BOARD_FEN_V1: &[u8] = include_bytes!("../fixtures/layout-board.fen");

/// Canonical Rust generated from the registered `.fen` fixture in `OUT_DIR`.
pub const LAYOUT_BOARD_GENERATED_RUST_V1: &str =
    include_str!(concat!(env!("OUT_DIR"), "/layout_board_fen_v1.rs"));

/// Constructs the raw IR triple generated from the registered `.fen` fixture.
#[must_use]
pub fn generated_layout_board_v1() -> (SchemaManifest, ConstructionProgram, StyleProgram) {
    include!(concat!(env!("OUT_DIR"), "/layout_board_fen_v1.rs"))
}

/// Constructs the raw IR triple expanded from the registered `ui!` fixture.
#[must_use]
pub fn macro_layout_board_v1() -> (SchemaManifest, ConstructionProgram, StyleProgram) {
    include!("../fixtures/layout-board.ui")
}
