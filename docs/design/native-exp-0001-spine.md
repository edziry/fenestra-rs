# Disposable native EXP-0001 spine

Status: active
Work unit: WU-0009
Branch: `experiment/native-spine`
Research baseline: `fenestra-research` commit
`176c42139776ed9f1ef879cd135bddadaf12a9da`
Last updated: 2026-08-09
Run and trace contract: [native WU-0009 reference run](native-exp-0001-run.md)

## Goal

WU-0009 connects the existing bounded scheduler and committed headless scene to
one real window and one real CPU presentation surface. It replaces only the
fake platform and fake renderer at the probe boundary. It does not select a
permanent native shell, renderer, platform support matrix, or public API.

The unit exits with one recorded result:

- `pass`: the named native environment exercised the complete replaceable seam;
- `adapt`: the seam remained valid but the candidate or contract needed a
  bounded documented change;
- `stop`: the candidate could not satisfy an ownership, correctness, safety, or
  reproducibility constraint.

This result is local to WU-0009. The pinned
[EXP-0001](https://github.com/edziry/fenestra-research/blob/176c42139776ed9f1ef879cd135bddadaf12a9da/init/experiments/EXP-0001-feasibility-spine.md)
remains proposed until its owners, environments, hardware, corpus, budgets, and
decision thresholds are ratified and executed.

## Definition of ready

The implementation may begin because this document fixes:

- the owned probe boundary and dependency direction;
- exact candidate versions, features, licenses, and known MSRV metadata;
- one named native reference environment and separate portability checks;
- platform, renderer, input, resize, scale, close, and failure semantics;
- bounded state, trace, privacy, and ownership rules;
- the correctness oracle, TDD sequence, replacement boundary, and exit checks.

The separate research repository remains the source of product research. This
document records only the implementation decision derived from that immutable
baseline and current primary candidate sources.

## Candidate screen

### Window and event loop

`winit 0.30.13` is the admitted candidate. It is the current stable release
screened for this unit; the newer `0.31` line is still beta. Its crate metadata
declares Apache-2.0 and Rust 1.70.0. Its low-level event-loop ownership, raw
window handle integration, and distinct drawing responsibility fit the seam.

Primary sources:

- [winit 0.30.13 documentation](https://docs.rs/winit/0.30.13/winit/);
- [winit 0.30.13 manifest](https://github.com/rust-windowing/winit/blob/v0.30.13/Cargo.toml);
- [winit platform matrix](https://github.com/rust-windowing/winit/blob/v0.30.13/FEATURES.md);
- [winit 0.30.13 release](https://github.com/rust-windowing/winit/releases/tag/v0.30.13).

`tao 0.36.0` was rejected for this unit because its Linux path brings GTK and
GDK responsibilities into the shell seam. `minifb 0.28.0` was rejected because
it couples polling, input, windowing, and framebuffer presentation and lacks an
equivalent explicit scale-event contract. Neither rejection is a permanent
project decision.

### Presentation

`softbuffer 0.4.8` is the admitted presenter. Its crate metadata declares
MIT OR Apache-2.0 and Rust 1.71.0. It consumes raw-window-handle 0.6 through
safe `Context` and `Surface` APIs and accepts an opaque CPU pixel buffer. The
probe writes no `unsafe` code; dependency-internal native code remains part of
the admitted dependency surface.

Primary sources:

- [softbuffer 0.4.8 documentation](https://docs.rs/softbuffer/0.4.8/softbuffer/);
- [softbuffer 0.4.8 manifest](https://github.com/rust-windowing/softbuffer/blob/v0.4.8/Cargo.toml);
- [softbuffer 0.4.8 release](https://github.com/rust-windowing/softbuffer/releases/tag/v0.4.8).

`pixels 0.17.2`, `wgpu 30.0.0`, and `Vello 0.9.0` were screened but deferred.
They introduce adapter, device, queue, shader, compute, and surface ownership
that WU-0009 does not need to validate the native seam. They remain candidates
for a later renderer experiment.

Softbuffer does not prove owned-surface export, GPU interoperability, damage
tracking, or compositor completion. The renderer input therefore remains the
candidate-neutral committed rectangle scene. EXP-0002 remains open.

### Maintenance and security review

The 2026-08-09 screen checked the exact crate manifests, tagged releases,
upstream repositories, changelogs, and package names in the RustSec advisory
index. No direct RustSec entry for `winit` or `softbuffer` was located. This is
not a complete transitive security audit or a future assurance.

Softbuffer's changelog records native safety fixes, including the older 0.4.1
macOS window double-free correction. Version 0.4.8 contains that fix, but the
history confirms that native dependency code remains part of the safety and
upgrade review. Winit and Softbuffer both have recent tagged activity under the
`rust-windowing` organization. Exact pins prevent silent candidate drift;
updates require a new dependency record, lockfile review, and native rerun.

## Exact dependency admission

The new unpublished package is:

```text
probes/exp-0001-native-spine
```

It depends on `fenestra-ui-runtime` and `fenestra-ui-testkit`. Product crates do
not depend on the probe. Candidate types do not enter runtime, IR, testkit, or
facade APIs.

All external versions are exact and default features are disabled. The named
Linux reference enables only the Wayland route:

```toml
winit = { version = "=0.30.13", default-features = false,
          features = ["rwh_06", "wayland", "wayland-dlopen"] }
softbuffer = { version = "=0.4.8", default-features = false,
               features = ["wayland", "wayland-dlopen"] }
```

The Windows compile route enables only `winit` raw-window-handle support and
the target-native Softbuffer backend:

```toml
winit = { version = "=0.30.13", default-features = false,
          features = ["rwh_06"] }
softbuffer = { version = "=0.4.8", default-features = false }
```

KMS, X11, client-side decoration, GPU, text, image, accessibility, and hosted
control dependencies are omitted. A later X11 probe must be a separately named
environment and feature decision, not an implicit fallback.

The Fedora route requires the Wayland client libraries available to the named
desktop; `wayland-dlopen` avoids turning that client library into an accidental
static distribution promise. The Windows route uses target-native Win32
dependencies. No installer, redistribution, static-link, or package-signing
claim is made by this unpublished probe.

## Reference environments

The native execution environment for this unit is the local Fedora 43 Wayland
desktop. The environment record must include only closed or numeric facts:

- OS family and target triple;
- effective window system;
- exact candidate versions and enabled feature sets;
- physical and logical surface dimensions;
- normalized scale factor;
- requested, detected, and effective closed capabilities.

The Windows machine reachable through the authorized SSH host is used for a
fresh native-target build and pure contract tests. An OpenSSH service session
does not by itself prove an interactive Win32 desktop. A Win32 presentation run
may be recorded only from an interactive desktop session and must be labeled as
that exact environment.

Hosted CI continues to prove compilation and deterministic tests, not native
window or compositor support.

## Ownership and replacement boundary

The probe contains five responsibilities:

```text
winit shell
  -> bounded native observation reducer
  -> existing runtime scheduler
  -> candidate-neutral rectangle rasterizer
  -> softbuffer presenter
```

- The winit shell uniquely owns `EventLoop` and creates one `Window` from
  `ApplicationHandler::resumed` on the calling thread.
- The shell owns `Context<OwnedDisplayHandle>` and one `Arc<Window>`.
  `Surface<OwnedDisplayHandle, Arc<Window>>` owns another window reference and
  is explicitly dropped before the shell's final window reference.
- `OwnedDisplayHandle` and `Arc<Window>` pass through safe handle traits. The
  probe does not extract, store, format, or export raw native handles.
- Runtime snapshots and frame work remain immutable and owned by the existing
  scheduler. No window, surface, buffer, or candidate error enters them.
- `NativeSurfaceGenerationV1` changes on each effective physical resize or
  scale change and remains independent from `RuntimeGeneration`.
- The presenter consumes a borrowed normalized scene and caller-owned staging
  pixels. It retains no runtime snapshot after admission or rejection.
- The private presenter port exists only for deterministic failure injection.

Replacing winit changes the shell and observation mapping. Replacing
Softbuffer changes the presenter. Neither replacement changes the runtime
scheduler, scene projection, trace vocabulary, or correctness oracle.

## Native event contract

The event loop uses `ControlFlow::Wait`. Winit may coalesce redraw requests, so
the trace records observed transitions rather than assuming callback counts or
latency.

### Surface and scale

- Winit physical sizes and positions are normalized before runtime use.
- Logical point coordinates use `floor(physical / scale)`.
- Logical extents use `ceil(physical / scale)`.
- Scale schema V1 stores `scale_micros: u32`: 1.0 is 1,000,000, input is
  rounded to the nearest micro, and the accepted range is 1 through 8,000,000.
- Logical scene rectangles scale their edges rather than their widths in
  isolation. Physical left/top use mathematical floor of `edge * scale`;
  right/bottom use mathematical ceil, followed by half-open physical clipping.
  Signed checked arithmetic defines negative origins and fractional scales.
- Scale must be finite, positive, and representable in the versioned fixed
  normalization. Invalid values fail before a transaction is staged.
- Physical zero width or height suspends allocation and presentation. The
  runtime surface becomes `0x0`; a later nonzero size restores rendering.
- Each effective observation creates one immutable tuple containing native
  surface generation, physical extent, fixed scale, and logical extent.
- `ScaleFactorChanged` samples current inner size; `Resized` samples current
  scale. Either ordering reduces into the same one-slot latest tuple.
- The first accepted tuple fixes the reference-run scale. Any later effective
  `ScaleFactorChanged` ends the run with `EnvironmentScaleChanged`; it does not
  advance either generation or displace a fixed milestone.
- Resize observations retain only the latest effective tuple. A changed logical
  extent publishes through the existing deferred callback transaction. A
  physical change with the same logical extent is retained and classified
  atomically, then ends WU-0009 with `SurfaceRepaintUnavailable` and `adapt`.
  This driver neither invents a `FrameId` nor bypasses `UiScheduler` to repaint.

### Pointer input

- Cursor movement updates one coalescible last physical point.
- A primary-button press captures the last committed projection and the native
  surface tuple paired with it. It converts with that tuple's scale and
  hit-tests the same runtime generation. A detected pending-resize tuple does
  not enter input until its logical resize publishes atomically.
- The trace stores only the closed target `none`, `static-control`, or keyed
  member. It stores no `NodeId`, pointer address, user text, or native handle.
- Input during a pending resize observes the prior committed generation until
  the deferred resize publishes atomically.

### Redraw and presentation

- One scheduler `RequestFrame` arms at most one native `request_redraw`.
- Only the corresponding armed `RedrawRequested` produces `FrameReady`.
  Spontaneous or duplicate redraws are observed without falsifying feedback.
- `OfferFrame` must carry a snapshot and headless projection with the same
  generation.
- The rasterizer clears the whole buffer, then emits authored-order clipped
  rectangles into `0x00RRGGBB` pixels. Clear is `0x00000000`; V1 accepts only
  opaque scene colors.
- After staging pixels, the shell calls `Window::pre_present_notify()` before
  Softbuffer presentation so Wayland can schedule its frame callback.
- Allocation, full rasterization, and presenter preflight happen before
  scheduler `AcceptFrame`. Once accepted, the presenter consumes the buffer.
- A successful Softbuffer `present` is followed by same-tick `Complete`. This
  proves CPU buffer retirement, not display scanout.
- Failure before acceptance rejects the outstanding offer. A failure after
  acceptance reports renderer loss rather than an ambiguous retry.

### Close and shutdown

- `CloseRequested` admits one idempotent shutdown control.
- New visual work is not accepted after shutdown admission.
- The shell exits only after `StopRenderer` is observed and scheduler state is
  `Stopped`.

## Bounds and trace

No unbounded platform queue is added. Winit callbacks are reduced
synchronously into the existing bounded scheduler. Coalescible pointer,
surface, and scale state each occupy one slot; close and renderer controls are
non-droppable typed transitions.

`NativeTraceV1` uses schema version 1 with an inclusive capacity of 128 events
and 24,576 protocol-accounted bytes at 192 bytes per event. Preflight order is
events, accounted bytes, then storage reservation. Each event carries:

- dense sequence and caller-supplied monotonic tick;
- native stage and closed observation/outcome;
- captured and published runtime generation when applicable;
- native surface generation, physical size, logical surface, and normalized
  scale when applicable;
- closed pointer target, frame, submission, and control identities when
  applicable;
- existing scheduler lane item/byte counts, redraw-arm state, pending reducer
  slots, and pending presenter state.

Exact enums, applicability, error priority, and same-turn scheduler correlation
are fixed in the [reference run contract](native-exp-0001-run.md).

One CPU frame may be pending. Correctness ceilings are 4,096 pixels per axis,
16,777,216 pixels, and 67,108,864 staging bytes. They are allocation guards,
not product budgets. Checked width, height, pixel, byte, and storage preflight
occurs before allocation. Native surface generation, runtime generation, and
frame identity are validated before presentation.
Storage reservation failure maps to the closed
`NativeFailureCauseV1::Storage` category after all numeric preflights pass.

The manifest and trace contain no hostname, username, environment-variable
value, path, wall clock, native handle, pixels, source text, clipboard data, or
candidate debug string. Candidate failures map into closed probe errors.

## Correctness oracle

After every runtime publication, the probe observes the committed headless
projection with the existing `HeadlessOracleV1` and requires exact equality.
The rasterizer is separately checked against manual pixel buffers and authored
order. Its stable digest is FNV-1a-64 with offset
`0xcbf29ce484222325` and prime `0x00000100000001b3`, consuming every pixel as
explicit `u32::to_le_bytes()`. It never hashes native `Vec` memory. A digest may
identify a staging buffer in a named run, but it is not a security or
display-integrity claim.

## TDD sequence

1. Dependency admission tests fail until exact packages, target features,
   publication locks, and dependency isolation are present.
2. Reducer tests cover duplicate resume, resize/scale ordering, 1.0/1.25/2.0
   conversions, invalid scales, overflow, zero size, recovery, coalescible
   cursor state, terminal environment-scale change, terminal native-only
   resize, and non-droppable close.
3. Scheduler tests cover prior-generation pointer capture, resize publication,
   redraw arming, duplicate redraw, storms, retained work, and repeated close.
4. Rasterizer tests cover clear-before-draw, half-open clipping, authored
   overlap, exact pixel packing, scale, zero size, and arithmetic failures.
5. Presenter-port tests inject resize, pre-present, post-present, slow, loss,
   and stop outcomes and prove accept/reject/completion and rollback behavior.
6. Trace tests cover inclusivity, overflow priority, dense sequence, privacy,
   deterministic scripted runs, and no retained snapshots or native objects.
7. The native process follows the exact bounded
   [reference run](native-exp-0001-run.md). On Wayland the window begins
   unmapped and the first buffer commit maps it; no unsupported visibility
   transition is requested.

## Verification

The unit requires:

- focused reducer, driver, rasterizer, trace, dependency, and process tests;
- workspace tests, format, Clippy with warnings denied, and rustdoc with missing
  documentation denied;
- Cargo metadata, feature tree, lockfile, ASCII, diff, and file-size audits;
- Linux and Windows all-target/all-feature builds and pure tests;
- one fresh native run on the named Fedora Wayland environment;
- independent source and evidence audit before squash integration.

## Non-goals

WU-0009 does not establish product support for Linux, Wayland, Windows, X11,
XWayland, mobile, or any compositor. It does not select a final shell,
renderer, GPU stack, layout engine, text engine, animation model, input model,
accessibility layer, hosted-control strategy, transparency path, special-window
behavior, damage algorithm, export ABI, or performance budget. Continuing after
an effective scale transition and repainting a native-only surface generation
without scheduler work remain explicit later decisions.

The project license, package publication, namespace reservation, final MSRV,
numeric product budgets, release support cadence, and full EXP-0001 governance
remain pending.

## Exit criteria

WU-0009 is complete only when the versioned verification record contains:

- the exact admitted dependency graph and replacement boundary;
- passing pure and cross-platform build evidence;
- the named native environment manifest and bounded correlated trace;
- pointer, resize, scale, redraw, close, slow, rejection, loss, and shutdown
  evidence;
- oracle equality after every publication and empty terminal owned queues;
- one explicit `pass`, `adapt`, or `stop` result;
- explicit remaining platform, budget, and feasibility nonclaims.
