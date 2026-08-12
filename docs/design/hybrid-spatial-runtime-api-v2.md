# WU-0013 runtime spatial publication API contract

Status: frozen for runtime RED/GREEN
Work unit: WU-0013
Spatial snapshot:
[hybrid-spatial-snapshot-api-v2.md](hybrid-spatial-snapshot-api-v2.md)
Reference semantics:
[hybrid-spatial-reference-v2.md](hybrid-spatial-reference-v2.md)
Runtime transactions: [runtime-transactions.md](runtime-transactions.md)
Runtime scheduler: [runtime-scheduler.md](runtime-scheduler.md)
Format: runtime spatial publication API version 2

## Boundary and public surface

This cut lets the single-owner runtime rebuild and publish one reference
spatial snapshot from an immutable logical draft. The runtime owns the rebuild
program, viewport, spatial limits, layout engine, accepted logical mapping,
and completed snapshot. The rebuild program owns only the translation from a
borrowed logical view to raw candidate-neutral spatial input.

The cut adds exactly six `prototype` exports, taking the runtime surface from
66 to 72 names and from 47 to 51 structs:

```text
RuntimeSpatialProgramV2
RuntimeSpatialBuildViewV2
RuntimeSpatialInputV2
RuntimeSpatialViewV2
RuntimeSpatialErrorV2
SpatialViewportChangeViewV2
```

It adds no public mode enum, runtime-spatial configuration object, mutable
view, candidate-output seam, presenter, renderer type, artifact type, or
dependency other than the existing workspace `fenestra-ui-spatial` package.
This cut adds no other public constructor, method, associated constant,
function, trait, alias, module, enum variant, or mutation-record variant beyond
the exact surface listed below; existing public method sets otherwise remain
unchanged.

## Runtime-owned build program

```rust
pub trait RuntimeSpatialProgramV2:
    Send + Sync + Unpin + UnwindSafe + RefUnwindSafe + 'static
{
    #[must_use]
    fn build(
        &self,
        runtime: RuntimeSpatialBuildViewV2<'_>,
        viewport: SpatialViewportV2,
    ) -> RuntimeSpatialInputV2;
}

pub struct RuntimeSpatialBuildViewV2<'a> { /* private fields */ }

impl<'a> RuntimeSpatialBuildViewV2<'a> {
    pub fn root(self) -> NodeId;
    pub fn node_count(self) -> usize;
    pub fn template(self, node: NodeId) -> Option<TemplateNodeId>;
    pub fn component(self, node: NodeId) -> Option<ComponentTypeId>;
    pub fn property(
        self,
        node: NodeId,
        property: PropertyId,
    ) -> Option<&'a PropertyValue>;
    pub fn parent(self, node: NodeId) -> Option<NodeId>;
    pub fn children(self, node: NodeId) -> Option<&'a [NodeId]>;
    pub fn fragment(
        self,
        owner: NodeId,
        descriptor: StructuralRegionId,
    ) -> Option<FragmentId>;
    pub fn keyed_members(
        self,
        fragment: FragmentId,
    ) -> Option<KeyedMemberIter<'a>>;
    pub fn keyed_member(
        self,
        fragment: FragmentId,
        key: u64,
    ) -> Option<NodeId>;
}
```

Every build-view method is `#[must_use]`. The view exposes only the listed
logical queries, with the same behavior as their same-named
`CommittedRuntimeSnapshot` methods. It omits generation, fragment and property
counts, allocation identity, headless projection, and spatial publication. It
borrows the private draft only for the `build` call and exposes no mutation,
raw arena, runtime domain, unchecked lookup, transaction, or retained snapshot
handle.

The view implements exactly `Clone + Copy` plus the runtime auto-trait set. It
has no public constructor and implements no `Debug`, `Display`, `Default`,
equality, ordering, or hash trait. The program is called synchronously through
one runtime-owned `Box<dyn RuntimeSpatialProgramV2>`. It must return only from
its arguments and retained immutable configuration. The runtime does not
retain the build view or invoke the program from any observation method.
Program callbacks occur only synchronously during initialization or rebuild.

```rust
pub struct RuntimeSpatialInputV2 { /* private fields */ }

impl RuntimeSpatialInputV2 {
    #[must_use]
    pub fn new(
        source: Arc<SpatialOwnedInputV2>,
        logical_nodes: Box<[NodeId]>,
    ) -> Self;
}
```

The input has no getter. It is consumed by runtime validation and implements
the runtime auto-trait set, but no `Clone`, `Copy`, `Debug`, `Display`,
`Default`, comparison, ordering, or hash trait. The source Arc moves unchanged
into spatial preparation. Resolution therefore does not clone image bytes or
any other raw table.

## Logical mapping and validation

Spatial key zero is the synthetic viewport sentinel and has no logical node.
For each mapping index `i`, `logical_nodes[i]` belongs to
`SpatialNodeKeyV2::new(i + 1)`. The mapping covers every non-sentinel topology
record exactly once. It may omit logical wrappers, and spatial parentage need
not match logical ownership, but one logical node cannot own two spatial keys
in this version.

