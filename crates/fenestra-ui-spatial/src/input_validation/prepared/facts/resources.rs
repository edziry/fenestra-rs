mod brushes_images;
mod items;
mod paths_shapes;

fn ordinal(index: usize) -> u32 {
    u32::try_from(index).expect("prepared table ordinal fits u32")
}
