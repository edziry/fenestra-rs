# WU-0004 runtime transactions verification

Status: complete locally
Result: pass
Date: 2026-08-08
Branch: `feat/runtime-transactions`
Research baseline: `fenestra-research` commit `176c42139776ed9f1ef879cd135bddadaf12a9da`

## Research and planning

The owned behavior, atomic publication boundary, capacities, invariants,
mutation records, invalidation semantics, non-goals, and replacement seams are
recorded in the [runtime transactions plan](../design/runtime-transactions.md).
The exact failure taxonomy and deterministic precedence are recorded in the
[runtime transaction error contract](../design/runtime-transaction-errors.md).

The immutable research baseline at the exact commit above supplies ADR-0001,
the feasibility spine, the authoring/runtime boundary, and the typed
invalidation finding. Those artifacts were sufficient for this pure Rust
correctness unit. No new external source, crate, or dependency was required.

## TDD and review evidence

The initial direct and keyed integration fixtures were written before the
runtime surface existed. Their first runs failed at compile time with unresolved
prototype imports for the runtime, capacity, snapshot, receipt, mutation, and
keyed lifecycle types. The first complete transaction implementation then made
both fixtures executable with local keyed fragments, shared property mutation,
expansion, movement, and retirement.

Bounded failure and fault-injection review tests followed the initial runtime
implementation. The capacity modifiers and error taxonomy already existed;
compilation failed because the private commit hooks and generation test seam did
not. The added controls remain private and `cfg(test)`.

Review found a deterministic capacity regression after the first green. Its
fixture creates an initially empty outer region whose repeat body contains a
binary multiplicity chain. The first insert failed with:

```text
assertion failed: left == right
  left: CapacityExceeded(LiveNodes)
 right: CapacityExceeded(StructuralChanges)
```

The fix retains overflow while measuring the factory, then applies the
context-specific priority during initialization or transaction preflight. The
same regression now reports `StructuralChanges` with operation index zero
without attempting the expansion.

The three corruption cases were written before their test-control variants
existed. The red run reported missing
`CorruptPropertiesBeforeValidation`, `CorruptTreeBeforeValidation`, and
`CorruptFragmentBeforeValidation` variants. The private hooks now corrupt only
the draft, and all three cases return `InvariantViolation` without publication.

The clean reconstruction, coalescing matrix, retention assertions, diagnostic
privacy checks, and several audit-strengthening cases were added after the
first behavior green. They are review evidence rather than being described as
individually red-first. The final runtime suite covers:

- initialization from defaults, overrides, static children, keyed factories,
  nested fragments, and distinct runtime identities for repeated factory
  occurrences;
- direct and keyed property changes through one typed mutation variant;
- keyed insert, move, update, remove, reinsert, local key scope, and ordered
  creation and retirement manifests;
- deterministic coalescing without erasing structural lifecycle records;
- true no-ops with the same state allocation and logical generation;
- complete rollback after earlier property, creation, movement, and retirement
  work when a later operation fails;
- caller and injected prepublication panics, plus property, tree, and fragment
  corruption, without changing the committed allocation;
- all transaction error kinds, every capacity kind, operation indices, stale
  and foreign identities, first-operation priority, and capacity tie-breaks;
- independent retained-generation pressure from snapshots, transactions, and
  receipts, including exact no-publication assertions;
- a fixture-scoped clean reconstruction after every deterministic direct and
  keyed step, comparing all declared properties, authored structure, fragments,
  keys, member roots, and global live counts;
- explicit identity survival, retirement, and reinsertion checks separate from
  normalized clean equality;
- opaque public diagnostics for errors, snapshots, receipts, mutation records,
  node and fragment handles, and lifecycle manifests.

## Verification

The following commands passed locally with Rust 1.97.1 and Cargo 1.97.1:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p fenestra-ui-runtime --all-targets --all-features --locked
cargo test --workspace --all-targets --all-features --locked
RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --workspace --no-deps --locked
cargo metadata --format-version 1 --no-deps --locked
cargo tree --workspace --locked
git diff --check
```

Observed result: all 68 runtime tests passed, consisting of 16 unit tests and
52 integration tests. All 90 workspace tests passed, consisting of those 68
runtime tests and the 22 existing IR tests. No test was ignored or failed.
Formatting, Clippy, and rustdoc completed with warnings denied.

Metadata confirms that every workspace package remains unpublished. The
dependency tree confirms that `fenestra-ui-runtime` depends only on
`fenestra-ui-ir`; the IR has no dependencies, and no external crate was added.
The facade still does not re-export the transaction prototype.

The ASCII scan over the changed runtime, tests, plans, and this artifact passed.
`git diff --check` passed. The runtime crate retains `forbid(unsafe_code)`, and
the compiler and Clippy gates found no unsafe block. The 400-line check passed:
the largest relevant file is the 392-line runtime transaction plan, while the
largest Rust file is the 363-line logical tree implementation.

## Result

Result: pass for WU-0004's local correctness boundary.

Every successful non-noop direct and keyed route now returns one ordered typed
log and publishes exactly one validated logical runtime generation. No-ops
return an empty receipt while retaining the exact allocation and generation.
Typed failures, capacity exhaustion, invariant corruption, and covered unwinds
publish neither a log nor a generation and preserve the exact prior committed
allocation.

This result is sufficient to start WU-0005's reusable generated oracle and
failure-artifact work. It is not a pass for EXP-0001 or any framework
feasibility gate.

## Limitations

- `prototype` remains documentation-hidden, unpublished, and absent from the
  `fenestra-ui` facade. None of these types or signatures is a stable API.
- `RuntimeGeneration` is logical state only. It is not a layout, semantic,
  hit-test, scene, renderer, frame, GPU-resource, or native-surface generation.
- The full-state clone is a correctness baseline. No persistence structure,
  incremental projection algorithm, or performance characteristic is selected.
- Capacities are reproducible test inputs, not ratified product budgets. They
  type configured and arithmetic exhaustion after validated IR inputs already
  exist; allocator OOM and process abort are not recoverable typed failures.
- Property values and operation records use a closed fixed-size bootstrap
  vocabulary. Variable-size text, assets, callbacks, or user values require
  byte budgets before entering this contract.
- Invalidation evidence covers declared cause selection, deterministic union,
  coalescing, and rollback only. It does not prove propagation, minimal work,
  dirty-root selection, or any final invalidation representation.
- The clean model is specific to the versioned WU-0004 fixture and deterministic
  sequence. WU-0005 still owns generators, seeds, traces, persisted artifacts,
  shrinking, and a reusable clean-rebuild oracle.
- There is no scheduler, queue, reentrant mutation, callback execution,
  lifecycle disposal, async work, lock-free publication, or cross-thread reader
  protocol. These boundaries remain with WU-0006 and later work.
- `NodeId` and `FragmentId` domains are process-local and use 64-bit atomics.
  They are not serialized identities, and target atomic support plus mobile
  portability remain future audits.
- Verification ran on the local Linux environment. No Windows CI result is
  versioned for this unit, so no Windows behavior or cross-platform result is
  claimed.
- No Miri, performance, memory-profile, MSRV, mobile-target, renderer, layout,
  style, accessibility, platform, or native-window result is claimed.
