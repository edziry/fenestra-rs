#![forbid(unsafe_code)]

//! Fixture boundary for the disposable EXP-0007 typed authoring probe.

/// Exact registered external authoring fixture.
pub const LAYOUT_BOARD_FEN_V1: &[u8] = include_bytes!("../fixtures/layout-board.fen");
