# Runtime observation diagnostics plan

Status: complete locally
Work unit: WU-0005
Branch: `test/runtime-oracles`
Research baseline: `fenestra-research` commit `176c42139776ed9f1ef879cd135bddadaf12a9da`
Last updated: 2026-08-09

## Research and repository evidence

The immutable research baseline requires every incremental result to be
compared with a correct reconstruction, requires the first failure to remain
reproducible, and treats traces and failure artifacts as a privacy boundary.
The [runtime oracle plan](runtime-oracles.md),
[artifact contract](runtime-oracle-artifacts.md), and
[V1 wire contract](runtime-oracle-v1.md) already fix the normalized comparison
order and legal failure-fingerprint grammar. No new external dependency or
source is required for this refinement.

Repository inspection establishes four constraints:

- `CommittedRuntimeSnapshot` exposes read-only queries but keeps its
  `Arc<RuntimeState>` private; its capacity is selected by the caller rather
  than fixed by the testkit ceilings.
- ordinary runtime publication validates the complete draft before swapping
  the committed state;
- corruption hooks are private unit-test seams inside `fenestra-ui-runtime`
  and are not compiled into the dependency consumed by the testkit;
- the workspace forbids unsafe code, so the testkit cannot manufacture a
  corrupt snapshot by bypassing those boundaries.

The current observer proves normal snapshots thoroughly, but it returns a
generic `StateMismatch` before preserving several malformed observations. It
also reconstructs some fields from validated construction rather than
retaining the candidate query result. Consequently, property-value and keyed-
order differences reach the structured comparator, while several template,
component, parent, binding, child, count, and alias failures do not.

This plan closes that diagnostic seam without weakening runtime validation or
claiming that the runtime can publish a corrupt state.

## Owned responsibility

`fenestra-ui-testkit` will own a private, replaceable observation pipeline that:

- reads the same public snapshot queries used by the existing observer;
- receives the transaction-specific expected `NormalizedStateV1` rather than
  assuming authored initial values and keys;
- retains bounded partial candidate observations before classifying them;
- compares in the exact V1 field order, independent of physical traversal;
- returns either a complete normalized snapshot or one structured fingerprint;
- retains runtime handles only in transient private indexes;
- hands identity comparison to the existing atomic identity ledger only after
  all normalized state fields match.

Its closed internal result is:

```text
Result<ObservationOutcomeV1, HarnessError>
  ObservationOutcomeV1::Complete(ObservedSnapshotV1)
  ObservationOutcomeV1::Mismatch(FailureFingerprintV1)
```

`HarnessError` remains the third path for a limit, arithmetic failure, invalid
fixture, or observation that V1 cannot represent honestly.

This work does not change or expose IR, runtime, facade, renderer, scheduler,
or platform APIs. It does not add a product fault hook. It does not add another
serializable `TraceFaultV1`; `OmitMove` remains the sole V1 candidate fault.

## Snapshot query boundary

A private query trait mirrors only public snapshot observations:

```text
SnapshotViewV1
  -> root and three global counts
  -> node template, component, property, parent, and children
  -> owner region binding and ordered keyed members
  -> child_count(node) and child_at(node, index) return owned scalars
  -> keyed_count(fragment) and keyed_at(fragment, index) return owned scalars
```

The diagnostic entry point is equivalent to:

```text
observe_snapshot_against_view_v1(
  construction,
  expected: &NormalizedStateV1,
  view: &V where V: SnapshotViewV1 + ?Sized,
  limits,
) -> Result<ObservationOutcomeV1, HarnessError>
```

The existing public `observe_snapshot_v1` signature remains compatible and
keeps its complete normal-snapshot path. Only replay and diagnostic unit tests
use the expected-state entry point.

