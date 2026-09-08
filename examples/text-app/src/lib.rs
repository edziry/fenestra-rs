use fenestra_ui::{Application, Color, Size, TextBuffer, TextStyle, View};
use fenestra_ui_text::TextRenderer;

#[cfg(all(feature = "native", any(target_os = "linux", target_os = "windows")))]
pub mod native;

pub const MAX_EDIT_BYTES: usize = 4096;
const FONT: &[u8] = include_bytes!("../../../assets/fonts/dejavu-sans-2.37/DejaVuSans.ttf");

pub fn panel() -> View {
    include!(concat!(env!("OUT_DIR"), "/panel.rs"))
}

pub fn macro_panel() -> View {
    include!("panel.ui")
}

pub fn application() -> Result<Application, Box<dyn std::error::Error>> {
    Ok(Application::with_text_engine(
        panel(),
        Size::new(640, 360),
        TextRenderer::new([FONT])?,
    )?)
}

pub fn update_headless(app: &mut Application) -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = TextBuffer::new(app.text("content")?, MAX_EDIT_BYTES)?;
    buffer.replace_selection("\nUpdated with TextBuffer: cafe\u{301}.")?;
    app.set_text("content", buffer.text())?;
    app.set_text_style(
        "content",
        TextStyle::new()
            .font_size(22)
            .line_height(30)
            .color(Color::rgba8(240, 208, 136, 255)),
    )?;
    app.set_text(
        "status",
        "Text and typography committed through the public API.",
    )?;
    Ok(())
}

pub fn checksum(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325_u64, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
    })
}
