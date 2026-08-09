#![forbid(unsafe_code)]

//! Disposable entry point for the EXP-0001 headless runtime spine.

use std::io::{self, Write};
use std::process::ExitCode;

use fenestra_ui_exp_0001_spine::run_headless_probe_v1;

fn main() -> ExitCode {
    let Ok(bytes) = run_headless_probe_v1() else {
        return ExitCode::FAILURE;
    };
    let stdout = io::stdout();
    let mut output = stdout.lock();
    if output
        .write_all(&bytes)
        .and_then(|()| output.flush())
        .is_err()
    {
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
