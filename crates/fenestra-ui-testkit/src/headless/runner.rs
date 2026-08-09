mod lifecycle;
mod record;
mod script;
mod state;
mod types;

pub use types::{HeadlessResultV1, HeadlessRunErrorV1, HeadlessRunV1};

/// Executes the registered deterministic headless feasibility spine.
pub fn run_headless_spine_v1() -> Result<HeadlessRunV1, HeadlessRunErrorV1> {
    script::run()
}
