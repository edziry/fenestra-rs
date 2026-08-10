use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{RecvTimeoutError, SyncSender, TryRecvError, sync_channel};
use std::time::{Duration, Instant};

use super::super::super::shell::watchdog::{
    NativeWatchdogProxyErrorV1, NativeWatchdogProxyV1, NativeWatchdogTokenV1, NativeWatchdogV1,
};

struct TestProxy {
    sender: SyncSender<NativeWatchdogTokenV1>,
    dropped: Arc<AtomicUsize>,
}

impl NativeWatchdogProxyV1 for TestProxy {
    fn send_timeout(&self, token: NativeWatchdogTokenV1) -> Result<(), NativeWatchdogProxyErrorV1> {
        self.sender
            .send(token)
            .map_err(|_| NativeWatchdogProxyErrorV1::Closed)
    }
}

impl Drop for TestProxy {
    fn drop(&mut self) {
        self.dropped.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn watchdog_deadline_and_command_channel_are_exact_and_monotonic() {
    assert_eq!(
        NativeWatchdogV1::<TestProxy>::TIMEOUT,
        Duration::from_secs(5)
    );
    assert_eq!(NativeWatchdogV1::<TestProxy>::COMMAND_CAPACITY, 1);

    let first = Instant::now();
    let second = first
        .checked_add(Duration::from_millis(1))
        .expect("test instant should advance");
    assert_eq!(
        NativeWatchdogV1::<TestProxy>::deadline_from(first)
            .expect("five-second deadline should fit"),
        first
            .checked_add(Duration::from_secs(5))
            .expect("test deadline should fit")
    );
    assert!(
        NativeWatchdogV1::<TestProxy>::deadline_from(second).expect("later deadline should fit")
            > NativeWatchdogV1::<TestProxy>::deadline_from(first)
                .expect("earlier deadline should fit")
    );
}

#[test]
fn worker_replaces_cancels_and_joins_through_the_bounded_proxy_seam() {
    let (signal_tx, signal_rx) = sync_channel(1);
    let dropped = Arc::new(AtomicUsize::new(0));
    let proxy = TestProxy {
        sender: signal_tx,
        dropped: Arc::clone(&dropped),
    };
    let mut watchdog =
        NativeWatchdogV1::spawn_with_timeout_for_test(proxy, Duration::from_millis(20))
            .expect("test watchdog worker should spawn");
    assert_eq!(watchdog.command_capacity(), 1);

    let first = watchdog.arm().expect("first token should arm");
    let replacement = watchdog.arm().expect("second token should replace first");
    assert_eq!(first.get(), 0);
    assert_eq!(replacement.get(), 1);
    assert_eq!(
        signal_rx
            .recv_timeout(Duration::from_millis(250))
            .expect("replacement deadline should signal"),
        replacement
    );
    assert_eq!(signal_rx.try_recv(), Err(TryRecvError::Empty));
    assert!(
        !watchdog
            .cancel(first)
            .expect("stale cancel should be typed")
    );

    let canceled = watchdog.arm().expect("third token should arm");
    assert_eq!(canceled.get(), 2);
    assert!(
        watchdog
            .cancel(canceled)
            .expect("active cancel should reach worker")
    );
    assert_eq!(
        signal_rx.recv_timeout(Duration::from_millis(60)),
        Err(RecvTimeoutError::Timeout)
    );

    watchdog
        .shutdown_and_join()
        .expect("worker shutdown should join cleanly");
    assert_eq!(dropped.load(Ordering::SeqCst), 1);
    assert_eq!(
        signal_rx.recv_timeout(Duration::from_millis(20)),
        Err(RecvTimeoutError::Disconnected)
    );
}
