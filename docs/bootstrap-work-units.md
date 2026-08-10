# Bootstrap work units

Status: active
Scope: research, planning, implementation, and verification trail
Research baseline: `fenestra-research` commit `176c42139776ed9f1ef879cd135bddadaf12a9da`
Last updated: 2026-08-10

This register makes the four required stages explicit for each work unit in the
[initial implementation plan](initial-implementation-plan.md). A unit ends with
one of three recorded results: `pass`, `adapt`, or `stop`. Passing means only
that the unit met its stated exit criteria; it does not imply product support or
that a feasibility gate passed.

## WU-0001: Workspace bootstrap

Branch: `build/bootstrap-workspace`

- Research: verify the registry family, current stable Rust release, Cargo 2024
  behavior, CI runner contract, existing repository state, license absence, and
  immutable ADR-0001 baseline.
- Planning: define package ownership, dependency direction, shared lints,
  toolchain, CI lanes, publication lock, and manifest audit criteria.
- Implementation: create `fenestra-ui`, `fenestra-ui-ir`,
  `fenestra-ui-runtime`, `fenestra-ui-testkit`, and
  `fenestra-ui-exp-0001-spine` as `publish = false` packages with no external
  dependencies.
- Verification: run Cargo metadata, format, Clippy, tests, documentation,
  lockfile, ASCII, file-size, dependency-direction, and Linux checks; require
  the Windows CI lane before calling cross-platform bootstrap reproducible.

Exit: the workspace is reproducible and contains no accidental public release
or product behavior.

## WU-0002: Generational identity and logical tree

Branch: `feat/runtime-identity-tree`

- Research: compare the baseline identity invariants with candidate arena and
  generational-handle techniques; record why reuse is safe and replaceable.
- Planning: define node ownership, root and parent rules, generation rollover,
  lifecycle transitions, stale-handle behavior, and model-test sequences.
- Implementation: start with failing creation, parentage, removal, reuse, and
  stale-handle tests; add the smallest typed IDs and logical tree that pass.
- Verification: run unit and generated lifecycle tests, invariant validation,
  deterministic replay, and Miri when its separate nightly lane exists.

Exit: no recycled slot can alias a retired logical identity.

## WU-0003: Provisional typed IR

Branch: `feat/provisional-typed-ir`

- Research: map the shared schema, specialized construction, source-anchor,
  keyed-region, value-type, and declared invalidation constraints from the
  baseline without selecting authoring syntax.
- Planning: define format and identity scope, validation domains and limits,
  validation order, immutable output, error anchors, explicit non-goals, and the
  runtime replacement seam.
- Implementation: write failing schema, graph, region, value, span, format,
  identity, limit, and invalidation tests before adding the smallest
  hand-authored fixture IR.
- Verification: prove deterministic validation and iteration, dependency
  direction, unpublished status, and rejection of every invariant class in the
  versioned malformed-fixture corpus.

Exit: the IR exposes one validated typed construction contract suitable for
later runtime consumption without a parser, DOM-like representation, or mutable
runtime state.

## WU-0004: Transactions and invalidation

Branch: `feat/runtime-transactions`

- Research: map ADR-0001 direct slots, local keyed fragments, invalidation
  classes, atomic publication, panic behavior, and clean-rebuild requirements.
- Planning: define the mutation state machine, validation boundary, rollback,
  keyed lifecycle, projection dependencies, and expected failure cases.
- Implementation: write failing direct-update, keyed-operation, invalidation,
  rollback, and no-op tests before adding transaction behavior.
- Verification: compare every incremental result with clean reconstruction;
  use a narrow local reference model, and prove failed or panicked transactions
  leave the committed generation intact. WU-0005 generalizes the model into the
  reusable clean-rebuild oracle, seeds, traces, and minimization artifacts.

Exit: successful non-noop mutation routes produce one validated typed log and
one new logical generation; no-ops retain the generation, and failures publish
neither state nor log.

## WU-0005: Test oracles and failure artifacts

Branch: `test/runtime-oracles`

- Research: evaluate model testing, deterministic generation, golden artifacts,
  shrinking, and seed persistence without selecting unnecessary dependencies.
- Planning: version the fixture, seed, trace, expected-state, and minimized
  failure schemas plus retention and privacy rules.
- Implementation: add the testkit clean-rebuild oracle, generated mutation
  sequences, deterministic traces, and minimized failure records.
- Verification: inject a known defect, prove the oracle detects it, reproduce it
  from the stored seed, and confirm minimization preserves the failure.

Exit: an incremental correctness failure is reproducible without private logs.

## WU-0006: Bounded scheduler and fake adapters

Branch: `feat/runtime-scheduler`

- Research: map event-loop affinity, frame backpressure, control-message
  delivery, reentrancy, clocks, shutdown, and resource-retirement constraints.
- Planning: define queue capacities in items and bytes, replacement rules,
  non-droppable controls, callback snapshots, trace fields, and fault outcomes.
- Implementation: add fake clock, platform, renderer, bounded mailboxes,
  deferred reentrant mutations, slow completion, loss, and shutdown injection.
- Verification: stress slow and failed consumers; prove bounds, control ordering,
  last-committed query semantics, deterministic traces, and idempotent shutdown.

