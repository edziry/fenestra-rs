# WU-0013 runtime symbolic spatial adapter API contract

Status: frozen for runtime IR adapter RED/GREEN
Work unit: WU-0013
Symbolic IR: [symbolic spatial IR](hybrid-spatial-ir-v2.md)
Runtime publication:
[runtime spatial publication](hybrid-spatial-runtime-api-v2.md)
Raw spatial API: [spatial API](hybrid-spatial-api-v2.md)
Format: runtime symbolic spatial adapter API version 2

## Boundary and compatibility

This cut adds one built-in, fallible path from a
`ValidatedSpatialProgramV2` to the existing atomic runtime spatial
publication. The validated program retains the exact validated style domain,
so the new constructor accepts no separate style or construction. Runtime
uses that retained style for generation-zero values and every later logical
draft.

The existing `RuntimeSpatialProgramV2` callback, `RuntimeSpatialInputV2`,
`UiRuntime::new_spatial`, and
`UiRuntime::new_spatial_with_layout_engine` retain their signatures and
behavior. The callback stays infallible and remains the manual raw-input and
fault-injection lane. `ValidatedSpatialProgramV2` does not implement or hide
behind that trait. The two lanes share publication validation and the raw
resolver but have separate private runtime configuration variants.

The runtime cut adds exactly two `prototype` exports, taking the runtime
surface from 72 to 74 names and from 51 to 52 structs:

```text
RuntimeSpatialIrErrorKindV2
RuntimeSpatialIrErrorV2
```

It adds exactly two methods to `UiRuntime`, taking that method set from eight
to ten. It adds no public mapper, mode enum, configuration type, trait, alias,
constant, mutation record, transaction method, observation method, or
dependency.

## Construction surface

```rust
impl UiRuntime {
    pub fn new_spatial_ir(
        program: ValidatedSpatialProgramV2,
        viewport: SpatialViewportV2,
        limits: SpatialLimitsV2,
        capacity: RuntimeCapacity,
    ) -> Result<Self, RuntimeInitializationError>;

    #[doc(hidden)]
    pub fn new_spatial_ir_with_layout_engine(
        program: ValidatedSpatialProgramV2,
        viewport: SpatialViewportV2,
        limits: SpatialLimitsV2,
        capacity: RuntimeCapacity,
        layout_engine: Box<dyn LayoutEngineV1>,
    ) -> Result<Self, RuntimeInitializationError>;
}
```

`new_spatial_ir` uses `ReferenceStackEngineV1`. Both constructors derive the
exact construction and style from `program.style()`, materialize logical
generation zero once, then materialize and resolve the symbolic spatial
program before returning. A symbolic materialization or spatial resolution
failure returns `RuntimeInitializationErrorKind::Spatial` and no runtime.
Logical generation-zero capacity and invariant failures retain their existing
initialization kinds and precede the spatial adapter.

Neither constructor is `#[must_use]`. The engine-injection constructor has
only `#[doc(hidden)]`. Existing constructor attributes and signatures remain
unchanged.

## Shared direct-count preflight

The spatial package adds exactly one `prototype` export, taking its surface
from 120 to 121 names without adding a struct or any other public item:

```rust
#[must_use = "direct-count preflight errors must be handled"]
pub fn preflight_spatial_direct_counts_v2(
    observed: [u128; 12],
    limits: SpatialLimitsV2,
) -> Result<(), SpatialResolveErrorV2>;
```

The array is in exact `SpatialLimitKindV2::DIRECT_ALL` order:

```text
Nodes, Shapes, Brushes, Clips, PaintItems, HitItems, SemanticItems,
Paths, PathVerbsTotal, PolygonPointsTotal, GradientStopsTotal, Images
```

The function checks entries in that order. For nodes, shapes, brushes, clips,
paint items, hit items, semantic items, paths, and images, the inclusive
maximum is the caller limit capped at `u32::MAX + 1`. For path verbs, polygon
points, and gradient stops, it is the caller limit. The first excess returns
the existing `SpatialResolveErrorV2` with
`LimitExceeded(kind)`, `SpatialErrorLocationV2::Input`, and the exact widened
observed and effective maximum values. Equality passes.

The raw resolver converts its direct slice lengths to `u128` and delegates its
entire phase-one direct-count check to this function. It performs no duplicate
effective-maximum calculation. The runtime IR materializer calls the same
function once as its pre-allocation check with checked widened counts. The
later call to `resolve_spatial_v2` invokes the helper again through the raw
resolver's unchanged phase one. This helper validates counts only; later raw
limits and semantics remain in their existing resolver phases.

