# Native WU-0009 reference run

Status: active
Work unit: WU-0009
Parent design: [disposable native EXP-0001 spine](native-exp-0001-spine.md)
Last updated: 2026-08-09

## Purpose

This contract makes the single Fedora Wayland execution reproducible and
decidable without pretending that winit can portably synthesize native pointer,
scale, resize, or close events. It separates observed OS events from a fixed
script that enters the same bounded adapter reducer.

## Fixed script

The event loop uses one monotonic integer turn counter. Each reducer input takes
the next tick; all scheduler calls caused by that input retain that tick. Wall
clock is absent from the trace. A five-second monotonic watchdog is used only
to return `Timeout` if the expected callback never arrives.

1. Construct runtime generation 0 and its clean oracle at logical surface
   `0x0`, then on `resumed` create one opaque, unmapped window requesting
   logical `320x240`. Record the actual physical extent and fixed effective
   scale, then publish the resulting logical resize. The publication must be
   generation 1.
2. Drain `RequestFrame`, arm one redraw, and wait for its `RedrawRequested`.
   Frame 0 must carry runtime generation 1 and native surface generation 0.
   Rasterize, preflight, call `pre_present_notify`, accept submission 0, present,
   and complete control 0. The first buffer commit maps the Wayland window.
3. Feed one explicitly labeled scripted primary press at logical point `(5,5)`
   through the same reducer used by `WindowEvent::MouseInput`. It must capture
   generation 1 and target `static-control`. It performs no mutation. Separate
   pure tests prove the winit position/button mapping.
4. Request logical inner size `360x260`. If winit returns an immediate physical
   size, feed it directly; otherwise wait for `Resized`. Reduce duplicate or
   superseded surface events into one slot. The effective changed logical
   surface must publish generation 2 and native surface generation 1. If the
   retained physical tuple changes but its logical extent does not, record
   `SurfaceRepaintUnavailable` and `adapt`; do not enter milestone 5.
5. Arm the next redraw. Frame 1 must carry runtime generation 2 and native
   surface generation 1. Accept submission 1, present, and complete control 1.
6. Feed one explicitly labeled scripted close through the same path as
   `CloseRequested`. Admit shutdown control 2, observe `StopRenderer(2)`, reach
   `SchedulerState::Stopped`, then exit the event loop.

Initial effective scale is evidence. The run does not require a scale
transition because winit provides no portable request for moving to another
DPI. Pure reducer and raster tests cover scale inputs at 1.0, 1.25, 1.5, and
2.0. After the first tuple is accepted, a real effective
`ScaleFactorChanged` returns `EnvironmentScaleChanged`; the accepted tuple and
both generation sequences remain unchanged. The probe records `adapt` if it can
finish its bounded trace and shutdown, otherwise `stop`. It never shifts the
fixed milestones.

If the compositor refuses the second size, produces a post-initial effective
scale change, never produces the armed redraw, or cannot create/present the
surface, the run returns the corresponding typed result. It does not silently
change the expected sequence.

## Trace vocabulary

`NativeTraceStageV1` is closed and ordered:

```text
Manifest, Shell, Platform, Scheduler, Renderer, Oracle
```

`NativeObservationV1` is closed:

```text
Build, Resumed, Surface, Scale, Pointer, Redraw, Frame, Present,
Close, Completion, Shutdown, Timeout
```

`NativeOutcomeV1` is closed:

```text
Observed, Coalesced, Ignored, Deferred, Published, Armed, Offered,
Accepted, Rejected, Completed, Matched, Stopped, Failed(cause)
```

`NativeFailureCauseV1` priority is:

```text
InvalidScale, InvalidPoint, Arithmetic, WidthLimit, HeightLimit,
PixelLimit, ByteLimit, UnsupportedAlpha, Storage, EnvironmentScaleChanged,
SurfaceRepaintUnavailable, Runtime, Oracle, Scheduler, PrePresent,
Presenter, Trace, Timeout, Invariant
```

Candidate errors are mapped immediately into this vocabulary. No upstream
`Debug`, string, path, handle, or environment payload is retained.

## Event applicability

Every event stores sequence, tick, stage, observation, outcome, redraw armed,
pending-surface count, pending-pointer count, presenter-pending count, and the
four scheduler lane item/byte pairs.

- Native surface generation, physical extent, fixed scale, and logical extent
  appear together or are all absent.
- Captured runtime generation appears only on pointer and deferred surface
  callback observations.
- Published runtime generation appears only on actual publications and oracle
  matches.
- Target appears only for pointer observations.
- Frame appears on offer, accept, reject, present, and the corresponding
  renderer failure.
- Submission appears only after acceptance and through completion.
- Control appears only on accepted completion, renderer loss, shutdown, and
  `StopRenderer`.
- `redraw_armed` is true only between one `RequestFrame` and the matching
  `RedrawRequested`. A spontaneous or duplicate redraw stays false and cannot
  produce `FrameReady`.
- Pending reducer counts are each zero or one. Presenter pending is zero or one.
- Environment-scale and native-only surface failures carry the observed surface
  tuple but no frame, submission, control, or presentation event. They cannot
  produce `FrameReady`, allocate a private frame identity, or bypass scheduler
  admission.

## Scheduler correlation

Each call to `begin_callback`, `next_action`, or `process_input` creates one
same-turn scheduler-stage event. Scheduler events carry a dense
`scheduler_turn` field; non-scheduler events carry none. The event is derived
from the typed call result and immediate post-state, not reconstructed later.

The scheduler projection matches `SchedulerTraceV1` fields for lifecycle,
runtime generation, frame, control, and all lane counts. The reference run
requires exact action/input progression:

```text
resize callback deferred
RequestFrame
FrameReady
OfferFrame(0)
AcceptFrame(0) -> submission 0
Complete(0) -> control 0
process completion 0
resize callback deferred
RequestFrame
FrameReady
OfferFrame(1)
AcceptFrame(1) -> submission 1
Complete(1) -> control 1
process completion 1
RequestShutdown -> control 2
StopRenderer(2)
```

The pointer query adds no scheduler turn because it is read-only. The trace
must end with runtime generation 2, scheduler state `Stopped`, all four lanes
empty, no redraw armed, no reducer slot occupied, and no presenter frame.

## Native-only and failure paths

After the initial tuple, an effective scale change is terminal
`EnvironmentScaleChanged`; it is never folded into the fixed run. A resized
tuple whose physical extent changes while logical extent does not is retained
with its next native surface generation and classified `NativeOnly`. Because
the current scheduler exposes no same-generation repaint admission, the driver
then returns terminal `SurfaceRepaintUnavailable` and records `adapt`. Neither
path creates a frame, calls `FrameReady`, or presents pixels.

Before scheduler acceptance, allocation, raster, buffer acquisition, or copy
failure rejects an outstanding scheduler frame when one exists. After
acceptance, `pre_present_notify` is complete and `present` owns the attempt;
failure admits renderer loss and never retries the ambiguous frame.

The fault presenter and a slow gate exercise both sides without provoking host
failures. Slow evidence retains exactly one CPU frame and one scheduler offer,
never exceeds configured residence, and releases both before the next input.

## Exit assertions

The native command succeeds only if:

- every fixed script milestone occurs before the watchdog;
- both frame generations and surface generations match this contract;
- no post-initial scale change or native-only resize entered a frame milestone;
- staging digests are recorded before presentation;
- the oracle matches after generations 1 and 2;
- close reaches stopped state with empty owned queues;
- the bounded ASCII trace is emitted with LF endings and stderr remains empty.
