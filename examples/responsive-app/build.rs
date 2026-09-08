fn main() {
    fenestra_ui_authoring::view::build_file("src/workspace.fen", "workspace.rs")
        .unwrap_or_else(|error| panic!("{error}"));
}
