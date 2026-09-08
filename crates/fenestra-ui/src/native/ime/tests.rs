use super::*;

#[derive(Debug, PartialEq)]
enum Call {
    Allowed(bool),
    Area(Bounds),
}

#[derive(Default)]
struct Sink(Vec<Call>);

impl ImeSink for Sink {
    fn allowed(&mut self, allowed: bool) {
        self.0.push(Call::Allowed(allowed));
    }
    fn area(&mut self, caret: Bounds) {
        self.0.push(Call::Area(caret));
    }
}

fn active(id: u64, x: i32) -> ImeContext {
    ImeContext::active(id, x, 20, 2, 24).unwrap()
}

fn refresh(bridge: &mut ImeBridge, context: Option<ImeContext>, sink: &mut Sink) {
    bridge
        .refresh(Ok::<_, ()>(context), true, true, false, sink)
        .unwrap();
}

#[test]
fn checked_caret_accepts_signed_positions_and_all_editor_identities() {
    for id in [0, u64::MAX] {
        let context = active(id, -12);
        assert_eq!(context.editor_id(), Some(id));
        let caret = context.caret().unwrap();
        assert_eq!(
            (caret.x(), caret.y(), caret.width(), caret.height()),
            (-12, 20, 2, 24)
        );
    }
    assert_eq!(ImeContext::disabled().editor_id(), None);
    assert_eq!(ImeContext::disabled().caret(), None);
}

#[test]
fn empty_or_unrepresentable_carets_are_rejected() {
    for (width, height) in [(0, 1), (1, 0), (0, 0)] {
        assert_eq!(
            ImeContext::active(1, 0, 0, width, height),
            Err(ImeContextError::EmptyCaret)
        );
    }
    for (x, y, width, height) in [
        (0, 0, u32::MAX, 1),
        (0, 0, 1, u32::MAX),
        (i32::MAX, 0, 1, 1),
        (0, i32::MAX, 1, 1),
    ] {
        assert_eq!(
            ImeContext::active(1, x, y, width, height),
            Err(ImeContextError::CaretOutOfRange)
        );
    }
    assert!(ImeContext::active(0, i32::MIN, i32::MIN, i32::MAX as u32, 1).is_ok());
}

#[test]
fn active_context_enables_before_positioning_and_coalesces_repeats() {
    let context = active(7, 10);
    let mut bridge = ImeBridge::new(false);
    let mut sink = Sink::default();
    refresh(&mut bridge, Some(context), &mut sink);
    refresh(&mut bridge, Some(context), &mut sink);
    assert_eq!(
        sink.0,
        [Call::Allowed(true), Call::Area(context.caret().unwrap())]
    );
    sink.0.clear();
    let moved = active(7, 14);
    refresh(&mut bridge, Some(moved), &mut sink);
    assert_eq!(sink.0, [Call::Area(moved.caret().unwrap())]);
}

#[test]
fn transferring_identity_resets_even_when_the_caret_does_not_move() {
    let first = active(0, 10);
    let second = active(u64::MAX, 10);
    let mut bridge = ImeBridge::new(false);
    let mut sink = Sink::default();
    refresh(&mut bridge, Some(first), &mut sink);
    sink.0.clear();
    refresh(&mut bridge, Some(second), &mut sink);
    assert_eq!(
        sink.0,
        [
            Call::Allowed(false),
            Call::Allowed(true),
            Call::Area(second.caret().unwrap())
        ]
    );
}

#[test]
fn focus_loss_and_minimization_suspend_and_restore_the_desired_editor() {
    let context = active(3, 10);
    let mut bridge = ImeBridge::new(false);
    let mut sink = Sink::default();
    refresh(&mut bridge, Some(context), &mut sink);
    sink.0.clear();
    for (drawable, focused) in [(true, false), (false, false), (false, true)] {
        bridge
            .refresh(
                Ok::<_, ()>(Some(context)),
                drawable,
                focused,
                false,
                &mut sink,
            )
            .unwrap();
        assert_eq!(bridge.desired, Some(context));
    }
    assert_eq!(sink.0, [Call::Allowed(false)]);
    sink.0.clear();
    refresh(&mut bridge, Some(context), &mut sink);
    assert_eq!(
        sink.0,
        [Call::Allowed(true), Call::Area(context.caret().unwrap())]
    );
}

#[test]
fn legacy_none_retains_static_policy_and_explicit_disabled_overrides_it() {
    for allowed in [false, true] {
        let mut bridge = ImeBridge::new(allowed);
        let mut sink = Sink::default();
        for (drawable, focused) in [(true, true), (false, false), (true, true)] {
            bridge
                .refresh(Ok::<_, ()>(None), drawable, focused, false, &mut sink)
                .unwrap();
        }
        assert_eq!(sink.0, [Call::Allowed(allowed)]);
        sink.0.clear();
        refresh(&mut bridge, Some(ImeContext::disabled()), &mut sink);
        assert_eq!(
            sink.0,
            if allowed {
                vec![Call::Allowed(false)]
            } else {
                vec![]
            }
        );
        sink.0.clear();
        refresh(&mut bridge, None, &mut sink);
        assert_eq!(
            sink.0,
            if allowed {
                vec![Call::Allowed(true)]
            } else {
                vec![]
            }
        );
    }
}

#[test]
fn callback_rejection_preserves_desired_applied_state_and_native_calls() {
    let context = active(3, 10);
    let mut bridge = ImeBridge::new(false);
    let mut sink = Sink::default();
    refresh(&mut bridge, Some(context), &mut sink);
    let before = bridge.clone();
    sink.0.clear();
    assert_eq!(
        bridge.refresh(Err("rejected"), false, false, true, &mut sink),
        Err("rejected")
    );
    assert_eq!(bridge, before);
    assert!(sink.0.is_empty());
}

#[test]
fn scale_changes_resend_area_without_restarting_composition() {
    let context = active(3, 10);
    let mut bridge = ImeBridge::new(false);
    let mut sink = Sink::default();
    refresh(&mut bridge, Some(context), &mut sink);
    sink.0.clear();
    bridge
        .refresh(Ok::<_, ()>(Some(context)), true, true, true, &mut sink)
        .unwrap();
    assert_eq!(sink.0, [Call::Area(context.caret().unwrap())]);
}
