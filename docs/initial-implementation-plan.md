# Initial implementation plan

Status: active
Scope: pre-alpha implementation bootstrap and EXP-0001 execution
Research baseline: `fenestra-research` commit `176c42139776ed9f1ef879cd135bddadaf12a9da`
Last updated: 2026-08-08

This plan translates the ratified research direction into the smallest useful
implementation sequence. It does not define a stable public API, select the
final dependency stack, or claim that a feasibility gate has passed.

The versioned research baseline is available in the
[Fenestra research repository](https://github.com/edziry/fenestra-research/tree/176c42139776ed9f1ef879cd135bddadaf12a9da/init).
Its product boundary is fixed by
[ADR-0001](https://github.com/edziry/fenestra-research/blob/176c42139776ed9f1ef879cd135bddadaf12a9da/init/decisions/ADR-0001-initial-product-direction.md).

## Intended outcome

The first implementation milestone is a deterministic, headless UI kernel and
one instrumented native-window probe. Together they must exercise the proposed
transaction, identity, projection, scheduling, and platform seams without
freezing the `.fen` grammar, `ui!` syntax, renderer, layout engine, or public
crate organization.

The first data flow is:

```text
hand-authored typed IR
  -> bounded UI transaction
  -> logical tree with generational identity
  -> minimal style and geometry projections
  -> semantics and hit-test projections
  -> immutable scene snapshot
  -> fake renderer and platform
  -> disposable native renderer and platform adapters
```

Hand-authored IR is deliberate. EXP-0001 needs a construction input before
EXP-0007 can validate the two real authoring frontends. Starting with a small
typed fixture breaks that dependency without turning the fixture syntax into a
framework contract. The fixture still uses distinct construction and style
programs linked by typed schemas; it is not a general DOM-like representation.

## Non-negotiable boundaries

- Production UI remains Rust AOT. No browser, DOM, CSSOM, WebView, JavaScript
  runtime, or VM enters the normal execution path.
- Construction and style programs remain specialized typed representations
  linked through shared schemas.
- Direct typed property updates are the normal reactive path. Keyed
  reconciliation is local to the dynamic fragment that owns the structure.
- Every mutation path enters one bounded transaction and typed invalidation
  protocol.
- A published generation is immutable and internally coherent across logical,
  style, layout, semantics, hit testing, and scene projections.
- Correct full reconstruction and full redraw remain oracles and fallbacks until
  incremental strategies prove both correctness and value.
- Platform APIs report requested, detected, and effective capability without
  silently emulating parity.
- Native capture, audio, codecs, encoding, transport, and remote-session policy
  remain outside this repository's core implementation program.
- Mobile work is limited to assumption audits and fake lifecycle inputs until a
  separate scope decision authorizes a backend.

## Required delivery cycle

Every implementation unit follows the same four-stage trail:

1. Research: inspect the current repository state and the smallest relevant
   primary or versioned sources. Record constraints, alternatives, unknowns,
   and the evidence that permits work to begin.
2. Planning: state the owned responsibility, explicit non-goals, dependency
   direction, failing test or correctness oracle, and exit criteria before
   introducing behavior.
3. Implementation: write the failing test first, confirm its failure, implement
   the smallest complete behavior, and refactor only while the tests remain
   green.
4. Verification: run formatting, Clippy with warnings denied, tests,
   documentation, manifest and dependency checks, ASCII and file-size checks,
   then record limitations without upgrading incomplete evidence into a claim.

Research artifacts live in the separate `fenestra-research` repository; this
source repository must not grow a `research/` subtree. Planning, code, tests,
and verification artifacts live here and link back to an immutable research
commit. If no new research is needed, the plan must cite the governing baseline
and explain why it is sufficient.

Research and planning are not substitutes for executable evidence. Likewise, a
passing implementation is not complete until its assumptions, verification,
and remaining limitations are versioned.

Definition of ready: a unit has a research baseline, owned responsibility,
non-goals, replacement boundary, correctness oracle or expected failing test,
and explicit exit checks. Definition of done: its implementation, required
oracles, verification commands, limitations, and research backlink are
versioned and independently reproducible.

## Governance gates before publication

These items do not block unpublished prototypes, but they block crate
publication or a stability promise:

1. The selected registry family is `fenestra-ui`. The `fenestra`,
   `fenestra-core`, and `fenestra-shell` package names are already used by
   another Rust GUI project; see the existing
   [`fenestra` 0.40.0 package](https://crates.io/crates/fenestra/0.40.0).
   Initial packages use `fenestra-ui` and `fenestra-ui-*` names, with Rust crate
   identifiers such as `fenestra_ui_runtime`, but set `publish = false`. A local
   name does not reserve its registry entry, so availability and publication
   timing remain explicit release gates.
2. Choose and version the project license. The repository currently has no
   license, so no manifest may imply one.
3. Separate the pinned development toolchain from the eventual MSRV. The
   bootstrap candidate is
   [Rust 1.97.1](https://blog.rust-lang.org/2026/07/16/Rust-1.97.1/), the
   current stable release when this plan was written. Pin it exactly for
   reproducible development, but leave the public `rust-version` promise open
   until the dependency graph and an explicit MSRV CI lane prove it.
4. Ratify numeric budgets, environment rows, test owners, and artifact-retention
   rules before an experiment result is interpreted as pass or fail.
5. Keep all APIs explicitly unstable until the relevant experiment and
   replacement boundary have evidence.

## Provisional workspace

The bootstrap should create a virtual Rust 2024 workspace with Cargo resolver
3, a committed lockfile, shared metadata and lints, and no public facade crate.

```text
fenestra-rs/
  Cargo.toml
  Cargo.lock
  rust-toolchain.toml
  rustfmt.toml
  crates/
    fenestra-ui/
    fenestra-ui-ir/
    fenestra-ui-runtime/
    fenestra-ui-testkit/
  probes/
    exp-0001-spine/
```

All five packages start at `0.0.0` with `publish = false`.

`fenestra-ui` is the unpublished facade. It owns no behavior during bootstrap
and exposes only contracts that have already passed their owning experiment.

`fenestra-ui-ir` owns only versioned component schemas, typed property IDs,
construction instructions, style instructions, source spans, structural-region
descriptors, and declared invalidation metadata. It owns no runtime geometry or
mutable UI state.

`fenestra-ui-runtime` owns generational identities, the logical tree, property
slots, keyed fragments, transactions, projection generations, the bounded
scheduler, and snapshot publication. Its initial modules should remain inside
one crate; style, layout, input, scene, and reactivity are not separate crates
until replacement or dependency boundaries require that split.

`fenestra-ui-testkit` owns fake clocks, fake platform and renderer adapters,
the clean-rebuild oracle, deterministic event logs, generated mutation
sequences, fixtures, and failure injection. Product code must not depend on it.

`fenestra-ui-exp-0001-spine`, stored at `probes/exp-0001-spine`, is a disposable
binary and integration harness. Candidate windowing, layout, text, renderer, or
GPU dependencies enter through this probe or a later adapter package, not
through an accidental public API.

The `fenestra-ui-macros` proc-macro package is added only when EXP-0007 begins.
Rust requires that technical crate boundary, but it must lower into the same
versioned schema as the future `.fen` compiler instead of embedding separate
runtime semantics.

## First vertical slice

The headless fixture is a small reactive layout board, not a production widget
set:

- a root surface and nested containers;
- typed width, height, color, visibility, and input-policy properties;
- one direct property binding;
- one keyed repeated fragment with insert, move, update, and remove operations;
- one semantic node and ordered hit-test region;
- rectangle scene primitives with declared bounds;
- pointer and resize events supplied by the fake platform;
- a fake renderer that can complete immediately, complete late, or fail;
- a bounded latest-useful-frame mailbox plus non-droppable control messages.

The mutation sequence must demonstrate:

1. A pointer event is routed against the last committed hit-test generation.
2. Application state enters a transaction.
3. A non-structural change updates a known property slot without tree
   reconciliation.
4. A structural change reconciles only its owning keyed fragment.
5. Typed invalidation reaches every affected projection.
6. Validation either publishes one complete generation or leaves the preceding
   generation untouched.
7. A slow renderer cannot grow queued frames or retained generations without a
   bound.
8. A reentrant query sees committed immutable state, while a reentrant mutation
   is deferred to a later transaction.

The initial native probe then replaces only the fake platform and renderer. It
opens one opaque window, renders rectangles, handles pointer input, resize,
scale, and close, and records an end-to-end correlated trace. Transparency,
text, accessibility, richer input, and special-window behavior follow as
separate increments. One reference environment validates only the probe seam;
it cannot establish Windows, Wayland, native X11, or XWayland support.

## Required oracles and tests

Implementation follows TDD. The first runtime work is not complete without
tests for:

- stale handles failing after an arena slot is reused;
- stable identities surviving unrelated direct property updates;
- keyed insert, move, update, and remove preserving the expected identity and
  lifecycle;
- transaction failure or panic preserving the previously published generation;
- direct updates and local reconciliation matching a clean reconstruction;
- incremental projection output matching the full-rebuild oracle;
- event capture, target, bubble, focus, cancellation, and reentrant ordering;
- no work being published for a true no-op transaction;
- bounded frame, control, and retirement queues under a slow consumer;
- typed capability absence never becoming a successful no-op;
- deterministic traces for the same input sequence and fake clock.

Use unit and model tests first, then property tests for mutation sequences and
generational reuse. Golden artifacts are appropriate for versioned IR,
diagnostics, geometry, semantic trees, and display lists. Image goldens wait
until fonts, assets, color behavior, and tolerances are fixed.

Fuzzing should later cover untrusted DSL and style input, SVG and assets,
dimensions and arithmetic, scene descriptors, shader or material input, and
shared-resource descriptors. Miri should cover the pure data structures once a
nightly toolchain lane exists.

## Safety and dependency policy

- Pure IR, runtime, authoring, and testkit crates use `forbid(unsafe_code)`.
- Native, renderer, GPU, and export adapters isolate unavoidable unsafe code
  behind safe facades with written invariants, small `SAFETY`-documented blocks,
  negative tests, and explicit ownership.
- No raw native pointer, handle, or descriptor enters the generic runtime.
- Dependencies are pinned in `workspace.dependencies`, use the smallest feature
  set, and avoid moving Git branches.
- Each admitted dependency records purpose, omitted responsibilities, exact
  version, features, MSRV, unsafe surface, maintenance, security history,
  license, native requirements, distribution constraints, and replacement cost.
- Competing layout, renderer, text, windowing, or reactive candidates remain in
  separate probes. They are not all linked into the runtime for convenience.
- Cargo features represent supported distributable capabilities, not internal
  algorithm choices or dependency brands.

## CI and measurement bootstrap

Every change should run the following on the pinned development toolchain:

- `cargo fmt --all -- --check`;
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`;
- `cargo test --workspace --all-targets --all-features`;
- `cargo doc --workspace --all-features --no-deps` with warnings denied;
- Linux and Windows build and headless test lanes.

Hosted CI proves portable compilation and deterministic tests, not real GPU,
compositor, IME, accessibility, or window-system support. Windows, Wayland,
native X11, and XWayland native runs remain separate environment records.

The first trace schema should be versioned before performance work. At minimum,
events carry run, environment, experiment, input, mutation, frame, surface, and
generation IDs; monotonic timestamp and clock domain; stage; result; queue
items, bytes, and residence time; and relevant node or resource counts. Default
telemetry contains no user text, clipboard contents, pixels, or native handle
values.

Performance numbers collected before budgets and hardware are ratified may
debug the harness, but cannot approve a feasibility gate.

## Implementation sequence

Each item is a focused squash-merge unit after this planning change. Its
research, planning, implementation, and verification artifacts are specified in
[bootstrap-work-units.md](bootstrap-work-units.md).

1. `build/bootstrap-workspace`: create the unpublished `fenestra-ui` workspace,
   pin the exact development toolchain, add shared lints, keep `Cargo.lock`, add
   Linux and Windows CI, and make all packages `publish = false`.
2. `feat/runtime-identity-tree`: add typed generational IDs and the minimal
   logical tree through failing lifecycle and stale-handle tests first.
3. `feat/runtime-transactions`: add typed property mutation, keyed fragment
   operations, validation, atomic generations, and rollback-on-failure tests.
4. `test/runtime-oracles`: add clean reconstruction, generated mutation
   sequences, deterministic tracing, and minimized failure artifacts.
5. `feat/runtime-scheduler`: add the fake platform, fake renderer, bounded
   mailboxes, reentrant callback rules, and slow-consumer failure injection.
6. `feat/probe-headless-spine`: assemble the reactive layout board and prove the
   complete headless data flow.
7. `experiment/native-spine`: admit one screened windowing and renderer
   candidate behind disposable adapters, retain the minimal geometry projection,
   preserve an owned-surface export route as an early screening criterion, and
   produce the first native frame and environment manifest.
8. `experiment/typed-authoring`: begin EXP-0007 by validating and evolving the
   provisional shared schemas, then lower equivalent `.fen` and `ui!` fixtures
   into them after the transaction seam is executable.

Layout conformance corpus preparation and the mobile-assumption inventory use
the same four-stage protocol and may run in parallel. Text, IME, accessibility,
rich 2D, owned-surface export, and the drawn-control design system build on the
spine as their prerequisite seams become testable.

## Decisions deliberately left open

- Facade shape, SemVer policy, registry reservation, and publication timing.
- Final MSRV and stable support cadence.
- Exact `.fen`, `ui!`, expression, style, selector, and reload syntax.
- Own runtime components versus reusable reactive machinery.
- Taffy or another layout implementation and the supported layout semantics.
- winit plus native escape hatches versus focused native shells.
- wgpu, Vello, Skia, CPU fallbacks, and the scene or display-list model.
- Parley, cosmic-text, or another text composition.
- One arena versus separate projection stores.
- UI-thread rendering versus a dedicated render owner.
- Incremental invalidation, damage, spatial index, cache, and crossover
  heuristics.
- Owned-surface export ABI and resource types before EXP-0002.
- Hosted native-control availability by platform.

An implementation choice moves out of this list only with a versioned fixture,
correctness oracle, dependency record, measured result where applicable, and an
ADR that states its replacement boundary.

## Exit criteria for the headless bootstrap goal

The active headless bootstrap goal is ready for deeper subsystem work when:

- the unpublished workspace is reproducible on Linux and Windows;
- the headless spine passes every identity, transaction, projection,
  reentrancy, boundedness, and oracle test;
- failures and unsupported capabilities are typed and reproducible;
- the trace and environment manifests are versioned;
- open dependency, platform, budget, crate-boundary, and registry decisions
  remain explicit;
- no result is described as product support or feasibility proof prematurely.
