use fenestra_ui::{Application, Dimension, Element, Size, Style, View};

#[test]
fn fill_policy_is_retained_even_when_it_initially_resolves_to_the_same_size() {
    let mut app = Application::new(
        View::new("responsive", Element::rect("root")),
        Size::new(64, 64),
    )
    .unwrap();
    let policy = app.style("root").unwrap().width_mode(Dimension::Fill(1));
    app.set_style("root", policy).unwrap();
    assert_eq!(app.bounds("root").unwrap().width(), 64);
    assert_eq!(app.style("root").unwrap(), policy);
    assert_eq!(app.generation(), 1);
    app.set_style("root", policy).unwrap();
    assert_eq!(app.generation(), 1);
    app.resize(Size::new(100, 64)).unwrap();
    assert_eq!(app.bounds("root").unwrap().width(), 100);
    assert_eq!(app.generation(), 2);
}

#[test]
fn resizing_many_fill_siblings_publishes_more_than_sixteen_properties_atomically() {
    let mut root =
        Element::row("root").style(Style::new().width_mode(Dimension::Fill(1)).height(1));
    for index in 0..20 {
        root = root.child(
            Element::rect(format!("item_{index}")).style(
                Style::new()
                    .width_mode(Dimension::Fill(1))
                    .height(1)
                    .input(true),
            ),
        );
    }
    let mut app = Application::new(View::new("many", root), Size::new(300, 1)).unwrap();
    assert_eq!(app.bounds("item_19").unwrap().width(), 15);
    app.resize(Size::new(600, 1)).unwrap();
    assert_eq!(app.generation(), 1);
    for index in 0..20 {
        let name = format!("item_{index}");
        let bounds = app.bounds(&name).unwrap();
        assert_eq!((bounds.x(), bounds.width()), (index * 30, 30));
        assert_eq!(app.hit_test(index as i32 * 30, 0), Some(name.as_str()));
    }
}

#[test]
fn intrinsic_containers_and_limits_preserve_authored_policy() {
    let policy = Style::new()
        .width_mode(Dimension::Auto)
        .height_mode(Dimension::Auto)
        .padding(2)
        .gap(3)
        .min_width(25)
        .max_width(40);
    let root = Element::row("root")
        .style(policy)
        .child(Element::rect("a").style(Style::new().width(10).height(4)))
        .child(Element::rect("b").style(Style::new().width(12).height(8)));
    let app = Application::new(View::new("auto", root), Size::new(80, 40)).unwrap();
    let bounds = app.bounds("root").unwrap();
    assert_eq!((bounds.width(), bounds.height()), (29, 12));
    assert_eq!(app.style("root").unwrap(), policy);
}