The implementation for `CommittedRuntimeSnapshot` reads slice and exact-size
iterator lengths, then returns individual copied entries through indexed
methods. A size-only streaming pass checks observed live memberships, generated
path depth, and normalized counts before allocating any variable-length record.
If several limits cross, it selects the first observer-relevant class in
`HarnessLimitKind::ALL`: live memberships, path depth, normalized nodes,
normalized fragments, then normalized properties. A second pass copies only
bounded property values, children, and keyed pairs. This avoids lifetime or GAT
complexity, prevents an observation from retaining a snapshot generation, and
does not mistake caller-selected `RuntimeCapacity` for a testkit bound.

Raw child counts contribute to the normalized-node preflight even if a faulty
view underreports its global node count.

Unit tests may use a private delegating view that introduces one semantic
observation defect into an otherwise valid synthetic snapshot. Simple defects
change one query. An alias adapter rewrites every correlated `children`,
`keyed_members`, and parent observation needed to keep normalized state
coherent. The adapter receives semantic fixture coordinates, resolves them to
transient handles for one snapshot, and never persists a runtime handle. It is
not exported, serialized, or recorded as fault provenance.

## Partial observation

The collector records a private partial state before deciding whether values
match:

```text
PartialObservedStateV1
  -> nodes in expected authored preorder
       -> optional observed template and component
       -> optional schema-ordered property values
       -> semantic parent result: known, none, or unknown
       -> raw bounded direct-child handles
  -> fragments in expected authored preorder
       -> optional binding
       -> ordered key and member-handle pairs
  -> reported node, fragment, and property counts
  -> transient semantic-path to opaque-handle indexes
```

The expected clean state drives semantic addressing. Physical handles never
enter `NormalizedStateV1`, `FailureFingerprintV1`, errors, debug output, trace
bytes, or artifacts.

Collection completes every reachable bounded query and checked preflight
before semantic diagnosis. An early field difference therefore cannot hide a
later reachable limit crossing.

Missing data remains `Option` or an internal closed observation state until
diagnosis. Parent is `None`, `Known`, or `Unknown`; children and keyed queries
distinguish a missing query from an empty result. A missing value must not be
replaced with an authored value merely to make a normalized record
constructible.

Present keyed pairs are addressed by key, never by zip or vector position, and
their raw order is retained for `KeyedOrder`. Raw direct children are retained
separately for the later flat-order comparison. The expected semantic paths,
not a visited-handle set, bound traversal, so two paths may retain one physical
handle for identity diagnosis without creating a physical cycle.

Parent and child checks first compare a raw handle with the handle already
bound to the expected semantic path. Only a real inequality invokes reverse
lookup, and every mismatching handle must map to exactly one path before a
`Node` or `Nodes` summary can be emitted. This prevents an alias from becoming
a false child-order mismatch. Unknown or ambiguous mismatching handles remain
generic `StateMismatch`.

## Deterministic diagnosis

The diagnostic pass follows one total order:

1. authored-preorder nodes;
2. for each node: template, component, schema-ordered properties, then parent;
3. authored-preorder fragments;
4. for each fragment: binding, then keyed order;
5. authored-preorder nodes again for flattened direct-child order;
6. node count, fragment count, then property count;
7. node aliases, fragment aliases, node lifecycle transitions, then fragment
   lifecycle transitions, each in authored order.

The existing normalized comparator remains the oracle for complete states.
The partial diagnostic uses the same fingerprint constructors and summary
families, so synthetic and ordinary paths cannot define two taxonomies.

State mismatch always wins over identity mismatch in one transaction. The
identity ledger continues to evaluate on a clone and publishes summary and
retirement maps only when every lifecycle check succeeds. Aliases report the
second semantic path in authored order with expected `distinct` and observed
`aliased`.

Initial observation calls an atomic alias check before incrementing the
alias-free snapshot count. A pre-step observation performs the same pure alias
check without incrementing that count. Post-commit observation delegates alias
and lifecycle checks to the existing transactional ledger.

The ledger therefore exposes separate internal operations equivalent to
`verify_initial_aliases`, `first_alias`, and `verify_transition`; only the first
and third may publish counters, and only after their complete check succeeds.

## Representable and non-representable observations

V1 can represent:

- template or component difference, including an absent observed value;
- one property difference or missing property;
- parent difference when both handles map to semantic paths, or when either
  expected or observed parent is `none` and the other is one uniquely known
  semantic path;
