use std::io::{self, Write};
use std::path::Path;

use fenestra_ui::Raster;

pub(super) fn write_ppm(path: &Path, raster: &Raster) -> io::Result<()> {
    let mut file = io::BufWriter::new(std::fs::File::create(path)?);
    write!(
        file,
        "P6\n{} {}\n255\n",
        raster.size().width(),
        raster.size().height(),
    )?;
    for pixel in raster.bytes().chunks_exact(4) {
        file.write_all(&pixel[..3])?;
    }
    file.flush()
}
