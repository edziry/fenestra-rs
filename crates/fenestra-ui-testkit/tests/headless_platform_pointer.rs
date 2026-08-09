#[path = "headless/fixture_support.rs"]
mod fixture_support;
#[path = "headless/platform_support.rs"]
mod support;

use fenestra_ui_ir::prototype::PropertyValue;
use fenestra_ui_runtime::prototype::{
    CallbackFinish, HeadlessPoint, SchedulerAction, SchedulerErrorKind, SchedulerTick,
    TransactionErrorKind,
};
use fenestra_ui_testkit::prototype::{
    FakeCallbackDepthV1, FakePlatformV1, HeadlessFixtureV1, HeadlessPlatformErrorKindV1,
    HeadlessPointerCaptureV1, HeadlessPointerMutationV1, HeadlessPointerScriptV1,
    HeadlessPointerTargetV1,
};

use fixture_support::{
    COLOR, SECOND_KEY, VISIBLE, WIDTH, control_path, fragment_id, item_path, items_path, node_id,
    rgba,
};

fn point(x: i32, y: i32) -> HeadlessPoint {
    HeadlessPoint::new(x, y)
}

fn mutation(
    property: fenestra_ui_ir::prototype::PropertyId,
    value: PropertyValue,
) -> HeadlessPointerMutationV1 {
    HeadlessPointerMutationV1::new(property, value)
}

fn assert_capture_type(capture: &HeadlessPointerCaptureV1) {
    assert_eq!(capture.generation().get(), 0);
    assert_eq!(capture.target(), HeadlessPointerTargetV1::Key(SECOND_KEY));
}

#[test]
fn nested_pointer_callback_targets_the_last_committed_projection_and_defers_publication() {
    let fixture = HeadlessFixtureV1::build().expect("registered fixture should validate");
    let mut scheduler = support::scheduler(&fixture);
    let mut platform = FakePlatformV1::new();
    let before = scheduler.committed();
    let control = node_id(&before, &control_path());
    let observed = platform
        .capture_headless_pointer(&scheduler, &fixture, point(5, 5))
        .expect("generation-zero control should be an accepting target");
    assert_eq!(observed.generation(), before.generation());
    assert_eq!(observed.target(), HeadlessPointerTargetV1::StaticControl);

    let report = platform
        .run_headless_pointer_callback(
            &mut scheduler,
            &fixture,
            HeadlessPointerScriptV1::new(
                point(5, 5),
                FakeCallbackDepthV1::Nested,
                Some(mutation(COLOR, rgba([20, 30, 40, 255]))),
            ),
            SchedulerTick::new(1),
        )
        .expect("accepting pointer target should stage one callback mutation");

    assert_eq!(report.captured_generation(), before.generation());
    assert_eq!(report.target(), HeadlessPointerTargetV1::StaticControl);
    assert_eq!(report.deepest_depth(), 2);
    assert!(report.shares_entry_snapshot());
    assert_eq!(
        report.finish(),
        CallbackFinish::Deferred {
            operation_count: 1,
            accounted_bytes: 80,
        }
    );
    assert!(scheduler.committed().shares_state_with(&before));
    assert_eq!(
        scheduler.committed().property(control, COLOR),
        Some(&rgba([10, 20, 30, 255]))
    );
    assert_eq!(scheduler.stats().deferred().items(), 1);
    assert_eq!(scheduler.stats().visual().items(), 0);

    let blocked = platform
        .run_headless_pointer_callback(
            &mut scheduler,
            &fixture,
            HeadlessPointerScriptV1::new(point(5, 5), FakeCallbackDepthV1::Outer, None),
            SchedulerTick::new(1),
        )
        .expect_err("a second callback must not overtake deferred publication");
    assert_eq!(
        blocked.kind(),
        HeadlessPlatformErrorKindV1::Scheduler(SchedulerErrorKind::ControlPending)
    );
    assert_eq!(blocked.operation_index(), None);

    assert_eq!(
        scheduler
            .next_action(SchedulerTick::new(2))
            .expect("later scheduler turn should publish the deferred mutation"),
        Some(SchedulerAction::RequestFrame)
    );
    let published = scheduler.committed();
    assert_eq!(published.generation().get(), 1);
    assert_eq!(
        published.property(control, COLOR),
        Some(&rgba([20, 30, 40, 255]))
    );
    assert_eq!(scheduler.stats().deferred().items(), 0);
    assert_eq!(
        scheduler.stats().visual().latest_tick(),
        Some(SchedulerTick::new(2))
    );
}

