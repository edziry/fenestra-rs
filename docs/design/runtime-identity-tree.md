# Runtime identity and logical tree plan

Status: active
Work unit: WU-0002
Branch: `feat/runtime-identity-tree`
Research baseline: `fenestra-research` commit `176c42139776ed9f1ef879cd135bddadaf12a9da`
Last updated: 2026-08-08

## Research

The versioned
[feasibility spine contract](https://github.com/edziry/fenestra-research/blob/176c42139776ed9f1ef879cd135bddadaf12a9da/init/architecture/feasibility-spine-contract.md)
already fixes the behavior required for the first identity experiment:

- every logical node has a stable slot and generation;
- every handle is scoped to one owning tree and fails closed in another tree;
- structural removal retires identities transactionally;
- derived data never trusts an unchecked stale handle;
- slot reuse changes the generation and fails closed;
- one owner controls mutable logical parentage.

No new external dependency is needed to test those rules. The required subset
is a private generational arena over `Vec`, `Option`, and a free list from the
Rust standard library. Keeping it private makes an arena crate replaceable if a
later dependency screen finds a better implementation. Adding a dependency now
would enlarge the graph before the necessary API and performance evidence exist.

## Planning

### Owned responsibility

`fenestra-ui-runtime` owns:

- opaque `NodeId` values containing a process-local tree domain, slot, and
  generation;
- allocation, removal, lookup, and safe slot reuse;
- one rooted logical tree with authoritative parent and ordered child links;
- subtree removal that invalidates every descendant handle;
- invariant validation used by tests and later transaction boundaries.

The generation and tree-domain counters are `u64`. A slot at the maximum
generation is retired instead of wrapping and risking aliasing. Creating a tree
after exhausting the domain counter fails closed with a panic. Allocation
failure at process address space limits follows `Vec` behavior and is not
converted into a framework error in this experiment.

The domain is unique only inside one process. It is deliberately not a stable
serialization or trace identifier. Physical arena details are tested inside the
runtime crate and are not exposed through the cross-crate prototype surface;
`NodeId` also uses an opaque debug representation.

### Non-goals

- component, fragment, surface, semantic, or render identities;
- reparenting, keyed reconciliation, transactions, rollback, or snapshots;
- thread safety, serialization, stable ABI, or public facade exposure;
- choosing a general-purpose arena dependency;
- memory or performance conclusions before workloads and budgets exist.

### Prototype boundary

The cross-crate test surface lives under a documentation-hidden `prototype`
module because Rust has no workspace-only visibility. Packages remain version
`0.0.0` and unpublished. The facade crate re-exports none of these types.

### Tests written before behavior

1. A removed handle fails after its slot is reused by a new node.
2. Removing a branch invalidates the branch and every descendant.
3. A handle from another or already destroyed tree cannot alias a live node.
4. Removing and inserting an unrelated node preserves surviving identities.
5. A second root, missing parent, and stale removal return typed errors without
   mutation.
6. Parent and ordered-child links remain reciprocal after each operation.
7. Removing the root empties the tree and permits a new distinct root.
8. A panicking value destructor cannot interrupt structural retirement.
9. Deterministic branching operations match an independent reference model.
10. Deliberately corrupted roots and links produce the expected invariant
    errors.
11. Cross-crate debug and error text do not reveal replaceable arena
    coordinates.

## Implementation constraints

- `#![forbid(unsafe_code)]` remains in force.
- The arena and stored node representation remain private.
- `NodeId` fields and physical arena coordinates remain private.
- Tree mutation validates the target before changing parent or root links.
- Subtree removal allocates its retirement buffer before mutation, unlinks once,
  retires every reachable node, and only then drops removed values.
- No behavior from later transaction or scheduler work is pulled into this unit.

## Verification and exit

The unit passes locally only after:

```text
cargo fmt --all -- --check
cargo clippy -p fenestra-ui-runtime --all-targets --all-features --locked -- -D warnings
cargo test -p fenestra-ui-runtime --all-targets --all-features --locked
RUSTDOCFLAGS="-D warnings" cargo doc -p fenestra-ui-runtime --no-deps --locked
```

The tests must prove stale-handle rejection and tree invariants. Miri remains a
future separate nightly lane and is not claimed here. Passing this unit proves
only the prototype identity boundary; it does not prove transaction atomicity,
reactivity, layout, rendering, or platform support.
