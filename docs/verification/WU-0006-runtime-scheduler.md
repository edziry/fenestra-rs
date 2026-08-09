# WU-0006 bounded runtime scheduler verification

Status: complete locally
Result: pass
Date: 2026-08-09
Branch: `feat/runtime-scheduler`
Research baseline: `fenestra-research` commit `176c42139776ed9f1ef879cd135bddadaf12a9da`

## Research and contract

The ownership rule, closed adapter protocol, queue accounting, callback rule,
fake-adapter behavior, trace schema, exit criteria, and nonclaims are recorded
in the [bounded runtime scheduler plan](../design/runtime-scheduler.md). The
immutable research baseline supplies the single-owner, bounded-publication,
non-droppable-control, completion-based-retirement, reentrancy, loss, and
shutdown constraints without selecting an event loop or renderer.

`fenestra-ui-runtime` owns the scheduler state machine. `fenestra-ui-testkit`
owns fake time, fake platform and renderer behavior, synthetic retirement
accounting, and scheduler tracing. The dependency remains one-way:

```text
fenestra-ui-testkit -> fenestra-ui-runtime -> fenestra-ui-ir
```

No package manifest or lockfile changed in WU-0006, and no external dependency
was added. Runtime code does not depend on the testkit. The prototype adds no
native adapter trait, OS handle, GPU resource, thread arrangement, async
executor, timer, or wake mechanism.

## TDD and review evidence

Focused tests were committed before each corresponding behavior. The retained
repository artifacts preserve those boundaries without depending on local
pre-squash commit identities:

- `runtime_scheduler.rs` fixes frame coalescing, timed actions, admission,
  tick regression, and mandatory-capacity behavior;
- `runtime_scheduler_submission.rs` fixes offer retry, submission retention,
  latest-useful pending work, and completion-based prefix release;
- `runtime_scheduler_residence.rs` fixes inclusive item, byte, arithmetic, and
  residence outcomes for the visual and in-flight lanes;
- `runtime_scheduler_callback.rs` fixes callback isolation, shutdown-latch
  priority, zero-storage callbacks, unwind, and later-turn publication;
- `runtime_scheduler_control.rs` plus its priority, residence, and protocol
  companions fix ordered control delivery, terminal obligations, projected
  completion, pressure priority, and the shutdown reserve;
- the testkit `scheduler_clock`, `scheduler_renderer`,
  `scheduler_renderer_backpressure`, `scheduler_platform`, and
  `scheduler_trace` fixtures fix each fake-adapter and trace contract before
  its corresponding implementation;
- `scheduler_trace/stress.rs` composes the already-tested seams as final
  integration evidence rather than claiming a new red behavior.

The review regressions strengthen observable boundaries rather than weakening
tests to fit the first implementation. They require rejected offers to preserve
the exact unsubmitted work without creating a submission, retries to use a new
checked frame ID, completion to be the only retirement route, and synthetic
resource reservation to be atomic. Callback tests require nested queries to
share one captured allocation, publication to occur only on a later scheduler
turn, and an accepted callback shutdown latch to survive drop or unwind.

Control tests require accepted work to block later publication until its
ordered disposition, preserve a shutdown reserve, prioritize the first
residence crossing, and retain terminal obligations until completion. The fake
platform retains at most one frame-ready observation during transient control
backpressure, retries it without replacement, and cancels it after a terminal
transition. Trace tests require typed steps and outcomes, real scheduler and
renderer projections, domain and tick validation, checked accounting, stable
error priority, and atomic retention of the accepted prefix.

## Registered capacities

The integrated fixture uses `RuntimeCapacity::new(4, 64, 256, 128, 1_024, 3)`:
four staged operations, 64 structural changes, 256 live nodes, 128 live
fragments, 1,024 live property slots, and three retained generations. The last
value is the exact in-flight ceiling plus one required by scheduler
construction.

Scheduler and fake-adapter capacities are inclusive:

| State | Items | Accounted bytes | Residence ticks |
| --- | ---: | ---: | ---: |
| Deferred callback batch | 1 | 80 | 8 |
| Ordered controls | 4 | 128 | 8 |
| Latest visual work | 1 | 40 | 8 |
| Submitted frames | 2 | 80 | 8 |
| Fake retirement records | 2 | 192 | 8 |

The four-control allowance includes one item and 32 bytes reserved for the
first shutdown request. One deferred one-operation batch weighs 80 bytes, one
visual or submitted frame weighs 40 bytes, and one control weighs 32 bytes.
Each stress resource declares 64 synthetic bytes and adds a 32-byte retirement
envelope, so two records weigh exactly 192 bytes.

The trace capacity is 256 events and 24,576 accounted bytes at 96 bytes per V1
event. Two runs of the same script each produce exactly 97 equal events and
9,312 accounted bytes. Every event is checked against every scheduler lane and
the fake retirement lane; the regression does not rely only on empty final
state.

