use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NativeWatchdogTokenV1(u64);

impl NativeWatchdogTokenV1 {
    #[cfg(test)]
    pub(crate) const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NativeWatchdogProxyErrorV1 {
    Closed,
}

pub(crate) trait NativeWatchdogProxyV1: Send + 'static {
    fn send_timeout(&self, token: NativeWatchdogTokenV1) -> Result<(), NativeWatchdogProxyErrorV1>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NativeWatchdogErrorV1 {
    Closed,
    TokenExhausted,
    DeadlineOverflow,
    SpawnFailed,
    JoinFailed,
}

enum CommandV1 {
    Arm {
        token: NativeWatchdogTokenV1,
        deadline: Instant,
    },
    Cancel {
        token: NativeWatchdogTokenV1,
        reply: SyncSender<bool>,
    },
    #[cfg(test)]
    ExpireNow(SyncSender<()>),
    Shutdown,
}

pub(crate) struct NativeWatchdogV1<P: NativeWatchdogProxyV1> {
    sender: SyncSender<CommandV1>,
    worker: Option<JoinHandle<()>>,
    timeout: Duration,
    next_token: u64,
    _proxy: std::marker::PhantomData<P>,
}

impl<P: NativeWatchdogProxyV1> NativeWatchdogV1<P> {
    pub(crate) const TIMEOUT: Duration = Duration::from_secs(5);
    pub(crate) const COMMAND_CAPACITY: usize = 1;

    pub(crate) fn spawn(proxy: P) -> Result<Self, NativeWatchdogErrorV1> {
        Self::spawn_with_timeout(proxy, Self::TIMEOUT)
    }

    #[cfg(test)]
    pub(crate) fn spawn_with_timeout_for_test(
        proxy: P,
        timeout: Duration,
    ) -> Result<Self, NativeWatchdogErrorV1> {
        Self::spawn_with_timeout(proxy, timeout)
    }

    fn spawn_with_timeout(proxy: P, timeout: Duration) -> Result<Self, NativeWatchdogErrorV1> {
        let (sender, receiver) = sync_channel(Self::COMMAND_CAPACITY);
        let worker = thread::Builder::new()
            .name(String::from("fenestra-native-watchdog"))
            .spawn(move || run_worker(proxy, receiver))
            .map_err(|_| NativeWatchdogErrorV1::SpawnFailed)?;
        Ok(Self {
            sender,
            worker: Some(worker),
            timeout,
            next_token: 0,
            _proxy: std::marker::PhantomData,
        })
    }

    #[cfg(test)]
    pub(crate) fn deadline_from(start: Instant) -> Option<Instant> {
        start.checked_add(Self::TIMEOUT)
    }

    #[cfg(test)]
    pub(crate) const fn command_capacity(&self) -> usize {
        Self::COMMAND_CAPACITY
    }

    pub(crate) fn arm(&mut self) -> Result<NativeWatchdogTokenV1, NativeWatchdogErrorV1> {
        let token = NativeWatchdogTokenV1(self.next_token);
        self.next_token = self
            .next_token
            .checked_add(1)
            .ok_or(NativeWatchdogErrorV1::TokenExhausted)?;
        let deadline = Instant::now()
            .checked_add(self.timeout)
            .ok_or(NativeWatchdogErrorV1::DeadlineOverflow)?;
        self.sender
            .send(CommandV1::Arm { token, deadline })
            .map_err(|_| NativeWatchdogErrorV1::Closed)?;
        Ok(token)
    }

    pub(crate) fn cancel(
        &mut self,
        token: NativeWatchdogTokenV1,
    ) -> Result<bool, NativeWatchdogErrorV1> {
        let (reply, response) = sync_channel(1);
        self.sender
            .send(CommandV1::Cancel { token, reply })
            .map_err(|_| NativeWatchdogErrorV1::Closed)?;
        response.recv().map_err(|_| NativeWatchdogErrorV1::Closed)
    }

    #[cfg(test)]
    pub(crate) fn expire_now_for_test(&mut self) -> Result<(), NativeWatchdogErrorV1> {
        let (reply, response) = sync_channel(1);
        self.sender
            .send(CommandV1::ExpireNow(reply))
            .map_err(|_| NativeWatchdogErrorV1::Closed)?;
        response.recv().map_err(|_| NativeWatchdogErrorV1::Closed)
    }

    pub(crate) fn shutdown_and_join(&mut self) -> Result<(), NativeWatchdogErrorV1> {
        let Some(worker) = self.worker.take() else {
            return Ok(());
        };
        if self.sender.send(CommandV1::Shutdown).is_err() {
            worker
                .join()
                .map_err(|_| NativeWatchdogErrorV1::JoinFailed)?;
            return Err(NativeWatchdogErrorV1::Closed);
        }
        worker.join().map_err(|_| NativeWatchdogErrorV1::JoinFailed)
    }
}

impl<P: NativeWatchdogProxyV1> Drop for NativeWatchdogV1<P> {
    fn drop(&mut self) {
        if let Some(worker) = self.worker.take() {
            let _ = self.sender.send(CommandV1::Shutdown);
            let _ = worker.join();
        }
    }
}

fn run_worker<P: NativeWatchdogProxyV1>(proxy: P, receiver: Receiver<CommandV1>) {
    let mut active = None;
    loop {
        let command = match active {
            Some((token, deadline)) => {
                let now = Instant::now();
                if now >= deadline {
                    let _ = proxy.send_timeout(token);
                    active = None;
                    continue;
                }
                match receiver.recv_timeout(deadline.saturating_duration_since(now)) {
                    Ok(command) => command,
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        let _ = proxy.send_timeout(token);
                        active = None;
                        continue;
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => return,
                }
            }
            None => match receiver.recv() {
                Ok(command) => command,
                Err(_) => return,
            },
        };
        match command {
            CommandV1::Arm { token, deadline } => active = Some((token, deadline)),
            CommandV1::Cancel { token, reply } => {
                let canceled = active.is_some_and(|(active_token, _)| active_token == token);
                if canceled {
                    active = None;
                }
                let _ = reply.send(canceled);
            }
            #[cfg(test)]
            CommandV1::ExpireNow(reply) => {
                if let Some((token, _)) = active.take() {
                    let _ = proxy.send_timeout(token);
                }
                let _ = reply.send(());
            }
            CommandV1::Shutdown => return,
        }
    }
}