```rust
pub enum RuntimeSpatialErrorV2 {
    ViewportMismatch,
    MappingLengthMismatch,
    MissingLogicalNode { key: SpatialNodeKeyV2 },
    DuplicateLogicalNode { key: SpatialNodeKeyV2 },
    Resolve(SpatialResolveErrorV2),
}
```

The error implements `Clone + Copy + Eq + PartialEq + Debug + Display + Error`
plus the runtime auto-trait set. It has no `ALL`, public method, `Default`,
`Hash`, `Ord`, or `PartialOrd`. `Debug` and `Display` are manual and redact the
logical identity, viewport, raw input, source allocation, and resolver
evidence. Their exact labels are:

```text
runtime-spatial-error(viewport-mismatch)
runtime-spatial-error(mapping-length-mismatch)
runtime-spatial-error(missing-logical-node)
runtime-spatial-error(duplicate-logical-node)
runtime-spatial-error(resolve)
```

`Debug` wraps the selected label as
`RuntimeSpatialErrorV2(<display-label>)`. The `Error` implementation reports no
source. Callers that need typed resolver detail pattern-match `Resolve`.

Runtime validates one returned wrapper in this exact order:

1. The source viewport must equal the viewport supplied to `build`.
2. Mapping length must equal `topology.nodes().len().saturating_sub(1)`.
3. If the topology contains more than `u32::MAX + 1` records, runtime invokes
   `resolve_spatial_v2` immediately. Its direct-count phase returns the exact
   `LimitExceeded(Nodes)` error before runtime forms an unrepresentable mapping
   key or scans an entry.
4. Otherwise mapping entries are scanned from spatial key one upward. At each
   key the logical node must be live in the draft, then must not have appeared
   at an earlier key. The first failure wins. A duplicate reports its second
   key.
5. `resolve_spatial_v2` runs with the runtime-owned engine and limits. Its
   complete existing diagnostic priority remains unchanged.

No hash iteration selects an error. Wrapper faults intentionally precede raw
spatial-input faults. The accepted mapping remains beside the exact resolved
snapshot; it is never reconstructed from logical or spatial parentage.

## Construction, resize, and observation

```rust
impl UiRuntime {
    pub fn new_spatial(
        style: ValidatedStyleProgram,
        program: Box<dyn RuntimeSpatialProgramV2>,
        viewport: SpatialViewportV2,
        limits: SpatialLimitsV2,
        capacity: RuntimeCapacity,
    ) -> Result<Self, RuntimeInitializationError>;

    #[doc(hidden)]
    pub fn new_spatial_with_layout_engine(
        style: ValidatedStyleProgram,
        program: Box<dyn RuntimeSpatialProgramV2>,
        viewport: SpatialViewportV2,
        limits: SpatialLimitsV2,
        capacity: RuntimeCapacity,
        layout_engine: Box<dyn LayoutEngineV1>,
    ) -> Result<Self, RuntimeInitializationError>;
}

impl UiTransaction {
    pub fn resize_spatial(
        &mut self,
        viewport: SpatialViewportV2,
    ) -> Result<(), TransactionError>;
}

impl CommittedRuntimeSnapshot {
    #[must_use]
    pub fn spatial(&self) -> Option<RuntimeSpatialViewV2<'_>>;
}

pub struct RuntimeSpatialViewV2<'a> { /* private fields */ }

impl<'a> RuntimeSpatialViewV2<'a> {
    #[must_use]
    pub fn snapshot(self) -> &'a SpatialResolvedSnapshotV2;
    #[must_use]
    pub fn logical_node(self, key: SpatialNodeKeyV2) -> Option<NodeId>;
    #[must_use]
    pub fn spatial_key(self, node: NodeId) -> Option<SpatialNodeKeyV2>;
}
```

`new_spatial` uses `ReferenceStackEngineV1`. The hidden constructor exists for
candidate-neutral layout conformance and fault injection. Both constructors
materialize typed property values from the validated style program, rather
than accepting a bare construction. The spatial build program therefore sees
the same final values stored in the generation.

`RuntimeSpatialViewV2` implements exactly `Clone + Copy` plus the runtime
auto-trait set. It has no public constructor and no other standard trait. Key
zero, an absent key, and a foreign or unmapped logical node return `None`.
Reverse lookup is unique because wrapper validation rejects duplicates; this
contract makes no indexing or complexity promise. A
`CommittedRuntimeSnapshot` retains its exact outer state Arc, so an old
snapshot and every spatial view borrowed from it remain valid and unchanged
while later program callbacks run and after a later commit publishes.

## Projection modes and transaction records

Ordinary, provisional headless, and spatial runtimes are three exclusive
private modes. There is no transaction that switches modes. Their observation
matrix is exact:

