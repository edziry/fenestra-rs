use std::path::Path;

use fenestra_controls_app::{DemoState, application, checksum, exercise, key, summary};
use fenestra_ui::{Application, Key, KeyState};

#[path = "../src/export.rs"]
mod export;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut arguments = std::env::args_os().skip(1);
    let directory = arguments.next().ok_or("usage: export-states DIRECTORY")?;
    if arguments.next().is_some() {
        return Err("usage: export-states DIRECTORY".into());
    }
    let directory = Path::new(&directory);
    std::fs::create_dir_all(directory)?;
    let mut app = application()?;
    let mut state = DemoState::default();
    save("initial", directory, &app, &state)?;
    app.focus(Some("compact"))?;
    state.input(&mut app, key(Key::Space, KeyState::Pressed))?;
    state.input(&mut app, key(Key::Space, KeyState::Released))?;
    save("changed", directory, &app, &state)?;
    let mut final_app = application()?;
    let mut final_state = DemoState::default();
    exercise(&mut final_app, &mut final_state)?;
    save("reset", directory, &final_app, &final_state)
}

fn save(
    stage: &str,
    directory: &Path,
    app: &Application,
    state: &DemoState,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = directory.join(format!("{stage}.ppm"));
    let raster = app.raster()?;
    export::write_ppm(&path, &raster)?;
    println!(
        "{} checksum={:016x} export={}",
        summary(stage, app, state)?,
        checksum(raster.bytes()),
        path.display()
    );
    Ok(())
}
