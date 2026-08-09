# Runtime oracle plan

Status: complete locally
Work unit: WU-0005
Branch: `test/runtime-oracles`
Research baseline: `fenestra-research` commit `176c42139776ed9f1ef879cd135bddadaf12a9da`
Last updated: 2026-08-09

## Research

The immutable research baseline already fixes the correctness and evidence
requirements for this unit:

- The [feasibility spine contract](https://github.com/edziry/fenestra-research/blob/176c42139776ed9f1ef879cd135bddadaf12a9da/init/architecture/feasibility-spine-contract.md)
  requires every incremental result to be compared with a correct full
  reconstruction, keeps that reconstruction as a replaceable fallback, and
  requires deterministic evidence for negative results.
- [EXP-0001](https://github.com/edziry/fenestra-research/blob/176c42139776ed9f1ef879cd135bddadaf12a9da/init/experiments/EXP-0001-feasibility-spine.md)
  requires correctness before timing and enough versioned input and output to
  reproduce a failed comparison.
- [EXP-0006](https://github.com/edziry/fenestra-research/blob/176c42139776ed9f1ef879cd135bddadaf12a9da/init/experiments/EXP-0006-invalidation-scale.md)
  requires generated mutation workloads to retain their seed and minimized
  failures as regression fixtures.
- The [priority and measurement plan](https://github.com/edziry/fenestra-research/blob/176c42139776ed9f1ef879cd135bddadaf12a9da/init/experiments/priority-and-measurement-plan.md)
  requires the oracle, fixture, identifiers, artifact format, fault behavior,
  privacy, and retention rules to be fixed before an experiment is run.
- The [threat model](https://github.com/edziry/fenestra-research/blob/176c42139776ed9f1ef879cd135bddadaf12a9da/init/security/threat-model.md)
  makes fixtures synthetic by default and treats durable traces and failure
  artifacts as a privacy boundary.

The alternatives considered for this narrow, closed operation grammar are:

1. fixed hand-authored tables alone;
2. an external property-testing, random-generation, snapshot, or serialization
   dependency;
3. a repository-owned deterministic generator, clean model, reducer, and small
   canonical codec built with `std`.

Tables alone cannot exercise enough valid state transitions. A general testing
or serialization dependency would add a dependency audit and would not remove
the need to own stable semantic paths, versions, bounds, and artifact privacy.
Option 3 is selected for WU-0005. It makes no statistical, fuzzing, global
minimality, or randomness-quality claim. The modules remain replaceable if a
later corpus justifies a dependency under REQ-DEP-001.

Current primary documentation was screened for
[Proptest 1.11.0](https://docs.rs/proptest/1.11.0/proptest/test_runner/struct.Config.html)
and [QuickCheck 1.1.0](https://docs.rs/quickcheck/1.1.0/quickcheck/struct.Gen.html).
Proptest exposes seeded generation, shrinking, and failure persistence;
QuickCheck exposes a seeded generator but scopes value stability to a release.
Neither removes the need for this repository's stable semantic case and
artifact versions. No dependency is admitted. Full EXP telemetry remains later
work; WU-0005 defines a smaller logical test trace.

## Owned responsibility and dependency boundary

`fenestra-ui-testkit` owns the unpublished prototype for:

- one versioned synthetic runtime fixture;
- semantic node and fragment addressing independent of runtime handles;
- a desired-state model and full clean reconstruction;
- observation and normalization of a committed runtime snapshot;
- deterministic bounded transaction generation from a seed;
- replay and comparison after every transaction boundary;
- transient identity-lifecycle checks;
- deterministic logical traces and failure fingerprints;
- test-only candidate fault adaptation and failure reduction;
- canonical, bounded, synthetic failure artifacts.

The dependency direction remains:

```text
fenestra-ui-testkit -> fenestra-ui-runtime -> fenestra-ui-ir
                   -> fenestra-ui-ir
```

IR, runtime, the facade, and future product packages must not depend on or
re-export the testkit. The disposable EXP-0001 probe may consume it, as already
declared by the workspace graph. No IR or runtime API change is required. The
fixture-scoped `CleanModel` in WU-0004 remains a runtime regression test;
moving it would create the wrong dependency direction and would share candidate
logic with the oracle.

All cross-crate testkit types remain under a documentation-hidden `prototype`
module. They are not stable API or serialization contracts outside the
versioned test artifacts.

## Synthetic fixture V1

`RuntimeOracleFixtureV1` uses only the validated provisional IR and closed
property values. It contains:

- a root with scalar, visibility, color, and input-policy properties;
- one static child;
- a primary keyed region with initial keys `7` and `8`;
- an item repeat body with a static descendant and a nested keyed region;
- an adjacent secondary keyed region that may reuse key `7` locally;
- explicit construction overrides and distinct invalidation declarations.

The fixture ID and revision are artifact data. They are not inferred from raw
IR IDs or an unstable hash. The fixture builder validates its schema and
construction through the ordinary bounded IR APIs, then initializes the runtime
with explicit capacity. Fixture values are synthetic and contain no user text,
asset, path, platform, or native data.

The directed prefix exercises direct set, true no-op, multiple writes in one
transaction, insert, move, keyed update, remove, reinsert, local equal keys,
and one nested region. A seeded tail adds valid state-dependent transactions.
Coverage of these lanes must not depend on a generated choice.

## Semantic addressing

Persisted operations never contain `NodeId`, `FragmentId`, arena coordinates,
domains, addresses, pointer-derived values, or their `Debug` output.

`NodePathV1` starts at `Root` and contains ordered segments:

- `Static { authored_slot: u16 }` selects the static child at an authored child
  slot of the current template;
- `Member { region_slot: u16, key: u64 }` selects the keyed member at an
  authored region slot of the current node.

`FragmentPathV1` contains an owner `NodePathV1` and an authored region-slot
ordinal. Slots are ordinals in `TemplateFactory::children()` and are checked
against the exact retained construction. `PropertyId` and closed
`PropertyValue` values remain authored semantic data tied to the fixture
revision. Path depth and every numeric conversion are bounded and checked.

Runtime handles are resolved only from the current committed snapshot while a
transaction is prepared. They are never retained in a generated case, trace,
fingerprint, reducer input, or artifact.

## Semantic transaction grammar

Each `TransactionV1` has an artifact-local `TransactionId(u32)` and one to four
ordered operations. Each operation retains a unique, monotonically assigned
`OperationId(u32)` even if later reduction removes surrounding operations.

The closed operation variants are:

- `SetProperty { node, property, value }`;
- `InsertKeyed { fragment, key, final_index: u32 }`;
- `MoveKeyed { fragment, key, final_index: u32 }`;
- `UpdateKeyed { fragment, key, property, value }`;
- `RemoveKeyed { fragment, key }`.

Authored indices use fixed-width artifact integers and are converted to
`usize` only after bounds checks. A transaction may exercise coalescing or a
no-op. Every node or fragment target must exist in the transaction's base
snapshot, and a target incarnation cannot have been retired by an earlier
operation in that transaction. A base fragment may insert a key and then move,
update, or remove that new key because all four operations use the already
resolved base fragment handle. `SetProperty` cannot target a node created in
the same transaction, and no operation can target a nested fragment created in
that transaction. The generator enforces these rules while it advances a
desired-state draft after each selected operation.

## Desired state and clean reconstruction

`DesiredStateV1` retains only semantic state:

- property overrides keyed by `(NodePathV1, PropertyId)`;
- ordered live keys keyed by `FragmentPathV1`;
- a logical incarnation ordinal for each removed and reinserted member path.

Initial values and unmodified key orders come from `ValidatedConstruction`.
Removing a member purges every descendant override and fragment order under its
path. Reinsertion therefore reconstructs the repeat body from authored defaults
and receives a new logical incarnation.

After each transaction the clean builder starts from the construction root and
the candidate desired state. It uses an iterative worklist and checked
arithmetic to produce `NormalizedStateV1` in authored preorder. The normalized
schema contains:

- semantic node path, parent path, template, and component;
- every declared property and its effective value in schema order;
- ordered authored child groups, with static paths and fragment paths, plus
  verified flattening against committed direct children;
- each fragment path, descriptor, ordered keys, and member paths;
- total node, fragment, and property-slot counts.

The builder does not call runtime mutation methods, use the runtime coalescer,
consume mutation manifests as truth, inspect private state, or parse `Debug`.
Physical identity is deliberately absent from normalized equality.

An independent iterative observer walks `CommittedRuntimeSnapshot` from its
root using public snapshot queries and the expected construction factories. It
checks root parentage, template and component resolution, all properties,
authored child segmentation, fragment descriptors, key order, direct
parentage, and global reported counts. Traversal is bounded by expected
semantic paths rather than unique physical handles: a repeated handle under
two distinct paths is retained for the identity ledger, while duplicate
semantic paths remain invalid. Semantic totals must equal the snapshot's node,
fragment, and property-slot counts so unreachable extras cannot pass. The first
unequal normalized field produces a structured semantic fingerprint. Adjacent
empty regions are checked as distinct public fragment descriptors, but the
oracle makes no claim about their unobservable private child-group layout.

## Transient identity ledger

An in-memory `IdentityLedger` separately maps semantic node and fragment paths
to opaque runtime handles during one replay. It verifies:

- paths that survive a direct update, move, or unrelated change preserve their
  handle;
- removed node and fragment handles no longer resolve;
- a later incarnation at the same semantic path receives a different handle;
- equal keys in different fragment paths do not alias.

The ledger stores retired handles only through the next relevant transition.
Each lifecycle token includes every keyed ancestor incarnation, so reinserting
an outer member refreshes its static descendants and nested fragments too.
Tokens are model bookkeeping, not normalized or serialized, because a snapshot
alone cannot observe them.

## Generator V1

`GeneratorV1` is a repository-owned deterministic word transition, not a
cryptographic or statistical random-number generator. `GeneratorConfigV1`
records the requested total transaction count, operations-per-transaction
ceiling, and live-key ceiling; the seed is a separate `SeedV1` value and has one
canonical location in an artifact. Configuration too small for the directed
prefix is a typed error rather than partial coverage.

The exact wrapping transition, directed prefix, value catalog, action order,
word consumption, and ID assignment are fixed in the
[V1 wire and generation contract](runtime-oracle-v1.md). No hash-map iteration,
process state, clock, thread, retry loop, or runtime identity affects
generation. The exact generated transaction sequence is persisted alongside
the seed because a future generator version must not make an old failure depend
on regenerated bytes.

The regression corpus uses 32 fixed seeds, at most 64 transactions per seed,
at most four operations per transaction, and at most 256 operations total. A
test pins exact output for selected seeds. Equal V1 fixture, config, and seed
must produce byte-identical cases. Selected unequal seeds must exercise a
different tail; no universal collision claim is made.

## Replay and candidate fault boundary

Replay starts a fresh fixture runtime and desired state. For every transaction:

1. compare the current committed state before staging;
2. prepare operations in order, resolving only base-snapshot targets while a
   desired-state clone advances and enforces the transaction target rules;
3. stage the corresponding operations in one `UiTransaction`;
4. commit once and record scalar observations;
5. validate receipt, allocation, mutation, invalidation, and generation shape;
6. rebuild and compare the complete normalized state;
7. check identity lifecycle, then release snapshot and receipt handles.

The desired clone is published only after the candidate commit succeeds. A
generated candidate rejection is a harness failure, not an accepted alternate
result. Snapshot and receipt data are normalized into owned semantic values and
dropped before the next step so a harness run succeeds with a retained-runtime-
generation capacity of one. Preparation retains a staged-index-to-operation-ID
map so a runtime error is correlated without serializing a handle.

The only known fault is a private testkit adapter, `OmitMoveV1`, targeting one
artifact-local operation ID. It validates the original move but omits that move
when staging the candidate; the desired model still applies it. It does not add
a runtime hook, feature, dependency, or product behavior. The directed fixture
moves an inserted member across a nonempty list, so omission produces:

```text
StateMismatch { location: fragment path, field: keyed order }
```

The same case without the adapter must pass. Detecting this injected defect
validates sensitivity of the harness; it does not prove the runtime defect-free.

## Bounds and deterministic failures

All ceilings are inclusive harness limits, not product budgets:

- 64 transactions, four operations per transaction, and 256 operations total;
- 12 live keyed memberships in the complete desired state;
- generated keys in `0..=31` and semantic path depth at most eight;
- 256 normalized nodes, 128 fragments, and 1,024 property slots;
- at most 16,384 applicable actions at one generated choice;
- 256 KiB for a transient logical trace and 512 KiB for a failure artifact;
- 128 KiB for either encoded case, 64 KiB for the embedded minimized trace,
  and 4,096 reducer evaluations.

Every length sum, index conversion, path extension, factory expansion, and
encoded-byte count uses checked arithmetic. The finite applicable-action set is
built once per choice and includes valid same-value sets. The registered fixture
always supplies one; an empty set is `NoApplicableAction`, never a retry or a
synthesized operation.

`HarnessErrorKind` has closed variants for unsupported versions, invalid
configuration, each limit, fixture validation, invalid semantic path, invalid
operation, runtime initialization, unexpected candidate rejection, state
mismatch, identity mismatch, trace mismatch, and arithmetic exhaustion. Errors
carry transaction and operation IDs only when those locations exist and contain
no runtime handle or free-form payload.

Priority is: envelope and config, harness limits, fixture validation and
initialization, pre-step comparison, authored semantic operation validation,
candidate stage and commit including commit-shape validation, normalized state
comparison, identity comparison, then trace capacity. Traversals use authored
order; the first mismatch in normalized schema order wins.

## Required executable evidence

1. The fixture starts equal under independent clean reconstruction and snapshot
   observation, including nested and adjacent regions.
2. The directed sequence covers all five operation variants, a true no-op,
   coalescing, remove and reinsert, local equal keys, and nested structure.
3. Every normal directed and generated transaction matches clean reconstruction
   before and after its commit.
4. The identity ledger observes preservation, retirement, and a fresh handle on
   reinsertion without persisting any handle.
5. The same V1 fixture, config, and seed produce identical operations and trace
   bytes; selected different seeds produce different seeded tails.
6. The 32-seed corpus completes within every declared bound and with retained
   generation capacity one.
7. Boundary and overflow tests terminate with the documented typed limit and
   first-error priority.
8. `OmitMoveV1` is detected at the exact first divergent fragment order, while
   the unmodified candidate passes.
9. The stored seed regenerates the exact original case, and the exact stored
   case replays without regeneration.
10. Reduction preserves the same fingerprint and the committed minimized
    artifact replays without private logs.

Artifact ownership, privacy, and retention are fixed in the
[runtime oracle artifact contract](runtime-oracle-artifacts.md). Exact V1
generation, wire encoding, trace fields, parser priority, cross-record
invariants are fixed in the [V1 wire and generation contract](runtime-oracle-v1.md).
Reduction order and metric are fixed in the
[V1 reduction contract](runtime-oracle-reduction-v1.md).

## TDD sequence

1. Add failing fixture, semantic-path, and initial clean-equality tests.
2. Implement the fixture, desired state, iterative builder, and observer.
3. Add failing seed determinism, directed-lane, bound, and long-replay tests.
4. Implement V1 generation and replay with transient identity checks.
5. Add failing canonical codec, malformed-input, trace, and privacy tests.
6. Implement the logical trace and bounded artifact codec.
7. Add the failing known-fault, reproduction, and reduction tests.
8. Implement the private fault adapter and deterministic reducer.
9. Commit the synthetic minimized golden and replay it as a regression.
10. Run the fixed seed corpus and all workspace gates before recording result.

Tests that expose a missing API should first fail to compile. Behavioral tests
must then fail for the expected mismatch or unimplemented result. Red output is
recorded in the eventual verification artifact without citing temporary logs or
ephemeral commits.

## Non-goals and replacement boundary

WU-0005 does not add or claim:

- layout, style, semantics, hit testing, scene, frame, renderer, platform, GPU,
  image, or pixel oracles;
- final invalidation propagation, dirty-region correctness, or full EXP-0001
  trace and environment manifests;
- property-testing quality, fuzzing, exhaustive state-space coverage, global
  minimality, benchmarks, or performance budgets;
- invalid runtime handle generation, stale-base races, transaction error
  priority, or capacity fault generation already owned by WU-0004;
- file-system writes, CI uploads, network transfer, arbitrary user fixtures,
  text, assets, callbacks, or variable-size property values;
- stable public APIs, facade exports, stable ABI, or a general serialization
  format.

Fixture IDs, path syntax, generator transition, value catalog, normalized
schema, trace schema, reduction transforms, bounds, and codec are versioned and
replaceable together. Passing validates only this logical harness and fixture.

## Verification and exit

The unit passes locally only after:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p fenestra-ui-testkit --all-targets --all-features --locked
cargo test -p fenestra-ui-runtime --all-targets --all-features --locked
cargo test --workspace --all-targets --all-features --locked
RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --workspace --no-deps --locked
cargo metadata --format-version 1 --no-deps --locked
cargo tree --workspace --edges normal --locked
git diff --check
```

The golden bytes, exact seed regeneration, trace determinism, malformed and
oversize rejection, privacy schema, dependency direction, unpublished status,
unchanged dependency lock, `forbid(unsafe_code)`, ASCII, and 400-line file limit
are separate required checks. Windows CI must pass before a cross-platform
determinism claim.

Exit: one injected incremental correctness failure is reproducible from a
committed synthetic artifact, with the same minimized semantic fingerprint and
without private logs or physical runtime identities. This is not an EXP-0001
gate, performance result, or platform-support claim.
