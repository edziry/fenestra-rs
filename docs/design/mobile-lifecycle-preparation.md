# WU-0012 mobile lifecycle preparation

Status: decision complete; implementation deferred
Decision date: 2026-08-10
Depends on: [WU-0011 layout plan](layout-conformance.md)

## Decision

WU-0012 is not a prerequisite for WU-0011. Its assumption inventory is complete
enough to constrain the layout boundary, but lifecycle implementation remains a
separate later work unit.

WU-0011 consumes only a present logical available extent and owns no surface,
density, lifecycle, platform thread-affinity, or resource-ownership vocabulary.
Zero extent is geometry, not suspension. The evidence neither implements nor
claims mobile support. Any trigger listed below stops WU-0011 and makes WU-0012
a prerequisite.

## Boundary required from WU-0011

- Input extent is logical, representable, nonnegative, and allows either axis
  to be zero.
- `0xN`, `Nx0`, and `0x0` are present degenerate viewports. They never encode
  background, app suspension, surface loss, or absence.
- WU-0011 reserves presence for a later outer coordinator; it does not implement
  that coordinator. A caller without presentation skips layout instead of
  fabricating a zero viewport and may retain the last immutable generation.
- Density, physical pixels, safe area, orientation, keyboard bounds, window and
  surface handles, lifecycle events, epochs, and thread ownership are absent.
- Physical scale is structurally absent from the layout input and artifact.
  Equal logical viewport and node constraints produce the same deterministic
  output, but WU-0011 does not execute labeled scale contexts or test physical
  conversion. That evidence belongs to a later platform work unit.
- The WU-0011 adapter retains no candidate cache. A later cache is allowed only
  as semantically invisible performance state whose purge cannot change output.
- Geometry, hit testing, and scene data correlate through runtime generation,
  not a platform surface epoch.

## Current assumption inventory

### Runtime and scheduler

The generic runtime accepts nonnegative logical surfaces and rebuilds from a
local `(0,0)` origin. It has no platform lifecycle state. Resize invalidates
surface, layout, semantics, hit testing, paint, and composition.

The scheduler has `Running`, `ShutdownQueued`, `Draining`, `Stopped`, and
`Faulted` states. It has no paused or background state, no surface epoch, and no
recovery path after renderer loss. Residence uses caller-supplied monotonic
ticks, and the native runner currently advances them by operation. No freeze,
advance, or rebase policy exists for lifecycle; a future wall-time clock must
not turn background time into a false residence failure.

### Native shell and ownership

The WU-0009 runner creates one window, context, surface, presenter, and driver
from `resumed`. It does not implement `ApplicationHandler::suspended` or
`memory_warning`. Its current `Suspended` and `Restored` trace words describe a
zero/nonzero logical-size transition, not app lifecycle.

The presenter permanently owns the surface, context, and `Arc<Window>` for the
run. A present failure becomes terminal renderer loss; there is no detach,
surface destruction, recreation, or resource epoch while preserving the
runtime. The existing native surface generation numbers accepted logical
resize state; it is not a recreated-resource epoch.

The runner also expects `run_app` to return before it extracts final evidence;
winit documents that this call does not return on iOS. That desktop-shaped
process boundary is outside WU-0011 and must be replaced before an iOS claim.

Scale is fixed after initial acceptance. Later environment scale change is a
typed terminal refusal. Pointer conversion floors physical coordinates into the
accepted logical space. This is desktop candidate behavior, not a mobile rule.
The current native adapter also collapses either zero physical axis to logical
`(0,0)`, so per-axis `0xN` and `Nx0` coverage belongs to pure layout tests, not
to a shell preservation claim.

### Input, memory, and targets

The native map covers cursor motion, primary mouse press, resize, scale,
redraw, and close. It has no touch IDs, move/up/cancel lifecycle, focus,
occlusion, virtual keyboard, or IME.

There is no background quiescence, memory-warning callback, cache eviction, or
heap budget. The watchdog uses a worker thread and `Instant`; app suspension is
not modeled and could otherwise appear as a timeout.

The native dependency lanes are Linux and Windows only. Global IDs use
`AtomicU64`; 32-bit and mobile target atomic availability remains an explicit
audit item.

## Platform evidence

