use fenestra_ui_runtime::prototype::HeadlessSurface;
use fenestra_ui_testkit::prototype::{HeadlessTraceEventV1, HeadlessTraceQueueStatsV1};

const ZERO: [usize; 2] = [0, 0];
const DEFERRED: [usize; 2] = [1, 80];
const CONTROL_ONE: [usize; 2] = [1, 32];
const CONTROL_TWO: [usize; 2] = [2, 64];
const VISUAL: [usize; 2] = [1, 40];
const IN_FLIGHT_ONE: [usize; 2] = [1, 40];
const IN_FLIGHT_TWO: [usize; 2] = [2, 80];
const RENDERER_ONE: [usize; 2] = [1, 96];
const RENDERER_TWO: [usize; 2] = [2, 192];
const BASE_COUNTS: [usize; 5] = [5, 5, 1, 3, 5];
const INSERTED_COUNTS: [usize; 5] = [6, 6, 1, 4, 6];
const HIDDEN_COUNTS: [usize; 5] = [5, 5, 0, 2, 4];

pub(super) const INITIAL_EMPTY: State = state(120, 90, BASE_COUNTS, ZERO, ZERO, ZERO, ZERO, ZERO);
pub(super) const INITIAL_DEFERRED: State =
    state(120, 90, BASE_COUNTS, DEFERRED, ZERO, ZERO, ZERO, ZERO);
pub(super) const INITIAL_VISUAL: State =
    state(120, 90, BASE_COUNTS, ZERO, ZERO, VISUAL, ZERO, ZERO);
pub(super) const INSERTED_VISUAL: State =
    state(120, 90, INSERTED_COUNTS, ZERO, ZERO, VISUAL, ZERO, ZERO);
pub(super) const INITIAL_DEFERRED_VISUAL: State =
    state(120, 90, BASE_COUNTS, DEFERRED, ZERO, VISUAL, ZERO, ZERO);
pub(super) const RESIZED_VISUAL: State = state(90, 70, BASE_COUNTS, ZERO, ZERO, VISUAL, ZERO, ZERO);
pub(super) const RESIZED_DEFERRED_VISUAL: State =
    state(90, 70, BASE_COUNTS, DEFERRED, ZERO, VISUAL, ZERO, ZERO);
pub(super) const RESIZED_SUBMITTED: State = state(
    90,
    70,
    BASE_COUNTS,
    ZERO,
    ZERO,
    ZERO,
    IN_FLIGHT_ONE,
    RENDERER_ONE,
);
pub(super) const HIDDEN_VISUAL_SUBMITTED: State = state(
    90,
    70,
    HIDDEN_COUNTS,
    ZERO,
    ZERO,
    VISUAL,
    IN_FLIGHT_ONE,
    RENDERER_ONE,
);
pub(super) const HIDDEN_TWO_SUBMITTED: State = state(
    90,
    70,
    HIDDEN_COUNTS,
    ZERO,
    ZERO,
    ZERO,
    IN_FLIGHT_TWO,
    RENDERER_TWO,
);
pub(super) const HIDDEN_COMPLETION_PENDING: State = state(
    90,
    70,
    HIDDEN_COUNTS,
    ZERO,
    CONTROL_ONE,
    ZERO,
    IN_FLIGHT_TWO,
    RENDERER_ONE,
);
pub(super) const HIDDEN_ONE_SUBMITTED: State = state(
    90,
    70,
    HIDDEN_COUNTS,
    ZERO,
    ZERO,
    ZERO,
    IN_FLIGHT_ONE,
    RENDERER_ONE,
);
pub(super) const HIDDEN_LOSS_PENDING: State = state(
    90,
    70,
    HIDDEN_COUNTS,
    ZERO,
    CONTROL_ONE,
    VISUAL,
    IN_FLIGHT_ONE,
    RENDERER_ONE,
);
pub(super) const HIDDEN_TWO_CONTROLS: State = state(
    90,
    70,
    HIDDEN_COUNTS,
    ZERO,
    CONTROL_TWO,
    VISUAL,
    IN_FLIGHT_ONE,
    RENDERER_ONE,
);
pub(super) const HIDDEN_SHUTDOWN_PENDING: State = state(
    90,
    70,
    HIDDEN_COUNTS,
    ZERO,
    CONTROL_ONE,
    ZERO,
    IN_FLIGHT_ONE,
    RENDERER_ONE,
);
pub(super) const HIDDEN_FINAL_COMPLETION: State = state(
    90,
    70,
    HIDDEN_COUNTS,
    ZERO,
    CONTROL_ONE,
    ZERO,
    IN_FLIGHT_ONE,
    ZERO,
);
pub(super) const HIDDEN_EMPTY: State = state(90, 70, HIDDEN_COUNTS, ZERO, ZERO, ZERO, ZERO, ZERO);

#[derive(Clone, Copy)]
pub(super) struct State {
    surface: HeadlessSurface,
    counts: [usize; 5],
    deferred: [usize; 2],
    controls: [usize; 2],
    visual: [usize; 2],
    in_flight: [usize; 2],
    renderer: [usize; 2],
}

#[allow(clippy::too_many_arguments)]
const fn state(
    width: i32,
    height: i32,
    counts: [usize; 5],
    deferred: [usize; 2],
    controls: [usize; 2],
    visual: [usize; 2],
    in_flight: [usize; 2],
    renderer: [usize; 2],
) -> State {
    State {
        surface: HeadlessSurface::new(width, height),
        counts,
        deferred,
        controls,
        visual,
        in_flight,
        renderer,
    }
}

pub(super) fn assert_state(event: HeadlessTraceEventV1, expected: State) {
    assert_eq!(event.surface(), expected.surface);
    let counts = event.projection_counts();
    assert_eq!(
        [
            counts.computed_styles(),
            counts.geometries(),
            counts.semantics(),
            counts.hit_regions(),
            counts.scene_rectangles(),
        ],
        expected.counts
    );
    assert_queue(event.deferred(), expected.deferred);
    assert_queue(event.controls(), expected.controls);
    assert_queue(event.visual(), expected.visual);
    assert_queue(event.in_flight(), expected.in_flight);
    assert_eq!(
        [event.renderer().items(), event.renderer().accounted_bytes()],
        expected.renderer
    );
}

fn assert_queue(stats: HeadlessTraceQueueStatsV1, expected: [usize; 2]) {
    assert_eq!([stats.items(), stats.accounted_bytes()], expected);
}
