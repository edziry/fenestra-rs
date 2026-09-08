use fenestra_text_app::{application, checksum, panel, update_headless};
use fenestra_ui::{Application, Color, Size, TextError, TextStyle};

#[test]
fn authored_text_has_named_geometry_metrics_and_visible_ink() {
    let app = application().unwrap();
    assert_eq!(
        app.node_names().collect::<Vec<_>>(),
        ["root", "title", "instructions", "content", "status"]
    );
    assert_eq!(app.hit_test(24, 136), Some("content"));
    let metrics = app.text_metrics("content").unwrap();
    assert!(metrics.glyphs() > 20);
    assert_eq!(metrics.lines(), 3);
    assert_eq!(metrics.missing_glyphs(), 0);
    let raster = app.raster().unwrap();
    assert_eq!(raster.size(), Size::new(640, 360));
    let bounds = app.bounds("content").unwrap();
    let ink = (bounds.y()..bounds.y() + i64::from(bounds.height())).any(|y| {
        (bounds.x()..bounds.x() + i64::from(bounds.width())).any(|x| {
            let offset = (y as usize * 640 + x as usize) * 4;
            raster.bytes()[offset..offset + 3] != [30, 40, 56]
        })
    });
    assert!(ink, "glyph pixels must differ from the text background");
}

#[test]
fn headless_edits_change_pixels_and_typography_repeatably() {
    let mut first = application().unwrap();
    let before = first.raster().unwrap();
    update_headless(&mut first).unwrap();
    assert!(
        first
            .text("content")
            .unwrap()
            .ends_with("TextBuffer: cafe\u{301}.")
    );
    assert!(first.text_metrics("content").unwrap().lines() >= 4);
    let updated = first.raster().unwrap();
    assert_ne!(before.bytes(), updated.bytes());
    let mut second = application().unwrap();
    update_headless(&mut second).unwrap();
    assert_eq!(updated, second.raster().unwrap());
    assert_ne!(checksum(updated.bytes()), checksum(before.bytes()));
}

#[test]
fn failed_content_and_style_updates_preserve_published_text_and_pixels() {
    let mut app = application().unwrap();
    let content = app.text("content").unwrap().to_owned();
    let raster = app.raster().unwrap();
    let generation = app.generation();
    assert!(matches!(
        app.set_text("content", "\u{4e2d}\u{1f680}"),
        Err(fenestra_ui::Error::Text(TextError::MissingGlyphs { .. }))
    ));
    assert!(
        app.set_text_style(
            "content",
            TextStyle::new()
                .font_size(0)
                .color(Color::rgba8(1, 2, 3, 255))
        )
        .is_err()
    );
    assert_eq!(app.text("content").unwrap(), content);
    assert_eq!(app.generation(), generation);
    assert_eq!(app.raster().unwrap(), raster);
    assert!(matches!(
        Application::new(panel(), Size::new(640, 360)),
        Err(fenestra_ui::Error::Text(TextError::EngineUnavailable))
    ));
}
