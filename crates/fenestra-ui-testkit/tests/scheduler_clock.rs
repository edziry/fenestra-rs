use fenestra_ui_runtime::prototype::SchedulerTick;
use fenestra_ui_testkit::prototype::{FakeClockDomainV1, FakeClockErrorKindV1, FakeClockV1};

#[test]
fn fake_clock_advances_only_when_directed_in_one_named_domain() {
    let domain = FakeClockDomainV1::new(7);
    let mut clock = FakeClockV1::new(domain, SchedulerTick::new(10));

    assert_eq!(clock.domain(), domain);
    assert_eq!(clock.now(), SchedulerTick::new(10));
    assert_eq!(
        clock.advance(0).expect("zero advance should be valid"),
        SchedulerTick::new(10)
    );
    assert_eq!(
        clock.advance(8).expect("manual advance should be valid"),
        SchedulerTick::new(18)
    );
    assert_eq!(clock.now(), SchedulerTick::new(18));

    let mut repeated = FakeClockV1::new(domain, SchedulerTick::new(10));
    assert_eq!(
        repeated
            .advance(0)
            .expect("equal script should accept zero advance"),
        SchedulerTick::new(10)
    );
    assert_eq!(
        repeated
            .advance(8)
            .expect("equal script should reproduce its tick"),
        clock.now()
    );
}

#[test]
fn fake_clock_overflow_is_typed_private_and_atomic() {
    let mut clock = FakeClockV1::new(FakeClockDomainV1::new(11), SchedulerTick::new(u64::MAX));
    let error = clock
        .advance(1)
        .expect_err("clock arithmetic must not wrap");

    assert_eq!(error.kind(), FakeClockErrorKindV1::ArithmeticExhausted);
    assert_eq!(clock.now(), SchedulerTick::new(u64::MAX));
    assert_eq!(
        format!("{error:?}"),
        "FakeClockErrorV1 { kind: ArithmeticExhausted }"
    );
    assert_eq!(
        error.to_string(),
        "fake scheduler clock failed: ArithmeticExhausted"
    );
}
