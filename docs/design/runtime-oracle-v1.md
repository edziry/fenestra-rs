# Runtime oracle V1 wire and generation contract

Status: complete locally
Work unit: WU-0005
Branch: `test/runtime-oracles`
Research baseline: `fenestra-research` commit `176c42139776ed9f1ef879cd135bddadaf12a9da`
Last updated: 2026-08-09

## Fixed ceilings

All ceilings are inclusive. `ArtifactLimitKind::ALL` in this order defines the
tie-break when one input crosses several limits:

1. `ArtifactBytes`: 524,288 bytes;
2. `LineBytes`: 1,024 bytes excluding LF;
3. `Lines`: 4,096 lines;
4. `CaseBytes`: 131,072 bytes for each case section;
5. `TraceBytes`: 65,536 bytes for the embedded minimized trace;
6. `Transactions`: 64 in each case;
7. `OperationsPerTransaction`: four;
8. `Operations`: 256 in each case;
9. `PathDepth`: eight node segments; `/r` does not add depth;
10. `TraceEvents`: 64;
11. `ReductionEvaluations`: 4,096.

The two case sections plus the embedded trace consume at most 327,680 bytes;
the fixed grammar has fewer than 24 non-section records, each subject to the
line ceiling, so every permitted section combination fits the envelope limit.
The in-memory normal-run trace has a separate 262,144-byte ceiling. Generator
choices have at most 16,384 applicable actions. Normalized state has at most 256
nodes, 128 fragments, 1,024 property slots, and 12 live keyed memberships.
Generated keys are limited to `0..=31`.

`HarnessLimitKind::ALL` fixes runtime-harness tie-break order:
`Transactions,OperationsPerTransaction,Operations,LiveMemberships,PathDepth,
NormalizedNodes,NormalizedFragments,NormalizedProperties,ApplicableActions,
TraceBytes`. `GeneratorErrorKind` is `InvalidConfig`,
`LimitExceeded(HarnessLimitKind)`, `NoApplicableAction`, or
`ArithmeticExhausted`. The first config field in declaration order wins; while
generating, authored transaction then operation order wins.

The caller's input bytes already exist; codec limits bound scanning and decoded
allocation, not that prior allocation. Encoders return a typed output-limit
error instead of partial bytes. The decoder can retain at most 128 transactions
and 512 operations across both cases. Every counter and conversion is checked.

## Generator V1

`GeneratorConfigV1` contains total transaction count, maximum operations per
transaction, and maximum live keyed memberships. The separate seed is `u64`.
Valid config requires at least eight transactions, two operations per
transaction, eight live memberships, and no field above its fixed ceiling. The
requested transaction count includes the eight-transaction directed prefix.

The repository-owned word transition uses wrapping `u64` arithmetic:

```text
state_0 = seed xor 0xa0761d6478bd642f
state_n = state_(n-1) * 0xd1342543de82ef95 + 0x9e3779b97f4a7c15
word_n  = state_n xor rotate_right(state_n, 29)
```

The directed prefix consumes no words and assigns transaction IDs `0..=7` and
operation IDs `0..=9`:

1. set root width to `320`, then `480` in one transaction;
2. set root width to `480` as a true no-op;
3. insert primary key `9` at index `2`, then move it to index `0`;
4. update primary key `9` value to `90`;
5. update secondary key `7` value to `70`;
6. insert nested key `2` below primary key `9` at index `1`;
7. remove primary key `9`;
8. reinsert primary key `9` at index `2`.

The known artifact targets operation ID `4`, the directed move. At that point
the expected primary order is `9,7,8`; omission leaves `7,8,9`.

The committed known-failure case uses seed `1592614637` and config
`transactions=16,max_operations=4,max_live_memberships=12`. The ordinary
regression corpus uses seeds `0..=31` with
`transactions=64,max_operations=4,max_live_memberships=12`; seeds `0` and `1`
must have different canonical seeded tails.

For every tail transaction, consume one word and choose
`1 + word % max_operations_per_transaction` operations. For each operation,
build the complete applicable-action vector, consume one word, and choose
`word % action_count`. Applying the selected operation to the desired draft
precedes enumeration of the next operation. IDs continue consecutively. No
other action consumes a word.

Semantic preorder is depth-first from root. At each template, visit authored
slots in order: a static slot visits its complete subtree; a region emits its
fragment, then visits each current key's complete member subtree in keyed order
before the next slot. Node and fragment target lists and their order are frozen
at transaction start. Later operations use current draft key order within those
base fragments; retired targets are skipped and newly created node or fragment
targets are not added.

Action vectors use this variant order:

1. `SetProperty` for base-snapshot nodes in semantic preorder, schema property
   order, then value-catalog order;
