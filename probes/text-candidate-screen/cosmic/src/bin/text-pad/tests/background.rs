use std::time::Instant;

use fenestra_ui::native::{Key, Modifiers, WindowContent};

use super::{key, pad};

#[test]
fn typing_reuses_background_until_a_successful_resize() {
    let mut pad = pad("Editable ink", 100);
    assert_eq!(pad.background_render_count.get(), 0);
    let first = pad.frame().unwrap();
    assert_eq!(pad.background_render_count.get(), 1);
    let editor = pad.app.bounds("editor").unwrap();
    let status = pad.app.bounds("status").unwrap();
    let inside = |index: usize, bounds: fenestra_ui::Bounds| {
        let (x, y) = ((index % 720) as i64, (index / 720) as i64);
        x >= bounds.x()
            && x < bounds.x() + i64::from(bounds.width())
            && y >= bounds.y()
            && y < bounds.y() + i64::from(bounds.height())
    };
    let mut previous = first.clone();
    let started = Instant::now();
    for text in ["x", "y", "z"] {
        key(
            &mut pad,
            Key::Character(text.into()),
            Some(text),
            Modifiers::default(),
            false,
        );
        let next = pad.frame().unwrap();
        assert_eq!(pad.app.bounds("editor").unwrap(), editor);
        assert!(
            previous
                .bytes()
                .chunks_exact(4)
                .zip(next.bytes().chunks_exact(4))
                .enumerate()
                .any(|(i, (a, b))| inside(i, editor) && a != b)
        );
        for (i, (initial, changed)) in first
            .bytes()
            .chunks_exact(4)
            .zip(next.bytes().chunks_exact(4))
            .enumerate()
        {
            if !inside(i, editor) && !inside(i, status) {
                assert_eq!(initial, changed);
            }
        }
        previous = next;
    }
    eprintln!("three typing frames: {:?}", started.elapsed());
    assert_eq!(pad.background_render_count.get(), 1);

    pad.resize(520, 360).unwrap();
    assert_eq!(pad.background_render_count.get(), 1);
    let resized = pad.frame().unwrap();
    assert_eq!(pad.background_render_count.get(), 2);
    assert!(pad.resize(0, 0).is_err());
    assert_eq!(pad.frame().unwrap(), resized);
    assert_eq!(pad.background_render_count.get(), 2);
}
