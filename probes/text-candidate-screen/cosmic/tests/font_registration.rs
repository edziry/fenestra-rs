//! Candidate facts for a future owned-font adapter; no host fonts are loaded.

use std::sync::Arc;

use cosmic_text::{Attrs, Buffer, Family, FontSystem, Metrics, Shaping, Wrap, fontdb};
use fenestra_text_screen_common::{FAMILY, FONT};
use fenestra_text_screen_cosmic::render;

fn binary(bytes: &[u8]) -> fontdb::Source {
    fontdb::Source::Binary(Arc::new(bytes.to_vec()))
}

fn fixture_fonts() -> FontSystem {
    let mut database = fontdb::Database::new();
    assert_eq!(database.faces().count(), 0);
    let ids = database.load_font_source(binary(FONT));
    assert_eq!(ids.len(), 1);
    database.set_sans_serif_family(FAMILY);
    FontSystem::new_with_locale_and_db("en-US".into(), database)
}

fn buffer(fonts: &mut FontSystem, text: &str, height: Option<f32>) -> Buffer {
    let mut buffer = Buffer::new(fonts, Metrics::new(20.0, 28.0));
    buffer.set_size(Some(130.0), height);
    buffer.set_wrap(Wrap::WordOrGlyph);
    buffer.set_text(
        text,
        &Attrs::new().family(Family::Name(FAMILY)),
        Shaping::Advanced,
        None,
    );
    buffer.shape_until_scroll(fonts, false);
    buffer
}

#[test]
fn empty_invalid_and_short_font_prefixes_return_no_faces_without_panicking() {
    let mut database = fontdb::Database::new();
    let malformed: &[&[u8]] = &[
        b"",
        b"this is not a font",
        &[0; 64],
        b"OTTO\0\0\0\0\0\0\0\0",
        b"ttcf\0\x01\0\0\0\0\0\x02",
    ];
    for bytes in malformed {
        assert!(database.load_font_source(binary(bytes)).is_empty());
        assert_eq!(database.faces().count(), 0);
    }
    for end in 0..=64 {
        assert!(database.load_font_source(binary(&FONT[..end])).is_empty());
        assert_eq!(database.faces().count(), 0);
    }
}

#[test]
fn failed_registration_preserves_the_existing_owned_font() {
    let mut fonts = fixture_fonts();
    let original_id = fonts.db().faces().next().unwrap().id;
    assert!(
        fonts
            .db_mut()
            .load_font_source(binary(b"invalid font"))
            .is_empty()
    );
    assert_eq!(fonts.db().faces().count(), 1);
    assert!(
        fonts
            .get_font(original_id, fontdb::Weight::NORMAL)
            .is_some()
    );
    assert!(matches!(
        fonts.db().face(original_id).unwrap().source,
        fontdb::Source::Binary(_)
    ));

    let layout = buffer(&mut fonts, "registered font", None);
    assert!(layout.layout_runs().next().is_some());
    for run in layout.layout_runs() {
        assert!(
            run.glyphs
                .iter()
                .all(|glyph| glyph.font_id == original_id && glyph.glyph_id != 0)
        );
    }
}

#[test]
fn unsupported_coverage_has_owned_utf8_ranges_without_hidden_font_fallback() {
    let text = "A \u{4e2d}\u{1f680}";
    let report = render(text, 130, 56, 20.0, 28.0);
    let missing: Vec<_> = report
        .glyphs
        .iter()
        .filter(|glyph| glyph.id == 0)
        .map(|glyph| glyph.source.clone())
        .collect();
    assert_eq!(missing, [2..5, 5..9]);
    for source in missing {
        assert!(text.is_char_boundary(source.start));
        assert!(text.is_char_boundary(source.end));
    }
    assert!(report.glyphs.iter().any(|glyph| glyph.id != 0));
    assert!(report.ink_pixels() > 0);
}

#[test]
fn bounded_viewport_measurements_exclude_hidden_paragraphs() {
    let mut fonts = fixture_fonts();
    let mut layout = buffer(&mut fonts, "first\nsecond\nthird\nfourth", Some(56.0));
    let visible: Vec<_> = layout
        .layout_runs()
        .map(|run| (run.line_top, run.line_height, run.line_w))
        .collect();
    assert_eq!(visible.len(), 2);
    assert_eq!(visible.last().unwrap().0 + visible.last().unwrap().1, 56.0);
    assert!(visible.iter().all(|(_, _, width)| width.is_finite()));
    assert_eq!(
        layout.lines.last().unwrap().layout_runs(None, 28.0).count(),
        0
    );

    layout.set_size(Some(130.0), None);
    layout.shape_until_scroll(&mut fonts, false);
    assert_eq!(layout.layout_runs().count(), 4);
    let measured_height = layout
        .layout_runs()
        .map(|run| run.line_top + run.line_height)
        .fold(0.0_f32, f32::max);
    assert_eq!(measured_height, 112.0);
}

#[test]
fn a_viewport_height_does_not_bound_single_paragraph_shaping_work() {
    let mut fonts = fixture_fonts();
    let text = "bounded output still shapes the entire paragraph ".repeat(12);
    let layout = buffer(&mut fonts, &text, Some(56.0));
    let visible_count = layout.layout_runs().count();
    let shaped_count = layout.lines[0].layout_runs(None, 28.0).count();
    assert_eq!(visible_count, 2);
    assert!(shaped_count > visible_count);
    assert!(layout.layout_runs().all(|run| run.line_w <= 130.0));
}
