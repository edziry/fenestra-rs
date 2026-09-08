//! Observational raster timings with byte-level checks and no timing assertions.
//!
//! Run with `cargo run --example raster-cost` and repeat with `--release`.

use std::error::Error;
use std::time::Instant;

use fenestra_controls_app::{application, checksum};
use serde_json::json;

fn main() -> Result<(), Box<dyn Error>> {
    let mut app = application()?;
    let initial_generation = app.generation();
    let mut original = None;
    let mut original_bytes = None;
    let mut changed = None;
    let mut samples = Vec::new();

    for iteration in 0..7 {
        if iteration == 5 {
            app.set_checked("compact", true)?;
            assert!(app.generation() > initial_generation);
        }
        let started = Instant::now();
        let raster = app.raster()?;
        let elapsed = started.elapsed();
        // Verification, checksums, retained frames and JSON are outside the timer.
        let checksum = format!("{:016x}", checksum(raster.bytes()));
        let old = original.get_or_insert_with(|| raster.clone());
        original_bytes.get_or_insert_with(|| old.bytes().to_vec());
        if iteration < 5 {
            assert_eq!(app.generation(), initial_generation);
            assert_eq!(raster.bytes(), old.bytes());
        } else {
            assert_ne!(raster.bytes(), old.bytes());
            let first_changed = changed.get_or_insert_with(|| raster.clone());
            assert_eq!(raster.bytes(), first_changed.bytes());
        }
        samples.push(json!({
            "phase": match iteration {
                0 => "cold",
                1..=4 => "warm",
                5 => "after_visual_change",
                _ => "warm_after_visual_change",
            },
            "iteration": iteration,
            "generation": app.generation(),
            "width": raster.size().width(),
            "height": raster.size().height(),
            "rgba_bytes": raster.bytes().len(),
            "rgba_checksum": checksum,
            "raster_microseconds": elapsed.as_secs_f64() * 1_000_000.0,
        }));
    }

    let original = original.expect("the cold frame is retained");
    assert_eq!(original.bytes(), original_bytes.unwrap());
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "schema": "fenestra-raster-cost-v1",
            "debug_assertions": cfg!(debug_assertions),
            "unchanged_calls": 5,
            "warm_repeats": 4,
            "mutation": "set_checked(compact, true)",
            "old_frame_unchanged": true,
            "samples": samples,
        }))?,
    );
    Ok(())
}
