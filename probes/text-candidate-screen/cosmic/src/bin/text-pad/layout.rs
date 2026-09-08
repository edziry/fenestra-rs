use fenestra_ui::{Application, Color, Element, Error, Limits, Size, Style, View};

pub(super) fn application(size: Size) -> Result<Application, Error> {
    // A small window clips this minimum layout instead of overlapping panes.
    let width = i32::try_from(size.width())
        .map_err(|_| Error::CapacityOverflow)?
        .max(320);
    let height = i32::try_from(size.height())
        .map_err(|_| Error::CapacityOverflow)?
        .max(300);
    let content_width = (width - 48).min(1100);
    let editor_height = (height - 190).min(1200);
    let view = View::new(
        "text_pad",
        Element::column("root")
            .style(
                Style::new()
                    .width(width)
                    .height(height)
                    .padding(24)
                    .gap(8)
                    .background(Color::rgba8(16, 23, 32, 255)),
            )
            .child(
                Element::row("heading")
                    .style(Style::new().width(content_width).height(42).gap(12))
                    .child(
                        Element::rect("accent").style(
                            Style::new()
                                .width(4)
                                .height(32)
                                .background(Color::rgba8(96, 216, 184, 255)),
                        ),
                    )
                    .child(
                        Element::rect("title")
                            .style(Style::new().width(content_width - 16).height(42)),
                    ),
            )
            .child(Element::rect("hint").style(Style::new().width(content_width).height(48)))
            .child(
                Element::column("editor_frame")
                    .style(
                        Style::new()
                            .width(content_width)
                            .height(editor_height)
                            .padding(12)
                            .background(Color::rgba8(28, 39, 52, 255))
                            .input(true),
                    )
                    .child(
                        Element::rect("editor").style(
                            Style::new()
                                .width(content_width - 24)
                                .height(editor_height - 24)
                                .input(true),
                        ),
                    ),
            )
            .child(Element::rect("status").style(Style::new().width(content_width).height(28))),
    );
    Application::with_limits(view, size, Limits::new(16, 5, 16_777_216))
}
