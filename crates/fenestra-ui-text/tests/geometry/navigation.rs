use crate::*;

#[test]
fn soft_wrap_keeps_two_visual_affinities_at_the_same_source_byte() {
    let mut renderer = TextRenderer::new([FONT]).unwrap();
    let text = "abcd efgh ijkl";
    let mut boundary = None;
    for byte in 1..text.len() {
        let upstream = TextSelection::caret(TextPosition::new(byte, TextAffinity::Upstream));
        let downstream = TextSelection::caret(TextPosition::new(byte, TextAffinity::Downstream));
        let before = renderer
            .geometry(query(text, Some(50), upstream, TextGeometryQuery::Current))
            .unwrap();
        let after = renderer
            .geometry(query(
                text,
                Some(50),
                downstream,
                TextGeometryQuery::Current,
            ))
            .unwrap();
        if before.focus_caret().y0() != after.focus_caret().y0() {
            assert_eq!(after.focus_caret().y0() - before.focus_caret().y0(), 28.0);
            assert!(before.focus_caret().x0() > after.focus_caret().x0());
            boundary = Some((upstream, downstream));
            break;
        }
    }
    let (before, after) = boundary.expect("wrapped text must have a shared byte boundary");
    let right = renderer
        .geometry(query(
            text,
            Some(50),
            before,
            TextGeometryQuery::Right { extend: false },
        ))
        .unwrap();
    let left = renderer
        .geometry(query(
            text,
            Some(50),
            after,
            TextGeometryQuery::Left { extend: false },
        ))
        .unwrap();
    assert_eq!(right.selection(), after);
    assert_eq!(left.selection(), before);
}

#[test]
fn vertical_navigation_retains_real_horizontal_position_across_short_lines() {
    let mut renderer = TextRenderer::new([FONT]).unwrap();
    let text = "iiiiWWWW\nx\niiiiWWWW";
    let start = renderer.geometry(request(text, None, 7)).unwrap();
    let x = start.focus_caret().x0();
    let first = renderer
        .geometry(query(
            text,
            None,
            start.selection(),
            TextGeometryQuery::Down {
                extend: false,
                preferred_x: None,
            },
        ))
        .unwrap();
    assert_eq!(first.selection().focus().byte(), 10);
    assert_eq!(first.preferred_x(), Some(x));
    let second = renderer
        .geometry(query(
            text,
            None,
            first.selection(),
            TextGeometryQuery::Down {
                extend: false,
                preferred_x: first.preferred_x(),
            },
        ))
        .unwrap();
    assert_eq!(second.selection().focus().byte(), 18);
    assert_eq!(second.focus_caret().x0(), x);
    let third = renderer
        .geometry(query(
            text,
            None,
            second.selection(),
            TextGeometryQuery::Up {
                extend: true,
                preferred_x: second.preferred_x(),
            },
        ))
        .unwrap();
    assert_eq!(third.selection().anchor().byte(), 18);
    assert_eq!(third.selection().focus().byte(), 10);
    assert_eq!(third.preferred_x(), Some(x));
}

#[test]
fn line_navigation_uses_raw_crlf_offsets_and_retains_the_anchor_when_extended() {
    let mut renderer = TextRenderer::new([FONT]).unwrap();
    let text = "abc\r\ndef";
    let selection = TextSelection::new(
        TextPosition::new(1, TextAffinity::Downstream),
        TextPosition::new(7, TextAffinity::Downstream),
    );
    let home = renderer
        .geometry(query(
            text,
            None,
            selection,
            TextGeometryQuery::LineStart { extend: true },
        ))
        .unwrap();
    assert_eq!(
        home.selection().byte_selection(),
        fenestra_ui::Selection::new(1, 5)
    );
    let end = renderer
        .geometry(query(
            text,
            None,
            selection,
            TextGeometryQuery::LineEnd { extend: false },
        ))
        .unwrap();
    assert_eq!(end.selection().focus().byte(), 8);
    let end = renderer
        .geometry(query(
            text,
            None,
            request(text, None, 1).selection(),
            TextGeometryQuery::LineEnd { extend: false },
        ))
        .unwrap();
    assert_eq!(end.selection().focus().byte(), 3);
}

#[test]
fn visual_rtl_navigation_moves_opposite_to_utf8_order_and_terminates() {
    let mut renderer = TextRenderer::new([FONT]).unwrap();
    let text = "\u{05d0}\u{05d1}\u{05d2}";
    let mut current = renderer.geometry(request(text, None, 0)).unwrap();
    for byte in [2, 4, 6] {
        let next = renderer
            .geometry(query(
                text,
                None,
                current.selection(),
                TextGeometryQuery::Left { extend: false },
            ))
            .unwrap();
        assert_eq!(next.selection().focus().byte(), byte);
        assert!(next.focus_caret().x0() < current.focus_caret().x0());
        current = next;
    }
    let end = renderer
        .geometry(query(
            text,
            None,
            current.selection(),
            TextGeometryQuery::Left { extend: false },
        ))
        .unwrap();
    assert_eq!(end.selection(), current.selection());
    for byte in [4, 2, 0] {
        let next = renderer
            .geometry(query(
                text,
                None,
                current.selection(),
                TextGeometryQuery::Right { extend: false },
            ))
            .unwrap();
        assert_eq!(next.selection().focus().byte(), byte);
        assert!(next.focus_caret().x0() > current.focus_caret().x0());
        current = next;
    }
}

#[test]
fn long_combining_graphemes_move_as_one_boundary_in_both_directions() {
    let mut renderer = TextRenderer::new([FONT]).unwrap();
    let mut text = String::from("e");
    text.push_str(&"\u{301}".repeat(200));
    text.push('x');
    let forward = renderer
        .geometry(query(
            &text,
            None,
            request(&text, None, 0).selection(),
            TextGeometryQuery::Right { extend: false },
        ))
        .unwrap();
    assert_eq!(forward.selection().focus().byte(), text.len() - 1);
    let backward = renderer
        .geometry(query(
            &text,
            None,
            forward.selection(),
            TextGeometryQuery::Left { extend: false },
        ))
        .unwrap();
    assert_eq!(backward.selection().focus().byte(), 0);
}
