use crate::Raster;

pub(super) fn copy_pixels(raster: &Raster, destination: &mut [u32]) -> Result<(), ()> {
    let pixels = raster.bytes().chunks_exact(4);
    if destination.len() != pixels.len() || !pixels.remainder().is_empty() {
        return Err(());
    }
    for (destination, source) in destination.iter_mut().zip(pixels) {
        *destination =
            u32::from(source[0]) << 16 | u32::from(source[1]) << 8 | u32::from(source[2]);
    }
    Ok(())
}
