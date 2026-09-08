//! Disposable caret and selection geometry from the screened cosmic adapter.

use cosmic_text::{Buffer, Cursor, LayoutRun};
use fenestra_text_screen_common::grapheme_boundaries;
use fenestra_ui::TextBuffer;

/// Owned text rectangle in local pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Owned geometry with no candidate font or buffer handles.
#[derive(Clone, Debug, PartialEq)]
pub struct EditorGeometry {
    pub caret: TextRect,
    pub highlights: Vec<TextRect>,
}

/// Returns disposable geometry, preferring the following row at soft wraps.
///
/// The owned selection has no visual affinity. Newline-only selections have
/// no highlight rectangle, and LFCR's interior boundary shares the next line.
pub fn geometry(text: &TextBuffer, width: u32, font_size: f32, line_height: f32) -> EditorGeometry {
    let (_, buffer) = crate::layout(text.text(), width, font_size, line_height);
    let range = text.selection().range();
    let start = cursor(&buffer, range.start);
    let end = cursor(&buffer, range.end);
    let focus = cursor(&buffer, text.selection().focus());
    let (x, y) = caret_position(&buffer, &focus);
    let mut highlights = Vec::new();
    if !range.is_empty() {
        for run in buffer.layout_runs() {
            if run.line_i < start.line || run.line_i > end.line {
                continue;
            }
            highlights.extend(highlight(&run, start, end));
        }
    }
    EditorGeometry {
        caret: TextRect {
            x,
            y,
            width: 2.0,
            height: line_height,
        },
        highlights,
    }
}

pub fn hit(
    text: &TextBuffer,
    width: u32,
    font_size: f32,
    line_height: f32,
    x: f32,
    y: f32,
) -> usize {
    let (_, buffer) = crate::layout(text.text(), width, font_size, line_height);
    let Some(hit) = rtl_cluster_hit(&buffer, x, y).or_else(|| buffer.hit(x, y)) else {
        return text.text().len();
    };
    let base: usize = buffer
        .lines
        .iter()
        .take(hit.line)
        .map(|line| line.text().len() + line.ending().as_str().len())
        .sum();
    let offset = (base + hit.index).min(text.text().len());
    // A candidate hit can resolve inside a shaping cluster. The editing model
    // accepts only complete extended graphemes, including missing-glyph input.
    grapheme_boundaries(text.text())
        .into_iter()
        .take_while(|boundary| *boundary <= offset)
        .last()
        .unwrap_or(0)
}

fn caret_position(buffer: &Buffer, cursor: &Cursor) -> (f32, f32) {
    // cosmic's cursor_position chooses the preceding row when an offset is
    // shared by wrapped rows. Prefer the next row's start deterministically.
    buffer
        .layout_runs()
        .filter(|run| run.line_i == cursor.line)
        .find_map(|run| {
            run.glyphs
                .iter()
                .any(|glyph| glyph.start == cursor.index)
                .then(|| run.cursor_position(cursor).map(|x| (x, run.line_top)))
                .flatten()
        })
        .or_else(|| buffer.cursor_position(cursor))
        .unwrap_or((0.0, 0.0))
}

fn highlight(run: &LayoutRun<'_>, start: Cursor, end: Cursor) -> Vec<TextRect> {
    // cosmic 0.19's helper subdivides every cluster from left to right, which
    // reverses partial RTL ligature selections. Use logical grapheme cells
    // from the correct visual edge, matching its caret width approximation.
    let mut spans = Vec::new();
    for glyph in run.glyphs {
        let boundaries = grapheme_boundaries(&run.text[glyph.start..glyph.end]);
        let count = boundaries.len() - 1;
        for (index, pair) in boundaries.windows(2).enumerate() {
            if (start.line == run.line_i && glyph.start + pair[1] <= start.index)
                || (end.line == run.line_i && glyph.start + pair[0] >= end.index)
            {
                continue;
            }
            let width = glyph.w / count as f32;
            if width <= 0.0 {
                continue;
            }
            let cell = if glyph.level.is_rtl() {
                count - index - 1
            } else {
                index
            };
            spans.push(TextRect {
                x: glyph.x + cell as f32 * width,
                y: run.line_top,
                width,
                height: run.line_height,
            });
        }
    }
    spans.sort_by(|a, b| a.x.total_cmp(&b.x));
    let mut merged: Vec<TextRect> = Vec::new();
    for span in spans {
        if let Some(previous) = merged.last_mut()
            && span.x <= previous.x + previous.width + 0.001
        {
            previous.width = (span.x + span.width).max(previous.x + previous.width) - previous.x;
        } else {
            merged.push(span);
        }
    }
    merged
}

fn rtl_cluster_hit(buffer: &Buffer, x: f32, y: f32) -> Option<Cursor> {
    // Buffer::hit also walks multi-grapheme RTL clusters from the left edge.
    // Correct only those clusters; retain candidate hit behavior elsewhere.
    let run = buffer
        .layout_runs()
        .find(|run| y >= run.line_top && y < run.line_top + run.line_height)?;
    let glyph = run.glyphs.iter().find(|glyph| {
        glyph.level.is_rtl() && glyph.w > 0.0 && x >= glyph.x && x <= glyph.x + glyph.w
    })?;
    let boundaries = grapheme_boundaries(&run.text[glyph.start..glyph.end]);
    let count = boundaries.len() - 1;
    if count < 2 {
        return None;
    }
    let from_right = (glyph.x + glyph.w - x) / glyph.w;
    let nearest = ((from_right * count as f32).round() as usize).min(count);
    Some(Cursor::new(run.line_i, glyph.start + boundaries[nearest]))
}

fn cursor(buffer: &Buffer, offset: usize) -> Cursor {
    let mut base = 0;
    for (index, line) in buffer.lines.iter().enumerate() {
        let end = base + line.text().len();
        if offset <= end {
            // LFCR contains two legal graphemes but the candidate treats it
            // as one ending. Its interior maps to the following line start.
            return Cursor::new(index, offset.saturating_sub(base));
        }
        base = end + line.ending().as_str().len();
    }
    let last = buffer.lines.len().saturating_sub(1);
    Cursor::new(
        last,
        buffer.lines.last().map_or(0, |line| line.text().len()),
    )
}
