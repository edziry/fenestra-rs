# WU-0002 runtime identity and tree verification

Status: complete locally
Result: pass
Date: 2026-08-08
Branch: `feat/runtime-identity-tree`
Research baseline: `fenestra-research` commit `176c42139776ed9f1ef879cd135bddadaf12a9da`

## Research and planning

The evidence, chosen private arena boundary, invariants, non-goals, test plan,
and replacement rule are recorded in the
[runtime identity and logical tree plan](../design/runtime-identity-tree.md).
The existing feasibility-spine baseline was sufficient; no dependency was added.

## TDD evidence

The integration tests were written before the `prototype` module existed. The
first run failed as expected:

```text
cargo test -p fenestra-ui-runtime --test logical_tree --locked
error[E0432]: unresolved import `fenestra_ui_runtime::prototype`
```

The minimum implementation then added a private generational arena and rooted
logical tree. Review exposed three missing failure cases. Tests for them were
added before their fixes, and the focused suite failed with:

```text
handles_are_scoped_to_their_originating_tree ... FAILED
identities_do_not_repeat_when_a_tree_is_recreated ... FAILED
subtree_retirement_finishes_before_removed_values_are_dropped ... FAILED
test result: FAILED. 7 passed; 3 failed
```

A later boundary audit found that derived debug output still exposed private
arena coordinates. Its regression test failed before the manual opaque formatter
was added:

```text
left: "NodeId { tree: TreeId(1), arena: ArenaId { slot: 0, generation: 0 } }"
right: "NodeId(..)"
```

The final suite contains two arena unit tests, seven internal corruption tests,
and eleven logical-tree integration tests covering:

- stale-handle rejection after slot reuse;
- rejection of foreign handles and identities from destroyed trees;
- full descendant invalidation after subtree removal;
- preservation of surviving identities under unrelated mutations;
- typed errors without partial mutation;
- root removal and creation of a distinct replacement root;
- structural consistency when a removed value's destructor panics;
- 1,000 deterministic branching operations checked against an independent
  model;
- negative validation of stale roots and children, missing roots, incorrect
  parentage, duplicate reachability, and unreachable nodes;
- opaque debug and error output across the prototype boundary;
- retirement rather than wraparound at generation `u64::MAX`.

## Verification

The following commands passed with Rust 1.97.1:

```text
cargo fmt --all -- --check
cargo clippy -p fenestra-ui-runtime --all-targets --all-features --locked -- -D warnings
cargo test -p fenestra-ui-runtime --all-targets --all-features --locked
RUSTDOCFLAGS="-D warnings" cargo doc -p fenestra-ui-runtime --no-deps --locked
```

Observed result: 20 tests passed, 0 failed. Clippy and rustdoc completed with
warnings denied.

## Limitations

- The `prototype` module is public only because Rust has no workspace-only
  visibility; it is documentation-hidden, unpublished, and not re-exported by
  `fenestra-ui`.
- Arena coordinates and ordering are private, so the arena remains a replaceable
  implementation detail.
- Tree domains are process-local safety tokens, not stable serialized or trace
  identities.
- Removed values still run their destructors synchronously after structural
  retirement. WU-0004 and WU-0006 must keep those destructors outside runtime
  locks and callback-forbidden transaction phases.
- A future trace needs its own opaque, correlatable token because `NodeId(..)`
  intentionally hides identity coordinates.
- The domain allocator currently requires 64-bit atomics. The mobile and target
  inventory must either verify that capability or replace the allocator behind
  the private boundary.
- Reparenting, keyed reconciliation, transaction rollback, snapshots,
  concurrency, serialization, and performance remain outside WU-0002.
- Miri remains pending until a separately governed nightly lane exists.

This result proves only the local identity and rooted-tree invariants. It does
not prove the later atomic transaction or headless-spine requirements.
