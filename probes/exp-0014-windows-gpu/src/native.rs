mod app;
mod gpu;

use std::process::Command;

use winit::event_loop::{ControlFlow, EventLoop};

use self::app::NativeGpuApplicationV1;
use crate::{GpuTargetV1, InteractiveArtifactBuilderV1};

pub(crate) fn run_interactive_probe_v1() -> Result<Vec<u8>, crate::InteractiveProbeErrorKindV1> {
    let builder = InteractiveArtifactBuilderV1::new(target(), &os_version())
        .map_err(|_| crate::InteractiveProbeErrorKindV1::Artifact)?;
    let event_loop = EventLoop::new().map_err(|_| crate::InteractiveProbeErrorKindV1::EventLoop)?;
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut application = NativeGpuApplicationV1::new(builder);
    event_loop
        .run_app(&mut application)
        .map_err(|_| crate::InteractiveProbeErrorKindV1::EventLoop)?;
    application.into_output()
}

pub(super) const fn target() -> GpuTargetV1 {
    if cfg!(target_os = "windows") {
        GpuTargetV1::WindowsDx12
    } else {
        GpuTargetV1::LinuxVulkan
    }
}

fn os_version() -> Vec<u8> {
    #[cfg(target_os = "windows")]
    let output = Command::new("cmd").args(["/C", "ver"]).output();
    #[cfg(target_os = "linux")]
    let output = Command::new("uname").args(["-sr"]).output();

    let mut bytes = output
        .ok()
        .filter(|output| output.status.success())
        .map_or_else(|| b"unknown".to_vec(), |output| output.stdout);
    bytes.truncate(128);
    while bytes.last().is_some_and(u8::is_ascii_whitespace) {
        bytes.pop();
    }
    if bytes.is_empty() {
        bytes.extend_from_slice(b"unknown");
    }
    bytes
}
