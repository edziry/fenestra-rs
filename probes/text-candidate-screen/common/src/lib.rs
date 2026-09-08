//! Owned, disposable candidate-screen reports. No product API promise.

pub mod contract;

use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;

pub const FONT: &[u8] = include_bytes!("../../../../assets/fonts/dejavu-sans-2.37/DejaVuSans.ttf");
pub const FAMILY: &str = "DejaVu Sans";

#[derive(Clone, Debug, PartialEq)]
pub struct Glyph {
    pub source: Range<usize>,
    pub id: u32,
    pub line: usize,
    pub x: f32,
    /// Baseline position including the glyph's vertical offset, in pixels.
    pub y: f32,
    pub advance: f32,
    pub rtl: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Line {
    pub source: Range<usize>,
    pub baseline: f32,
    pub top: f32,
    pub height: f32,
    pub width: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Report {
    pub width: u32,
    pub height: u32,
    pub measured_width: f32,
    pub measured_height: f32,
    pub lines: Vec<Line>,
    pub glyphs: Vec<Glyph>,
    /// White foreground with coverage alpha, transparent background.
    pub rgba: Vec<u8>,
}

impl Report {
    pub fn new(width: u32, height: u32) -> Self {
        assert!(width > 0 && height > 0 && width <= 4096 && height <= 4096);
        Self {
            width,
            height,
            measured_width: 0.0,
            measured_height: 0.0,
            lines: Vec::new(),
            glyphs: Vec::new(),
            rgba: vec![0; width as usize * height as usize * 4],
        }
    }

    pub fn paint(&mut self, x: i32, y: i32, alpha: u8) {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return;
        }
        let offset = (y as usize * self.width as usize + x as usize) * 4;
        let destination = u32::from(self.rgba[offset + 3]);
        let source = u32::from(alpha);
        let result = source + (destination * (255 - source) + 127) / 255;
        self.rgba[offset..offset + 4].copy_from_slice(&[255, 255, 255, result as u8]);
    }

    pub fn missing_glyphs(&self) -> usize {
        self.glyphs.iter().filter(|glyph| glyph.id == 0).count()
    }

    pub fn ink_pixels(&self) -> usize {
        self.rgba
            .chunks_exact(4)
            .filter(|pixel| pixel[3] > 0)
            .count()
    }
}

pub struct Case {
    pub name: &'static str,
    pub text: &'static str,
    pub width: u32,
}

pub fn cases() -> Vec<Case> {
    macro_rules! fixture {
        ($name:literal, $width:literal) => {
            Case {
                name: $name,
                text: include_str!(concat!("../../corpus/", $name, ".txt")),
                width: $width,
            }
        };
    }
    vec![
        fixture!("latin", 360),
        fixture!("narrow", 360),
        fixture!("wide", 360),
        fixture!("wrap", 130),
        fixture!("long-word", 80),
        fixture!("combining", 360),
        fixture!("ligatures", 360),
        fixture!("mixed-bidi", 360),
        fixture!("arabic", 360),
        fixture!("greek", 360),
        fixture!("unsupported", 360),
        fixture!("hard-lines", 360),
    ]
}

pub fn grapheme_boundaries(text: &str) -> Vec<usize> {
    text.grapheme_indices(true)
        .map(|(index, _)| index)
        .chain(std::iter::once(text.len()))
        .collect()
}

pub fn assert_geometry(text: &str, report: &Report) {
    assert!(!report.lines.is_empty());
    assert!(report.measured_width.is_finite());
    assert!(report.measured_height.is_finite());
    assert_eq!(
        report.rgba.len(),
        report.width as usize * report.height as usize * 4
    );
    for glyph in &report.glyphs {
        assert!(glyph.source.start <= glyph.source.end);
        assert!(text.is_char_boundary(glyph.source.start));
        assert!(text.is_char_boundary(glyph.source.end));
        assert!(glyph.x.is_finite() && glyph.y.is_finite() && glyph.advance.is_finite());
        assert!(glyph.line < report.lines.len());
    }
    for line in &report.lines {
        assert!(line.width.is_finite() && line.height.is_finite() && line.baseline.is_finite());
        assert!(line.height > 0.0);
    }
}

pub fn print_report(candidate: &str, render: fn(&str, u32, u32, f32, f32) -> Report) {
    println!(
        "text-candidate-screen-v1 candidate={candidate} font=DejaVuSans-2.37 size=20 line-height=28 raster=white-alpha"
    );
    for case in cases() {
        let report = render(case.text, case.width, 240, 20.0, 28.0);
        assert_geometry(case.text, &report);
        println!(
            "case={} width={:.3} height={:.3} lines={} glyphs={} missing={} ink={} graphemes={}",
            case.name,
            report.measured_width,
            report.measured_height,
            report.lines.len(),
            report.glyphs.len(),
            report.missing_glyphs(),
            report.ink_pixels(),
            grapheme_boundaries(case.text).len() - 1
        );
        for (index, line) in report.lines.iter().enumerate() {
            println!(
                "  line={index} bytes={:?} width={:.3} top={:.3} baseline={:.3}",
                line.source, line.width, line.top, line.baseline
            );
        }
        for glyph in &report.glyphs {
            println!(
                "  glyph={} bytes={:?} line={} x={:.3} y={:.3} advance={:.3} rtl={}",
                glyph.id, glyph.source, glyph.line, glyph.x, glyph.y, glyph.advance, glyph.rtl
            );
        }
    }
}