#[test]
fn pointer_target_is_captured_before_a_visibility_mutation_can_change_hit_testing() {
    let fixture = HeadlessFixtureV1::build().expect("registered fixture should validate");
    let mut scheduler = support::scheduler(&fixture);
    let mut platform = FakePlatformV1::new();
    let before = scheduler.committed();
    let control = node_id(&before, &control_path());

    let report = platform
        .run_headless_pointer_callback(
            &mut scheduler,
            &fixture,
            HeadlessPointerScriptV1::new(
                point(5, 5),
                FakeCallbackDepthV1::Outer,
                Some(mutation(VISIBLE, PropertyValue::Bool(false))),
            ),
            SchedulerTick::new(1),
        )
        .expect("committed target should accept one visibility mutation");

    assert_eq!(report.target(), HeadlessPointerTargetV1::StaticControl);
    assert_eq!(report.captured_generation(), before.generation());
    assert!(scheduler.committed().shares_state_with(&before));
    assert_eq!(
        before
            .headless_projection()
            .expect("old snapshot should retain its projection")
            .hit_test(point(5, 5)),
        Some(control)
    );

    assert_eq!(
        scheduler
            .next_action(SchedulerTick::new(2))
            .expect("visibility mutation should publish later"),
        Some(SchedulerAction::RequestFrame)
    );
    let fresh = platform
        .capture_headless_pointer(&scheduler, &fixture, point(5, 5))
        .expect("fresh committed projection should remain queryable");
    assert_eq!(fresh.generation().get(), 1);
    assert_eq!(fresh.target(), HeadlessPointerTargetV1::None);
    assert_eq!(
        before
            .headless_projection()
            .expect("retained snapshot should remain coherent")
            .hit_test(point(5, 5)),
        Some(control)
    );
}

#[test]
fn pointer_miss_with_a_mutation_is_a_query_only_no_op() {
    let fixture = HeadlessFixtureV1::build().expect("registered fixture should validate");
    let mut scheduler = support::scheduler(&fixture);
    let mut platform = FakePlatformV1::new();
    let before = scheduler.committed();

    let report = platform
        .run_headless_pointer_callback(
            &mut scheduler,
            &fixture,
            HeadlessPointerScriptV1::new(
                point(100, 0),
                FakeCallbackDepthV1::Nested,
                Some(mutation(WIDTH, PropertyValue::ScalarI32(41))),
            ),
            SchedulerTick::new(0),
        )
        .expect("a pointer miss should not invent a mutation target");

    assert_eq!(report.target(), HeadlessPointerTargetV1::None);
    assert_eq!(report.deepest_depth(), 2);
    assert!(report.shares_entry_snapshot());
    assert_eq!(report.finish(), CallbackFinish::NoChanges);
    assert!(scheduler.committed().shares_state_with(&before));
    assert_eq!(scheduler.stats().deferred().items(), 0);
    assert_eq!(scheduler.stats().visual().items(), 0);
}

#[test]
fn retired_pointer_capture_fails_only_when_its_deferred_mutation_publishes() {
    let fixture = HeadlessFixtureV1::build().expect("registered fixture should validate");
    let mut scheduler = support::scheduler(&fixture);
    let mut platform = FakePlatformV1::new();
    let initial = scheduler.committed();
    let retired = node_id(&initial, &item_path(SECOND_KEY));
    let capture = platform
        .capture_headless_pointer(&scheduler, &fixture, point(5, 22))
        .expect("second keyed member should be an accepting target");

    assert_capture_type(&capture);
    assert_eq!(
        format!("{capture:?}"),
        "HeadlessPointerCaptureV1 { generation: 0, target: Key(20) }"
    );

    let mut removal = scheduler.begin_transaction();
    removal
        .remove_keyed(fragment_id(&initial, &items_path()), SECOND_KEY)
        .expect("live key removal should stage");
    scheduler
        .commit(removal, SchedulerTick::new(6))
        .expect("live key removal should publish");
    let after_removal = scheduler.committed();
    let visual_before_stale = scheduler.stats().visual();
    assert_eq!(after_removal.generation().get(), 1);
    assert_eq!(after_removal.template(retired), None);

    let report = platform
        .run_headless_captured_callback(
            &mut scheduler,
            &capture,
            FakeCallbackDepthV1::Nested,
            mutation(WIDTH, PropertyValue::ScalarI32(41)),
            SchedulerTick::new(6),
        )
        .expect("stale identity is validated only at deferred publication");
    assert_eq!(report.captured_generation(), after_removal.generation());
    assert_eq!(report.target(), HeadlessPointerTargetV1::Key(SECOND_KEY));
    assert_eq!(report.deepest_depth(), 2);
    assert!(report.shares_entry_snapshot());
    assert_eq!(
        report.finish(),
        CallbackFinish::Deferred {
            operation_count: 1,
            accounted_bytes: 80,
        }
    );
    assert!(scheduler.committed().shares_state_with(&after_removal));

    let error = scheduler
        .next_action(SchedulerTick::new(6))
        .expect_err("retired target must fail when the deferred transaction publishes");
    assert_eq!(
        error.kind(),
        SchedulerErrorKind::Transaction(TransactionErrorKind::MissingNode)
    );
    assert_eq!(error.operation_index(), Some(0));
    assert!(scheduler.committed().shares_state_with(&after_removal));
    assert_eq!(scheduler.stats().visual(), visual_before_stale);
    assert_eq!(scheduler.stats().deferred().items(), 0);
    assert_eq!(
        scheduler
            .next_action(SchedulerTick::new(6))
            .expect("failed deferred work must not consume the earlier frame request"),
        Some(SchedulerAction::RequestFrame)
    );
    assert!(scheduler.committed().shares_state_with(&after_removal));
}

