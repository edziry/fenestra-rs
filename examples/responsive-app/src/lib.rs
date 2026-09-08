use fenestra_ui::{Application, Color, Error, Size, View};
use fenestra_ui_text::TextRenderer;

const FONT: &[u8] = include_bytes!("../../../assets/fonts/dejavu-sans-2.37/DejaVuSans.ttf");
const SHORT_NOTES: &str = "Notes updated. Auto height moves the cards with the text.";

pub fn workspace() -> View {
    include!(concat!(env!("OUT_DIR"), "/workspace.rs"))
}

pub fn macro_workspace() -> View {
    include!("workspace.ui")
}

pub fn application() -> Result<Application, Box<dyn std::error::Error>> {
    Ok(Application::with_text_engine(
        workspace(),
        Size::new(820, 520),
        TextRenderer::new([FONT])?,
    )?)
}

pub fn exercise(app: &mut Application) -> Result<Vec<String>, Error> {
    let mut trace = vec![summary("initial", app)?];
    app.resize(Size::new(520, 520))?;
    trace.push(summary("narrow", app)?);
    let mut state = DemoState::default();
    state.activate(app, "highlight")?;
    state.activate(app, "revise")?;
    app.set_text_style(
        "paragraph",
        app.text_style("paragraph")?.font_size(18).line_height(26),
    )?;
    app.resize(Size::new(960, 520))?;
    trace.push(summary("updated", app)?);
    Ok(trace)
}

pub fn summary(stage: &str, app: &Application) -> Result<String, Error> {
    let main = app.bounds("main")?;
    let paragraph = app.bounds("paragraph")?;
    Ok(format!(
        "{stage} viewport={}x{} sidebar={} main={}x{} paragraph={}x{} lines={} cards_y={}",
        app.size().width(),
        app.size().height(),
        app.bounds("sidebar")?.width(),
        main.width(),
        main.height(),
        paragraph.width(),
        paragraph.height(),
        app.text_metrics("paragraph")?.lines(),
        app.bounds("actions")?.y(),
    ))
}

pub fn checksum(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325_u64, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
    })
}

#[derive(Default)]
pub struct DemoState {
    highlighted: bool,
    concise: bool,
    original_notes: Option<String>,
}

impl DemoState {
    pub fn activate(&mut self, app: &mut Application, name: &str) -> Result<(), Error> {
        let message = match name {
            "highlight" => {
                let color = if self.highlighted {
                    Color::rgba8(48, 128, 192, 255)
                } else {
                    Color::rgba8(208, 144, 48, 255)
                };
                app.set_background("highlight", color)?;
                self.highlighted = !self.highlighted;
                if self.highlighted {
                    "Highlight enabled."
                } else {
                    "Highlight cleared."
                }
            }
            "revise" => {
                let original = self
                    .original_notes
                    .get_or_insert(app.text("paragraph")?.to_owned());
                app.set_text(
                    "paragraph",
                    if self.concise {
                        original.as_str()
                    } else {
                        SHORT_NOTES
                    },
                )?;
                self.concise = !self.concise;
                if self.concise {
                    "Notes shortened."
                } else {
                    "Full notes restored."
                }
            }
            _ => return Ok(()),
        };
        app.set_text("readout", message)
    }
}
