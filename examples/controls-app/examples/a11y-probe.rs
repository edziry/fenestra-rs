//! Native AT-SPI/UIA probe using the public application and hosting APIs.

use std::error::Error;
use std::io;
use std::path::PathBuf;

use fenestra_controls_app::{DemoState, checksum, preferences};
use fenestra_ui::native::{self, WindowContent, WindowEvent, WindowOptions};
use fenestra_ui::{
    AccessibilityActionRequest, AccessibilityTree, Application, Color, ControlRole, Dimension,
    Element, Event, Raster, Size, Style, View,
};
use fenestra_ui_text::TextRenderer;
use serde_json::json;

#[path = "../src/export.rs"]
mod export;

type ProbeError = Box<dyn Error + Send + Sync>;

struct Probe {
    app: Application,
    state: DemoState,
    output: PathBuf,
    presentations: u64,
    finish: bool,
    finished_presented: bool,
}

impl Probe {
    fn events(&mut self, events: Vec<Event>) -> Result<(), ProbeError> {
        for event in events {
            if matches!(&event, Event::Activated { target } if target == "finish") {
                self.finish = true;
            }
            self.state.event(&mut self.app, &event)?;
        }
        Ok(())
    }

    fn record(&self) -> Result<(), ProbeError> {
        let raster = self.app.raster()?;
        let frame = format!("frame-{:04}.ppm", self.presentations);
        export::write_ppm(&self.output.join(&frame), &raster)?;
        let controls = self.app.control_snapshots()?.into_iter().map(|node| {
            let bounds = node.bounds();
            let state = node.state();
            json!({
                "name": node.name(), "label": node.label(),
                "role": match node.role() { ControlRole::Button => "button", ControlRole::Checkbox => "checkbox" },
                "bounds": [bounds.x(), bounds.y(), i64::from(bounds.width()), i64::from(bounds.height())],
                "disabled": state.disabled(), "checked": state.checked(), "focused": state.focused(),
            })
        }).collect::<Vec<_>>();
        let record = json!({
            "pid": std::process::id(), "generation": self.app.generation(),
            "presentations": self.presentations, "apply_count": self.state.apply_count(),
            "rgba_checksum": format!("{:016x}", checksum(raster.bytes())),
            "frame": frame, "controls": controls, "finished": self.finish,
            "viewport": [raster.size().width(), raster.size().height()],
            "readout": self.app.text("readout")?,
        });
        let temporary = self.output.join("state.pending.json");
        std::fs::write(&temporary, serde_json::to_vec_pretty(&record)?)?;
        std::fs::rename(temporary, self.output.join("state.json"))?;
        Ok(())
    }
}

impl WindowContent for Probe {
    type Error = io::Error;

    fn resize(&mut self, width: u32, height: u32) -> io::Result<()> {
        let size = Size::new(width, height);
        self.app.resize(size).map_err(io::Error::other)?;
        self.state
            .event(&mut self.app, &Event::Resized { size })
            .map_err(io::Error::other)
    }

    fn event(&mut self, input: WindowEvent) -> io::Result<()> {
        let events = self.app.dispatch_input(input).map_err(io::Error::other)?;
        self.events(events).map_err(io::Error::other)
    }

    fn frame(&self) -> io::Result<Raster> {
        self.app.raster().map_err(io::Error::other)
    }

    fn accessibility(&self) -> io::Result<Option<AccessibilityTree>> {
        self.app
            .accessibility_tree()
            .map(Some)
            .map_err(io::Error::other)
    }

    fn accessibility_action(&mut self, request: AccessibilityActionRequest) -> io::Result<()> {
        let events = self
            .app
            .dispatch_accessibility_action(request)
            .map_err(io::Error::other)?;
        self.events(events).map_err(io::Error::other)
    }

    fn presented(&mut self) -> io::Result<()> {
        self.presentations += 1;
        self.record().map_err(io::Error::other)?;
        self.finished_presented = self.finish;
        Ok(())
    }

    fn should_close(&self) -> bool {
        self.finished_presented
    }
}

fn application() -> Result<Application, ProbeError> {
    let view = preferences();
    let finish = Element::button("finish", "Finish accessibility verification")
        .style(
            Style::new()
                .width_mode(Dimension::Fill(1))
                .height(40)
                .padding(8)
                .background(Color::rgba8(48, 128, 192, 255)),
        )
        .child(
            Element::text("finish_caption", "Finish verification").style(
                Style::new()
                    .width_mode(Dimension::Fill(1))
                    .height_mode(Dimension::Auto),
            ),
        );
    let view = View::new(view.name(), view.root().clone().child(finish));
    let font = include_bytes!("../../../assets/fonts/dejavu-sans-2.37/DejaVuSans.ttf");
    Ok(Application::with_text_engine(
        view,
        Size::new(640, 520),
        TextRenderer::new([font.as_slice()])?,
    )?)
}

fn main() -> Result<(), ProbeError> {
    let output = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .ok_or("usage: a11y-probe OUTPUT_DIRECTORY")?;
    std::fs::create_dir_all(&output)?;
    let mut probe = Probe {
        app: application()?,
        state: DemoState::default(),
        output,
        presentations: 0,
        finish: false,
        finished_presented: false,
    };
    native::run(
        &mut probe,
        WindowOptions::new("Fenestra accessibility verification").size(640, 520),
    )?;
    if !probe.finished_presented {
        return Err("probe closed before its final action was presented".into());
    }
    println!(
        "native accessibility probe completed after {} presentations",
        probe.presentations
    );
    Ok(())
}
