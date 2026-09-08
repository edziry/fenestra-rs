# Fenestra completion goal

Status: active; product completion is not yet achieved
Baseline: `a32025e14ceb9fdc88c385732437cf2b8d4ac344`, workspace `0.2.0`
Audit date: 2026-09-08

Latest checkpoint: [application handoff, 2026-09-08](handoffs/2026-09-08-application-checkpoint.md).
Product work stops at this requested handoff; the goal remains incomplete.

## Intended outcome

Deliver the native Rust UI framework described in the [README](../README.md):
practical typed component and styling authoring, application-owned state,
native execution on Windows and Linux, and the advanced window and graphics
capabilities promised there. Keep browser, WebView, JavaScript, media capture,
encoding, and transport outside the framework core.

The governing scope and research remain those of the
[initial implementation plan](initial-implementation-plan.md). Its boundaries
are sufficient for the development-loop repairs below. New language, platform,
text, and release decisions require their own versioned design and evidence.

## Audited baseline before WU-0016 and WU-0017

| Area | Evidence available | Remaining gap |
| --- | --- | --- |
| Kernel | Typed IR, identity, transactions, scheduler, replay and reconstruction oracles | Product-facing application lifecycle and API |
| Authoring | Equivalent format-1/2 `.fen` and `ui!`, canonical Rust and source maps | Component syntax, bindings, events, imports and practical diagnostics |
| Layout and graphics | Bounded layout, free placement, transforms, clips, paths, images and reference raster | General application scale, production renderer and measured incremental work |
| Application | WU-0015 inspector and registered Windows artifact | Content beyond fixtures, node/property UX, focus and robust repeated input |
| Native platform | Windows DX12 probe; Windows CPU inspector; Linux Wayland shell | Current Linux execution evidence, X11 path and qualified capability matrix |
| Public package | Workspace packages and pinned toolchain | `fenestra-ui` facade is empty; no release license, MSRV or packaging commitment |
| Normal UI | Spatial hit and semantic records | Text shaping, keyboard, IME, native accessibility, controls and scrolling |

The audited baseline passed 2,155 tests across 174 test suites with no ignored
tests using `cargo test --workspace --all-targets --all-features --locked`.
That is a regression baseline, not proof of complete product behavior.

Relevant versioned records:

- [Windows GPU spine](verification/WU-0014-windows-interactive-gpu-spine.md)
- [First usable application](verification/WU-0015-first-usable-application.md)
- [Inspector and authoring loop](design/inspector-authoring-loop.md)
- [Format-2 source contract](design/hybrid-spatial-authoring-source-v2.md)

## Ordered acceptance gates

### 1. Reproducible development and execution loop

- [x] Document one working path from clone to validation, headless run and
  native window, including the exact toolchain and platform prerequisites.
- [x] Supply a small `.fen`/`ui!` example with equivalent programs, observable
  pixels, pointer selection, repeated keyed insertion and resize tests.
- [x] Expose compiler diagnostics with a source path, line, byte column,
  diagnostic category and underlying typed validation reason.
- [x] Keep idle native windows idle, accept repeated supported input, and
  avoid presentation at zero size.
- [x] Preserve the registered native evidence while executing the current
  native shell on the available host.
- [x] Pass formatting, tests, Clippy, documentation, manifest and dependency
  checks; record exact native evidence limits.

This gate passed under [WU-0016](verification/WU-0016-inspector-authoring-loop.md):
2,175 tests, strict quality checks, Windows cross-compilation and local Wayland
native presentation. Its result does not close the later gates or the
completion goal.

### 2. Practical authoring and application API

[WU-0017](design/typed-application-api.md) adds a public facade, named nested
views, format 3, host-only compilation, Rust event handlers and shared native
hosting. [WU-0019](design/authored-text-views.md) extends this vocabulary with
fixed-size text leaves and Rust string literals.
[WU-0020](design/responsive-layout.md) adds typed auto/fill dimensions and
minimum/maximum bounds to both frontends, with intrinsic text measurement.
WU-0021 adds [authored controls](design/keyboard-controls.md), explicit state
colors and semantic action events through the same public facade.
WU-0022 adds [owned accessibility trees and native action dispatch](design/native-accessibility.md)
without changing the authored control vocabulary. This is an implemented
foundation; general reusable authored components, imports, bound expressions
and the inspector's property UX remain open.

- [ ] Design and ratify ergonomic component/property/style syntax using the
  example and inspector as the user scenario; remove author-facing numeric
  schema bookkeeping from the normal application path.
- [ ] Define roles for `.fen` and `ui!`, Rust state/event bindings, component
  composition, source maps and build integration.
