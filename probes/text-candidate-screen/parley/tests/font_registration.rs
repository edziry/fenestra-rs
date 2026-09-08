//! Explicit-font candidate facts; the only valid font is the versioned fixture.

use std::sync::Arc;

use fenestra_text_screen_common::{FAMILY, FONT};
use fenestra_text_screen_parley::render;
use parley::fontique::{Blob, Collection, CollectionOptions, SourceKind};

fn blob(bytes: &[u8]) -> Blob<u8> {
    Blob::new(Arc::new(bytes.to_vec()))
}

fn collection() -> Collection {
    Collection::new(CollectionOptions {
        system_fonts: false,
        shared: false,
    })
}

#[test]
fn empty_invalid_and_short_font_prefixes_return_no_families_without_panicking() {
    let mut fonts = collection();
    assert_eq!(fonts.family_names().count(), 0);
    let malformed: &[&[u8]] = &[
        b"",
        b"this is not a font",
        &[0; 64],
        b"OTTO\0\0\0\0\0\0\0\0",
        b"ttcf\0\x01\0\0\0\0\0\x02",
        b"ttcf\0\x01\0\0\0\0\0\x20",
    ];
    for bytes in malformed {
        assert!(fonts.register_fonts(blob(bytes), None).is_empty());
        assert_eq!(fonts.family_names().count(), 0);
    }
    for end in 0..=64 {
        assert!(fonts.register_fonts(blob(&FONT[..end]), None).is_empty());
        assert_eq!(fonts.family_names().count(), 0);
    }
}

#[test]
fn owned_registration_reports_one_family_face_and_usable_raster_source() {
    let mut fonts = collection();
    let registered = fonts.register_fonts(blob(FONT), None);
    assert_eq!(registered.len(), 1);
    let (family_id, faces) = &registered[0];
    assert_eq!(fonts.family_name(*family_id), Some(FAMILY));
    assert_eq!(faces.len(), 1);
    assert_eq!(faces[0].index(), 0);
    assert!(matches!(faces[0].source().kind, SourceKind::Memory(_)));
    let retained = faces[0].load(None).unwrap();
    assert_eq!(retained.as_ref(), FONT);
    assert!(swash::FontRef::from_index(retained.as_ref(), 0).is_some());
}

#[test]
fn failed_registration_preserves_the_registered_family() {
    let mut fonts = collection();
    let registered = fonts.register_fonts(blob(FONT), None);
    let original_id = registered[0].0;
    assert!(fonts.register_fonts(blob(b"invalid font"), None).is_empty());
    assert_eq!(fonts.family_names().collect::<Vec<_>>(), [FAMILY]);
    assert_eq!(fonts.family_id(FAMILY), Some(original_id));
    assert_eq!(fonts.family(original_id).unwrap().fonts().len(), 1);
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
fn complete_measurement_is_independent_of_clipped_raster_height() {
    let text = "first\nsecond\nthird\nfourth";
    let short = render(text, 130, 28, 20.0, 28.0);
    let tall = render(text, 130, 140, 20.0, 28.0);
    assert_eq!(short.measured_width, tall.measured_width);
    assert_eq!(short.measured_height, 112.0);
    assert_eq!(short.measured_height, tall.measured_height);
    assert_eq!(short.lines, tall.lines);
    assert_eq!(short.glyphs, tall.glyphs);
    assert_eq!(short.rgba.len(), 130 * 28 * 4);
    assert!(short.ink_pixels() < tall.ink_pixels());
}
