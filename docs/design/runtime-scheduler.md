# Bounded runtime scheduler plan

Status: complete locally
Work unit: WU-0006
Branch: `feat/runtime-scheduler`
Research baseline: `fenestra-research` commit `176c42139776ed9f1ef879cd135bddadaf12a9da`
Last updated: 2026-08-09

## Research

The immutable research baseline fixes the safety and ownership constraints for
this unit without selecting a native event loop, async runtime, renderer, or
thread arrangement:

- The [feasibility spine contract](https://github.com/edziry/fenestra-research/blob/176c42139776ed9f1ef879cd135bddadaf12a9da/init/architecture/feasibility-spine-contract.md)
  assigns mutable UI state to one owner, sends immutable publications through a
  bounded renderer seam, requires non-droppable control transitions, and
  forbids a native callback from observing partial UI state.
- The [Windows initial map](https://github.com/edziry/fenestra-research/blob/176c42139776ed9f1ef879cd135bddadaf12a9da/init/platforms/windows-initial-map.md)
  records per-GUI-thread queues, direct and queued message delivery,
  reentrancy, synchronous-send hazards, and shutdown questions. A native event
  loop therefore cannot be treated as an ordinary async stream.
- The [Linux initial map](https://github.com/edziry/fenestra-research/blob/176c42139776ed9f1ef879cd135bddadaf12a9da/init/platforms/linux-initial-map.md)
  separates Wayland frame callbacks from presentation feedback and buffer
  reuse. A redraw request is not a presentation or completion observation.
- [FND-0030](https://github.com/edziry/fenestra-research/blob/176c42139776ed9f1ef879cd135bddadaf12a9da/init/findings/FND-0030-typed-invalidation-and-coalescing.md)
  supports request coalescing as a candidate but does not claim that request
  coalescing bounds queued work, submissions, or retained memory.
- [FND-0034](https://github.com/edziry/fenestra-research/blob/176c42139776ed9f1ef879cd135bddadaf12a9da/init/findings/FND-0034-gpu-retirement-by-completion.md)
  requires resource retirement to follow observed completion, not frame age,
  cache eviction, mailbox replacement, or presentation alone.
- The [invalidation and scene alternatives](https://github.com/edziry/fenestra-research/blob/176c42139776ed9f1ef879cd135bddadaf12a9da/init/architecture/invalidation-and-scene-alternatives.md)
  proposes one outstanding platform request, one successor demand during a
  build, a latest-useful publication seam, and separate submission,
  completion, and presentation observations. It labels that state machine an
  experiment candidate rather than a selected framework contract.
- The [priority and measurement plan](https://github.com/edziry/fenestra-research/blob/176c42139776ed9f1ef879cd135bddadaf12a9da/init/experiments/priority-and-measurement-plan.md)
  requires named clock domains and calibration before cross-domain latency
  claims. Fake time may establish deterministic ordering, not real pacing.
- The [threat model](https://github.com/edziry/fenestra-research/blob/176c42139776ed9f1ef879cd135bddadaf12a9da/init/security/threat-model.md)
  requires nested callbacks, slow consumers, shutdown during callbacks, loss,
  and update storms to be tested without torn state or unbounded growth.

`REQ-BP-001` is ratified in the baseline, but `REQ-FRAME-001`,
`REQ-RESOURCE-001`, and `REQ-RECOVERY-001` remain proposed. Numeric values for
`BUD-QUEUE-001` remain pending. Capacities in this work unit are therefore
reproducible experiment inputs, not product budgets.

## Ownership and dependency boundary

`fenestra-ui-runtime` owns an unpublished, single-owner scheduler state machine
around `UiRuntime`. It accepts typed inputs, advances deterministically, and
returns typed actions. It invokes no application, platform, or renderer
callback and holds no mutable tree, draft, or runtime-state lock across foreign
code. A callback scope may exclusively borrow the scheduler owner, but exposes
only its captured immutable snapshot and detached mutation plan.

`fenestra-ui-testkit` owns the fake logical clock, fake platform, fake renderer,
synthetic resource accounting, fault scripts, and scheduler trace. The
dependency remains:

```text
fenestra-ui-testkit -> fenestra-ui-runtime -> fenestra-ui-ir
```

The runtime does not depend on the testkit. Native adapter traits, OS handles,
GPU resources, async executors, wake mechanisms, and thread-affinity checks do
not enter this unit. A later probe may translate runtime actions into one
candidate backend without changing the scheduler's ownership rule.

The scheduled payload is provisionally a `CommittedRuntimeSnapshot`. It is a
logical and property generation, not a scene, layout, hit-test, pixel, surface,
or GPU snapshot. WU-0007 and WU-0008 may replace the payload with coherent
projections while preserving the queue and completion contract.

## Runtime state machine

The candidate owner is:

```text
UiScheduler
  -> UiRuntime
  -> lifecycle: Running | ShutdownQueued | Draining | Stopped | Faulted
  -> one optional deferred callback transaction
  -> one coalesced platform-frame request
  -> one optional latest-useful unsubmitted frame
  -> bounded submitted frames retained until observed completion
  -> bounded non-droppable controls ordered by acceptance sequence
  -> one reserved idempotent shutdown latch
```

The closed V1 adapter protocol has one coalescible `FrameReady` input and four
control families: submission acceptance or rejection, ordered frame
completion, renderer loss, and shutdown. It emits `RequestFrame`,
`OfferFrame(FrameWork)`, and `StopRenderer` actions. An offer remains in a
bounded awaiting-acceptance state and becomes submitted only after adapter
acceptance; rejection restores the same latest visual work without inventing a
submission. The fake renderer's injected `Fail` mode rejects an offer before
acceptance. Only ordered completion ends an accepted submission. Loss alone
does not authorize resource retirement. Acceptance assigns a monotonic token
in one renderer epoch. V1 starts at renderer epoch zero and submission token
zero, using explicit optional state rather than either value as a sentinel.
Every emitted offer, including a retry after rejection, consumes a fresh,
checked, monotonically increasing `FrameId`. The ID names an offer attempt, not
a generation. An equal completion watermark is idempotent. Regressing,
foreign-epoch, or beyond-last-accepted watermarks are typed errors.

The scheduler is clock-agnostic. Every input carries a caller-supplied
`SchedulerTick`, represented by a checked monotonic integer. The runtime never
reads wall time, sleeps, advances a clock, or compares different clock domains.
Tick regression is a typed error.

The scheduler owns all calls that can publish `UiRuntime`. It exposes committed
snapshot queries and transaction creation, but it does not allow an ordinary
commit to overtake a deferred callback transaction. One accepted nonempty
commit produces a small owned summary, drops the original `CommitReceipt`, and
marks frame work dirty. The summary retains only generation, mutation count,
and invalidation. It does not retain the prior `Arc<RuntimeState>`.

An accepted control or outstanding offer disposition must be processed before
another ordinary or deferred commit. Until `next_action` and the corresponding
adapter feedback resolve it, commit returns typed `ControlPending`
backpressure. A state publication therefore cannot overtake completion, loss,
or shutdown already owned by the scheduler.

A true no-op retains the exact state allocation and requests no frame. Several
nonempty commits before a platform frame-ready input retain one frame request.
If an unsubmitted frame already exists, the newer generation replaces its
snapshot and unions all invalidation since the last submission. Replacement
preserves the earliest unconsumed request tick and records the latest request
tick separately, so an update storm cannot reset residence age. Submitted
frames are never replaced.

One frame-ready input consumes the outstanding request and makes the latest
committed generation eligible for submission. The renderer action contains an
opaque frame ID, committed generation, immutable snapshot, accumulated
invalidation, request ticks, and accounted message bytes. Submission moves the
same generation into the bounded in-flight set. Completion, loss, and
presentation remain distinct observations; V1 needs completion and injected
loss, but makes no presentation claim.

Renderer loss is a typed terminal outcome in this prototype. Recovery is not
silently treated as success. The scheduler stops accepting visual work, keeps
already accepted completion obligations bounded, and allows shutdown control
delivery. Loss or shutdown cancels a request, pending publication, or unaccepted
offer without creating a submission. It does not remove an accepted
submission. Completion feedback for accepted submissions remains admissible
after loss, shutdown, or residence pressure and is the only route from
`Draining` to `Stopped`. If completion never arrives, the scheduler remains
`Draining` or `Faulted` with bounded obligations rather than claiming
`Stopped`. A future backend-specific experiment must define safe recovery and
device-loss retirement before replacing this policy.

## Reentrant callback rule

`begin_callback` captures one committed snapshot and a transaction based on the
same exact state allocation. The returned `CallbackScope<'_>` exclusively
borrows the scheduler owner and exposes:

- immutable queries against that captured snapshot;
- staged mutation through its one detached transaction;
- explicit nesting depth for a fake nested callback;
- an idempotent shutdown request flag.

Nested callback guards borrow the outer scope and share the same captured
snapshot and detached transaction. No operation publishes while the callback
scope exists. Finishing the outer scope may accept its nonempty transaction
into the one-item deferred lane; only a later scheduler turn commits it. A
second outer callback cannot begin while that lane is occupied. It receives
typed backpressure rather than creating another transaction against the same
base and later failing accidentally as stale.

The first implementation stores no closure, future, arbitrary application
value, or variable-size callback payload. Dropping an unfinished scope drops
its uncommitted transaction. Its RAII drop path still latches an idempotent
shutdown requested through that scope. A callback panic therefore cannot expose
a draft or erase an accepted shutdown request. The scheduler invokes no
foreign code while the exclusive scope exists; the adapter owns invocation and
uses the scope for explicitly nested fake callbacks.

This is a conservative experiment rule. It does not establish safe nested
transactions, callback thread affinity, application panic recovery, event
capture or bubble order, or native dispatch latency.

## Bounded lanes and byte accounting

Every capacity is inclusive and has no unbounded default. `QueueCapacity`
records maximum items, maximum accounted bytes, and a residence deadline in
ticks. `SchedulerCapacity` contains separate capacities for:

1. the one deferred callback transaction;
2. non-droppable controls;
3. the one latest-useful unsubmitted frame;
4. submitted frames in flight.

`RequestOutstanding`, `PendingPublication`, and `OfferAwaitingDisposition` are
mutually exclusive states of the same one-item visual lane. Each costs 40
accounted bytes. Scheduler construction requires visual capacity of at least
one item and 40 bytes, so a nonempty runtime commit cannot publish and only then
discover that its mandatory frame request has no bounded slot.

The testkit's separate `FakeRendererCapacityV1` bounds synthetic retirement
records in items, bytes, and residence ticks. Runtime configuration does not
name or own a fake-adapter resource.

V1 protocol accounting is independent of Rust layout and allocator behavior:

- a deferred transaction costs a 16-byte envelope plus 64 bytes per staged
  closed operation;
- a frame request, pending frame, or submitted frame costs a 40-byte envelope;
- a control or completion costs a 32-byte envelope;
- a fake retirement record costs a 32-byte envelope plus its declared
  synthetic resource bytes.

All multiplication and addition is checked before acceptance. The closed
operation vocabulary contains no variable-size text or asset. Snapshot payload
growth is bounded separately by `RuntimeCapacity` live-node, live-fragment,
live-property-slot, and retained-generation limits. The accounted byte values
are protocol weights for this experiment, not exact heap measurements.

The registered scheduler fixture uses one deferred batch, one pending frame,
two in-flight frames, four control records, and a fake renderer with two
retirement records. Its byte ceilings are the exact V1 weights for those maxima
and its residence limit is eight fake ticks. These values exercise every
transition. The paired runtime uses a retained-generation ceiling of three so
two distinct submitted generations plus the next publication edge are
representable. These values do not ratify a production budget.

An unconsumed visual publication may be replaced without reporting a drop.
Non-droppable means one of the following occurs:

- the control is accepted into bounded scheduler-owned state;
- an idempotent duplicate is reported as already accepted; or
- a typed backpressure result returns ownership to the producer.

Shutdown has one reserved item and byte allowance, so an accepted earlier
control cannot prevent the first shutdown request from being recorded. Once
shutdown is accepted, later visual work is rejected and later shutdown requests
produce no duplicate action or cleanup.

The reserve is inside, not in addition to, the declared control capacity.
Construction requires at least one item and 32 bytes. Ordinary control
admission uses at most `max_items - 1` and `max_bytes - 32`; the remaining
allowance belongs exclusively to the first shutdown request.

Control records and completion observations carry an acceptance sequence.
The next scheduler turn processes the smallest sequence before emitting visual
work, so frame replacement cannot overtake a loss or shutdown transition.

Residence is a deadline-to-pressure threshold, not a promise that an external
consumer completes. At the first scheduler-observed crossing, the lane enters
a typed terminal pressure outcome and stops accepting new work. Pending visual
work may be canceled explicitly. An in-flight snapshot or synthetic resource
is not retired merely because its deadline elapsed; it remains in the bounded
terminal state until completion or the test ends. Item and byte bounds continue
to hold even while residence exceeds the threshold. This proves bounded
failure behavior without claiming that a real backend may reuse unfinished GPU
resources.

`CommittedRuntimeSnapshot`, `UiTransaction`, and `CommitReceipt` each retain an
`Arc<RuntimeState>`. The scheduler never queues a `CommitReceipt`. With `N`
distinct old submitted generations still retained, publishing another
generation may temporarily require capacity for `N + 1` retired generations.
Scheduler construction therefore requires the runtime retained-generation
ceiling to be at least the in-flight item ceiling plus one. Caller-cloned
snapshots remain outside scheduler item accounting, but their generations still
participate in the runtime's existing typed retained-generation backpressure.

## Fake adapters

`FakeClockV1` has one named logical domain and advances only when directed by a
test. Equal-tick work is ordered by scheduler acceptance sequence. Checked
overflow is a typed harness failure. No test uses `sleep`, wall time, or an
automatic clock.

`FakePlatformV1` can:

- acknowledge a coalesced frame request at a chosen tick;
- open nested callback scopes;
- stage a reentrant mutation and query the captured generation;
- request shutdown during a callback;
- retain at most one frame-ready observation across transient typed
  backpressure from an earlier accepted control, retry it without replacement,
  and cancel it after a terminal transition.

`FakeRendererV1` can complete immediately, retain a bounded late submission,
advance an ordered completion watermark to a selected token, or inject loss.
It owns synthetic resource counts and bytes. Before accepting a frame offer it
atomically reserves every required submission and retirement record; failure
returns typed adapter pressure and the scheduler keeps the offer unsubmitted.
Each retirement record names the latest ordered submission token that used the
resource. Advancing the watermark means every earlier token in that renderer
epoch completed, and releases every submitted frame and retirement record at or
below the watermark. An equal watermark is idempotent. A regressing watermark,
foreign epoch, or watermark beyond the last accepted token is typed. A full
retirement lane never accepts new work and never retires early.

The fake adapters run on one test thread. They prove protocol ordering and
bounds, not OS thread affinity, GPU safety, pacing, presentation accuracy, or
backend recovery.

## Scheduler trace V1

The testkit owns a bounded typed `SchedulerTraceV1`. Its
`SchedulerTraceCapacityV1` contains maximum events and accounted bytes. One V1
event costs 96 accounted bytes. Recording checks item and byte addition before
append; a full trace returns a typed trace failure and retains the accepted
prefix. The registered stress fixture allows 256 events and 24,576 bytes.
Each event records:

- schema revision, sequence, fake tick, and the fixed fake clock domain;
- scheduler stage, typed input or action, and typed outcome;
- lifecycle, callback depth, committed generation, and optional frame or
  control sequence;
- item, accounted-byte, and oldest-residence values for every lane;
- submitted and retirement item and byte counts.

The trace contains no wall-clock value, thread ID, native handle, runtime
identity handle, property value, application text, local filesystem path,
panic text, or arbitrary message. Equal fixture, capacity, fake-clock script,
and runtime behavior must produce equal event vectors. WU-0008 may place this
typed trace in a wider experiment envelope; WU-0006 does not add another
durable wire format.

## TDD sequence

Implementation proceeds in focused red and green commits:

1. Frame mailbox tests require a nonempty commit to request one frame, true
   no-ops to request none, several pre-ready commits to coalesce, and a slow
   renderer to retain at most one latest pending frame plus the configured
   in-flight set.
2. Capacity tests require exact item, byte, arithmetic, tick-regression, and
   residence outcomes. Construction rejects visual capacity below one item or
   40 bytes and retained-generation capacity below the in-flight ceiling plus
   one. A completion watermark releases its submitted prefix and opens those
   submission slots. Pending replacement may allow later logical publications
   without renderer completion only while the runtime retained-generation
   bound permits them.
3. Callback tests require outer and nested queries to share the last committed
   allocation, reentrant mutation to remain invisible until a later turn, a
   second pending batch to receive typed pressure, and dropped or panicking
   staging to publish nothing. Dropping or unwinding a scope after its shutdown
   request must preserve exactly one shutdown latch.
4. Control tests fill the ordinary control allowance, then require shutdown to
   remain accepted, ordered, unique, and idempotent. Loss before shutdown must
   remain observable and visual work after shutdown must be rejected. Accepted
   controls block ordinary and deferred commits until processed.
5. Fake-renderer tests delay completion through many update attempts, inject
   loss, and prove an offer is not in flight before acceptance. Rejection
   restores it without phantom retirement. Watermark tests cover prefix
   release, equal duplicates, regression, foreign epochs, and values beyond the
   last accepted token. Loss and shutdown cancel an outstanding offer, then
   admitted completion drives accepted submissions through draining.
6. Determinism tests run the same nested-callback, slow-completion, loss, and
   shutdown script twice from the same fake tick and compare every trace event.

Stress scripts use fixed iteration counts and inspect every intermediate
bound. They must test accepted work, replacement, typed rejection, recovery to
a usable state where defined, and terminal behavior. A test that merely checks
the final queue length is insufficient.

## Exit and nonclaims

WU-0006 passes locally only when:

- every scheduler-owned lane remains within its item, accounted-byte, and
  residence policy under slow, failed, nested, and shutdown workloads, with the
  first observed deadline crossing entering typed pressure while item and byte
  bounds continue to hold;
- unsubmitted visual generations coalesce to the latest useful snapshot while
  controls and submitted generations are never inferred from a replacement;
- completion alone releases submitted and synthetic retirement state;
- nested queries observe one committed immutable allocation and nested
  mutation publishes only in a later scheduler turn;
- shutdown during a callback is ordered, non-droppable, terminal, and
  idempotent;
- equal fake inputs and ticks produce equal typed traces;
- trace recording either remains within its declared item and byte capacity or
  returns its typed full-trace failure without exceeding either bound;
- formatting, Clippy with warnings denied, tests, rustdoc, dependency, ASCII,
  and file-size checks pass on the complete workspace.

This unit does not select or validate winit, an async runtime, a render thread,
a GPU API, a native window backend, a timer source, or numeric product budgets.
It does not prove real latency, fairness, presentation, device-loss recovery,
GPU resource safety, multi-window arbitration, mobile lifecycle behavior, or
platform support. Those claims require their owning probes and environment
records.
