use fenestra_ui::Raster;

use super::*;

fn pixel(raster: &Raster, x: usize, y: usize) -> [u8; 4] {
    let start = (y * raster.size().width() as usize + x) * 4;
    raster.bytes()[start..start + 4].try_into().unwrap()
}

#[test]
fn build_script_view_matches_the_real_ui_macro_expression() {
    let from_macro: View = include!("panel.ui");
    assert_eq!(panel(), from_macro);
}

#[test]
fn source_layout_has_literal_pixels_and_named_input_targets() {
    let app = Application::new(panel(), Size::new(320, 192)).unwrap();
    assert_eq!(
        app.node_names().collect::<Vec<_>>(),
        ["root", "cards", "primary", "secondary", "footer"]
    );
    let raster = app.raster().unwrap();
    assert_eq!(pixel(&raster, 0, 0), [18, 24, 36, 255]);
    assert_eq!(pixel(&raster, 16, 16), [48, 128, 192, 255]);
    assert_eq!(pixel(&raster, 80, 16), [18, 24, 36, 255]);
    assert_eq!(pixel(&raster, 92, 16), [88, 168, 112, 255]);
    assert_eq!(pixel(&raster, 16, 92), [36, 48, 64, 255]);
    assert_eq!(app.hit_test(16, 16), Some("primary"));
    assert_eq!(app.hit_test(80, 16), None);
    assert_eq!(app.hit_test(92, 16), Some("secondary"));
    assert_eq!(app.hit_test(16, 92), None);
}

#[test]
fn executable_updates_color_size_and_viewport_with_coherent_results() {
    let app = Application::new(panel(), Size::new(320, 192)).unwrap();
    let app = update_headless(app).unwrap();
    assert_eq!(app.generation(), 3);
    assert_eq!(app.size(), Size::new(360, 220));
    assert_eq!(app.hit_test(100, 16), Some("primary"));
    assert_eq!(app.hit_test(128, 16), None);
    assert_eq!(app.hit_test(140, 16), Some("secondary"));
    let raster = app.raster().unwrap();
    assert_eq!(pixel(&raster, 100, 16), [240, 176, 64, 255]);
    assert_eq!(pixel(&raster, 140, 16), [88, 168, 112, 255]);
    assert_eq!(pixel(&raster, 340, 200), [0, 0, 0, 0]);
}
