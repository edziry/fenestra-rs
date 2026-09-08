fn main() {
    fenestra_ui_authoring::view::build_file("src/panel.fen", "panel.rs")
        .unwrap_or_else(|error| panic!("{error}"));
}
