mod raster;
mod surface;
mod trace;
mod trace_applicability;
mod trace_capacity;

use fenestra_ui_runtime::prototype::{RuntimeGeneration, UiRuntime};
use fenestra_ui_testkit::prototype::HeadlessFixtureV1;

pub(super) fn generation_zero() -> RuntimeGeneration {
    let fixture = HeadlessFixtureV1::build().expect("registered fixture should remain valid");
    UiRuntime::new_headless(
        fixture.style().clone(),
        fixture.spec(),
        fixture.surface(),
        fixture.runtime_capacity(),
    )
    .expect("registered runtime should initialize")
    .committed()
    .generation()
}
