mod app;
mod flow;
mod lifecycle;
mod script;

#[cfg(test)]
mod tests;

use winit::event_loop::{ControlFlow, EventLoop, EventLoopProxy};

use self::app::NativeApplicationV1;
use super::shell::watchdog::{
    NativeWatchdogErrorV1, NativeWatchdogProxyErrorV1, NativeWatchdogProxyV1,
    NativeWatchdogTokenV1, NativeWatchdogV1,
};
use crate::NativeProbeErrorV1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum NativeUserEventV1 {
    Timeout(NativeWatchdogTokenV1),
}

#[derive(Clone)]
struct WinitWatchdogProxyV1(EventLoopProxy<NativeUserEventV1>);

impl NativeWatchdogProxyV1 for WinitWatchdogProxyV1 {
    fn send_timeout(&self, token: NativeWatchdogTokenV1) -> Result<(), NativeWatchdogProxyErrorV1> {
        self.0
            .send_event(NativeUserEventV1::Timeout(token))
            .map_err(|_| NativeWatchdogProxyErrorV1::Closed)
    }
}

pub(crate) fn run_native_probe_v1() -> Result<Vec<u8>, NativeProbeErrorV1> {
    let event_loop = EventLoop::<NativeUserEventV1>::with_user_event()
        .build()
        .map_err(|_| NativeProbeErrorV1::EventLoop)?;
    event_loop.set_control_flow(ControlFlow::Wait);
    let watchdog = NativeWatchdogV1::spawn(WinitWatchdogProxyV1(event_loop.create_proxy()))
        .map_err(map_watchdog_error)?;
    let mut application = NativeApplicationV1::new(watchdog);
    event_loop
        .run_app(&mut application)
        .map_err(|_| NativeProbeErrorV1::EventLoop)?;
    application.into_output()
}

pub(super) const fn map_watchdog_error(_: NativeWatchdogErrorV1) -> NativeProbeErrorV1 {
    NativeProbeErrorV1::Watchdog
}