- missing expected fragment binding;
- fragment-local keyed order;
- flattened direct-child order when every observed handle maps to a semantic
  path;
- the three global count differences;
- node or fragment alias and supported lifecycle transitions.

The public snapshot does not expose authored child-group metadata, so
`ChildOrder` with `children` summaries can only be exercised by a semantic
unit-test adapter. The product observation path uses flat `nodes` summaries.
An expected descriptor queried through `fragment(owner, descriptor)` cannot
distinguish a wrong descriptor from an absent binding.

An unknown parent or child handle has no honest `NodePathV1`, and V1 has no
summary for an opaque or unknown handle. Such a case remains a privacy-safe
generic `StateMismatch`; the testkit must not invent a semantic path. Adding a
new summary requires a future format revision.

## Runner phase rules

The runner consumes observation outcomes differently by phase:

- initial observation mismatch is a setup error and cannot become a reducible
  artifact;
- pre-transaction mismatch is a continuity error located at that transaction;
- after a successful runtime commit, `CommitShapeV1` is validated before any
  post-commit state diagnosis; an invalid shape remains `TraceMismatch` and is
  never converted to a fingerprint;
- post-commit structured mismatch creates one terminal `Mismatch` event and a
  `ReplayFailureV1(transaction, None, fingerprint)`;
- limit, arithmetic, invalid fixture, or non-representable observation errors
  remain `HarnessError` and do not produce a fingerprint;
- identity runs only after a complete state match;
- snapshot and receipt handles are released before another transaction.

A small private post-commit helper receives the owned `CommitShapeV1` plus a
lazy observation closure. It validates the shape first and invokes the closure
only on success. The production runner uses this helper, and its unit test can
combine an invalid synthetic shape with a closure that would return a semantic
mismatch. This proves both error priority and that observation was not called,
without asking the runtime to publish an impossible shape.

The normal runner uses the unmodified snapshot view. A one-shot `cfg(test)`
runner seam selects exactly one phase:

```text
Initial
Before(TransactionIdV1)
AfterCommit(TransactionIdV1)
```

It may apply a private observation adapter only at that phase. Initial and
pre-step injections return an internal harness error. A post-commit injection
may return a private `ReplayRunV1` for assertions, but it must not construct a
`LogicalTraceV1`, invoke the encoder or reducer, or produce an artifact with
`fault = none`. The adapter is not `TraceFaultV1`, has no artifact encoding,
and cannot be confused with `OmitMove` provenance.

## TDD matrix

The first red corpus will introduce one semantic observation defect into a
valid registered snapshot:

| Case | Expected first fingerprint |
| --- | --- |
| wrong root template | node `root`, `template` |
| wrong root component | node `root`, `component` |
| changed root property | node `root`, `property` |
| missing root property | node `root`, `property` to `none` |
| missing static-child parent | node `root/s:0`, `parent` |
| hidden expected empty fragment | fragment path, `fragment-binding` |
| swapped known flat children | owner node, `child-order` with `nodes` |
| wrong node count | global `node-count` |
| wrong fragment count | global `fragment-count` |
| wrong property count | global `property-count` |
| duplicate node handle | later node path, `identity-lifecycle` |
| duplicate fragment handle | later fragment path, `identity-lifecycle` |

The scalar count cases preserve all records and change only the reported
initial counts from `9/4/14` to `10/5/15`. The child-order case changes only
`children(root)` positions one and two, yielding semantic paths
`root/m:1:8,root/m:1:7` while `keyed_members` remains authored. It never swaps a
static node with a member.

Binding and fragment-alias cases first use real transactions to empty both
adjacent root fragments: remove keys `7` and `8` from `root/r:1`, then key `7`
from `root/r:2`. Binding hides `root/r:2`. Fragment alias makes its descriptor
query return the handle for `root/r:1`, so both fragments remain otherwise
empty and the later `root/r:2` path wins. Node alias coherently substitutes the
handle for `root/m:1:7` wherever `root/m:1:8` appears in the initial
`children` and `keyed_members` results; their otherwise equal subtrees and
shared parent keep state comparison equal.

