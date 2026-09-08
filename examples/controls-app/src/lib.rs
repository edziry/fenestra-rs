use fenestra_ui::{Application, Size, View};
use fenestra_ui_text::TextRenderer;

mod exercise;
mod state;

pub use exercise::{exercise, key, summary};
pub use state::{DemoState, Settings};

const FONT: &[u8] = include_bytes!("../../../assets/fonts/dejavu-sans-2.37/DejaVuSans.ttf");

pub fn preferences() -> View {
    include!(concat!(env!("OUT_DIR"), "/preferences.rs"))
}

pub fn macro_preferences() -> View {
    include!("preferences.ui")
}

pub fn application() -> Result<Application, Box<dyn std::error::Error>> {
    Ok(Application::with_text_engine(
        preferences(),
        Size::new(640, 520),
        TextRenderer::new([FONT])?,
    )?)
}

pub fn checksum(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325_u64, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
    })
}
