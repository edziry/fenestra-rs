#![forbid(unsafe_code)]

//! Interactive native entry point for the layout inspector.

use std::process::ExitCode;

use fenestra_layout_inspector::native::{NativeInspectorError, run_native};

fn main() -> ExitCode {
    match run_native() {
        Ok(()) => ExitCode::SUCCESS,
        Err(NativeInspectorError::Application(error)) => {
            eprintln!("fenestra-layout-inspector-native-error={error:?}");
            ExitCode::FAILURE
        }
        Err(error) => {
            eprintln!("fenestra-layout-inspector-native-error={error:?}");
            ExitCode::FAILURE
        }
    }
}
