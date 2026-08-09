# Runtime oracle artifact contract

Status: complete locally
Work unit: WU-0005
Branch: `test/runtime-oracles`
Research baseline: `fenestra-research` commit `176c42139776ed9f1ef879cd135bddadaf12a9da`
Last updated: 2026-08-09

## Boundary and ownership

This contract defines the durable synthetic evidence emitted by the
[runtime oracle plan](runtime-oracles.md). Exact generation, encoding, parsing,
trace, and fingerprint rules are in the
[V1 wire and generation contract](runtime-oracle-v1.md); transforms and metrics
are in the [V1 reduction contract](runtime-oracle-reduction-v1.md). Together
they are the Definition of Ready for the WU-0005 codec and golden tests.

The artifact is a logical test record. It is not an EXP-0001 telemetry stream,
stable public format, content-security envelope, or container for private
runtime diagnostics. The library accepts and returns caller-owned bytes. It
never writes a file, uploads data, reads environment state, or chooses a path.

The first committed artifact lives at:

```text
crates/fenestra-ui-testkit/tests/artifacts/known-move-omission-v1.txt
```

Its owner is `fenestra-ui-testkit`. It remains an immutable regression fixture
for the lifetime of failure format V1. Replacement or cleanup requires a
reviewed commit that adds a successor or records why the regression no longer
applies. Passing traces and rejected reduction candidates are transient.

## Versioned schemas

The V1 envelope names independent version `1` values for:

1. failure envelope;
2. fixture and fixture revision;
3. generator and generator configuration;
4. semantic case grammar;
5. normalized-state schema;
6. logical-trace schema;
7. failure-fingerprint schema;
8. reducer and reduction metric.

Identifiers are closed enum words. Unknown or inconsistent versions fail before
semantic replay. An incompatible field, algorithm, transform, or meaning gets a
new version instead of silently reinterpreting V1 bytes.

`GeneratedCaseV1` contains the fixture revision, generator configuration, one
separate `SeedV1(u64)`, and the exact ordered transactions and operations. The
artifact stores both seed and exact original case. Replay uses the stored case;
verification regenerates from the seed and requires identical canonical case
bytes. This keeps an old artifact replayable after later generators exist.

`ReplayConfigV1` separately records all six `RuntimeCapacity` values in their
runtime contract order. IR validation limits are fixed by the registered
fixture revision, not selected by artifact input. The verifier requires replay
capacity and fixture metadata to equal the registered V1 values.

Original transaction and operation IDs are unique and increasing. Reduction
preserves the IDs of surviving records, so gaps are valid in the minimized
case. They are artifact-local correlation values, never runtime identities.

## Logical trace V1

The in-memory `LogicalTraceV1` retains fixture, replay, generator, seed, exact
case, and fault provenance. The failure envelope is its only persistent V1
encoding: provenance comes from the surrounding records and the trace section
contains canonical event bytes. Each transaction emits one event with:

- a dense zero-based event sequence;
- transaction ID and ordered operation IDs;
- logical generation before and after;
- one outcome: `commit`, `noop`, or `reject` with a closed rejection code;
- observed mutation count and invalidation classes;
- one comparison: `match` or `mismatch`.

`commit` means a nonempty receipt and exactly one logical generation advance.
`noop` means an empty receipt with the same generation. `reject` means no
generation advance, zero observed mutations, empty invalidation, and a terminal
candidate-rejection fingerprint. Any mismatch is terminal, so the trace ends at
the first failed comparison. A successful trace covers every case transaction.

Invalidation names use IR declaration order:
`structure,style-match,intrinsic,layout,semantics,hit-test,paint,composition,surface`.
Duplicates are impossible and an empty set encodes as `-`. Mutation records are
observations only; neither they nor invalidation are used as expected-state
truth.

The runner copies these semantic scalars, then drops snapshot and receipt
handles before the next event. Trace V1 contains no clock, process, thread,
machine, platform, local path, runtime handle, panic text, or arbitrary message.
Equal fixture, case, fault, and runtime behavior must produce identical bytes.
WU-0006 owns a separate scheduler trace; WU-0008 may correlate both into the
headless EXP envelope without treating this logical trace as full telemetry.

## Normalized state and fingerprints

`NormalizedStateV1` contains authored-preorder nodes, all schema-ordered
properties, authored static-or-region child groups, fragment paths, descriptor
slots, ordered keys, member paths, and global counts. Observation also verifies
that groups flatten to committed direct children. It contains neither physical
identity nor model incarnation ordinals; those remain in the identity ledger.

The first comparison failure in normalized schema order becomes
`FailureFingerprintV1`. The closed kinds are:

- `CandidateRejected` at global location and field `candidate-outcome`;
- `StateMismatch` at global, node, or fragment location;
- `IdentityMismatch` at one semantic node or fragment location.

