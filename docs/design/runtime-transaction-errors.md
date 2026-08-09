# Runtime transaction error contract

Status: active
Work unit: WU-0004
Branch: `feat/runtime-transactions`
Research baseline: `fenestra-research` commit `176c42139776ed9f1ef879cd135bddadaf12a9da`
Last updated: 2026-08-08

This contract is part of the
[runtime transactions plan](runtime-transactions.md). It exists separately to
keep the main design and the deterministic error corpus focused.

## Taxonomy

`CapacityKind::ALL` contains, in order:

```text
Operations
StructuralChanges
LiveNodes
LiveFragments
LivePropertySlots
RetainedGenerations
```

This `ALL` order is also the capacity tie-break. When one initialization or
applied operation would cross several applicable limits, the earliest
applicable kind in this list wins. Semantic reference validation precedes
apply-time capacity. `Operations` is the exception: staging detects it before
semantic validation and stores that failure as the transaction poison.

`TransactionErrorKind` is the closed prototype taxonomy:

```text
CapacityExceeded(CapacityKind)
StaleBase
MissingNode
MissingFragment
MissingKey
DuplicateKey
UnknownProperty
PropertyTypeMismatch
IndexOutOfBounds
GenerationExhausted
InvariantViolation
```

`TransactionError` has private fields plus typed `kind()` and
`operation_index()` accessors. Operation indices are zero-based. Poisoned
staging and per-operation failures retain the responsible attempted index.
Stale-base, generation, retained-generation, and global invariant failures have
no operation index.

Initialization uses the narrower `RuntimeInitializationErrorKind` taxonomy:

```text
CapacityExceeded(CapacityKind)
InvariantViolation
```

Only `LiveNodes`, `LiveFragments`, and `LivePropertySlots` can arise during
initialization. No transaction or retired generation exists yet.

## Deterministic priority

Commit selects one failure in this order:

1. A stored staging poison wins and prevents draft creation.
2. The exact base `Arc` must still match; a stale or foreign transaction fails.
3. Operations run in authored order, and the first failing operation wins.
4. Within `set_property`, node, property, then value type are checked.
5. Within keyed insert, fragment, duplicate key, then final index are checked.
6. Within keyed move or remove, fragment, then existing key are checked; move
   checks the final index afterward.
7. Within keyed update, fragment, existing key, property, then value type are
   checked.
8. Capacity is checked only after the current operation's semantic references
   are valid and immediately before bounded work would cross its limit.
9. Complete draft invariants are checked after all operations.
10. An empty final log is a true no-op and returns before retention or
    generation-exhaustion checks.
11. Retained-generation capacity is checked, then generation increment is
    checked, then final artifacts are prepared and published.

The operation-capacity check occurs while staging. The rejected operation is
not appended, the transaction stores that exact error, and every later staging
call or commit returns it. The attempted operation index is the length the
operation vector had before rejection.

No diagnostic is selected by hash-map iteration. Identical input, capacity, and
base generation therefore return the same kind and operation index.
