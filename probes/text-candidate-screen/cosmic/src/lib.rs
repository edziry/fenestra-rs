//! Disposable cosmic-text adapter using only the versioned font fixture.

use cosmic_text::{
    Attrs, Buffer, Color, Family, FontSystem, Metrics, Shaping, SwashCache, Wrap, fontdb,
};
use fenestra_text_screen_common::{FAMILY, FONT, Glyph, Line, Report};

pub mod editor;

/// Shape all lines and paint clipped white glyph coverage into owned RGBA pixels.
///
/// Dimensions are in pixels at scale 1. This fresh-context screening adapter is
/// deliberately not a production cache or a performance reference.
pub fn render(text: &str, width: u32, height: u32, font_size: f32, line_height: f32) -> Report {
    assert!(font_size.is_finite() && font_size > 0.0);
    assert!(line_height.is_finite() && line_height > 0.0);
    let mut report = Report::new(width, height);
    let (mut fonts, mut buffer) = layout(text, width, font_size, line_height);

    let mut paragraph_start = 0;
    let starts: Vec<_> = buffer
        .lines
        .iter()
        .map(|line| {
            let start = paragraph_start;
            paragraph_start += line.text().len() + line.ending().as_str().len();
            start
        })
        .collect();
    for (line_index, run) in buffer.layout_runs().enumerate() {
        let base = starts[run.line_i];
        let source_start = run
            .glyphs
            .iter()
            .map(|glyph| glyph.start)
            .min()
            .unwrap_or(0);
        let source_end = run.glyphs.iter().map(|glyph| glyph.end).max().unwrap_or(0);
        report.measured_width = report.measured_width.max(run.line_w);
        report.measured_height = report.measured_height.max(run.line_top + run.line_height);
        report.lines.push(Line {
            source: base + source_start..base + source_end,
            baseline: run.line_y,
            top: run.line_top,
            height: run.line_height,
            width: run.line_w,
        });
        report.glyphs.extend(run.glyphs.iter().map(|glyph| Glyph {
            source: base + glyph.start..base + glyph.end,
            id: u32::from(glyph.glyph_id),
            line: line_index,
            x: glyph.x + glyph.x_offset * glyph.font_size,
            y: run.line_y + glyph.y - glyph.y_offset * glyph.font_size,
            advance: glyph.w,
            rtl: glyph.level.is_rtl(),
        }));
    }
    buffer.draw(
        &mut fonts,
        &mut SwashCache::new(),
        Color::rgb(255, 255, 255),
        |x, y, w, h, color| {
            for row in 0..h {
                for col in 0..w {
                    report.paint(x + col as i32, y + row as i32, color.a());
                }
            }
        },
    );
    report
}

pub(crate) fn layout(
    text: &str,
    width: u32,
    font_size: f32,
    line_height: f32,
) -> (FontSystem, Buffer) {
    assert!(width > 0 && width <= 4096);
    assert!(font_size.is_finite() && font_size > 0.0);
    assert!(line_height.is_finite() && line_height > 0.0);
    let mut database = fontdb::Database::new();
    database.load_font_data(FONT.to_vec());
    database.set_sans_serif_family(FAMILY);
    let mut fonts = FontSystem::new_with_locale_and_db("en-US".into(), database);
    let mut buffer = Buffer::new(&mut fonts, Metrics::new(font_size, line_height));
    buffer.set_size(Some(width as f32), None);
    buffer.set_wrap(Wrap::WordOrGlyph);
    buffer.set_text(
        text,
        &Attrs::new().family(Family::Name(FAMILY)),
        Shaping::Advanced,
        None,
    );
    buffer.shape_until_scroll(&mut fonts, false);

    (fonts, buffer)
}
