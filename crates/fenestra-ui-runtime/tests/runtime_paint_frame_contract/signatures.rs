use fenestra_ui_runtime::prototype::{FrameWork, RuntimeGeneration};
use fenestra_ui_spatial::prototype::SpatialPaintFrameV2;

use crate::RuntimePaintFrameV2;

#[test]
fn runtime_paint_frame_signatures_are_exact() {
    assert_signatures(&());
}

fn assert_signatures<'a>(_: &'a ()) {
    let _: for<'work> fn(&'work FrameWork) -> Option<RuntimePaintFrameV2<'work>> =
        FrameWork::paint_frame;
    let _: fn(RuntimePaintFrameV2<'a>) -> RuntimeGeneration = RuntimePaintFrameV2::generation;
    let _: fn(RuntimePaintFrameV2<'a>) -> SpatialPaintFrameV2<'a> = RuntimePaintFrameV2::spatial;
}
