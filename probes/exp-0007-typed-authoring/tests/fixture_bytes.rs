use fenestra_ui_exp_0007_typed_authoring::LAYOUT_BOARD_FEN_V1;

#[test]
fn registered_fen_fixture_is_exact_canonical_ascii() {
    assert_eq!(LAYOUT_BOARD_FEN_V1.len(), 1_350);
    assert_eq!(LAYOUT_BOARD_FEN_V1.last(), Some(&b'\n'));
    assert!(!LAYOUT_BOARD_FEN_V1.contains(&b'\r'));
    assert!(
        LAYOUT_BOARD_FEN_V1
            .iter()
            .all(|byte| *byte == b'\n' || *byte == b'\t' || (0x20..=0x7e).contains(byte))
    );
}