- [ ] Define resource budgets that permit real applications. Format 2 retains
  its depth-4/eight-instance fixture contract. WU-0017 introduces independent
  node, depth and pixel budgets, with tests covering 301 nodes and 40 levels;
  these still need validation with the normal UI and performance scenarios.
- [x] Implement and test the new syntax/version without silently changing the
  frozen experiment fixtures or their diagnostic contracts.
- [x] Provide an application-facing facade and compile examples as consumers
  of it, without importing internal `prototype` APIs.
- [ ] Add inspector node navigation, property display/editing and visible
  diagnostics; keep authored input, compiled state and UI feedback traceable.

### 3. Normal application interfaces

[WU-0018](design/text-input-foundation.md) adds bounded grapheme editing,
owned keyboard/focus/composition events, committed bounds and a native text-pad
probe. Its isolated same-font Parley/cosmic-text screen informs the next text
view contract. [WU-0019](design/authored-text-views.md) now integrates authored
text, complete shaped measurements, ordered explicit font fallback and atomic
content/style updates through a replaceable Parley adapter. Its standalone
consumer has real Wayland presentation evidence. WU-0020 now resolves intrinsic
and weighted dimensions before wrapping text and publishing sibling geometry,
with a standalone responsive consumer. WU-0021 now provides buttons and
checkboxes, Tab navigation, visible focus, release-based pointer activation,
explicit state colors and owned semantic snapshots. A preferences consumer
exercises these behaviors with Rust state and has native Wayland presentation
evidence. Its [verification](verification/WU-0021-keyboard-controls.md) records
the original snapshot-only scope. WU-0022 now connects owned trees and semantic
actions to a replaceable native adapter. Its
[verification](verification/WU-0022-native-accessibility.md) records four actual
Linux AT-SPI stages covering labels, roles, bounds, focus, checked and disabled
state, button/checkbox activation and agreement with presented frames. A private
activation-status fixture selected the real desktop accessibility bus while
global accessibility and screen-reader settings remained unchanged. The local
AT-SPI disabled-button correction retains its provenance and regressions.
WU-0024 adds [source-indexed editing geometry](design/text-editing-geometry.md),
grapheme-safe queries, selection fragments, clipped text viewports and an owned
native IME caret-context hook. Its public gallery exercises these prerequisites;
an integrated authored field and real text accessibility remain open.
Broad font qualification, general alignment, editing controls, clipboard,
scrolling, native IME, real Windows UIA and screen-reader usability remain open.

- [ ] Specify, implement and verify text measurement, shaping, rendering and
  font fallback with multilingual examples.
- [ ] Add focus, keyboard navigation, editable text, selection, clipboard and
  IME composition on each qualified platform.
- [x] Connect bounded button, checkbox and standalone-label semantics to the
  native bridge and verify real Linux AT-SPI queries and actions against
  accepted application state and presented frames.
- [ ] Verify platform accessibility and screen-reader usability, including real
  Windows UIA, accessible scrolling and layout under resize and scale changes.
- [ ] Build a complete small application using those controls and bindings,
  and turn observed usability failures into regression tests.

### 4. Platform and graphics promises

- [ ] Record requested, detected and effective Windows, Wayland and X11
  capabilities, with explicit unsupported rows and tested environments.
- [ ] Resolve Linux native/GPU evidence and the renderer replacement boundary;
  test lifecycle, resize, suspend/resume, device loss and multiple windows.
- [ ] Verify transparency, overlays, notifications and safe framework-owned
  surface export under the relevant platform authority and lifetime rules.
- [ ] Measure startup, idle work, input latency, text and scene scaling;
  establish budgets before asserting performance or incremental benefits.

[WU-0023](verification/WU-0023-committed-raster-cache.md) addresses repeated
reference rasterization of an unchanged committed frame and records bounded
native memory observations. One cached raster has an explicit pixel-storage
bound. General latency, scene scaling and whole-process memory budgets remain
open; this bounded optimization does not close the performance gate.

### 5. Release readiness

- [ ] Record the project owner's license and distribution decisions, an MSRV
  backed by CI, compatibility/versioning rules and the supported platform set.
- [ ] Build reproducible installable artifacts, public API documentation,
  tutorials and examples from a clean checkout.
- [ ] Run release gates and publish only after explicit release authorization.

## Completion rule

Every mandatory gate needs implemented behavior, executable tests or native
evidence, documentation and recorded limitations. A plan, a passing fixture,
or a platform cross-build alone does not satisfy a product gate. Keep this
goal active while required behavior is missing; record independently completed
increments in their verification documents.
