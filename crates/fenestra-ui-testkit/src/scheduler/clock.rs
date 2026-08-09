use std::error::Error;
use std::fmt;

use fenestra_ui_runtime::prototype::SchedulerTick;

/// Stable identity of one deterministic fake-clock domain.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FakeClockDomainV1(u32);

impl FakeClockDomainV1 {
    /// Creates an explicit fake-clock domain identity.
    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    /// Returns the numeric domain identity used by scheduler traces.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

/// Closed failures produced by the deterministic fake clock.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FakeClockErrorKindV1 {
    /// Checked tick arithmetic could not represent the requested advance.
    ArithmeticExhausted,
}

/// Privacy-safe failure returned by the deterministic fake clock.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct FakeClockErrorV1 {
    kind: FakeClockErrorKindV1,
}

impl FakeClockErrorV1 {
    const fn new(kind: FakeClockErrorKindV1) -> Self {
        Self { kind }
    }

    /// Returns the closed fake-clock failure category.
    #[must_use]
    pub const fn kind(self) -> FakeClockErrorKindV1 {
        self.kind
    }
}

impl fmt::Debug for FakeClockErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("FakeClockErrorV1")
            .field("kind", &self.kind)
            .finish()
    }
}

impl fmt::Display for FakeClockErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "fake scheduler clock failed: {:?}", self.kind)
    }
}

impl Error for FakeClockErrorV1 {}

/// Manually advanced logical clock used by deterministic scheduler scripts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FakeClockV1 {
    domain: FakeClockDomainV1,
    tick: SchedulerTick,
}

impl FakeClockV1 {
    /// Creates a clock at an explicit tick without reading wall time.
    #[must_use]
    pub const fn new(domain: FakeClockDomainV1, tick: SchedulerTick) -> Self {
        Self { domain, tick }
    }

    /// Returns the clock domain shared by every reading from this clock.
    #[must_use]
    pub const fn domain(self) -> FakeClockDomainV1 {
        self.domain
    }

    /// Returns the current logical tick without advancing it.
    #[must_use]
    pub const fn now(self) -> SchedulerTick {
        self.tick
    }

    /// Advances by an explicit delta and returns the resulting logical tick.
    pub fn advance(&mut self, delta: u64) -> Result<SchedulerTick, FakeClockErrorV1> {
        let value = self
            .tick
            .get()
            .checked_add(delta)
            .ok_or_else(|| FakeClockErrorV1::new(FakeClockErrorKindV1::ArithmeticExhausted))?;
        let tick = SchedulerTick::new(value);
        self.tick = tick;
        Ok(tick)
    }
}
