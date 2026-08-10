# WU-0009 disposable native spine verification

Status: complete locally
Result: pass
Date: 2026-08-09
Branch: `experiment/native-spine`
Evidence commit: `5a99b3b`
Research baseline: `fenestra-research` commit
`176c42139776ed9f1ef879cd135bddadaf12a9da`

## Research and contract

The candidate screen, exact dependencies, ownership boundary, event reducer,
raster contract, trace bounds, oracle, safety constraints, and non-goals are
recorded in the
[disposable native spine design](../design/native-exp-0001-spine.md). The fixed
milestones and failure decisions are recorded separately in the
[native reference run](../design/native-exp-0001-run.md).

WU-0009 replaces only the headless fake platform and fake renderer at the probe
boundary. The real shell uses winit, the real presenter uses Softbuffer, and the
existing runtime scheduler and headless rectangle projection remain unchanged.
Candidate types do not enter the IR, runtime, testkit, or facade APIs. The
package remains unpublished and the native seam remains disposable.

This unit closes one named local native run and cross-platform build evidence.
It does not execute or close EXP-0001. The experiment still lacks ratified
owners, platform and hardware sets, corpus, product budgets, and decision
thresholds.

## TDD and review evidence

The branch retains contract tests before each corresponding implementation
slice. The versioned evidence divides responsibility as follows:

- [surface and raster tests](../../probes/exp-0001-native-spine/src/native/tests/surface.rs)
  fix directed scale conversion, generation, coalescing, suspension, inclusive
  limits, authored-order pixels, clipping, alpha rejection, and storage
  failures;
- [driver tests](../../probes/exp-0001-native-spine/src/native/tests/driver.rs)
  fix scheduler correlation, accepted-surface input, redraw admission,
  acceptance and completion, failure atomicity, renderer retirement, and empty
  shutdown;
- [trace tests](../../probes/exp-0001-native-spine/src/native/tests/trace.rs)
  and the [stage matrix](../../probes/exp-0001-native-spine/src/native/tests/trace_matrix.rs)
  fix closed identities, applicability, accounting, priority, privacy, and
  environment failures;
- [shell tests](../../probes/exp-0001-native-spine/src/native/tests/shell.rs)
  fix winit mapping, owned Softbuffer resources, bounded watchdog behavior,
  slow and failed presentation, exact script milestones, and artifact grammar;
- [runner tests](../../probes/exp-0001-native-spine/src/native/runner/tests.rs)
  require the one-slot post-present barrier, surface preemption, and exact
  `360x260` resize result;
- package integration tests fix the
  [closed process API](../../probes/exp-0001-native-spine/tests/native_api.rs),
  [target-scoped manifests](../../probes/exp-0001-native-spine/tests/native_manifest.rs),
  and [frozen Fedora artifact](../../probes/exp-0001-native-spine/tests/native_artifact.rs).

Review confirmed that no test was weakened or skipped to admit the
implementation. The final exact-resize regression rejects both an unchanged
accepted surface and a different pending logical extent with
`EnvironmentSurfaceChanged`; neither can be mistaken for generation 2.

## Native seam result

The Fedora runner creates one opaque winit window, one owned Softbuffer context
and surface, and one candidate-neutral CPU staging frame. It processes winit
callbacks through `ControlFlow::Wait` and the bounded reducer, then presents the
same committed headless scene used by the scheduler.

The fixed run observed:

1. Fedora Wayland supplied physical `400x300` at scale `1.25`, normalized as
   logical `320x240`; publication reached runtime generation 1 and native
   surface generation 0.
2. Frame 0 was offered, accepted as submission `0:0`, presented, completed by
   control 0, and matched the committed headless oracle.
3. A scripted primary press at logical `(5,5)` entered the same reducer as a
   native button callback, captured generation 1, and hit `static-control`.
4. The exact logical `360x260` request was observed as physical `450x325` at
   scale `1.25`; publication reached runtime generation 2 and native surface
   generation 1 and matched the oracle.
5. Frame 1 was accepted as submission `0:1`, presented, and completed by
   control 1.
6. Scripted close admitted control 2, observed `StopRenderer(2)`, and reached
   `SchedulerState::Stopped` with all owned lanes and slots empty.

Pointer and close are explicitly scripted native-adapter inputs. Pure mapping
tests show that real winit callbacks enter the same candidate-free reducer, but
this run does not claim that the OS generated those two inputs.