2. `InsertKeyed` for base-snapshot fragments in semantic preorder, free key in
   ascending order, then final index ascending;
3. `MoveKeyed` for base-snapshot fragments, current draft key order, then final
   index ascending, including the current index;
4. `UpdateKeyed` for base-snapshot fragments, current draft key order, repeat
   body schema property order, then value-catalog order;
5. `RemoveKeyed` for base-snapshot fragments and current draft key order.

Targets retired earlier in the transaction are excluded. A key inserted into a
base fragment may be moved, updated, or removed later in that transaction.
Nodes and fragments created in that transaction are never direct targets. An
action crossing configured live membership or harness bounds is excluded.

The value catalogs are ordered exactly:

- Bool: `false, true`;
- ScalarI32: `-1024, -1, 0, 1, 1024`;
- Rgba8: `00000000, 000000ff, ffffffff, 336699ff`;
- InputPolicy: `accept, ignore`.

If the vector is empty, generation fails with `NoApplicableAction`; it does not
retry or synthesize a value. The registered fixture always has at least one
same-value `SetProperty` action. Selected config and seed pin exact case bytes.

## Registered fixture and replay config

Fixture `runtime-oracle`, revision `1`, uses schema format `1`, namespace
`5001`, schema revision `1`, and construction format `1`. Its IR validation
limits are compile-time fixture data and cannot be overridden by an artifact.

Root template `0` uses component `0` and child slots: static template `1` at
slot `0`, primary region `0` at slot `1`, and secondary region `2` at slot `2`. Root
properties are width `0`, visible `1`, color `2`, and input policy `3`.
Primary initial keys are `7,8`; its item template is `2` with scalar value
property `0`. Item slot `0` is static template `3`; item slot `1` is a nested
region `1` with initial key `1` and repeat template `4`. The secondary region has
initial key `7` and its separately owned repeat template `5`, also with scalar
value property `0`. Templates `1..=5` use components `1..=5` respectively.
Repeat templates are not shared between regions.

All anchors are synthetic. Component and property declarations are:

```text
component 0: width 0 = i32:100 [layout,paint]
             visible 1 = bool:true [semantics,hit-test,paint]
             color 2 = rgba8:000000ff [paint]
             input 3 = input:accept [hit-test]
component 1: visible 0 = bool:true [semantics,paint]
component 2: value 0 = i32:10 [intrinsic,layout,paint]
             visible 1 = bool:true [semantics,hit-test,paint]
component 3: color 0 = rgba8:ffffffff [paint]
component 4: value 0 = i32:1 [intrinsic,layout,paint]
component 5: value 0 = i32:20 [intrinsic,layout,paint]
```

Root width has construction override `i32:120`; no other template has an
override. Primary and nested regions invalidate `structure,layout,paint`;
secondary invalidates `structure,paint`. Validation limits, in constructor
order, are `6,10,6,3,5,1,4,3,9`. Changing any fixture field requires revision
`2`; an artifact cannot select an alternate builder.

`ReplayConfigV1` is exactly, in runtime contract order:

```text
operations=4, structural_changes=64, live_nodes=256,
live_fragments=128, live_property_slots=1024, retained_generations=1
```

## Primitive encoding

Encoding is ASCII with `|` fields and exactly one final LF. Empty files, CR,
tabs, spaces, non-ASCII bytes, missing final LF, and blank lines are invalid.
Line length excludes LF. Unsigned decimal has no leading zero except `0`.
Signed decimal has no `+`, leading zero, or negative zero. RGBA8 is eight
lowercase hexadecimal digits.
Canonical digits outside their declared fixed-width type are
`NonCanonicalValue`, evaluated before semantic or configured-limit checks.

Property values are one field: `bool:false`, `bool:true`, `i32:<signed>`,
`rgba8:<hex>`, `input:accept`, or `input:ignore`. Lists use comma-separated
canonical elements or `-` when empty. No primitive contains `|`.

A node path is `root` followed by zero or more `/s:<u16>` or
`/m:<u16>:<u64>` segments. A fragment path is a node path followed by
`/r:<u16>`. Path depth counts only `s` and `m` segments.

## Envelope grammar

Every record below is one line. Literal words appear exactly as shown:

```text
fenestra-oracle-failure|1
versions|fixture|1|generator|1|case|1|state|1|trace|1|fingerprint|1|reducer|1
fixture|runtime-oracle|1|<schema-format>|<schema-namespace>|<schema-revision>|<construction-format>
replay|<operations>|<structural>|<nodes>|<fragments>|<property-slots>|<retained-generations>
generator|<transaction-count>|<max-operations>|<max-live-memberships>
seed|<u64>
original-begin|<transaction-count>|<operation-count>|<canonical-case-bytes>
<case records>
original-end
fault|omit-move|<operation-id>
failure|original|<transaction-id>|<operation-id-or->|<fingerprint fields>
reducer|<max-evaluations>|<used-evaluations>|<fixed-point|budget-exhausted>
minimized-begin|<transaction-count>|<operation-count>|<canonical-case-bytes>
<case records>
minimized-end
failure|minimized|<transaction-id>|<operation-id-or->|<fingerprint fields>
trace-begin|<event-count>|<canonical-trace-bytes>
<event records>
trace-end
end
```

