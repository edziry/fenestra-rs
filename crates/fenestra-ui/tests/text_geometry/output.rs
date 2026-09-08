use fenestra_ui::{TextGeometry, TextHighlight, TextPoint, TextRect};

use crate::*;

fn caret() -> TextRect {
    TextRect::new(0.0, 0.0, 1.0, 10.0)
}

fn extent() -> TextRect {
    TextRect::new(0.0, 0.0, 20.0, 10.0)
}

fn metrics() -> fenestra_ui::TextMetrics {
    fenestra_ui::TextMetrics::new(20.0, 10.0, 1, 2, 0)
}

fn output(selected: TextSelection, highlights: Vec<TextHighlight>) -> TextGeometry {
    TextGeometry::new(
        metrics(),
        selected,
        caret(),
        caret(),
        highlights,
        extent(),
        None,
    )
    .unwrap()
}

#[test]
fn every_output_box_rejects_nonfinite_reversed_and_out_of_domain_edges() {
    let maximum = f64::from(i32::MAX);
    for invalid in [
        TextRect::new(f64::NAN, 0.0, 1.0, 1.0),
        TextRect::new(0.0, f64::NEG_INFINITY, 1.0, 1.0),
        TextRect::new(0.0, 0.0, f64::INFINITY, 1.0),
        TextRect::new(0.0, 0.0, 1.0, f64::NAN),
        TextRect::new(1.0, 0.0, 0.0, 1.0),
        TextRect::new(0.0, 1.0, 1.0, 0.0),
        TextRect::new(-maximum - 1.0, 0.0, 0.0, 1.0),
        TextRect::new(0.0, 0.0, maximum + 1.0, 1.0),
    ] {
        for (anchor, focus, highlights, bounds) in [
            (invalid, caret(), Vec::new(), extent()),
            (caret(), invalid, Vec::new(), extent()),
            (
                caret(),
                caret(),
                vec![TextHighlight::new(invalid, 0)],
                extent(),
            ),
            (caret(), caret(), Vec::new(), invalid),
        ] {
            assert_eq!(
                TextGeometry::new(
                    metrics(),
                    selection(0, 2),
                    anchor,
                    focus,
                    highlights,
                    bounds,
                    None,
                )
                .err(),
                Some(TextError::InvalidGeometry),
                "{invalid:?}",
            );
        }
    }
}

#[test]
fn local_output_validation_rejects_invalid_metrics_highlights_and_preferred_x() {
    assert_eq!(
        TextGeometry::new(
            fenestra_ui::TextMetrics::new(f32::NAN, 10.0, 1, 2, 0),
            selection(0, 0),
            caret(),
            caret(),
            Vec::new(),
            extent(),
            None,
        )
        .err(),
        Some(TextError::InvalidMetrics),
    );
    for (selected, highlights, preferred_x) in [
        (selection(0, 2), vec![TextHighlight::new(extent(), 1)], None),
        (selection(0, 0), vec![TextHighlight::new(extent(), 0)], None),
        (selection(0, 0), Vec::new(), Some(f64::NAN)),
        (selection(0, 0), Vec::new(), Some(f64::INFINITY)),
        (selection(0, 0), Vec::new(), Some(f64::from(i32::MAX) + 1.0)),
    ] {
        assert_eq!(
            TextGeometry::new(
                metrics(),
                selected,
                caret(),
                caret(),
                highlights,
                extent(),
                preferred_x,
            )
            .err(),
            Some(TextError::InvalidGeometry),
        );
    }
    for invalid in [
        TextRect::new(0.0, 0.0, 0.0, 10.0),
        TextRect::new(0.0, 0.0, 2.0, 10.0),
        TextRect::new(0.0, 0.0, 1.0, 0.0),
    ] {
        for (anchor, focus) in [(invalid, caret()), (caret(), invalid)] {
            assert_eq!(
                TextGeometry::new(
                    metrics(),
                    selection(0, 0),
                    anchor,
                    focus,
                    Vec::new(),
                    extent(),
                    None,
                )
                .err(),
                Some(TextError::InvalidGeometry),
            );
        }
    }
    for fragments in [[(10.0, 20.0), (0.0, 10.0)], [(0.0, 11.0), (10.0, 20.0)]] {
        let highlights =
            fragments.map(|(x0, x1)| TextHighlight::new(TextRect::new(x0, 0.0, x1, 10.0), 0));
        assert_eq!(
            TextGeometry::new(
                metrics(),
                selection(0, 2),
                caret(),
                caret(),
                highlights.to_vec(),
                extent(),
                None,
            )
            .err(),
            Some(TextError::InvalidGeometry),
        );
    }
}