After each successful present, the following directive waits in one bounded
slot. The runner arms a five-second settlement watchdog, requests one unarmed
redraw, and releases the directive only when that callback records `Ignored`.
At `about_to_wait`, a pending surface observation runs first and terminates as
`EnvironmentSurfaceChanged`/`adapt`. This prevents a fixed milestone from
overtaking native state already delivered to the application. It is a callback
ordering barrier, not evidence of compositor completion, display scanout, or
pixel visibility.

The resize milestone also compares the published logical extent with the exact
`360x260` request. Refusal, supersession, an effective surface change around a
frame or settlement barrier, and a later scale change are typed environment
outcomes rather than silent changes to the passing script.

## Fedora Wayland artifact

The canonical run evidence is
[`fedora-wayland-v1.txt`](../../probes/exp-0001-native-spine/tests/artifacts/fedora-wayland-v1.txt):

```text
LF-terminated lines: 37
bytes: 12975
maximum line bytes: 389
SHA-256: 413b9c53c73d1b2a5dfb8de6aa760f08b034d467cac192df73b6d02a0880ce73
```

The artifact contains one schema line, one closed manifest, 34 dense events,
and one terminal record. Every event accounts 192 protocol bytes, for 6,528 of
the 24,576-byte trace ceiling. Every artifact byte is printable ASCII or LF,
the final byte is LF, and the complete output stays below its 65,536-byte and
131-line limits.

The manifest records Linux, `x86_64-unknown-linux-gnu`, Wayland, exact candidate
versions and features, initial `400x300` physical and `320x240` logical extents,
scale `1250000` micros, and closed requested/detected/effective capability masks
of 31. These masks report the five adapter capabilities exercised by the fixed
run; they are not a product platform-support declaration.

Only the two scheduler `Accepted` frame events carry staging digests, before
their matching presenter events:

```text
frame 0, generation 1, native generation 0: 99e3b9d20c7b7cbd
frame 1, generation 2, native generation 1: 0471ed0561dc931d
```

No offered, rejected, presented, or completion event carries a digest. The
digest identifies exact CPU staging pixels through the documented FNV-1a
procedure; it is not an authenticity, security, scanout, or display-integrity
check.

The terminal record is `pass`, runtime generation 2, scheduler `stopped`, and
zero deferred, control, visual, in-flight, redraw, surface, pointer, and
presenter work. Five consecutive executions in the named desktop produced the
same frozen bytes and empty stderr. That observation does not become a general
cross-run determinism rule: winit and the compositor may schedule, coalesce, or
reorder live callbacks. Exact encoding is guaranteed only for an identical
typed event stream; the versioned artifact freezes this one observed stream.

The artifact contains no hostname, username, path, environment value, wall
clock, native handle, node identity, source text, pixel buffer, or candidate
error string. Its SHA-256 identifies the versioned bytes and is not an
authenticity mechanism.

## Dependencies, licenses, and safety

The normal direct package tree is exactly the two existing workspace packages
plus `winit 0.30.13` and `softbuffer 0.4.8`. Linux enables only winit
`rwh_06,wayland,wayland-dlopen` and Softbuffer
`wayland,wayland-dlopen`. Windows enables only winit `rwh_06` and a Softbuffer
build with default features disabled and its target-native Win32 backend. X11,
KMS, client-side decoration, GPU, text, image, accessibility, and hosted-control
features are not admitted.

Winit declares Apache-2.0 and Rust 1.70.0. Softbuffer declares MIT OR
Apache-2.0 and Rust 1.71.0. The exact pins and lockfile prevent silent candidate
drift. The package screen found no direct RustSec entry for either exact package
on 2026-08-09, but this is not a complete transitive audit or future assurance.
The Fenestra project license remains pending; dependency licenses do not select
or supply that project license. No installer, redistribution, static-link,
package-signing, or publication conclusion is made.

Both probe roots use `forbid(unsafe_code)`, workspace lints deny unsafe code and
undocumented unsafe blocks, and the probe adds no unsafe block. Upstream native
implementation remains part of the dependency safety surface. The shell keeps
owned display, surface, and window resources private and drops the Softbuffer
surface before its final window reference. It never exports or formats a raw
native handle.