```text
mode       headless_projection()    spatial()
ordinary   None                     None
headless   Some                     None
spatial    None                     Some
```

`resize_headless` committed against an ordinary or spatial runtime returns
`HeadlessUnavailable`. `resize_spatial` committed against an ordinary or
headless runtime returns `SpatialUnavailable`. An available spatial resize is
coalesced like a headless resize. Equal extent is a no-op; several staged
resizes retain only the original and final viewport; returning to the original
viewport removes the record.

The mutation surface gains:

```rust
MutationRecordView::SpatialViewportChanged(SpatialViewportChangeViewV2<'a>)

pub struct SpatialViewportChangeViewV2<'a> { /* private fields */ }

impl SpatialViewportChangeViewV2<'_> {
    #[must_use]
    pub const fn old_viewport(self) -> SpatialViewportV2;
    #[must_use]
    pub const fn new_viewport(self) -> SpatialViewportV2;
}
```

The view implements exactly `Clone + Copy` plus the runtime auto-trait set and
no other standard trait. A retained viewport change contributes exactly
`Layout | Semantics | HitTest | Paint | Composition`. It does not contribute
`Surface`: this viewport is present logical spatial input, not native surface
or lifecycle state.

The closed error vocabularies gain these variants in the shown positions:

```text
RuntimeInitializationErrorKind:
CapacityExceeded, Headless, Spatial(RuntimeSpatialErrorV2), InvariantViolation

TransactionErrorKind:
CapacityExceeded, Headless, Spatial(RuntimeSpatialErrorV2),
HeadlessUnavailable, SpatialUnavailable, StaleBase, MissingNode,
MissingFragment, MissingKey, DuplicateKey, UnknownProperty,
PropertyTypeMismatch, IndexOutOfBounds, GenerationExhausted,
InvariantViolation
```

`SpatialUnavailable` retains the responsible resize operation index.
`Spatial(_)` has no operation index in this first seam, including when final
viewport input came from a resize. Fine source attribution belongs to the
later typed IR mapper and is not guessed from arbitrary program reads.

## Rebuild and atomic publication order

Initialization performs these phases in order:

1. Materialize generation zero and its styled property slots.
2. Validate logical invariants and runtime capacities.
3. Call the spatial build program over that immutable state and viewport.
4. Validate the wrapper and resolve a complete reference snapshot.
5. Own the accepted mapping and snapshot in generation zero, then return the
   runtime owner.

Every effective nonempty transaction rebuilds spatial state after logical
draft validation and mutation coalescing. This is required even when its
reported invalidation excludes spatial-looking classes: the program can read
any exposed property, and this API has no declared read dependency set. The
receipt preserves the exact union of mutation invalidation; rebuilding does
not invent extra invalidation bits.

Commit order remains poison, exact-base check, authored operation application,
logical validation, mutation coalescing, no-op return, spatial rebuild,
retained-generation capacity, checked generation increment, final Arc
preparation, and one outer state pointer replacement. A true no-op calls no
program or layout engine and preserves the outer state allocation, inner
spatial allocation, source Arc, mapping, snapshot, and generation.

All program output, wrapper validation, layout calls, spatial resolution,
mapping ownership, and Arc allocation complete before publication. A typed
failure drops the replacement and preserves the prior logical state, viewport,
source, mapping, snapshot, generation, and scheduler visual state. A program
or engine panic is not converted to a typed error, but unwinding before the
pointer replacement has the same rollback property. No foreign callback is
retained or invoked after publication.

The immutable generation retains one private Arc-owned spatial publication.
That publication owns the completed `Arc<SpatialResolvedSnapshotV2>` and its
matching logical mapping. A rebuild never mutates either in place. This needs
no `ArcSwap`, lock, unsafe code, pinning, self-reference, or public Arc getter.

## Scheduler integration and deferrals

The scheduler continues to retain only `CommittedRuntimeSnapshot` as visual
work. That outer snapshot now retains the spatial publication transitively, so
pending, offered, and submitted frames observe the exact logical and spatial
generation together. Replacement, submission, completion, retirement,
retained-generation backpressure, and the fixed 40-byte protocol weight do not
change. `FrameWork::snapshot().spatial()` is the only new access path; the
scheduler never rebuilds spatial state or holds the program.

Runtime always calls `resolve_spatial_v2`, which materializes the reference
output. It does not accept `PreparedSpatialV2`, candidate output, a
candidate-validated snapshot, or an admitted candidate token from callers.
Candidate field comparison and admission remain evidence-only later work.

This cut also defers typed spatial IR and format versioning, manual and `.fen`
and `ui!` lowering, precise property-read invalidation, presentation-frame
types, logical hit-result joins, native presenters, candidate renderers,
artifacts, and clean-reconstruction evidence. Density, physical pixels,
safe-area and keyboard insets, lifecycle, surface epochs, platform handles,
and multi-scene state remain excluded with WU-0012.