Exit: no fake workload can create unbounded queued or retired state.

## WU-0007: Provisional typed style program

Branch: `feat/provisional-style-ir`

- Research: map typed properties, exact-target fixture styling, source anchors,
  schema linking, and invalidation metadata without selecting selectors,
  cascade, inheritance, or public syntax.
- Planning: keep style instructions distinct from construction and define one
  replaceable exact-target program plus typed diagnostics and equivalence data.
- Implementation: write failing schema-link, target, property, value, duplicate,
  version, span, and deterministic-order tests before adding the minimum style
  program needed by the headless fixture.
- Verification: compare linked defaults and exact assignments with a manual
  expected result and prove no style parser or runtime state enters the IR.

Exit: construction and style are distinct linked typed programs before the
headless spine begins, without implying final style-language semantics.

## WU-0008: Headless EXP-0001 spine

Branch: `feat/probe-headless-spine`

- Research: map the headless workload to EXP-0001 requirements and record which
  pending budgets prevent performance or support conclusions.
- Planning: fix the reactive layout-board fixture, environment manifest, input
  sequence, expected projections, trace schema, failures, and exit evidence.
- Implementation: connect typed IR, transactions, logical state, minimal style
  and geometry, semantics, hit testing, scene rectangles, scheduler, and fakes.
- Verification: replay the fixture against clean reconstruction and every fault
  path; record a `pass`, `adapt`, or `stop` result with raw deterministic data.

Exit: the full headless flow is correct, bounded, observable, and reproducible.

## WU-0009: Disposable native spine

Branch: `experiment/native-spine`
Design: [disposable native EXP-0001 spine](design/native-exp-0001-spine.md)

- Research: screen windowing and renderer candidates by version, features,
  maintenance, unsafe surface, license, platform reach, replacement cost, and
  ability to preserve future owned-surface export.
- Planning: isolate adapters, select one reference environment, define native
  callback and capability traces, and state that one run is not platform support.
- Implementation: replace fake platform and renderer only; retain the minimal
  geometry projection and keep candidate types out of generic runtime APIs.
- Verification: run on the named native environment with manifest, trace,
  resize, scale, input, close, slow render, and failure evidence.

Exit: the native seam is measurable and replaceable, not selected permanently.

## WU-0010: Typed dual authoring

Branch: `experiment/typed-authoring`
Design: [typed dual authoring plan](design/typed-dual-authoring.md)
Reference: [typed authoring format-1 fixture](design/typed-authoring-reference.md)
Verification: [typed dual authoring verification](verification/WU-0010-typed-authoring.md)

- Research: evaluate grammar, proc-macro, build integration, source maps,
  diagnostics, typed style semantics, runtime contracts, and toolchain costs.
- Planning: evolve one shared schema and define equivalent `.fen`, `ui!`, and
  style fixtures with diagnostic, span, build, memory, and behavior criteria.
- Implementation: lower both construction frontends and typed styles into their
  distinct linked programs without a target parser or general DOM-like IR.
- Verification: compare schemas, programs, diagnostics, source spans, direct
  slots, keyed fragments, observable runtime state, and bounded workflow costs.

Exit: both authoring paths share semantics rather than merely similar syntax.

## WU-0011: Layout conformance

Branch: `experiment/layout-conformance`
Design: [layout conformance plan](design/layout-conformance.md)
Reference: [layout reference contract](design/layout-conformance-reference.md)
Verification: [layout conformance verification](verification/WU-0011-layout-conformance.md)

- Research: inspect the current projection, runtime transaction, invalidation,
  scheduler, hit, scene, and native seams; screen current layout candidates,
  exact dependency closures, maintenance, MSRV, license, unsafe, and replacement
  cost.
- Planning: fix a candidate-neutral integer stack contract, validation order,
  rounding, limits, corpus, independent oracle, TDD sequence, and
  pass/adapt/stop criteria.
- Implementation: add the unpublished neutral boundary, reference engine,
  independent oracle, disposable candidate adapter, and atomic runtime seam
  through focused RED/GREEN commits.
- Verification: compare every supported case and the registered runtime script
  field by field, retain negative controls and deterministic artifacts, and run
  Linux plus Windows pure gates.

Exit: the bounded stack subset is reproducible and replaceable without selecting
a final layout engine or leaking candidate types.

## WU-0012: Mobile lifecycle preparation

Branch: lifecycle implementation deferred; preparation recorded with WU-0011
Decision: [mobile lifecycle preparation](design/mobile-lifecycle-preparation.md)

- Research: inventory desktop assumptions in lifecycle, surface ownership,
  scale, input, backgrounding, memory pressure, scheduler clocks, and targets.
- Planning: define the stop-the-line triggers, later lifecycle axes, fake ports,
  surface epochs, invariants, and independent version decision.
- Implementation: deferred because WU-0011 needs only a present logical extent;
  no mobile lifecycle code or platform target is added in parallel.
- Verification: audit that zero extent is geometry rather than absence, that no
  platform vocabulary enters layout, and that every trigger remains false.

Exit: preparation records a separate post-WU-0011 unit and makes no mobile
support claim. If a trigger becomes true, WU-0011 stops and WU-0012 becomes its
prerequisite.
