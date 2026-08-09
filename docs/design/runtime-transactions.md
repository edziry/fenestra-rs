# Runtime transactions plan

Status: active
Work unit: WU-0004
Branch: `feat/runtime-transactions`
Research baseline: `fenestra-research` commit `176c42139776ed9f1ef879cd135bddadaf12a9da`
Last updated: 2026-08-08

## Research

The versioned research baseline already fixes the transaction behavior needed
for this unit:

- [ADR-0001](https://github.com/edziry/fenestra-research/blob/176c42139776ed9f1ef879cd135bddadaf12a9da/init/decisions/ADR-0001-initial-product-direction.md)
  makes direct typed property updates the normal lane, confines keyed
  reconciliation to owning structural regions, and requires one transactional
  mutation and invalidation model.
- The [feasibility spine contract](https://github.com/edziry/fenestra-research/blob/176c42139776ed9f1ef879cd135bddadaf12a9da/init/architecture/feasibility-spine-contract.md)
  requires stable node and fragment identities, transactional creation and
  retirement, immutable committed generations, bounded state, and no partially
  observed mutation.
- The [authoring and runtime boundary](https://github.com/edziry/fenestra-research/blob/176c42139776ed9f1ef879cd135bddadaf12a9da/init/architecture/authoring-style-runtime-boundary.md)
  requires compiled bindings and every state mechanism to enter the same
  mutation protocol without bypassing later projections.
- [FND-0030](https://github.com/edziry/fenestra-research/blob/176c42139776ed9f1ef879cd135bddadaf12a9da/init/findings/FND-0030-typed-invalidation-and-coalescing.md)
  supports explicit typed causes and coalescing as candidates but does not fix
  final flags, propagation, or performance.

The baseline is sufficient for a pure Rust correctness prototype. No new
external source or dependency is required. This unit does not choose among the
full-rebuild, explicit-dirty, dependency-query, or hybrid invalidation
alternatives. It establishes the mutation seam they must all consume.

## Owned responsibility

`fenestra-ui-runtime` owns an unpublished prototype with:

- one runtime instantiated from an exact `ValidatedConstruction` domain;
- stable generational `NodeId` and `FragmentId` values;
- typed property slots initialized from schema defaults and construction
  overrides;
- keyed member state scoped to one runtime fragment;
- bounded transaction staging and structural work;
- one typed mutation log shared by direct and keyed update routes;
- deterministic invalidation accumulation from validated IR metadata;
- immutable committed logical generations and atomic publication;
- typed validation and capacity errors that leave the prior generation intact.

The existing logical tree remains authoritative for node identity, parentage,
and child order. A private fragment arena owns region instance identity, its
owner node, the validated region descriptor, and ordered `(key, root NodeId)`
members. Every occurrence of a nested region receives a distinct `FragmentId`,
even when several occurrences use the same construction `StructuralRegionId`.
Each `FragmentId` combines a private runtime domain with an arena slot and
generation. Lookup verifies all three so equal physical coordinates from
another runtime cannot alias a local fragment. Its debug form is opaque.

## Runtime state and views

The candidate state is:

```text
UiRuntime
  -> exact ValidatedConstruction
  -> Arc<RuntimeState> at RuntimeGeneration
       -> LogicalTree<RuntimeNode>
            -> component symbol
            -> typed property slots
            -> authored child groups: Static(NodeId) or Region(FragmentId)
       -> generational FragmentStore
            -> owner NodeId
            -> StructuralRegionId
            -> ordered keyed member roots
```

Initialization expands the validated root factory iteratively. Static child
factories instantiate once. Every initial region key instantiates the same
validated repeat-body factory, including its static descendants and nested
regions. Property slots copy the component schema defaults, then apply the
validated construction overrides.

The initial committed generation is zero. `CommittedRuntimeSnapshot` is an
immutable receiver-scoped view retaining one `Arc<RuntimeState>`. It can query
root, parent, children, component, property values, owner fragments, region
keys, and keyed member roots. A snapshot captured before a later commit remains
unchanged and participates in the retained-generation bound below.

This is a committed logical and property generation, not a scene, frame,
layout, semantic, hit-test, renderer, or platform snapshot. Those coherent
projections begin in later work units.

Raw arenas, slots, child groups, mutable trees, and fragment maps remain
private. The prototype module exposes no `&mut LogicalTree`, mutable property
slot, unchecked runtime-state constructor, domain token, or raw arena index.

## Explicit capacity

`RuntimeCapacity` has six inclusive `usize` fields and no unbounded default:

1. maximum staged operations per transaction;
2. maximum structural changes per transaction;
3. maximum live logical nodes;
4. maximum live fragment instances;
5. maximum live typed property slots across all nodes;
6. maximum distinct retired runtime generations retained by snapshots,
   transactions, or commit receipts.

A structural change counts each created or retired node and fragment instance.
Move and property operations do not consume this counter. Initialization obeys
the live-node, live-fragment, and live-property-slot limits but is not a
transaction. The first crossing returns `CapacityExceeded` with a typed
`CapacityKind`.

Exceeding staging capacity poisons the transaction. The rejected operation is
not appended, but the already staged prefix cannot later be committed as if no
failure occurred; committing the poisoned transaction returns the same typed
capacity failure without creating a draft.

The runtime keeps only `Weak` bookkeeping for retired generations and removes
expired entries before a real publication. Retiring the current generation
would cross the inclusive distinct-generation limit, the commit fails before
publication. Multiple handles to one generation share its state allocation and
count once. This bounds framework-observable retained state; caller-owned handle
objects themselves remain outside framework memory accounting.

The current property values and operation records have a closed fixed-size
vocabulary, so operation, structural, and property-slot counts bound staged and
live payload growth for this prototype. A later variable-size text, asset,
callback, or user value must add an explicit byte budget before entering this
contract. These capacities are reproducible harness inputs, not ratified product
budgets or allocator-OOM handling.

## Transaction state machine

```text
last committed Arc
  -> staging fixed typed operations
  -> fork private full-state draft
  -> apply operations in authored order
  -> validate draft invariants and capacities
  -> prepare new Arc, typed log, invalidation, and receipt
  -> one infallible pointer replacement
```

`begin_transaction` retains the exact committed `Arc` as its base. A commit
fails as stale if another real commit replaced that base. This pointer check
also prevents a transaction created by another runtime from being accepted;
no public runtime-domain token is needed.

The prototype ownership signatures are:

```text
UiRuntime::committed(&self) -> CommittedRuntimeSnapshot
UiRuntime::begin_transaction(&self) -> UiTransaction
UiRuntime::commit(&mut self, UiTransaction) -> Result<CommitReceipt, TransactionError>
```

`UiRuntime` and `UiTransaction` are not cloneable. A detached transaction may
be staged while `committed()` continues to answer the current generation. The
transaction is consumed by commit. Safe Rust's exclusive `&mut UiRuntime`
enforces one publication owner. Atomic means all-or-nothing replacement for
that owner; it is not a lock-free cross-thread `Arc` acquisition protocol,
thread-safety claim, or scheduler design.

The first implementation deep-clones the complete private state into a draft.
That is a correctness baseline, not a selected persistent-data structure or
performance claim. Draft-created identities never escape before publication.
Every lookup during staging or draft validation observes either the base or the
private draft; ordinary runtime queries continue to observe only the last
committed `Arc`.

All fallible allocation, application, invariant validation, log construction,
and `Arc` construction occurs before publication. Pointer replacement is the
only publication step. The replaced prior `Arc` and deterministic retirement
manifests move into `CommitReceipt`; they are not released inside the mutable
commit phase. A typed error or unwind at any earlier checkpoint drops the draft
and leaves the runtime's committed `Arc` and generation unchanged. The closed
bootstrap values run no application destructor or callback during precommit.
General lifecycle disposal and callback scheduling remain later
responsibilities.

Generation increment uses checked arithmetic. Exhaustion is a typed failure
before publication. A real commit increments exactly once regardless of its
operation count. A true no-op retains both the same generation and the same
state `Arc` and publishes no log or invalidation.

Private unit-test fault hooks may unwind at draft creation, operation apply,
invariant validation, and final preparation. They are not re-exported through
the prototype surface. Caller panic while staging is tested with `catch_unwind`;
the uncommitted transaction is simply discarded.

## Draft invariants and private seams

Every initialization and candidate commit validates all of these invariants:

1. The underlying `LogicalTree` root, parentage, reachability, identities, and
   child links are valid.
2. Flattening each runtime node's authored child groups exactly equals that
   logical node's flat ordered children.
3. Every static child group names one live direct child. Every region group
   names one live fragment with that node as owner and the expected authored
   region descriptor.
4. Every live fragment appears in exactly one reachable region group. No
   fragment-store entry is orphaned or referenced twice.
5. Fragment keys are unique; every ordered member root is live, is a direct
   child of the fragment owner, and appears in the matching flattened position.
6. Nested fragments are reachable through exactly one live member subtree, and
   no node or fragment reference is stale or retired.
7. Each runtime node's template and component resolve through the retained
   construction domain. Its property slots are unique, type-correct, complete,
   and ordered like that component schema.
8. Live node, fragment, and property-slot counts equal stored counts and remain
   within capacity.

The existing arena and logical tree gain only private transaction seams:
`fork_for_transaction` preserves domains and arena generations; live-entry and
subtree traversal support invariant and retirement manifests; internal value
mutation and checked child reordering update the draft. `LogicalTree` never
implements public `Clone`, because two independently mutable trees with the same
identity domain would alias handles. No new seam is re-exported from the
existing logical-tree prototype.

## Typed operations

One transaction stages these operations:

- `set_property(node, property, value)` updates a known typed slot directly;
- `insert_keyed(fragment, key, final_index)` instantiates that fragment's
  validated repeat-body factory;
- `move_keyed(fragment, key, final_index)` reorders only that fragment;
- `update_keyed(fragment, key, property, value)` resolves the member root and
  enters the same property-update helper as `set_property`;
- `remove_keyed(fragment, key)` retires that member root subtree and every
  nested fragment it owns.

Insert indices are valid in `0..=len`; move destinations are final positions in
`0..len`. Moving to the current position is a no-op. A key inserted earlier in
the same transaction may be updated, moved, or removed by later operations.

Keys are unique only within one fragment. Equal keys in two fragment instances
do not collide. Duplicate insert, absent key, out-of-range index, missing or
foreign node or fragment, wrong property for a component, wrong value type,
stale base, generation exhaustion, invariant failure, and each capacity class
are typed errors. Any such failure rejects the whole transaction.

Reinserting a removed key creates a distinct node identity. Insert followed by
remove and remove followed by insert preserve their structural transitions and
identity retirement even when the normalized content appears equal. Structural
operations never trigger whole-tree reconciliation and cannot reorder a static
child or a sibling fragment.

The exact typed taxonomy, payloads, operation indices, and first-error order are
fixed in the [transaction error contract](runtime-transaction-errors.md). It
specifies poisoned staging, stale bases, per-operation semantic checks,
capacity, invariant validation, no-op handling, retained generations, and
generation exhaustion without relying on hash-map iteration.

## Mutation log and invalidation

Every real commit returns one immutable ordered log containing only:

- `PropertyChanged` with node, property, old value, and new value;
- `KeyInserted` with fragment, key, member root, final index, and ordered
  manifests of every created node and nested fragment;
- `KeyMoved` with fragment, key, member root, old index, and final index;
- `KeyRemoved` with fragment, key, retired member root, old index, and ordered
  manifests of every retired node and nested fragment.

Direct and keyed property updates produce the same `PropertyChanged` variant.
Replaceable writes to one slot coalesce at their first operation position while
retaining the original old value and final new value. A write sequence that
returns to the original value disappears. Structural records never coalesce
away because lifecycle and retirement transitions are not replaceable.

Creation manifests list nodes in factory pre-order and fragments when their
authored region slot is encountered. Retirement manifests list descendants and
nested fragments before their owning member root. Their combined lengths are
charged to the structural-change capacity. Property records targeting a node
that is retired later in the same transaction remain in operation order; the
later retirement does not erase an observable transition.

Log fields are private with typed accessors. Manual debug output omits property
values, arena coordinates, validation domains, and internal indices.

Each retained record carries the validated invalidation metadata for its cause.
The commit receipt recomputes the deterministic union after coalescing:

- property changes use the property's schema invalidation;
- insert, move, and remove use the owning region's structural invalidation.

The nine IR classes remain experimental artifact labels. This unit proves cause
selection, union, and rollback only. It does not claim complete propagation,
minimal downstream work, a final bit layout, or selection of invalidation
alternative I1 through I4. Full reconstruction remains available.

## Narrow reference model

WU-0004 tests use a small clean reconstruction with ordinary maps and vectors.
After every step it starts from `ValidatedConstruction` plus fixture-scoped
desired maps for the exercised root property, keyed region, and keyed member
values. It reconstructs without calling runtime mutators or its coalescer and
compares normalized component, template, every declared property, child order,
fragment descriptor, key order, member structure, and global live counts with
the runtime. Physical arena coordinates are excluded from normalized equality;
explicit identity assertions separately check survival, retirement, and
reinsertion.

The model is deliberately local to this unit. WU-0005 generalizes clean
reconstruction into reusable generated sequences, seeds, traces, persisted
failure artifacts, and minimization. WU-0004 does not claim the full projection
or EXP-0001 oracle.

## Required executable evidence

1. Initialization materializes defaults, overrides, static children, initial
   keys, nested factories, and distinct fragment identities.
2. A direct update preserves all identities, emits one `PropertyChanged`, and
   accumulates exactly its declared invalidation.
3. Keyed insert, move, update, remove, and reinsert preserve or retire the
   expected identities and affect only the owning fragment.
4. Keyed update and direct update use the same mutation-record variant.
5. Equal keys in different fragments are independent; foreign and stale node
   and fragment handles fail closed.
6. Empty transactions, same-value writes, and same-position moves retain the
   generation and state `Arc` with an empty receipt.
7. Repeated property writes coalesce, while insert and remove lifecycle records
   remain ordered and observable.
8. Each real multi-operation commit increments generation exactly once, and a
   previous committed view remains immutable afterward.
9. A late invalid operation rolls back every earlier draft change, created or
   retired identity, log entry, and invalidation cause.
10. Operation, structural-change, live-node, live-fragment,
    live-property-slot, and retained-generation capacities fail with their typed
    kind and do not publish.
11. Generation exhaustion and each injected prepublication panic preserve the
    exact prior committed generation.
12. Queries while a transaction is staged observe only the committed state.
13. Deterministic operation tables match the local reference and identity
    models after every step.
14. Internal tree and fragment corruption is rejected before publication.
15. Error and debug formatting disclose no property value, physical slot,
    validation domain, or private draft state.
16. The full typed error corpus verifies priority and operation index, including
    poisoned staging, stale and foreign bases, and multiple failures in one
    operation sequence.
17. A real commit is blocked at the retained-generation limit until an old
    snapshot, transaction, or receipt releases its generation; a no-op does not
    consume retention capacity or stale another transaction.

## Non-goals and replacement boundary

This unit does not add:

- signals, reducers, effects, async tasks, reactive dependency evaluation, or
  component lifecycle callbacks;
- nested or concurrent mutation, callback reentrancy, locks, queues,
  scheduling, frame requests, or renderer and platform backpressure policy;
- computed style, intrinsic measurement, layout, semantics, hit testing,
  scene state, renderer state, damage, GPU resources, or native surfaces;
- final projection invalidation, dirty-root selection, equality cutoffs,
  dependency queries, or performance thresholds;
- serialization, stable ABI, stable public API, facade re-export, or mobile and
  platform behavior;
- arbitrary property bags, DOM mutation, full-tree authoring reconciliation,
  parser behavior, `.fen`, `ui!`, or style syntax.

The full-state draft, arena layouts, capacity values, log storage, coalescing
indexes, and invalidation-set representation remain private and replaceable.
Only semantic operation results, immutable views, typed errors, and atomic
generation behavior cross the documentation-hidden prototype boundary.

## Verification and exit

The unit passes locally only after:

```text
cargo fmt --all -- --check
cargo clippy -p fenestra-ui-runtime --all-targets --all-features --locked -- -D warnings
cargo test -p fenestra-ui-runtime --all-targets --all-features --locked
RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc -p fenestra-ui-runtime --no-deps --locked
```

Workspace tests, dependency direction, `publish = false`, no new dependencies,
`forbid(unsafe_code)`, ASCII, diff cleanliness, and the 400-line file limit are
also required. Windows CI must pass before any cross-platform claim.

Exit: every successful non-noop direct or keyed mutation route returns one
validated typed log and publishes exactly one atomic logical generation. A true
no-op returns an empty receipt with the same generation, and every failed route
publishes neither a log nor a generation. Passing does not establish projection
correctness, performance, reentrancy, scheduling, rendering, native behavior,
or a feasibility gate.