The closed state fields are `template,component,property,parent,child-order,
fragment-binding,keyed-order,node-count,fragment-count,property-count`. The only
identity field is `identity-lifecycle`; for handle aliasing it reports the
later semantic node or fragment path in authored order as `aliased`.
Commit/no-op, generation, mutation count, and
invalidation remain trace observations already covered as runtime behavior by
WU-0004; only an unexpected rejection becomes a fingerprint. Valid combinations
and summaries are fixed by the V1 wire contract.

The step or transaction is stored outside the fingerprint, so deleting
unrelated prefix work does not change failure identity. The known fault must
produce `StateMismatch` at the primary fragment's `keyed-order`, with the exact
expected and observed key summaries fixed by the directed prefix.
Every fingerprint writes the expected summary before the observed summary.

## Minimized failure V1

The envelope stores, in canonical order:

- every version and fixture identity field;
- replay capacity, generator configuration, and seed;
- the exact original case;
- test-only fault kind and target operation ID;
- original first-failure transaction, optional operation, and fingerprint;
- reducer configuration, evaluation count, and completion status;
- exact minimized case;
- minimized first-failure transaction, optional operation, and identical
  fingerprint;
- canonical minimized logical trace.

Completion is `fixed-point` or `budget-exhausted`. Only `fixed-point` means no
V1 transform remaining in the documented search order preserved the same
fingerprint. Neither status means global minimality.

The committed known-failure artifact is fixed-point. With `OmitMoveV1`, both
stored cases reproduce the same fingerprint. The minimized case without the
fault passes, proving the artifact records the injected candidate defect rather
than an invalid operation sequence.

## Cross-record consistency

Structural decoding and semantic verification are separate operations. Before
an artifact is accepted as replayable, verification requires:

- all repeated version, fixture, replay, generator, config, seed, and fault
  metadata to be identical and to match the registered fixture;
- declared transaction, operation, event, and byte counts to match their
  sections and hard ceilings;
- minimized transaction and operation IDs to be ordered subsequences of the
  originals, with retained transaction boundaries;
- the target operation to exist in both cases and remain a `MoveKeyed`;
- each failure transaction and optional operation to exist in its case;
- original and minimized failure transaction and optional operation IDs to be
  identical;
- original and minimized fingerprints to be exactly equal;
- regeneration from seed to equal the stored original case bytes;
- original replay to fail first at its stored transaction and fingerprint;
- the minimized trace to match the minimized case from start through its first
  failure, including transaction and operation IDs;
- the terminal trace mismatch to equal the minimized failure record;
- fault-free replay of the minimized case to pass;
- rerunning reduction from the original to reproduce the exact minimized case,
  status, and used-evaluation count;
- only for `fixed-point`, reducing the minimized case again to return the same
  case with zero accepted transforms. `budget-exhausted` carries no such claim.

A parser cannot establish synthetic provenance from an arbitrary number. The
closed fixture registry supplies the synthetic seed and data used by the
committed artifact; publication review confirms that provenance.

## Privacy and retention

V1 has no field for user text, pixels, clipboard data, tokens, secrets, URLs,
local file paths, titles, source content, native handles, runtime IDs, arena
domains, addresses, panic payloads, process data, or machine identity. Authored
numeric IDs and closed property values are permitted only inside the registered
synthetic fixture revision.

Unexpected failures return an in-memory bounded representation. They are never
automatically written, uploaded, or committed. Before adding another artifact,
its canonical decoder, semantic verification, closed-schema privacy test, ASCII
scan, size limit, fixture owner, and retention decision must pass review. WU-0005
performs no CI upload.

The committed artifact remains in Git until explicit reviewed cleanup or
migration. If text, assets, images, platform data, or variable-size values enter
a later fixture, they require a new schema version, byte budget, redaction
policy, owner, retention period, and cleanup path before persistence.

## Required executable evidence

1. Encoding is canonical and decode-encode returns identical bytes.
2. Every syntax, version, canonicality, limit, ordering, reference, and semantic
   verification class returns its typed error without panic.
3. Equal replay inputs produce byte-identical traces and artifacts.
4. The schema exposes no field capable of carrying a physical runtime identity
   or arbitrary private payload.
5. Stored seed regeneration equals the exact stored original case.
6. Original and minimized replay produce the stored first fingerprint under
   `OmitMoveV1`; fault-free minimized replay passes.
7. Reduction is strictly smaller for the known case, deterministic, bounded,
   and fixed-point idempotent while preserving the exact fingerprint.
8. At least one rejected candidate fails differently, proving that any-failure
   matching is not accepted.
9. The committed artifact remains within every V1 limit and passes the closed
   synthetic-fixture privacy audit.

Passing establishes one durable logical regression artifact. It does not
establish cryptographic integrity, academic minimality, exhaustive coverage,
full telemetry, platform determinism, or product support. TM-0018 and EXP
artifact-integrity work remain open because V1 intentionally defines no digest.
