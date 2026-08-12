pub mod dynamic;
pub mod engine;
pub mod facts;
pub mod input;
pub mod none;
pub mod program;

use fenestra_ui_ir::prototype::{PropertyValue, ValidatedStyleProgram};
use fenestra_ui_spatial::prototype::{SpatialLimitsV2, SpatialViewportV2};

use crate::support::headless::{CONTROL, HEIGHT, ITEM, WIDTH, exact_style_with};

pub const VIEWPORT: SpatialViewportV2 = SpatialViewportV2::new(90, 70);

pub fn limits() -> SpatialLimitsV2 {
    SpatialLimitsV2::new([usize::MAX; 30])
}

pub fn nodes_limit(maximum: usize) -> SpatialLimitsV2 {
    let mut values = [usize::MAX; 30];
    values[0] = maximum;
    SpatialLimitsV2::new(values)
}

pub fn styled_program() -> ValidatedStyleProgram {
    exact_style_with(vec![
        (CONTROL, WIDTH, PropertyValue::ScalarI32(37)),
        (ITEM, HEIGHT, PropertyValue::ScalarI32(17)),
    ])
}
