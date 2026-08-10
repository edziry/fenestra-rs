use std::error::Error;

use fenestra_ui_exp_0001_native_spine::{NativeProbeErrorV1, run_native_probe_v1};

#[test]
fn native_probe_api_returns_owned_bytes_and_closed_errors() {
    let _: fn() -> Result<Vec<u8>, NativeProbeErrorV1> = run_native_probe_v1;
    assert_error::<NativeProbeErrorV1>();
    assert_eq!(
        NativeProbeErrorV1::ALL,
        [
            NativeProbeErrorV1::EventLoop,
            NativeProbeErrorV1::Window,
            NativeProbeErrorV1::Presenter,
            NativeProbeErrorV1::Watchdog,
            NativeProbeErrorV1::Driver,
            NativeProbeErrorV1::Artifact,
        ]
    );
    for (error, debug) in NativeProbeErrorV1::ALL.into_iter().zip([
        "EventLoop",
        "Window",
        "Presenter",
        "Watchdog",
        "Driver",
        "Artifact",
    ]) {
        assert_eq!(format!("{error}"), "native probe failed");
        assert_eq!(format!("{error:?}"), debug);
    }
}

fn assert_error<T: Copy + Eq + Error>() {}
