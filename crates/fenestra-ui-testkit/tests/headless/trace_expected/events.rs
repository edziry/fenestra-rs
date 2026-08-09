use fenestra_ui_testkit::prototype::{
    HeadlessFailureCauseV1, HeadlessInputKindV1, HeadlessOutcomeV1, HeadlessPointerTargetV1,
    HeadlessTraceStageV1,
};

use super::ExpectedEvent;
use super::state::*;

type Outcome = HeadlessOutcomeV1;
type Target = HeadlessPointerTargetV1;

macro_rules! trace_event {
    (
        $tick:expr, $stage:ident, $input:ident, $outcome:expr;
        captured $captured:expr; published $published:expr; target $target:expr;
        frame $frame:expr; control $control:expr; $state:ident
    ) => {
        ExpectedEvent {
            tick: $tick,
            stage: HeadlessTraceStageV1::$stage,
            input: HeadlessInputKindV1::$input,
            outcome: $outcome,
            captured: $captured,
            published: $published,
            target: $target,
            frame: $frame,
            control: $control,
            state: $state,
        }
    };
}

pub(super) fn expected_events() -> [ExpectedEvent; 55] {
    use HeadlessFailureCauseV1::Runtime;

    [
        trace_event!(
            0, Build, None, Outcome::Observed;
            captured None; published Some(0); target Target::None;
            frame None; control None; INITIAL_EMPTY
        ),
        trace_event!(
            0, Projection, None, Outcome::Matched;
            captured None; published Some(0); target Target::None;
            frame None; control None; INITIAL_EMPTY
        ),
        trace_event!(
            0, Input, Pointer, Outcome::Observed;
            captured Some(0); published None; target Target::StaticControl;
            frame None; control None; INITIAL_EMPTY
        ),
        trace_event!(
            0, Input, Pointer, Outcome::Observed;
            captured Some(0); published None; target Target::Key(20);
            frame None; control None; INITIAL_EMPTY
        ),
        trace_event!(
            1, Callback, Pointer, Outcome::Deferred;
            captured Some(0); published None; target Target::StaticControl;
            frame None; control None; INITIAL_DEFERRED
        ),
        trace_event!(
            2, Scheduler, Pointer, Outcome::Action;
            captured Some(0); published Some(1); target Target::StaticControl;
            frame None; control None; INITIAL_VISUAL
        ),
        trace_event!(
            2, Projection, None, Outcome::Matched;
            captured None; published Some(1); target Target::None;
            frame None; control None; INITIAL_VISUAL
        ),
        trace_event!(
            3, Transaction, Insert, Outcome::Published;
            captured None; published Some(2); target Target::Key(30);
            frame None; control None; INSERTED_VISUAL
        ),
        trace_event!(
            3, Projection, None, Outcome::Matched;
            captured None; published Some(2); target Target::None;
            frame None; control None; INSERTED_VISUAL
        ),
        trace_event!(
            3, Scheduler, None, Outcome::Action;
            captured None; published None; target Target::None;
            frame None; control None; INSERTED_VISUAL
        ),
        trace_event!(
            4, Transaction, Move, Outcome::Published;
            captured None; published Some(3); target Target::Key(30);
            frame None; control None; INSERTED_VISUAL
        ),
        trace_event!(
            4, Projection, None, Outcome::Matched;
            captured None; published Some(3); target Target::None;
            frame None; control None; INSERTED_VISUAL
        ),
        trace_event!(
            4, Scheduler, None, Outcome::Action;
            captured None; published None; target Target::None;
            frame None; control None; INSERTED_VISUAL
        ),
        trace_event!(
            5, Transaction, Update, Outcome::Published;
            captured None; published Some(4); target Target::Key(30);
            frame None; control None; INSERTED_VISUAL
        ),
        trace_event!(
            5, Projection, None, Outcome::Matched;
            captured None; published Some(4); target Target::None;
            frame None; control None; INSERTED_VISUAL
        ),
        trace_event!(
            5, Scheduler, None, Outcome::Action;
            captured None; published None; target Target::None;
            frame None; control None; INSERTED_VISUAL
        ),
        trace_event!(
            6, Transaction, Remove, Outcome::Published;
            captured None; published Some(5); target Target::Key(20);
            frame None; control None; INITIAL_VISUAL
        ),
        trace_event!(
            6, Projection, None, Outcome::Matched;
            captured None; published Some(5); target Target::None;
            frame None; control None; INITIAL_VISUAL
        ),
        trace_event!(
            6, Callback, Pointer, Outcome::Deferred;
            captured Some(0); published None; target Target::Key(20);
            frame None; control None; INITIAL_DEFERRED_VISUAL
        ),
        trace_event!(
            6, Scheduler, Pointer, Outcome::Failed(Runtime);
            captured Some(0); published None; target Target::Key(20);
            frame None; control None; INITIAL_VISUAL
        ),
        trace_event!(
            6, Scheduler, None, Outcome::Action;
            captured None; published None; target Target::None;
            frame None; control None; INITIAL_VISUAL
        ),
        trace_event!(
            7, Callback, Resize, Outcome::Deferred;
            captured Some(5); published None; target Target::None;
            frame None; control None; INITIAL_DEFERRED_VISUAL
        ),
        trace_event!(
            7, Scheduler, Resize, Outcome::Published;
            captured Some(5); published Some(6); target Target::None;
            frame None; control None; RESIZED_VISUAL
        ),
        trace_event!(
            7, Projection, None, Outcome::Matched;
            captured None; published Some(6); target Target::None;
            frame None; control None; RESIZED_VISUAL
        ),
        trace_event!(
            8, Callback, Resize, Outcome::Deferred;
            captured Some(6); published None; target Target::None;
            frame None; control None; RESIZED_DEFERRED_VISUAL
        ),
        trace_event!(
            8, Scheduler, Resize, Outcome::NoChange;
            captured Some(6); published None; target Target::None;
            frame None; control None; RESIZED_VISUAL
        ),
        trace_event!(
            8, Input, FrameReady, Outcome::Accepted;
            captured None; published None; target Target::None;
            frame None; control None; RESIZED_VISUAL
        ),
        trace_event!(
            8, Scheduler, None, Outcome::Action;
            captured None; published None; target Target::None;
            frame Some(0); control None; RESIZED_VISUAL
        ),
        trace_event!(
            8, Renderer, None, Outcome::Accepted;
            captured None; published None; target Target::None;
            frame Some(0); control None; RESIZED_SUBMITTED
        ),
        trace_event!(
            9, Transaction, Direct, Outcome::Published;
            captured None; published Some(7); target Target::StaticControl;
            frame None; control None; HIDDEN_VISUAL_SUBMITTED
        ),
        trace_event!(
            9, Projection, None, Outcome::Matched;
            captured None; published Some(7); target Target::None;
            frame None; control None; HIDDEN_VISUAL_SUBMITTED
        ),
        trace_event!(
            9, Scheduler, None, Outcome::Action;
            captured None; published None; target Target::None;
            frame None; control None; HIDDEN_VISUAL_SUBMITTED
        ),
        trace_event!(
            9, Input, Pointer, Outcome::Observed;
            captured Some(7); published None; target Target::None;
            frame None; control None; HIDDEN_VISUAL_SUBMITTED
        ),
        trace_event!(
            10, Transaction, Direct, Outcome::Published;
            captured None; published Some(8); target Target::None;
            frame None; control None; HIDDEN_VISUAL_SUBMITTED
        ),
        trace_event!(
            10, Projection, None, Outcome::Matched;
            captured None; published Some(8); target Target::None;
            frame None; control None; HIDDEN_VISUAL_SUBMITTED
        ),
        trace_event!(
            10, Scheduler, None, Outcome::Action;
            captured None; published None; target Target::None;
            frame None; control None; HIDDEN_VISUAL_SUBMITTED
        ),
        trace_event!(
            11, Input, FrameReady, Outcome::Accepted;
            captured None; published None; target Target::None;
            frame None; control None; HIDDEN_VISUAL_SUBMITTED
        ),
        trace_event!(
            11, Scheduler, None, Outcome::Action;
            captured None; published None; target Target::None;
            frame Some(1); control None; HIDDEN_VISUAL_SUBMITTED
        ),
        trace_event!(
            11, Renderer, None, Outcome::Rejected;
            captured None; published None; target Target::None;
            frame Some(1); control None; HIDDEN_VISUAL_SUBMITTED
        ),
        trace_event!(
            11, Scheduler, None, Outcome::Action;
            captured None; published None; target Target::None;
            frame Some(2); control None; HIDDEN_VISUAL_SUBMITTED
        ),
        trace_event!(
            11, Renderer, None, Outcome::Accepted;
            captured None; published None; target Target::None;
            frame Some(2); control None; HIDDEN_TWO_SUBMITTED
        ),
        trace_event!(
            12, Renderer, Completion, Outcome::Completed;
            captured None; published None; target Target::None;
            frame None; control Some(0); HIDDEN_COMPLETION_PENDING
        ),
        trace_event!(
            13, Scheduler, Completion, Outcome::Completed;
            captured None; published None; target Target::None;
            frame None; control None; HIDDEN_ONE_SUBMITTED
        ),
        trace_event!(
            14, Transaction, Direct, Outcome::Published;
            captured None; published Some(9); target Target::None;
            frame None; control None; HIDDEN_VISUAL_SUBMITTED
        ),
        trace_event!(
            14, Projection, None, Outcome::Matched;
            captured None; published Some(9); target Target::None;
            frame None; control None; HIDDEN_VISUAL_SUBMITTED
        ),
        trace_event!(
            14, Scheduler, None, Outcome::Action;
            captured None; published None; target Target::None;
            frame None; control None; HIDDEN_VISUAL_SUBMITTED
        ),
        trace_event!(
            15, Input, FrameReady, Outcome::Accepted;
            captured None; published None; target Target::None;
            frame None; control None; HIDDEN_VISUAL_SUBMITTED
        ),
        trace_event!(
            15, Scheduler, None, Outcome::Action;
            captured None; published None; target Target::None;
            frame Some(3); control None; HIDDEN_VISUAL_SUBMITTED
        ),
        trace_event!(
            15, Renderer, Loss, Outcome::Lost;
            captured None; published None; target Target::None;
            frame Some(3); control Some(1); HIDDEN_LOSS_PENDING
        ),
        trace_event!(
            15, Input, Shutdown, Outcome::Accepted;
            captured None; published None; target Target::None;
            frame None; control Some(2); HIDDEN_TWO_CONTROLS
        ),
        trace_event!(
            15, Input, Shutdown, Outcome::NoChange;
            captured None; published None; target Target::None;
            frame None; control Some(2); HIDDEN_TWO_CONTROLS
        ),
        trace_event!(
            16, Scheduler, Loss, Outcome::Lost;
            captured None; published None; target Target::None;
            frame None; control None; HIDDEN_SHUTDOWN_PENDING
        ),
        trace_event!(
            17, Scheduler, Shutdown, Outcome::Action;
            captured None; published None; target Target::None;
            frame None; control Some(2); HIDDEN_ONE_SUBMITTED
        ),
        trace_event!(
            18, Renderer, Completion, Outcome::Completed;
            captured None; published None; target Target::None;
            frame None; control Some(3); HIDDEN_FINAL_COMPLETION
        ),
        trace_event!(
            19, Scheduler, Completion, Outcome::Stopped;
            captured None; published None; target Target::None;
            frame None; control None; HIDDEN_EMPTY
        ),
    ]
}