## IR materialization diagnostics

```rust
pub enum RuntimeSpatialIrErrorKindV2 {
    ArithmeticExhausted,
    InvariantViolation,
    Resolve(SpatialResolveErrorV2),
}

pub struct RuntimeSpatialIrErrorV2 { /* private fields */ }

impl RuntimeSpatialIrErrorV2 {
    #[must_use]
    pub const fn kind(self) -> RuntimeSpatialIrErrorKindV2;

    #[must_use]
    pub const fn span(self) -> SourceSpan;
}
```

`ArithmeticExhausted` means checked count, cursor, key, ordinal, range, or
byte-length arithmetic could not be represented before a raw input was
formed. `InvariantViolation` means the validated program, retained
construction, live build view, generated wrapper, or provenance table did not
agree where successful earlier validation requires agreement. It is not used
for a typed raw spatial failure. `Resolve` preserves the exact raw resolver
error, including limit evidence.

The kind implements `Clone + Copy + Debug + Eq + PartialEq` plus the runtime
auto-trait set. It has no `ALL`, public method, `Default`, `Hash`, ordering, or
display/error implementation. The stored error contains exactly a private
kind and `SourceSpan`. It has no public constructor or associated constant and
implements `Clone + Copy + Eq + PartialEq + Debug + Display + Error` plus the
runtime auto-trait set. It implements no `Default`, `Hash`, `Ord`, or
`PartialOrd`.

`Display` uses these exact redacted labels:

```text
runtime-spatial-ir-error(arithmetic-exhausted)
runtime-spatial-ir-error(invariant-violation)
runtime-spatial-ir-error(resolve)
```

`Debug` wraps the display label as
`RuntimeSpatialIrErrorV2(<display-label>)`. `Error::source()` returns `None`.
No formatting exposes symbols, logical identities, keys, property values,
counts, limits, bytes, allocation details, or resolver evidence. Callers use
`kind()` and `span()` for typed detail.

`RuntimeSpatialErrorV2` gains one variant immediately before its existing
`Resolve` variant:

```rust
pub enum RuntimeSpatialErrorV2 {
    ViewportMismatch,
    MappingLengthMismatch,
    MissingLogicalNode { key: SpatialNodeKeyV2 },
    DuplicateLogicalNode { key: SpatialNodeKeyV2 },
    Ir(RuntimeSpatialIrErrorV2),
    Resolve(SpatialResolveErrorV2),
}
```

Its new display label is exactly `runtime-spatial-error(ir)`. Its existing
manual `Debug`, `Display`, `Error`, traits, lack of methods, and other labels
remain unchanged.

Every direct-count failure and every later raw resolver failure in the IR lane
is wrapped as
`RuntimeSpatialErrorV2::Ir(RuntimeSpatialIrErrorV2 { kind:
RuntimeSpatialIrErrorKindV2::Resolve(error), ... })`. The IR lane never
reclassifies one as the outer `Resolve` variant. A generated-wrapper mismatch
is `Ir(InvariantViolation)`. The manual callback lane continues to report its
four wrapper variants and `Resolve` exactly as before; it never reports `Ir`.

## Exact source attribution

The materializer retains source provenance parallel to every expanded raw
record and independently fallible raw field. It maps one trusted
`SpatialErrorLocationV2` to one `SourceSpan` as follows:

- `Input` maps to the spatial program span.
- `Viewport` maps to `SourceSpan::Synthetic` because the viewport is supplied
  by runtime, not authored in the symbolic program.
- A whole sentinel `Node` maps to the program span. A sentinel container field
  maps to its exact viewport-container field span. A runtime-derived sentinel
  key, parent, placement, or viewport-dependent field maps to
  `SourceSpan::Synthetic`.
- A non-sentinel `Node` maps to its expanded symbolic node record. A derived
  node key maps to that record span. A `Node(...)` spatial parent maps to its
  node-symbol field; a `Viewport` parent maps to the containing node record.
  Placement, transform, size, and container fields map to their exact authored
  leaf when one exists and otherwise to their containing node record.
- Each `Path`, `Shape`, `Brush`, `Image`, `Clip`, `Paint`, `Hit`, and `Semantic`
  raw record retains one field-to-span map. A field produced by an authored
  leaf or symbolic reference maps to that leaf or reference field. A derived
  key, owner, ordinal, range cursor, discriminant, or other unspanned literal
  maps to its containing record. A `Viewport` or `Parent` anchor target has no
  authored source field and therefore maps to its containing node record.
