use fenestra_text_app::{macro_panel, panel};

#[test]
fn external_fen_and_real_ui_macro_construct_the_same_public_view() {
    assert_eq!(panel(), macro_panel());
    assert_eq!(panel().name(), "text_demo");
}
