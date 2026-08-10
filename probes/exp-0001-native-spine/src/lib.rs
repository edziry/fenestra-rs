#![forbid(unsafe_code)]

//! Disposable native-window feasibility probe for Fenestra.

use std::error::Error;
use std::fmt;

mod native;

/// Closed failures that prevent the native probe from emitting evidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeProbeErrorV1 {
    /// The native event loop could not be created or completed.
    EventLoop,
    /// The native window could not be created.
    Window,
    /// The native presentation resources could not be created.
    Presenter,
    /// The bounded watchdog could not be started or controlled.
    Watchdog,
    /// The runtime driver could not reach a clean terminal state.
    Driver,
    /// The bounded evidence artifact could not be formed.
    Artifact,
}

impl NativeProbeErrorV1 {
    /// Every failure in stable priority order.
    pub const ALL: [Self; 6] = [
        Self::EventLoop,
        Self::Window,
        Self::Presenter,
        Self::Watchdog,
        Self::Driver,
        Self::Artifact,
    ];
}

impl fmt::Display for NativeProbeErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("native probe failed")
    }
}

impl Error for NativeProbeErrorV1 {}

/// Runs the fixed native reference probe and returns its bounded ASCII artifact.
pub fn run_native_probe_v1() -> Result<Vec<u8>, NativeProbeErrorV1> {
    native::run_native_probe_v1()
}
