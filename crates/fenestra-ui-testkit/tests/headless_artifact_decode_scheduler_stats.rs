#[path = "headless/artifact_decode_support.rs"]
mod support;

use fenestra_ui_testkit::prototype::HeadlessArtifactDecodeErrorKindV1 as Kind;

#[test]
fn scheduler_lane_bytes_use_the_closed_accounting_weights() {
    let canonical = support::canonical_bytes();
    for (headless, scheduler, headless_field, scheduler_field, bytes) in [
        ("h-event|1|4|", "s-event|1|0|", 21, 15, "79"),
        ("h-event|1|48|", "s-event|1|34|", 23, 18, "31"),
        ("h-event|1|48|", "s-event|1|34|", 25, 21, "39"),
        ("h-event|1|48|", "s-event|1|34|", 27, 24, "39"),
        ("h-event|1|48|", "s-event|1|34|", 29, 27, "95"),
    ] {
        let changed = mapped_value(
            &canonical,
            headless,
            scheduler,
            headless_field,
            scheduler_field,
            bytes,
        );
        assert_invalid(&changed, &canonical, scheduler);
    }
}

#[test]
fn standalone_headless_events_also_enforce_accounting_weights() {
    let canonical = support::canonical_bytes();
    let changed = support::set_field(&canonical, "h-event|1|1|", 20, "1");
    let changed = support::set_field(&changed, "h-event|1|1|", 21, "79");

    support::assert_decode_error(
        &changed,
        Kind::InvalidReference,
        Some(support::line_number(&canonical, "h-event|1|1|")),
    );
}

#[test]
fn weight_correct_lane_usage_cannot_exceed_configured_capacity() {
    let canonical = support::canonical_bytes();
    for (headless, scheduler, headless_item, scheduler_item, items, bytes) in [
        ("h-event|1|4|", "s-event|1|0|", 20, 14, "2", "160"),
        ("h-event|1|48|", "s-event|1|34|", 22, 17, "5", "160"),
        ("h-event|1|48|", "s-event|1|34|", 24, 20, "2", "80"),
        ("h-event|1|48|", "s-event|1|34|", 26, 23, "3", "120"),
        ("h-event|1|48|", "s-event|1|34|", 28, 26, "3", "288"),
    ] {
        let changed = mapped_value(
            &canonical,
            headless,
            scheduler,
            headless_item,
            scheduler_item,
            items,
        );
        let changed = mapped_value(
            &changed,
            headless,
            scheduler,
            headless_item + 1,
            scheduler_item + 1,
            bytes,
        );
        assert_invalid(&changed, &canonical, scheduler);
    }
}

#[test]
fn occupied_renderer_requires_a_last_accepted_submission() {
    let canonical = support::canonical_bytes();
    let changed = support::set_field(&canonical, "s-event|1|18|", 29, "-");

    assert_invalid(&changed, &canonical, "s-event|1|18|");
}

#[test]
fn scheduler_lane_residence_presence_matches_occupancy() {
    let canonical = support::canonical_bytes();
    for residence_field in [16, 19, 22, 25, 28] {
        let changed = support::set_field(&canonical, "s-event|1|40|", residence_field, "0");
        assert_invalid(&changed, &canonical, "s-event|1|40|");
    }
    for (scheduler, residence_field) in [
        ("s-event|1|0|", 16),
        ("s-event|1|34|", 19),
        ("s-event|1|34|", 22),
        ("s-event|1|34|", 25),
        ("s-event|1|34|", 28),
    ] {
        let changed = support::set_field(&canonical, scheduler, residence_field, "-");
        assert_invalid(&changed, &canonical, scheduler);
    }
}

#[test]
fn scheduler_lane_residence_cannot_exceed_the_event_tick() {
    let canonical = support::canonical_bytes();
    for (scheduler, residence_field, residence) in [
        ("s-event|1|0|", 16, "2"),
        ("s-event|1|34|", 19, "16"),
        ("s-event|1|34|", 22, "16"),
        ("s-event|1|34|", 25, "16"),
        ("s-event|1|34|", 28, "16"),
    ] {
        let changed = support::set_field(&canonical, scheduler, residence_field, residence);
        assert_invalid(&changed, &canonical, scheduler);
    }
}

#[test]
fn scheduler_lane_residence_cannot_exceed_its_configured_capacity() {
    let canonical = support::canonical_bytes();
    let deferred_capacity = support::set_field(&canonical, "capacity-scheduler|", 3, "0");
    let deferred = support::set_field(&deferred_capacity, "s-event|1|0|", 16, "1");
    assert_invalid(&deferred, &canonical, "s-event|1|0|");

    for (scheduler_field, residence) in [(19, "9"), (22, "9"), (25, "9"), (28, "9")] {
        let changed = support::set_field(&canonical, "s-event|1|34|", scheduler_field, residence);
        assert_invalid(&changed, &canonical, "s-event|1|34|");
    }
}

fn mapped_value(
    bytes: &[u8],
    headless: &str,
    scheduler: &str,
    headless_field: usize,
    scheduler_field: usize,
    value: &str,
) -> Vec<u8> {
    let changed = support::set_field(bytes, headless, headless_field, value);
    support::set_field(&changed, scheduler, scheduler_field, value)
}

fn assert_invalid(bytes: &[u8], canonical: &[u8], scheduler: &str) {
    support::assert_decode_error(
        bytes,
        Kind::InvalidReference,
        Some(support::line_number(canonical, scheduler)),
    );
}
