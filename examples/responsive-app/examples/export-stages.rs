use std::path::Path;

use fenestra_responsive_app::{application, checksum, exercise, summary};
use fenestra_ui::{Application, Size};

#[path = "../src/export.rs"]
mod export;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut arguments = std::env::args_os().skip(1);
    let directory = arguments.next().ok_or("usage: export-stages DIRECTORY")?;
    if arguments.next().is_some() {
        return Err("usage: export-stages DIRECTORY".into());
    }
    let directory = Path::new(&directory);
    std::fs::create_dir_all(directory)?;
    let mut app = application()?;
    save("initial", directory, &app)?;
    app.resize(Size::new(520, 520))?;
    save("narrow", directory, &app)?;
    let mut updated = application()?;
    exercise(&mut updated)?;
    save("updated", directory, &updated)
}

fn save(
    stage: &str,
    directory: &Path,
    app: &Application,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = directory.join(format!("{stage}.ppm"));
    let raster = app.raster()?;
    export::write_ppm(&path, &raster)?;
    println!(
        "{} checksum={:016x} export={}",
        summary(stage, app)?,
        checksum(raster.bytes()),
        path.display(),
    );
    Ok(())
}
