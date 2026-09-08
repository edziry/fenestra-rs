use crate::{assert_geometry, cases, grapheme_boundaries};

pub fn shared_corpus_has_owned_geometry_and_raster(
    render: fn(&str, u32, u32, f32, f32) -> crate::Report,
) {
    for case in cases() {
        let report = render(case.text, case.width, 240, 20.0, 28.0);
        assert_geometry(case.text, &report);
        assert!(report.ink_pixels() > 0, "{}", case.name);
        if case.name == "unsupported" {
            assert!(report.missing_glyphs() > 0);
        } else {
            assert_eq!(report.missing_glyphs(), 0, "{}", case.name);
        }
        if case.name == "mixed-bidi" {
            assert!(report.glyphs.iter().any(|glyph| glyph.rtl));
            assert!(
                report
                    .glyphs
                    .windows(2)
                    .any(|pair| pair[0].source.start > pair[1].source.start)
            );
        }
        if case.name == "combining" {
            let boundaries = grapheme_boundaries(case.text);
            for glyph in &report.glyphs {
                assert!(boundaries.contains(&glyph.source.start));
                assert!(boundaries.contains(&glyph.source.end));
            }
        }
        if case.name == "hard-lines" {
            assert_eq!(report.lines.len(), 2);
            assert_eq!(report.lines[1].source.start, 11);
        }
    }
}

pub fn measurements_are_proportional_wrapped_and_repeatable(
    render: fn(&str, u32, u32, f32, f32) -> crate::Report,
) {
    let narrow = render("iiiiiiii", 360, 240, 20.0, 28.0);
    let wide = render("WWWWWWWW", 360, 240, 20.0, 28.0);
    assert!(wide.measured_width > narrow.measured_width * 2.0);
    let text = "One two three four five six seven eight nine ten.";
    let wrapped = render(text, 130, 240, 20.0, 28.0);
    let unwrapped = render(text, 900, 240, 20.0, 28.0);
    assert!(wrapped.lines.len() > unwrapped.lines.len());
    assert!(wrapped.measured_height > unwrapped.measured_height);
    assert_eq!(wrapped, render(text, 130, 240, 20.0, 28.0));
    let long_word = render("abcdefghijklmnopqrstuvwxyz", 80, 240, 20.0, 28.0);
    assert!(long_word.lines.len() > 1);
    assert!(long_word.lines.iter().all(|line| line.width <= 80.0));
    let empty = render("", 130, 240, 20.0, 28.0);
    assert_eq!(empty.lines.len(), 1);
    assert_eq!(empty.ink_pixels(), 0);
    let clipped = render(text, 130, 10, 20.0, 28.0);
    assert_eq!(wrapped.lines, clipped.lines);
    assert_eq!(clipped.rgba.len(), 130 * 10 * 4);
}