#[test]
fn returned_positions_are_checked_against_source_and_current_keeps_raw_endpoints() {
    let query = TextGeometryQuery::Right { extend: false };
    let original = request("e\u{301}", selection(0, 0), query).unwrap();
    for byte in [1, 2, 4] {
        assert_eq!(
            output(selection(byte, byte), Vec::new()).validate_request(original),
            Err(TextError::InvalidTextPosition { byte }),
        );
    }
    let current = request("ab", selection(0, 0), TextGeometryQuery::Current).unwrap();
    assert_eq!(
        output(selection(1, 1), Vec::new()).validate_request(current),
        Err(TextError::InvalidGeometry),
    );
    let upstream = TextPosition::new(0, TextAffinity::Upstream);
    output(TextSelection::caret(upstream), Vec::new())
        .validate_request(current)
        .unwrap();

    let current = request("ab", selection(2, 0), TextGeometryQuery::Current).unwrap();
    let highlights = vec![TextHighlight::new(extent(), 0)];
    output(selection(2, 0), highlights.clone())
        .validate_request(current)
        .unwrap();
    assert_eq!(
        output(selection(0, 2), highlights).validate_request(current),
        Err(TextError::InvalidGeometry),
    );
}

#[test]
fn moving_queries_preserve_the_anchor_when_extending_and_otherwise_collapse() {
    for extend in [false, true] {
        for query in [
            TextGeometryQuery::Hit {
                point: TextPoint::new(10.0, 5.0),
                extend,
            },
            TextGeometryQuery::Left { extend },
            TextGeometryQuery::Right { extend },
            TextGeometryQuery::LineStart { extend },
            TextGeometryQuery::LineEnd { extend },
            TextGeometryQuery::Up {
                extend,
                preferred_x: None,
            },
            TextGeometryQuery::Down {
                extend,
                preferred_x: None,
            },
        ] {
            let request = request("ab", selection(0, 2), query).unwrap();
            let valid = if extend {
                TextSelection::new(TextPosition::new(0, TextAffinity::Upstream), position(1))
            } else {
                selection(1, 1)
            };
            let valid_highlights = if extend {
                vec![TextHighlight::new(extent(), 0)]
            } else {
                Vec::new()
            };
            output(valid, valid_highlights)
                .validate_request(request)
                .unwrap();
            let invalid = if extend {
                selection(1, 2)
            } else {
                selection(0, 1)
            };
            assert_eq!(
                output(invalid, vec![TextHighlight::new(extent(), 0)]).validate_request(request),
                Err(TextError::InvalidGeometry),
                "{query:?}",
            );
        }
    }
}

#[test]
fn rectangle_limits_are_inclusive_and_cannot_be_loosened_past_the_byte_bound() {
    let highlights = (0..4)
        .map(|index| {
            let x = f64::from(index) * 5.0;
            TextHighlight::new(TextRect::new(x, 0.0, x + 5.0, 10.0), 0)
        })
        .collect::<Vec<_>>();
    let geometry_request = request("ab", selection(0, 2), TextGeometryQuery::Current).unwrap();
    output(selection(0, 2), highlights[..3].to_vec())
        .validate_request(geometry_request)
        .unwrap();
    for (maximum, count, expected_limit) in [(2, 3, 2), (usize::MAX, 4, 3)] {
        assert_eq!(
            output(selection(0, 2), highlights[..count].to_vec())
                .validate_request(geometry_request.with_max_rects(maximum)),
            Err(TextError::LimitExceeded {
                resource: "text geometry rectangles",
                actual: count,
                limit: expected_limit,
            }),
        );
    }
    let collapsed = request("ab", selection(0, 0), TextGeometryQuery::Current)
        .unwrap()
        .with_max_rects(0);
    output(selection(0, 0), Vec::new())
        .validate_request(collapsed)
        .unwrap();
}

#[test]
fn empty_and_newline_only_text_have_bounded_lines_even_without_glyphs() {
    for (text, lines) in [("", 1), ("\n\n", 3)] {
        let request = request(text, selection(0, 0), TextGeometryQuery::Current)
            .unwrap()
            .with_max_rects(0);
        for actual in [lines, lines + 1] {
            let height = actual as f32 * 10.0;
            let geometry = TextGeometry::new(
                fenestra_ui::TextMetrics::new(0.0, height, actual, 0, 0),
                selection(0, 0),
                caret(),
                caret(),
                Vec::new(),
                TextRect::new(0.0, 0.0, 1.0, f64::from(height)),
                None,
            )
            .unwrap();
            assert_eq!(
                geometry.validate_request(request),
                if actual == lines {
                    Ok(())
                } else {
                    Err(TextError::InvalidGeometry)
                },
                "{text:?}, {actual} lines",
            );
        }
    }
    assert_eq!(
        TextGeometry::new(
            fenestra_ui::TextMetrics::new(0.0, 0.0, 0, 0, 0),
            selection(0, 0),
            caret(),
            caret(),
            Vec::new(),
            extent(),
            None,
        )
        .err(),
        Some(TextError::InvalidGeometry),
    );
}

#[test]
fn request_validation_applies_glyph_budget_without_a_raster_pixel_budget() {
    let measure =
        TextMeasureRequest::new("ab", TextStyle::new(), None, TextLimits::new(2, 0, 1)).unwrap();
    let bounded =
        TextGeometryRequest::new(measure, selection(0, 0), TextGeometryQuery::Current).unwrap();
    assert_eq!(
        output(selection(0, 0), Vec::new()).validate_request(bounded),
        Err(TextError::LimitExceeded {
            resource: "text glyphs",
            actual: 2,
            limit: 1
        }),
    );
    output(selection(0, 0), Vec::new())
        .validate_request(request("ab", selection(0, 0), TextGeometryQuery::Current).unwrap())
        .unwrap();
}