`schema-format`, `schema-revision`, and `construction-format` are canonical
`u32`; `schema-namespace` and seed are canonical `u64`. Replay capacities,
generator config, section counts and byte counts, transaction and operation
IDs, fault target, failure IDs, reducer counts, and trace counts are canonical
`u32`. The registered fixture supplies exact expected metadata. Byte counts
cover only records between paired begin and end lines, including record LF.

Case records are:

```text
tx|<transaction-id>|<operation-count>
op|<operation-id>|set|<node-path>|<property-u32>|<value>
op|<operation-id>|insert|<fragment-path>|<key-u64>|<index-u32>
op|<operation-id>|move|<fragment-path>|<key-u64>|<index-u32>
op|<operation-id>|update|<fragment-path>|<key-u64>|<property-u32>|<value>
op|<operation-id>|remove|<fragment-path>|<key-u64>
```

Every transaction has at least one operation. Original transaction and
operation IDs increase strictly. Minimized IDs increase strictly and retain
their original relative order and transaction boundary IDs.

## Fingerprint grammar

Fingerprint fields after the failure transaction and optional operation IDs
are; the first summary is expected and the second is observed:

```text
candidate-rejected|global|candidate-outcome|kind:accept|kind:<rejection-code>
state-mismatch|<location>|<state-field>|<summary>|<summary>
identity-mismatch|<node-or-fragment-location>|identity-lifecycle|<lifecycle>|<lifecycle>
```

Locations are `global`, `node:<node-path>`, or `fragment:<fragment-path>`.
State fields are `template,component,property,parent,child-order,
fragment-binding,keyed-order,node-count,fragment-count,property-count`. Legal
summaries are:

- `none`, `count:<u32>`, `template:<u32>`,
  `component:<u32>`, or `property:<u32>:<value>`;
- `node:<node-path>`, `nodes:<node-path-list>`,
  `children:<child-group-list>`, or
  `keys:<u64-list>`;
- `binding:present` or `binding:absent`;
- `kind:accept` or `kind:<closed-rejection-code>`;
- lifecycle `absent`, `preserved`, `fresh`, or `retired`.

Lists inside one summary use commas. A child-order field requires `nodes` on
both sides for flat-order failure or `children` on both sides for authored
group failure. Child-group entries are `s:<node-path>` or
`r:<fragment-path>`. Keyed-order requires `keys`; each count field requires `count`;
template, component, property, parent, and fragment binding require their
matching summary families. Identity mismatch requires two lifecycle words;
`distinct` and `aliased` are additional node or fragment lifecycle words. Candidate
rejection uses one closed mapped runtime error code and never an error message.

Valid fingerprint combinations are exactly:

- candidate rejection: global, `candidate-outcome`, expected `kind:accept`,
  observed rejection kind; V1 does not independently predict commit versus noop;
- node state: template, component, or property with matching typed expected
  summary and matching typed or `none` observed summary; parent with `node` or
  `none`; child order with equal summary families on both sides;
- fragment state: `fragment-binding` with expected `binding:present` and
  observed `binding:absent`, or `keyed-order` with two key lists;
- global state: each count field with two counts;
- node or fragment identity: unequal lifecycle words; handle aliasing uses the
  later authored path with expected `distinct` and observed `aliased`.

No other location, field, summary family, equality, or use of `none` is valid.

Rejection codes are `capacity-operations,capacity-structural,
capacity-live-nodes,capacity-live-fragments,capacity-live-properties,
capacity-retained-generations,stale-base,missing-node,missing-fragment,
missing-key,duplicate-key,unknown-property,property-type-mismatch,
index-out-of-bounds,generation-exhausted,invariant-violation`. They map one to
one to the runtime transaction taxonomy. Initialization failure is a harness
setup error and cannot be persisted as a reducible transaction fingerprint.
State and identity mismatch records use `-` for the optional operation. A
candidate rejection maps `TransactionError::operation_index()` through the
prepared staged-index table; `None` remains `-`.

The comparison order is authored-preorder nodes, each node's template,
component, properties, and parent; then authored-preorder fragments, fragment
binding and keyed order; then node child order, global counts, and identity lifecycle. State mismatch
wins over lifecycle at the same transaction. Candidate rejection wins because
no post-commit state exists.

## Trace event grammar

Trace events are:

```text
event|<sequence>|<transaction-id>|<operation-id-list>|<before-generation>|<after-generation>|<outcome>|<mutation-count>|<invalidation-list>|<match|mismatch>
```

Operation IDs use a nonempty comma list. Outcome is `commit`, `noop`, or
`reject:<closed-rejection-code>`. Invalidation uses the ordered names from the
artifact contract or `-`. Event sequence is dense from zero. Transaction and
operation IDs exactly match successive minimized-case records.
Sequence, transaction ID, operation IDs, and mutation count are canonical
`u32`; before and after generations are canonical `u64`.

Commit advances generation by one and has positive mutation count. Noop keeps
generation and has zero mutations and empty invalidation. Reject keeps
generation, has zero mutations and empty invalidation. Reject or mismatch is
terminal; the last event must be mismatch and correspond to the minimized
failure transaction. The fault metadata comes from the envelope and is part of
the trace replay input rather than repeated in every event.

## Decode and verification priority

`decode_failure_artifact` performs only structural work. First error wins in
this total order:

1. `LimitExceeded(ArtifactBytes)` before scanning;
2. `InvalidAscii` at the first CR or byte outside printable ASCII plus LF;
3. first `LineBytes` crossing, then first `Lines` crossing; missing final LF is
   `MalformedRecord` at the final one-based line;
4. header and `UnsupportedVersion` in declared version-field order;
5. the section state machine: the expected tag is consumed; an already consumed
   singleton or section marker is `DuplicateSection` at its repeated line; a
   known future marker is `MissingSection(expected)` at that line; EOF reports
   the missing expected marker without a line; a case record outside its section
   is `OrderingViolation`; an unknown tag, bad field count, or bad enum word is
   `MalformedRecord`;
6. numeric and path canonicality in field order;
7. configured-limit crossings in `ArtifactLimitKind::ALL` order, then declared
   count equality in section order; begin-line fields use left-to-right order
   and transaction operation counts use transaction order;
8. ID order, section references, fault target, and trace cross-references;
9. legal fingerprint location, field, and summary combinations;
10. trailing records after `end`.

`ArtifactDecodeErrorKind` is closed: `LimitExceeded(ArtifactLimitKind)`,
`InvalidAscii`, `MalformedRecord`, `UnsupportedVersion(VersionKind)`,
`NonCanonicalValue`, `MissingSection(SectionKind)`,
`DuplicateSection(SectionKind)`,
`OrderingViolation`, `CountMismatch(CountKind)`, `InvalidReference`,
`InvalidFingerprint`, and `TrailingData`. `VersionKind`
uses `Envelope,Fixture,Generator,Case,State,Trace,Fingerprint,Reducer` order.
Encoding typed data can return only `LimitExceeded`; it never emits partial
bytes.

`SectionKind` follows grammar order: `Header,Versions,Fixture,Replay,Generator,
Seed,Original,Fault,OriginalFailure,Reducer,Minimized,MinimizedFailure,Trace,End`.
`CountKind` is `Transactions,Operations,CaseBytes,OperationsPerTransaction,
TraceEvents,TraceBytes`. A mismatch is anchored to the begin, transaction, or
trace-begin line that declared it.

Errors use one-based line numbers; an error not tied to a line has `None`.
`InvalidAscii` includes invalid UTF-8 because every valid byte is ASCII.
`LimitExceeded` carries one `ArtifactLimitKind`. Limits apply separately to
original and minimized cases except the whole-envelope byte and line ceilings.

`verify_failure_artifact_v1` then applies this order: fixture identity and
schema metadata, replay configuration, semantic paths, semantic operations,
seed regeneration, original replay, minimized replay, exact fingerprint
equality, trace replay, fault-free replay, then reduction reproduction and
conditional fixed-point verification.

The path phase visits original operation targets, original fingerprint
location/expected/observed, minimized operation targets, then minimized
fingerprint location/expected/observed. It validates authored topology only;
dynamic key presence and relationships among individually valid fingerprint
paths are checked by operation or replay phases. Only after this global path
pass are original then minimized operations validated in stored order. The
verifier returns a separate closed verification error
with the first transaction or operation ID when available. Neither error type
echoes source lines, values, runtime handles, or `Debug` text.

`ArtifactVerificationErrorKind` is closed: `FixtureMismatch`,
`ReplayConfigMismatch`, `InvalidSemanticPath`, `InvalidSemanticOperation`,
`SeedMismatch`, `OriginalFailureMismatch`, `MinimizedFailureMismatch`,
`FingerprintMismatch`, `TraceMismatch`, `FaultFreeReplayFailed`, and
`ReductionMismatch`, in that precedence order.

Reduction and its metric are fixed separately in the
[V1 reduction contract](runtime-oracle-reduction-v1.md).