[winit 0.30.13 ApplicationHandler](https://docs.rs/winit/0.30.13/winit/application/trait.ApplicationHandler.html)
documents redundant resume/suspend callbacks, Android surface availability and
destruction, iOS active/inactive mapping, and mobile memory warnings.
[WindowEvent](https://docs.rs/winit/0.30.13/winit/event/enum.WindowEvent.html)
separates touch, occlusion, scale, and resize events. The
[DPI model](https://docs.rs/winit/0.30.13/winit/dpi/) separates logical and
physical coordinates.

Android's
[SurfaceView contract](https://developer.android.com/reference/android/view/SurfaceView)
has explicit surface creation and destruction. Apple's
[app lifecycle](https://developer.apple.com/documentation/uikit/managing-your-app-s-life-cycle)
and
[memory warning guidance](https://developer.apple.com/documentation/uikit/responding-to-memory-warnings)
separate application and scene activity from reclaimable memory. Winit's
[event-loop builder](https://docs.rs/winit/0.30.13/winit/event_loop/struct.EventLoopBuilder.html)
and [window contract](https://docs.rs/winit/0.30.13/winit/window/struct.Window.html)
also constrain platform-resource ownership without implying one window. These
primary contracts keep Android presentation lifetime, activity, size, and
memory pressure as distinct concerns and make a zero-extent alias for absence
incorrect.

The complete later scope is anchored by the immutable
[EXP-0009 mobile-awareness audit](https://github.com/edziry/fenestra-research/blob/176c42139776ed9f1ef879cd135bddadaf12a9da/init/experiments/EXP-0009-mobile-awareness-audit.md).
Android's
[activity lifecycle](https://developer.android.com/guide/components/activities/activity-lifecycle)
also documents visible paused states, process death, and state restoration.
Winit `resumed` and `suspended` feed presentation availability on Android but
activity on iOS; they must not collapse into one portable lifecycle state.

## Stop-the-line triggers

WU-0012 becomes a prerequisite before WU-0011 can finish if any of these occur:

1. layout input requires physical scale, winit, window, surface, handle, thread,
   platform, or ownership types;
2. layout must distinguish absent from zero internally;
3. safe area, orientation, or keyboard insets are required by the corpus;
4. surface loss, resume, or scheduler pause is needed to prove geometry;
5. memory warning or candidate cache purge changes output;
6. intrinsic measurement requires a native or surface-bound resource;
7. geometry identity is tied to a surface epoch;
8. global or multi-window coordinates enter the boundary;
9. a 32-bit or mobile target claim requires unresolved atomic behavior; or
10. any WU-0011 result is described as mobile compilation, lifecycle, safe-area,
    input correctness, or support.

No trigger is present in the documented WU-0011 stack contract.

## Later WU-0012 design seed

WU-0012 should model three independent axes rather than one overloaded state:

- activity keyed by an application-or-scene scope: cold, foreground active,
  foreground inactive, or background; process lifetime and unannounced death
  remain separate;
- presentation availability: absent or available with a typed surface epoch,
  physical extent, logical extent, and scale;
- memory pressure: normal or warning.

The later replacement boundary should provide a fake `LifecyclePort`, a
detachable and recreatable presenter, `SurfaceEpoch`, and a
`ResourcePressurePort`. The runtime logical generation must survive surface
loss. Recreated presentation must request a fresh frame even when the runtime
generation is unchanged.

This seed is a minimum lifecycle subset, not the complete mobile work unit. The
deterministic fake corpus must cover duplicate resume and suspend,
available/lost/recreated surfaces, zero and nonzero resize, scale change,
foreground/background, memory warning, multi-ID touch down/move/up/cancel,
device loss, capability and permission change, and shutdown during each state.
It must mutate logical state while presentation is absent and resume with both
unchanged and changed viewports. The full WU-0012 plan must cover at least
gesture arbitration, stylus input, focus, IME and accessibility without mouse,
orientation, safe area, virtual keyboard and visible area, reduced GPU budget,
constrained filesystem and services, AOT packaging and dynamic-loading
assumptions, and cold restart after an unannounced prior process death.

Required invariants include idempotent callbacks; on Android, dropping render
surfaces derived from `SurfaceView` before `ApplicationHandler::suspended`
returns; absent distinct from zero; stale epoch work rejection; gesture
cancellation; quiesced visual, frame, and animation demand with explicit
freeze, advance, or rebase policy per clock domain; reconstructible cache and
resource eviction without changing logical correctness; one event-loop owner
for platform resources without assuming one window or scene; and distinct
surface-loss and device-loss paths.

## Version and result

This document changes no executable contract and does not advance the package
line. WU-0012 will make its own version decision under
[versioning-policy.md](../versioning-policy.md). In particular, changing the
meaning of zero extent, scheduler ownership, or renderer-loss behavior would be
an incompatible pre-1.0 change and advance `MINOR` without a shim.

```text
WU-0012 preparation result: go after WU-0011
WU-0012 implementation status: deferred
mobile support status: not claimed
```