- `PathVerb`, `PolygonPoint`, and `GradientStop` use the captured provenance of
  their containing raw path, shape, or brush plus the resolver's local nested
  index. An independently spanned nested coordinate, gradient offset, or color
  maps to its exact leaf; an unspanned kind or literal maps to the nested
  record. `Image` width, height, and stride map to their fields, while key and
  byte length map to the image record.
- `Dependency { ordinal }` treats the stable ordinal as an expanded spatial
  node key and maps to that node record.
- `Island { index }` maps to that island's expanded host node record.
- `ImagePixel` maps to its containing image record because pixels have no
  separate source records.
- `OutputRecord` maps by table and ordinal to its source raw record: geometry
  to node, clip to clip, paint to paint, hit to hit, and semantic to semantic.
  A sentinel geometry output record maps to the program span.
- A table-wide `Output` maps to the spatial program span.

The mapper records these associations while assigning dense raw ordinals; it
does not reconstruct them from authored symbol values. It validates complete
provenance cardinality before resolver invocation. If a returned trusted
location nevertheless has an impossible ordinal or missing entry, it reports
`InvariantViolation` at the program span. A live-tree walk, generated wrapper,
or provenance invariant also uses the program span. A missing property or
unexpected runtime value shape uses the responsible binding field. Mapper
arithmetic uses the most specific participating record or field span; global
count or cursor arithmetic uses the program span.

## Deterministic staging and atomicity

Initialization and each effective spatial transaction use this exact order:

1. complete logical validation and, for a transaction, remove no-op records;
2. walk the live tree by retained construction slots and keyed regions,
   validating build-view invariants while accumulating only checked `u128`
   counts and bounded provenance scratch;
3. call `preflight_spatial_direct_counts_v2` once as the materializer's
   pre-allocation direct-count check;
4. check every raw `u32` key, item ordinal, payload range, and allocation-length
   conversion with checked arithmetic;
5. allocate the raw tables, evaluate current owner-local bindings, and flatten
   records in the symbolic contract's fixed order;
6. validate the generated viewport and logical mapping wrapper as internal IR
   invariants;
7. invoke the existing raw spatial resolver once;
8. retain the accepted mapping and snapshot, advance the generation, and
   publish atomically.

A checked-arithmetic failure during the live walk is selected in
construction-guided depth-first order. After the direct counts pass, raw key,
ordinal, range, and allocation-length representation checks use
`SpatialLimitKindV2::DIRECT_ALL` table order, then expanded owner and local
declaration order, then raw record-field and nested-record order. Global images
use declaration order. These orders select the span when more than one
representation failure exists.

A transaction that is a logical no-op does not enter materialization. Every
effective nonempty transaction in an IR spatial runtime rebuilds once. The
count pass may allocate bounded logical-enumeration and provenance scratch, but
allocates no raw table, logical mapping, payload buffer, or image copy.
Standard allocator exhaustion has no typed result in this contract and is not
translated into `ArithmeticExhausted`. The later resolver repeats the shared
direct-count helper through its normal phase one before continuing with the
remaining raw phases.

An owner-local `ScalarI32` used as Fixed16 is multiplied exactly by 65,536 in
`i64`. A dynamic result outside the canonical spatial scalar domain is still
written to the raw field; the raw resolver then returns its field-specific
typed error, which the IR lane wraps as `Ir(Resolve(...))` with that binding's
span.

An initialization failure publishes nothing. A rebuild failure preserves the
prior logical and spatial generation, mapping, and snapshot. It appears as
`TransactionErrorKind::Spatial(RuntimeSpatialErrorV2::Ir(...))` with
`operation_index() == None`, because materialization validates the complete
post-operation draft rather than one mutation. Existing transaction atomicity,
retained snapshot lifetime, resize coalescing, scheduling, and generation
rules remain unchanged.

## RED/GREEN obligations

1. Freeze exact export counts, constructor signatures and attributes, error
   variants, traits, labels, getters, and negative surface assertions.
2. Prove the public direct-count helper and the raw resolver select identical
   first failures and widened evidence at exact and one-over limits.
3. Prove construction-guided nested keyed expansion, empty regions, omitted
   logical wrappers, spatial preorder, dense mappings, and source provenance.
4. Prove every materializer failure kind and every resolver location maps to
   the required span without leaking evidence through formatting.
5. Prove manual callback behavior is unchanged and IR failures use only the
   `Ir` variant.
6. Prove initialization and transaction failures are atomic, transaction
   operation indices are absent, no-op drafts do not rebuild, and successful
   commits retain old snapshots.