These values are deterministic experiment inputs and logical protocol weights.
They are not product budgets, allocator measurements, heap bounds, or evidence
of real backend capacity.

## Integrated stress result

The stress script covers a grandchild nested callback, deferred mutation,
coalesced platform requests, two slow accepted submissions, an update storm,
latest-useful visual replacement, and exact request residence preservation.
It then forces an atomic fake-renderer capacity rejection, retries the same
generation under a fresh frame ID, and releases capacity only through ordered
completion.

An earlier accepted completion makes a frame-ready observation receive typed
`ControlPending` backpressure. The fake platform retains and later retries that
observation. The script then injects renderer loss, accepts shutdown through
its reserve, requires a duplicate shutdown to reuse the same sequence, reaches
the inclusive eight-tick in-flight and retirement ages, completes the remaining
submission, and ends in `Stopped` with every lane empty. Repeating the complete
script from the same domain and ticks yields the identical event vector.
The script stops at the inclusive deadline and does not claim a crossing.

Separate focused regressions cover configured-capacity rejection, arithmetic
exhaustion, tick and clock-domain errors, first deadline crossings, control
ordering, frame acceptance and rejection, completion watermark validation,
loss, callback drop and unwind, renderer pressure, trace prefix preservation,
and fake-clock overflow.

## Verification

The following final commands passed locally with warnings denied where
required:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --workspace --all-features --no-deps --locked
cargo metadata --format-version 1 --no-deps --locked
cargo tree --workspace --edges normal --locked
git diff --check
```

The workspace test gate passed 318 tests: 22 IR tests, 93 runtime tests (16 unit
and 77 integration), and 203 testkit tests (141 unit and 62 integration). The
facade and headless probe contain no tests. No executed test was ignored or
failed. Formatting, Clippy, and rustdoc completed with warnings denied where
applicable.

Metadata confirms that every package remains unpublished. The normal dependency
tree contains only workspace packages and confirms the one-way dependency
boundary above. The package manifests and lockfile did not change during
WU-0006.

The complete tracked-workspace ASCII scan and `git diff --check` passed. Every
Rust and Markdown file remains below 400 lines. The largest
changed Rust file is the 390-line fake-renderer integration test, and the
scheduler plan is 389 lines. All five crate and probe targets retain
`forbid(unsafe_code)`, while workspace lints deny unsafe code and undocumented
unsafe blocks.

## Result

Result: pass for WU-0006's local deterministic scheduler boundary.

Scheduler-owned work, accepted submissions, fake retirement records, and typed
trace events remain within their configured item and accounted-byte bounds.
Nested callbacks observe one immutable committed allocation, deferred mutation
publishes only on a later turn, controls cannot be overtaken, completion alone
retires accepted work, and shutdown remains ordered and idempotent. Equal fake
inputs and ticks reproduce the same complete trace. The first observed
residence crossing latches typed terminal pressure without dropping retained
obligations or exceeding bounds, while trace-capacity failure preserves the
exact accepted prefix.

This result is sufficient for WU-0007 and the later WU-0008 headless spine to
consume the scheduler seam. It is not a pass for EXP-0001, a native probe, or
any framework feasibility gate.

## Limitations and nonclaims

- The `prototype` surfaces remain documentation-hidden, unpublished, absent
  from the product facade, and unstable.
- Capacities and accounted bytes describe closed logical records only. They do
  not measure allocator overhead, total process memory, snapshot payload size,
  caller-cloned snapshots, or GPU allocation.
- The trace is bounded typed test evidence, not a durable wire format,
  production telemetry system, log-retention policy, or privacy assessment for
  future application payloads.
- Fake ticks prove deterministic ordering and residence decisions, not pacing,
  wall-clock latency, throughput, fairness, or cross-domain calibration.
- The fakes run on one test thread. They do not prove thread affinity,
  concurrent callback safety, lock freedom, or cross-thread publication.
- Completion-based synthetic retirement proves the protocol distinction from
  presentation, replacement, loss, and age. It does not prove GPU fence safety,
  buffer reuse, presentation accuracy, or device-loss recovery.
- Renderer loss is terminal in V1. Recovery, adapter restart, and preservation
  across device or surface recreation remain unspecified.
- No native event loop, window, surface, renderer, GPU API, timer, wake source,
  async runtime, or platform adapter is implemented or validated. There is no
  Windows, Linux desktop, mobile, or other platform-support result.
- Verification ran on local Linux x86_64 with Rust and Cargo 1.97.1. The
  configured Windows CI lane was not observed for this unit and must pass
  before any cross-platform determinism or Windows execution result is claimed.
- No multi-window arbitration, mobile lifecycle, accessibility, layout, style,
  hit testing, scene construction, pixels, benchmark, memory profile, Miri,
  MSRV, or performance result is claimed.
