use fenestra_ui::{Application, Element, Error, Size, Style, View};

#[test]
fn bounds_follow_committed_nested_layout_and_style_updates() {
    let view = View::new(
        "bounds",
        Element::column("root")
            .style(Style::new().width(200).height(160).padding(8).gap(4))
            .child(Element::rect("header").style(Style::new().height(24)))
            .child(
                Element::row("row")
                    .style(Style::new().width(100).height(30).padding(3).gap(5))
                    .child(Element::rect("first").style(Style::new().width(20).height(20)))
                    .child(Element::rect("second").style(Style::new().width(15).height(12))),
            ),
    );
    let mut app = Application::new(view, Size::new(200, 160)).unwrap();
    let bounds = app.bounds("second").unwrap();
    assert_eq!(
        (bounds.x(), bounds.y(), bounds.width(), bounds.height()),
        (36, 39, 15, 12)
    );
    app.set_size("first", 40, 20).unwrap();
    let bounds = app.bounds("second").unwrap();
    assert_eq!(
        (bounds.x(), bounds.y(), bounds.width(), bounds.height()),
        (56, 39, 15, 12)
    );
    let generation = app.generation();
    assert!(app.set_size("first", -1, 20).is_err());
    assert_eq!(app.bounds("second").unwrap(), bounds);
    assert_eq!(app.generation(), generation);
}

#[test]
fn bounds_remain_unclipped_and_unknown_names_are_errors() {
    let view = View::new(
        "overflow",
        Element::column("root").style(Style::new().width(80).height(90)),
    );
    let app = Application::new(view, Size::new(20, 20)).unwrap();
    let bounds = app.bounds("root").unwrap();
    assert_eq!(
        (bounds.x(), bounds.y(), bounds.width(), bounds.height()),
        (0, 0, 80, 90)
    );
    assert_eq!(
        app.bounds("absent"),
        Err(Error::UnknownNode {
            name: "absent".into()
        })
    );
}

#[test]
fn zero_size_bounds_preserve_their_nested_layout_origin() {
    let view = View::new(
        "empty_bounds",
        Element::column("root")
            .style(Style::new().width(80).height(80).padding(5).gap(7))
            .child(Element::rect("first").style(Style::new().width(10).height(11)))
            .child(Element::rect("empty").style(Style::new().width(0).height(0)))
            .child(Element::rect("last").style(Style::new().width(2).height(3))),
    );
    let mut app = Application::new(view, Size::new(80, 80)).unwrap();
    let bounds = app.bounds("empty").unwrap();
    assert_eq!(
        (bounds.x(), bounds.y(), bounds.width(), bounds.height()),
        (5, 23, 0, 0)
    );
    assert_eq!(app.bounds("last").unwrap().y(), 30);

    app.set_size("empty", 0, 9).unwrap();
    let bounds = app.bounds("empty").unwrap();
    assert_eq!(
        (bounds.x(), bounds.y(), bounds.width(), bounds.height()),
        (5, 23, 0, 9)
    );
    assert_eq!(app.bounds("last").unwrap().y(), 39);

    app.set_size("empty", 4, 0).unwrap();
    let bounds = app.bounds("empty").unwrap();
    assert_eq!(
        (bounds.x(), bounds.y(), bounds.width(), bounds.height()),
        (5, 23, 4, 0)
    );
    assert_eq!(app.bounds("last").unwrap().y(), 30);
}

#[test]
fn large_bounds_preserve_numeric_edges_and_reject_overflow_atomically() {
    for horizontal in [true, false] {
        let root = if horizontal {
            Element::row("root")
        } else {
            Element::column("root")
        };
        let large = if horizontal {
            Style::new().width(i32::MAX - 2).height(1)
        } else {
            Style::new().width(1).height(i32::MAX - 2)
        };
        let view = View::new(
            "large_bounds",
            root.style(Style::new().width(1).height(1))
                .child(Element::rect("first").style(large))
                .child(Element::rect("last").style(Style::new().width(2).height(2))),
        );
        let mut app = Application::new(view, Size::new(1, 1)).unwrap();
        let bounds = app.bounds("last").unwrap();
        let far = i64::from(i32::MAX) - 2;
        let expected = if horizontal { (far, 0) } else { (0, far) };
        assert_eq!((bounds.x(), bounds.y()), expected);
        assert_eq!((bounds.width(), bounds.height()), (2, 2));
        let generation = app.generation();
        let overflow = if horizontal {
            app.set_size("first", i32::MAX, 1)
        } else {
            app.set_size("first", 1, i32::MAX)
        };
        assert!(matches!(overflow, Err(Error::Runtime(_))));
        assert_eq!(app.bounds("last").unwrap(), bounds);
        assert_eq!(app.generation(), generation);
    }
}
