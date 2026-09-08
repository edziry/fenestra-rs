fn main() {
    fenestra_ui_authoring::view::build_file("src/preferences.fen", "preferences.rs")
        .unwrap_or_else(|error| panic!("{error}"));
}
