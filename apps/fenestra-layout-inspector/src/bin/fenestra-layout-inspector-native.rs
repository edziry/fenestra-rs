#![forbid(unsafe_code)]

//! Interactive native entry point for the layout inspector.

use std::process::ExitCode;

use fenestra_layout_inspector::native::{NativeInspectorError, run_native, run_native_smoke};

fn main() -> ExitCode {
    let result = if std::env::args_os().any(|argument| argument == "--smoke") {
        run_native_smoke()
    } else {
        run_native()
    };
    match result {
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
