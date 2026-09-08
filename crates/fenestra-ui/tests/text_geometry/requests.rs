use fenestra_ui::{Selection, TextPoint, TextRequest, TextViewportRequest};

use crate::*;

#[test]
fn request_endpoints_use_the_same_grapheme_boundaries_as_text_buffer() {
    for (text, byte) in [
        ("e\u{301}", 1),
        ("e\u{301}", 2),
        ("\r\n", 1),
        ("a\u{e9}", 2),
        ("", 1),
        ("a", usize::MAX),
    ] {
        for selected in [selection(byte, 0), selection(0, byte)] {
            assert_eq!(
                request(text, selected, TextGeometryQuery::Current).err(),
                Some(TextError::InvalidTextPosition { byte }),
                "{text:?}, {selected:?}",
            );
        }
    }
    for text in ["", "e\u{301}", "\r\n", "a\u{e9}"] {
        for affinity in [TextAffinity::Upstream, TextAffinity::Downstream] {
            let selected = TextSelection::new(
                TextPosition::new(text.len(), affinity),
                TextPosition::new(0, affinity),
            );
            request(text, selected, TextGeometryQuery::Current).unwrap();
            assert_eq!(selected.byte_selection(), Selection::new(text.len(), 0));
        }
    }
    for (text, expected) in [
        ("", vec![0]),
        ("a\u{e9}", vec![0, 1, 3]),
        ("e\u{301}\r\n", vec![0, 3, 5]),
        ("\u{1f469}\u{200d}\u{1f4bb}", vec![0, 11]),
    ] {
        let geometry = request(text, selection(0, 0), TextGeometryQuery::Current).unwrap();
        assert_eq!(geometry.grapheme_boundaries().collect::<Vec<_>>(), expected);
        let mut buffer = fenestra_ui::TextBuffer::new(text, text.len()).unwrap();
        for byte in 0..=text.len() {
            let from_geometry = request(text, selection(byte, byte), TextGeometryQuery::Current);
            let from_buffer = buffer.set_selection(Selection::caret(byte));
            assert_eq!(
                from_geometry.is_ok(),
                from_buffer.is_ok(),
                "{text:?}, {byte}"
            );
        }
    }
}

#[test]
fn query_coordinates_reject_nonfinite_and_out_of_domain_values() {
    let maximum = f64::from(i32::MAX);
    for invalid in [
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        maximum + 1.0,
        -maximum - 1.0,
    ] {
        for query in [
            TextGeometryQuery::Hit {
                point: TextPoint::new(invalid, 0.0),
                extend: false,
            },
            TextGeometryQuery::Hit {
                point: TextPoint::new(0.0, invalid),
                extend: true,
            },
            TextGeometryQuery::Up {
                extend: false,
                preferred_x: Some(invalid),
            },
            TextGeometryQuery::Down {
                extend: true,
                preferred_x: Some(invalid),
            },
        ] {
            assert_eq!(
                request("a", selection(0, 0), query).err(),
                Some(TextError::InvalidGeometry),
                "{query:?}",
            );
        }
    }
    for coordinate in [-maximum, -1.0, 0.0, maximum] {
        request(
            "a",
            selection(0, 0),
            TextGeometryQuery::Hit {
                point: TextPoint::new(coordinate, coordinate),
                extend: false,
            },
        )
        .unwrap();
    }
}

#[test]
fn viewport_coordinates_validate_offset_plus_size_without_changing_wrap_policy() {
    let raster = TextRequest::new(
        "a",
        TextStyle::new(),
        fenestra_ui::Size::new(2, 3),
        TextLimits::new(1, 6, 1),
    )
    .unwrap();
    let maximum = i32::MAX as u32;
    for (x, y) in [
        (maximum - 1, 0),
        (0, maximum - 2),
        (u32::MAX, 0),
        (0, u32::MAX),
    ] {
        assert_eq!(
            TextViewportRequest::new(raster, None, x, y).err(),
            Some(TextError::InvalidViewportOffset),
            "({x}, {y})",
        );
    }
    TextViewportRequest::new(raster, None, maximum - 2, maximum - 3).unwrap();
    for width in [None, Some(0), Some(1), Some(maximum)] {
        TextViewportRequest::new(raster, width, 0, 0).unwrap();
    }
    assert_eq!(
        TextViewportRequest::new(raster, Some(u32::MAX), 0, 0).err(),
        Some(TextError::InvalidMeasurementWidth { width: u32::MAX }),
    );
}
