# Headless EXP-0001 spine plan

Status: complete locally
Work unit: WU-0008
Branch: `feat/probe-headless-spine`
Research baseline: `fenestra-research` commit `176c42139776ed9f1ef879cd135bddadaf12a9da`
Last updated: 2026-08-09

## Research boundary

The [feasibility spine contract](https://github.com/edziry/fenestra-research/blob/176c42139776ed9f1ef879cd135bddadaf12a9da/init/architecture/feasibility-spine-contract.md)
requires one atomically published immutable generation across logical state and
all projections, a clean-rebuild fallback, and committed-only callback views.
The [invalidation alternatives](https://github.com/edziry/fenestra-research/blob/176c42139776ed9f1ef879cd135bddadaf12a9da/init/architecture/invalidation-and-scene-alternatives.md)
retain full rebuild, rectangle output, and reverse hit testing as candidates,
without performance or incrementality evidence. The [authoring boundary](https://github.com/edziry/fenestra-research/blob/176c42139776ed9f1ef879cd135bddadaf12a9da/init/architecture/authoring-style-runtime-boundary.md)
keeps construction and style distinct and leaves runtime precedence open.

[EXP-0001](https://github.com/edziry/fenestra-research/blob/176c42139776ed9f1ef879cd135bddadaf12a9da/init/experiments/EXP-0001-feasibility-spine.md)
remains proposed: owner, native environments, hardware, corpus, measurements,
and thresholds are not ratified. WU-0008 owns only a deterministic synthetic
headless result; it neither executes EXP-0001 nor satisfies a feasibility gate.

## Ownership and dependency direction

`fenestra-ui-runtime` owns the provisional runtime materialization rule, one
headless surface input, full projection rebuild, atomic publication, immutable
projection views, and committed hit-test queries. These records live in the
same private `RuntimeState` allocation as the logical tree. A transaction either
publishes every record in one new generation or preserves the complete previous
allocation.

`fenestra-ui-testkit` owns a new headless fixture, an independent clean-rebuild
oracle, normalized comparisons, fake pointer and resize delivery, fault
injection, the fixed script runner, bounded tracing, and artifact support. Existing
runtime-oracle V1 fixtures and wire artifacts remain unchanged.

`fenestra-ui-exp-0001-spine` owns only runner invocation, the synthetic
environment manifest, canonical result emission, and integration tests.
`main.rs` does not implement runtime behavior. The product facade is unchanged.

The direct dependency edges remain:

```text
probe -> IR
probe -> runtime
probe -> testkit
testkit -> IR
testkit -> runtime
runtime -> IR
```

No package, external dependency, native handle, backend trait, async runtime,
layout library, renderer library, or platform API is added by this unit.

## Fixed typed fixture

Fixture V1 uses schema namespace `8001`, revision `1`, component `0`, property
IDs width `0`, height `1`, color `2`, visible `3`, and input policy `4`.
Schema defaults are respectively `40`, `10`, `[32,32,32,255]`, `true`, and
`Ignore`. Template IDs are root `0`, container `1`, control `2`, and item `3`;
region `0` belongs to the container, repeats item `3`, and starts with keys
`[10,20]`. The root has static container slot `0`; the container has static
control slot `0` and region slot `1`. Semantic label symbol `1` designates the
static control. Surface V1 starts at `120 x 90` logical units.

| Template | Construction overrides | Exact style |
| --- | --- | --- |
| root | width `100`, height `80`, color `[1,1,1,255]` | none |
| container | width `80`, height `50`, color `[2,2,2,255]` | none |
| control | width `30`, color `[3,3,3,255]`, input `Accept` | color `[10,20,30,255]` |
| item | height `12`, color `[4,4,4,255]`, input `Accept` | color `[80,90,100,255]` |

Width and height invalidate `Layout`, `Semantics`, `HitTest`, `Paint`, and
`Composition`; color invalidates `Paint`; visible invalidates `Semantics`,
`HitTest`, and `Paint`; input policy invalidates `HitTest`. Region `0`
invalidates `Structure`, `Layout`, `Semantics`, `HitTest`, `Paint`, and
`Composition`. Resize invalidates `Layout`, `Semantics`, `HitTest`, `Paint`,
`Composition`, and `Surface`; semantic V1 is `(Control,1,Activate)`.

The values have no CSS units, device-pixel or color-space meaning. Runtime
materialization uses this disposable precedence:

```text
exact StyleAssignment for (template, property)
  > TemplateFactory::effective_value(property)
  > schema default through effective_value
```

The implementation must call `ValidatedStyleProgram::assignment` for the first
step. It must not use `linked_value` for the fallback because that API
intentionally returns the schema default and does not inspect construction
initial values. The selected value initializes the runtime property slot. A
later direct mutation replaces that slot. A newly inserted keyed instance
materializes the same exact style rule again.

Generation zero has five nodes in authored preorder at paths `root`,
`root/s:0`, `root/s:0/s:0`, `root/s:0/m:1:10`, and
`root/s:0/m:1:20`. Their absolute rectangles are `(0,0,100,80)`,
`(0,0,80,50)`, `(0,0,30,10)`, `(0,10,40,12)`, and `(0,22,40,12)`.
Computed colors are `[1,1,1,255]`, `[2,2,2,255]`, `[10,20,30,255]`,
and `[80,90,100,255]` for both items. The generation has semantic tuple
`(root/s:0/s:0,Control,1,Activate)`, three hit regions in
control/key-10/key-20 scene order, and five scene rectangles in authored order.

## Provisional headless specification

`HeadlessProjectionSpec` is runtime-owned, unpublished, and replaceable. It
names the five component-local property IDs, the single semantic template and
label symbol, and explicit projection capacities. Initialization validates the
four reachable templates, each property type, the initial surface, and every
capacity before publication. The semantic template must have exactly one
static authored expansion and must not be a region repeat body.

The ordinary `UiRuntime::new` path remains unchanged. A separate provisional
constructor consumes a `ValidatedStyleProgram`, `HeadlessProjectionSpec`,
initial `HeadlessSurface`, and `RuntimeCapacity`. It obtains the exact
construction from the style program and retains no independent source program.
An ordinary snapshot returns `None` from `headless_projection`; a headless
snapshot returns its immutable view. A resize staged against an ordinary
runtime fails at commit with `TransactionErrorKind::HeadlessUnavailable` and
the operation index.

The surface is a pair of non-negative logical extents. Resize is one staged
`UiTransaction` operation and one typed mutation record. It participates in
the existing operation ceiling and emits `Surface`, `Layout`, `HitTest`,
`Semantics`, `Paint`, and `Composition` invalidation. A resize to the current
extent is a true no-op and retains the exact committed allocation.

Registered IR validation capacities for components/properties/templates/regions/
child-slots/initial-properties/initial-keys/template-depth/initial-instances are
`1/5/4/1/3/12/2/3/5`; style assignments are `2`; runtime
operations/structural/live-nodes/live-fragments/live-properties/retained
`8/8/8/2/40/3`, and projection computed/geometry/semantic/hit/scene
`8/8/1/8/8`. Scheduler deferred/controls/visual/in-flight capacities are
`1/80/8`, `4/128/8`, `1/40/8`, and `2/80/8` in items/bytes/ticks.
The fake renderer uses `2/192/8`; scheduler trace uses `256/24576`; headless
trace uses `128/20480` with `160` accounted bytes per event; the canonical
artifact ceiling is `65536` bytes.

Pointer input is not a `SchedulerInput`. That closed protocol remains renderer
and frame-control feedback. The fake platform captures one committed snapshot,
queries its hit-test projection, and may stage a mutation through the existing
callback transaction rule.

## Full projection rebuild

WU-0008 uses a complete deterministic rebuild for every effective logical or
surface publication. This is alternative I1, L1, S1, and H1 from the research
record. It is a correctness candidate and fallback, not an incremental layout
or damage result. This unit produces no damage record and makes no D1 claim.

Each live logical node produces one computed-style record and one geometry
record containing raw `bounds` and effective `clip`, in authored tree order.
Geometry uses signed checked arithmetic and this fixture-only vertical rule:

1. Every stored rectangle uses absolute logical coordinates. The root geometry
   is `(0,0,declared_width,declared_height)`; its clip is its intersection with
   `(0,0,surface_width,surface_height)`.
2. A parent's child cursor starts at its geometry `(x,y)`. A child geometry has
   that `x`, the current cursor `y`, `min(child_width,parent_geometry.width)`,
   and the child's declared height. The cursor advances by that declared height
   with checked addition.
3. Nested children repeat step 2. Clip intersection keeps the maximum origin,
   clamps each extent at zero, and never changes geometry or the cursor. Hidden,
   empty, or fully clipped siblings still advance the cursor.
4. Effective visibility is the conjunction of the node and all ancestor
   visibility values. `InputPolicy::Ignore` excludes only that node and does
   not suppress an accepting descendant.
5. Geometry remains present for every node. Negative dimensions, coordinate or
   cursor overflow, or an impossible parent relation fail before publication.

A node with effective visibility and a non-empty clip produces one scene
rectangle storing that clip. One that also accepts input produces one hit
region storing that clip. Both retain authored scene order; hit testing scans
the hit collection in reverse and uses half-open rectangles. The semantic template produces one
closed control record only while its runtime instance is effectively visible
and non-empty. No event handler or application closure is stored in a hit or
semantic record.

The immutable projection snapshot exposes its `RuntimeGeneration`, surface,
computed-style views, geometry views, semantic views, hit regions, scene
rectangles, and `hit_test(point)`. Every view belongs to the same private state
allocation. Public prototype formatting discloses counts and generation only;
it does not disclose property payloads, physical arena slots, source text, host
paths, or private domain tokens.

## Atomic publication and scheduler integration

The transaction path applies logical operations to a draft, validates the
logical state, rebuilds and validates the complete headless projection, and
only then swaps one `Arc<RuntimeState>`. Projection failure returns a typed
initialization or transaction error and preserves logical state, surface,
projection vectors, identities, generation, and allocation.

`CommittedRuntimeSnapshot` is extended rather than paired with a second
publication channel. Existing callback scopes, `FrameWork`, the replaceable
visual lane, and accepted submissions already retain that exact snapshot.
Consequently a slow accepted frame may retain an older complete projection
while a successor publication contains a newer complete projection, subject to
the existing retained-generation capacity.

No scheduler action, lane, control, ID, accounting weight, residence rule, or
completion rule changes. A testkit wrapper reads the scene from the snapshot
carried by `FrameWork`, derives one bounded synthetic resource use separately,
and passes that testkit-owned slice to `FakeRendererV1::offer`. Runtime never
depends on a synthetic-resource type.

## Bounds and diagnostic order

`HeadlessProjectionCapacity` has inclusive ceilings for computed-style records,
geometry records, semantic records, hit regions, and scene rectangles. Counts
are inspected before allocating their corresponding vectors. The runtime live
node and property-slot ceilings remain authoritative for logical storage.
Retained projection memory follows the existing retained-generation bound.

Transaction semantics and fixed-record admission precede final rebuild. That
rebuild validates only the final surface, then uses this closed order:

1. missing specification target or property;
2. property type mismatch;
3. invalid or non-unique semantic template;
4. surface validity;
5. computed-style and geometry limits;
6. negative geometry;
7. arithmetic exhaustion;
8. semantic, hit, and scene limits;
9. internal invariant failure.

`HeadlessProjectionLimitKind::ALL` orders simultaneously available limit facts.
Computed and geometry counts exist before layout; semantic, hit, and scene
counts become applicable only after negative and checked-arithmetic validation.
Initialization has no operation anchor. A commit identifies a directly
responsible operation when possible; derived failures may omit it. Every check
precedes publication. Runtime-oracle wire V1 gains no enum: its unreachable
headless failure maps to the existing invariant rejection, preserving its bytes.

The experiment capacities are correctness inputs, not allocator-byte
measurements or product budgets. `BUD-UI`, `BUD-INPUT`, `BUD-START`, `BUD-MEM`,
`BUD-BIN`, `BUD-POWER`, `BUD-RECOVERY`, and the numeric product interpretation
of `BUD-QUEUE` remain pending.

## Independent oracle and normalized identity

The testkit clean rebuild does not call the runtime projection builder. It
walks the desired logical fixture in semantic path order, applies runtime
overrides, exact style assignments, and construction effective values, then
builds the same documented vertical geometry and derived outputs independently.
Traversal uses authored child-slot and current keyed-member vector order;
`NodePathV1` identifies records but never sorts them.

Comparisons normalize logical nodes to `NodePathV1` and fragments to
`FragmentPathV1`. Golden and trace artifacts never serialize `NodeId`,
`FragmentId`, arena indices, pointer values, or process-local validation
domains. The oracle compares complete ordered records after generation is
projected separately.

Fault adapters independently perturb computed style, geometry order, semantic
membership, hit order, and scene output. `HeadlessMismatchKindV1` selects those
five families in that order. Within one family records compare in authored
order and fields compare path first, then style width/height/color/visible/input,
geometry bounds/clip, semantic role/label/action, hit clip/order, or scene
rectangle/color/order. The first differing index is reported with the expected
path, otherwise the observed path, otherwise `End`; values are never included
in the fingerprint. Each defect changes only the testkit view.

## Input sequence

The versioned script uses these exact fake ticks and inputs:

1. At tick `0`, build and compare generation zero; pointer `(5,5)` resolves to
   `root/s:0/s:0` in that snapshot.
2. At tick `1`, a depth-two callback stages control color
   `[20,30,40,255]`; `next_action` at tick `2` publishes generation `1` and
   emits the sole frame request.
3. Commit key `30` at region index `1` on tick `3`, move it to final index `2`
   on tick `4`, set its height to `14` on tick `5`, remove key `20` on tick
   `6`, then prove its captured `NodeId` fails `MissingNode` without publication.
   Resize from `120x90` to `90x70` on tick `7`. Before frame-ready at tick `8`,
   repeat that resize and require a no-op sharing generation `6`, allocation,
   request state, and latest visual tick `7`. The real operations publish
   generations `2` through `6` under one request.
4. Deliver frame-ready and offer the latest generation later on tick `8`; fake mode
   `Late` accepts submission token `0` with one generation-ID resource of `64`
   synthetic bytes.
5. Set control visibility to `false` on tick `9` and emit one successor request.
   The latest snapshot now misses `(5,5)` while token `0`'s retained snapshot
   still targets the control. Set root width to `84` on tick `10`. Frame-ready
   on tick `11` offers generation `8`;
   `Fail` rejects the first frame ID, the fresh retry is different, and `Late`
   accepts token `1` with its separate `64`-byte resource.
6. Admit completion of token `0` on tick `12` and process it on tick `13`.
   On tick `14` set root color to `[9,9,9,255]` and emit its request.
7. On tick `15`, frame-ready offers generation `9`; fake mode `Loss` admits
   renderer loss without accepting that offer. Admit shutdown twice on the same
   tick and require one accepted control sequence plus one idempotent result.
8. Process loss on tick `16` and `StopRenderer` on tick `17`; token `1` and its
   resource remain live. Admit its completion on tick `18` and process it on
   tick `19`, reaching `Stopped` with every lane and resource ledger empty.

After every logical or surface publication, the candidate snapshot is compared
with a clean rebuild. The script also verifies stable unaffected identities,
callback deferral, queue statistics, and retained projection lifetime. After
resize, root bounds remain `(0,0,100,80)` while its clip is `(0,0,90,70)`;
scene and hit records use clips, not bounds.

## Headless trace and artifact

The scheduler trace schema remains fixed. `HeadlessTraceEventV1` has schema `1`,
dense sequence, scheduler-domain tick, `HeadlessTraceStageV1` (`Build`, `Input`,
`Callback`, `Transaction`, `Projection`, `Scheduler`, `Renderer`),
`HeadlessInputKindV1` (`None`, `Pointer`, `Direct`, `Insert`, `Move`, `Update`,
`Remove`, `Resize`, `FrameReady`, `Completion`, `Loss`, `Shutdown`), and
`HeadlessOutcomeV1` (`Observed`, `Deferred`, `Published`, `NoChange`, `Matched`,
`Action`, `Accepted`, `Rejected`, `Completed`, `Lost`, `Stopped`, or `Failed`
with closed runtime/projection/oracle/scheduler/renderer/trace cause).
It also stores optional captured and published numeric generations, a closed
target (`None`, `StaticControl`, or `Key(u64)`), optional frame and control
numbers, surface extent, the five projection counts, four scheduler lane
item/byte pairs, and renderer item/byte counts. It retains no snapshot,
property value, source text, native handle, wall clock, or arbitrary string.

Event and accounted-byte ceilings are independent and inclusive. Append
preflights both with checked arithmetic, gives event count priority over bytes,
and preserves the accepted prefix and sequence on failure. Events encode in
sequence order and fields encode in the order above.

Canonical artifact V1 starts with envelope, fixture, schema, construction,
style, trace, and projection version `1`; synthetic platform `headless-fake`,
fake clock `scheduler`, projection choices `full/vertical/rebuilt/reverse`; then
the registered capacities, headless events, complete `SchedulerTraceEventV1`
records, final computed-style, geometry, semantic, hit, and scene sections,
closed `HeadlessResultV1` (`pass`, `adapt`, or `stop`), and end marker. Both
traces correlate by tick, generation, frame, and control; scheduler records
retain their typed stage, outcome, and lane residence. Final projection
sections contain the fixture's closed bounded property values;
the privacy statement above applies to trace events. Neither area contains
user text or host data. Artifact text is preflighted against its separate
`65536`-byte ceiling.

Testkit owns the fixture, closed canonical decoder, encoder, and semantic
verifier. Decode enforces ASCII, final LF, grammar, versions, limits, counts,
and references before verification replays the complete script and compares
both traces and projections. Unknown fields are rejected and errors disclose no
source record. Two fresh runs produce identical bytes. The golden is retained
at `probes/exp-0001-spine/tests/artifacts/headless-spine-v1.txt` as WU-0008
evidence until a later artifact version supersedes it; the probe emits those
bytes without host data.

## File boundaries

Runtime additions use `runtime/headless.rs` plus
`runtime/headless/{spec,types,build,view}.rs`; testkit additions use
`headless.rs`, `headless/{fixture,oracle,platform,renderer,trace,artifact,runner}.rs`,
and `headless/artifact/{decode,encode,verify}.rs` with split tests; the
probe uses `src/lib.rs`, a thin `main.rs`, and split integration support. This
unit does not grow existing near-limit fixture, semantic, scheduler, mutation,
transaction, or fake-platform modules beyond small re-exports and call seams.

## TDD slices

1. Style materialization: construction fallback, exact style override, later
   direct override, keyed insertion, and specification failures.
2. Initial projection: manual computed style, geometry, semantic, hit, and scene
   tuples; a `25x15` clip; hidden ancestor and zero-height retention; half-open
   edges and reverse-order control-over-accepting-root hit; capacity/arithmetic.
3. Atomic updates: direct, insert, move, update, remove, resize, no-op, invalid
   dimensions, overflow, and rollback of the exact prior allocation.
4. Headless fixture and independent oracle: normalized clean rebuild, every
   projection defect, identity preservation, and typed mismatch priority.
5. Fake input: last-committed pointer query, nested callback snapshot, deferred
   publication, stale target rejection, and resize delivery.
6. Scheduler integration: coalescing, immediate and late completion, recoverable
   rejection, loss, idempotent shutdown, projection retention, and all queue and
   resource bounds.
7. Trace and probe: limits and accepted-prefix behavior, closed canonical
   codec, semantic replay verifier, privacy and full-workspace ASCII checks,
   deterministic golden, binary output, and versioned verification record.

Each behavioral slice starts with a focused failing test, implements the
smallest passing behavior, runs its owning package gates, and commits red and
green work separately when cohesive.

## Exit and nonclaims

WU-0008 may pass only when the full fixed headless script is correct, bounded,
observable, reproducible, compared with its clean oracle, and exercised through
all declared fault paths. The verification record must state separately:

```text
WU-0008 result: pass
EXP-0001 status: open and not executed
```

This unit does not validate final style semantics, layout conformance,
incrementality, damage, pixels, text, native accessibility or input, hosted
controls, presentation, GPU use, transparency, export, recovery, security, or
platform support. It provides no timing, memory, binary-size, energy, queue
sizing, scalability, or product-support conclusion.