#[test]
fn ordinary_snapshot_pointer_query_returns_a_closed_headless_error() {
    let fixture = HeadlessFixtureV1::build().expect("registered fixture should validate");
    let scheduler = support::ordinary_scheduler(&fixture);
    let platform = FakePlatformV1::new();
    let before = scheduler.committed();

    let error = platform
        .capture_headless_pointer(&scheduler, &fixture, point(5, 5))
        .expect_err("ordinary committed state has no headless projection");

    assert_eq!(
        error.kind(),
        HeadlessPlatformErrorKindV1::HeadlessUnavailable
    );
    assert_eq!(error.operation_index(), None);
    assert!(scheduler.committed().shares_state_with(&before));
    assert!(!format!("{error:?}").contains("NodeId"));
}

#[test]
fn pointer_captures_do_not_retain_committed_snapshot_allocations() {
    let fixture = HeadlessFixtureV1::build().expect("registered fixture should validate");
    let mut scheduler = support::scheduler(&fixture);
    let platform = FakePlatformV1::new();
    let initial = scheduler.committed();
    let root = initial.root();
    drop(initial);
    let mut captures = Vec::new();

    for generation in 0..4_u64 {
        let capture = platform
            .capture_headless_pointer(&scheduler, &fixture, point(5, 5))
            .expect("each committed generation should remain queryable");
        assert_eq!(capture.generation().get(), generation);
        captures.push(capture);

        let mut transaction = scheduler.begin_transaction();
        transaction
            .set_property(
                root,
                WIDTH,
                PropertyValue::ScalarI32(99 - i32::try_from(generation).expect("small fixture")),
            )
            .expect("effective width mutation should stage");
        scheduler
            .commit(transaction, SchedulerTick::new(generation + 1))
            .expect("opaque captures must not consume retained-generation capacity");
    }

    assert_eq!(captures.len(), 4);
    assert_eq!(scheduler.committed().generation().get(), 4);
}

#[test]
fn accepting_node_outside_the_closed_target_vocabulary_is_an_identity_error() {
    let fixture = HeadlessFixtureV1::build().expect("registered fixture should validate");
    let mut scheduler = support::scheduler(&fixture);
    let platform = FakePlatformV1::new();
    let root = scheduler.committed().root();
    let mut transaction = scheduler.begin_transaction();
    transaction
        .set_property(
            root,
            fixture.spec().input(),
            PropertyValue::InputPolicy(fenestra_ui_ir::prototype::InputPolicy::Accept),
        )
        .expect("root input policy should stage");
    scheduler
        .commit(transaction, SchedulerTick::new(1))
        .expect("root input policy should publish");
    let before = scheduler.committed();

    let error = platform
        .capture_headless_pointer(&scheduler, &fixture, point(90, 5))
        .expect_err("the closed target vocabulary cannot name the accepting root");
    assert_eq!(error.kind(), HeadlessPlatformErrorKindV1::IdentityMismatch);
    assert_eq!(error.operation_index(), None);
    assert!(scheduler.committed().shares_state_with(&before));
}

#[test]
fn target_vocabulary_is_closed_and_does_not_expose_runtime_node_ids() {
    fn discriminant(target: HeadlessPointerTargetV1) -> u8 {
        match target {
            HeadlessPointerTargetV1::None => 0,
            HeadlessPointerTargetV1::StaticControl => 1,
            HeadlessPointerTargetV1::Key(_) => 2,
        }
    }

    assert_eq!(discriminant(HeadlessPointerTargetV1::None), 0);
    assert_eq!(discriminant(HeadlessPointerTargetV1::StaticControl), 1);
    assert_eq!(discriminant(HeadlessPointerTargetV1::Key(SECOND_KEY)), 2);
}

#[test]
fn headless_platform_error_vocabulary_is_closed() {
    fn discriminant(kind: HeadlessPlatformErrorKindV1) -> u8 {
        match kind {
            HeadlessPlatformErrorKindV1::Scheduler(_) => 0,
            HeadlessPlatformErrorKindV1::HeadlessUnavailable => 1,
            HeadlessPlatformErrorKindV1::IdentityMismatch => 2,
        }
    }

    assert_eq!(
        discriminant(HeadlessPlatformErrorKindV1::Scheduler(
            SchedulerErrorKind::InputOutOfOrder,
        )),
        0
    );
    assert_eq!(
        discriminant(HeadlessPlatformErrorKindV1::HeadlessUnavailable),
        1
    );
    assert_eq!(
        discriminant(HeadlessPlatformErrorKindV1::IdentityMismatch),
        2
    );
}
