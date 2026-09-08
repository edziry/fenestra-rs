mod cursors;

use fenestra_ui::{
    TextError, TextGeometry, TextGeometryQuery, TextGeometryRequest, TextHighlight, TextMetrics,
    TextRect, TextSelection,
};
use parley::{Cursor, Layout, Selection};

use crate::source::Source;
use cursors::Cursors;

pub(crate) fn query(
    layout: &Layout<()>,
    source: &Source<'_>,
    request: TextGeometryRequest<'_>,
    metrics: TextMetrics,
) -> Result<TextGeometry, TextError> {
    let cursors = Cursors::new(layout, source, request)?;
    let anchor = cursors.cursor(request.selection().anchor());
    let focus = cursors.cursor(request.selection().focus());
    let current = Selection::new(anchor, focus);
    let (selection, preferred_x) = match request.query() {
        TextGeometryQuery::Current => (current, None),
        TextGeometryQuery::Hit { point, extend } => {
            let target = cursors.hit(point.x(), point.y());
            (with_focus(anchor, target, extend), None)
        }
        TextGeometryQuery::Left { extend } | TextGeometryQuery::Right { extend } => {
            let right = matches!(request.query(), TextGeometryQuery::Right { .. });
            let target = if !extend && !current.is_collapsed() {
                if right {
                    current.next_visual(layout, false).focus()
                } else {
                    current.previous_visual(layout, false).focus()
                }
            } else {
                cursors.step(focus, right)
            };
            (with_focus(anchor, target, extend), None)
        }
        TextGeometryQuery::LineStart { extend } | TextGeometryQuery::LineEnd { extend } => {
            let target = if matches!(request.query(), TextGeometryQuery::LineStart { .. }) {
                current.line_start(layout, extend).focus()
            } else {
                current.line_end(layout, extend).focus()
            };
            let rect = target.geometry(layout, 0.0);
            let target = cursors.valid_near(target, rect.x0, rect.y0);
            (with_focus(anchor, target, extend), None)
        }
        TextGeometryQuery::Up {
            extend,
            preferred_x,
        }
        | TextGeometryQuery::Down {
            extend,
            preferred_x,
        } => {
            let down = matches!(request.query(), TextGeometryQuery::Down { .. });
            let x = preferred_x.unwrap_or_else(|| focus.geometry(layout, 0.0).x0);
            let target = move_line(&cursors, current, down, x);
            (with_focus(anchor, target, extend), Some(x))
        }
    };
    let owned = TextSelection::new(
        cursors.to_position(selection.anchor()),
        cursors.to_position(selection.focus()),
    );
    let mut highlights = Vec::new();
    let mut failure = None;
    let mut count = 0usize;
    selection.geometry_with(layout, |rect, line| {
        count = count.saturating_add(1);
        if failure.is_some() {
            return;
        }
        if count > request.max_rects() {
            failure = Some(TextError::LimitExceeded {
                resource: "text geometry rectangles",
                actual: count,
                limit: request.max_rects(),
            });
        } else if highlights.try_reserve(1).is_err() {
            failure = Some(TextError::Engine("text geometry allocation failed".into()));
        } else {
            highlights.push(TextHighlight::new(
                TextRect::new(rect.x0, rect.y0, rect.x1, rect.y1),
                line,
            ));
        }
    });
    if let Some(error) = failure {
        return Err(error);
    }
    let output = TextGeometry::new(
        metrics,
        owned,
        caret(layout, selection.anchor()),
        caret(layout, selection.focus()),
        highlights,
        extent(layout, source.text().len()),
        preferred_x,
    )?;
    output.validate_request(request)?;
    Ok(output)
}

fn with_focus(anchor: Cursor, focus: Cursor, extend: bool) -> Selection {
    Selection::new(if extend { anchor } else { focus }, focus)
}

fn move_line(cursors: &Cursors<'_, '_>, selection: Selection, down: bool, x: f64) -> Cursor {
    let layout = cursors.layout;
    let y = selection.focus().geometry(layout, 0.0).y0;
    let index = layout
        .lines()
        .position(|line| {
            let metrics = line.metrics();
            f64::from(metrics.block_min_coord) <= y && y < f64::from(metrics.block_max_coord)
        })
        .unwrap_or(layout.len().saturating_sub(1));
    let target = if down {
        index.checked_add(1)
    } else {
        index.checked_sub(1)
    };
    if let Some(line) = target.and_then(|index| layout.get(index)) {
        let metrics = line.metrics();
        let y = (f64::from(metrics.block_min_coord) + f64::from(metrics.block_max_coord)) * 0.5;
        cursors.hit(x, y)
    } else {
        let target = if down {
            selection.line_end(layout, false).focus()
        } else {
            selection.line_start(layout, false).focus()
        };
        let rect = target.geometry(layout, 0.0);
        cursors.valid_near(target, rect.x0, rect.y0)
    }
}

fn caret(layout: &Layout<()>, cursor: Cursor) -> TextRect {
    let rect = cursor.geometry(layout, 0.0);
    // Add thickness in owned f64 space: f32 cannot represent x + 1 at large x.
    TextRect::new(rect.x0, rect.y0, rect.x0 + 1.0, rect.y1)
}

fn extent(layout: &Layout<()>, text_len: usize) -> TextRect {
    let mut left = 0.0f64;
    let mut right = 0.0f64;
    for line in layout
        .lines()
        .filter(|line| line.text_range().start < text_len)
    {
        let metrics = line.metrics();
        let x = f64::from(metrics.offset + metrics.inline_min_coord);
        left = left.min(x);
        right = right.max(x + f64::from(metrics.advance));
    }
    TextRect::new(left, 0.0, right, f64::from(layout.height()))
}