Priority tests combine several alterations and require the earliest field in
the documented order. Exact inclusive count tests require `256/128/1024` to
remain diagnosable and `257/129/1025` to return the corresponding limit before
any field mismatch. Multi-limit cases pair an excessive normalized count with
observed live-membership overflow and require the earlier class from
`HarnessLimitKind::ALL`. A table test of the private limit selector pairs a
path-depth flag with each normalized-count flag; the registered fixture has no
authored path deeper than the V1 ceiling, so the query adapter does not pretend
to synthesize one. Unknown parent or child handles remain generic errors.

With seed `0` and the directed `8/2/8` generator config, a root-template defect
at `AfterCommit(0)` must terminate with transaction zero, no operation,
`verified=0`, one publication, generation one, `preserved=0`, and exactly one
alias-free initial snapshot, with this exact event:

```text
event|0|0|0,1|0|1|commit|1|layout,paint|mismatch
```

The same defect at `Initial` is an unlocated setup error; at `Before(0)` it is
a continuity error located at transaction zero. A separate unit priority
regression passes an invalid owned commit shape and a lazy post-commit semantic
defect to the private helper; it requires `TraceMismatch`, no
`ReplayFailureV1`, and zero observation calls.

Normal replay, the fixed 32-seed corpus, and the exact `OmitMove` trace bytes
must remain unchanged.

These tests prove the collector, diagnostic order, and runner handoff against
the public query shape. They do not claim that `UiRuntime` can publish the
altered snapshot.

## Bounds and privacy

Before copying, the observer completes its size-only streaming preflight. All
copied records use the existing harness ceilings for path depth, live
memberships, normalized nodes, fragments, and properties. Every count and
offset uses checked arithmetic. Multiple failures use their relative
`HarnessLimitKind::ALL` order, so a stored normalized-count crossing cannot
beat a later-discovered live-membership or path-depth crossing. Limits and
arithmetic errors are never converted into a semantic fingerprint.

The partial state and adapters are private. No trace or error contains a
`NodeId`, `FragmentId`, runtime domain, arena coordinate, local path, clock,
free-form or candidate-supplied message payload, user text, pixel, or platform
handle. Debug output for public prototype values remains closed and
physical-identity-free.

## Implementation cut

The current observer, runner, identity ledger, and fixture files are already
near the repository's 400-line limit. The behavior lands only after a green
mechanical split:

```text
observe.rs                 public wrapper and private entry points
observe/view.rs            bounded snapshot query boundary
observe/partial.rs         owned collection records and traversal
observe/diagnose.rs        ordered state classification
observe/tests/             fields, priorities, aliases, and limits
replay/observer.rs         phase selection and runner handoff
```

The refactor must preserve the existing public prototype and pass its tests
before any new red test is committed. No source file may be compressed merely
to satisfy the line limit.

## Replacement boundary and exit

The query trait, partial records, and diagnostic collector are correctness
scaffolding, not a renderer snapshot protocol. They may be replaced when a
future runtime provides a richer immutable description, provided the V1
artifact verifier remains able to reproduce its committed fixtures.

This subphase exits when:

- every representable state field in the matrix produces its exact structured
  fingerprint through the private snapshot-view boundary;
- node and fragment aliases reach identity diagnosis rather than generic state
  errors when normalized state otherwise matches, including the initial alias
  check before the first alias-free count;
- initial, pre-step, and post-commit phase behavior is distinct and tested;
- invalid commit shape retains priority over post-commit state diagnosis;
- state-before-identity priority and ledger atomicity remain executable;
- normal corpus results and known-fault bytes are unchanged;
- formatting, Clippy with warnings denied, tests, rustdoc, dependency
  direction, ASCII, diff, and file-size checks pass.

This does not close WU-0005. Failure-envelope decoding, cross-record
verification, deterministic reduction, the committed minimized fixture, and
the WU verification artifact remain later work.