Trace storage is limited to 128 events and 24,576 accounted bytes. Reducer
surface, pointer, presenter, and deferred-script state each occupy at most one
slot. The raster guards are 4,096 pixels per axis, 16,777,216 pixels, and
67,108,864 staging bytes, with checked arithmetic and preflight before
allocation. These are correctness ceilings, not measured allocator use or
product budgets.

Every Rust and Markdown file remains below 400 lines. The workspace maximum is
399 lines; the WU-0009 maximums are 384 production Rust lines, 394 test Rust
lines, and 390 design lines. Tracked ASCII, whitespace, and LF-attribute checks
pass.

## Linux verification

The native run and final package gate passed on Fedora 43 x86_64, kernel
`7.1.5-101.fc43.x86_64`, with rustc and Cargo 1.97.1, target
`x86_64-unknown-linux-gnu`, and LLVM 22.1.6.

The following final commands passed with warnings denied where shown:

```text
cargo fmt --all -- --check
cargo test -p fenestra-ui-exp-0001-native-spine --all-targets --all-features --locked
cargo clippy -p fenestra-ui-exp-0001-native-spine --all-targets --all-features --locked -- -D warnings
RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc -p fenestra-ui-exp-0001-native-spine --all-features --no-deps --locked
cargo test --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --workspace --all-features --no-deps --locked
cargo metadata --format-version 1 --no-deps --locked
cargo tree -p fenestra-ui-exp-0001-native-spine --edges normal --locked
git check-attr text eol -- probes/exp-0001-native-spine/tests/artifacts/fedora-wayland-v1.txt
git diff --check
```

The package gate passed 96 library unit tests and four integration tests: one
process API test, one artifact test, and two manifest tests. The binary target
has no unit tests. The workspace gate passed 602 tests: 2 facade, 32 IR, 145
runtime, 316 testkit, 7 headless-probe, and 100 native-probe tests. No executed
test was ignored or failed. The native command completed within its 20-second
process bound, emitted the versioned artifact on stdout, and emitted no stderr.

## Windows verification

Status: pass for build and pure tests in a fresh native checkout of commit
`5a99b3b`.

The 64-bit Windows host reported Windows NT `10.0.26200`, rustc 1.97.1, and
Cargo 1.97.1. Formatting, the locked package all-target/all-feature test gate,
Clippy with warnings denied, and rustdoc with warnings and missing documentation
denied all passed. The test breakdown matched Linux: 96 library tests and four
integration tests, with no ignored or failed test.

The verification used the authorized OpenSSH service session. That service did
not provide an interactive Windows desktop and therefore did not run the probe
through Win32 presentation. Windows evidence proves the target-native package
build and pure contract tests only; it does not prove Win32 window creation,
input, resizing, Softbuffer presentation, compositor behavior, or shutdown in
an interactive session.

## Result

```text
WU-0009 result: pass
EXP-0001 status: open and not executed
```

WU-0009 passes its local disposable native seam: one real Fedora Wayland
window, CPU surface, two accepted and presented frames, exact resize, bounded
pointer input, correlated scheduler lifecycle, independent oracle matches,
closed artifact, and empty stopped state agree with the fixed contract. The
Windows package and pure tests pass in a fresh checkout.

This is not selection of winit or Softbuffer for a product, an EXP-0001 pass,
or a platform-support declaration.

## Limitations and nonclaims

- No product support is established for Linux, Wayland, Windows, Win32, X11,
  XWayland, mobile, any compositor, or any hardware set.
- Scripted pointer and close prove the shared reducer path, not native OS input
  synthesis. The SSH Windows result is not an interactive Win32 run.
- Softbuffer `present` and the settlement callback prove CPU retirement and
  callback ordering, not scanout, displayed pixels, compositor completion,
  latency, damage, synchronization, or presentation timing.
- The staging digest is not a screen capture, security check, content proof,
  or broad pixel oracle. One rectangle fixture is not renderer conformance.
- Winit and Softbuffer remain exact experimental candidates behind a private
  replacement boundary. Their APIs and behavior are not public compatibility
  promises.
- No final shell, renderer, GPU, layout, text, animation, input, accessibility,
  hosted-control, transparency, special-window, export, or packaging design is
  selected.
- No timing, startup, memory, binary-size, energy, queue sizing, recovery,
  performance, security, fuzz, Miri, sanitizer, benchmark, or profiler result
  is claimed. Registered capacities are correctness inputs, not product
  budgets.
- The project license, final MSRV, publication, namespace, support cadence,
  release process, and full EXP-0001 governance remain pending.
