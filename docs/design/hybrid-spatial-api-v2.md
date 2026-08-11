# WU-0013 spatial API contract

Parent plan: [hybrid-spatial-composition.md](hybrid-spatial-composition.md)
Core semantics: [hybrid-spatial-reference-v2.md](hybrid-spatial-reference-v2.md)
Fields: [hybrid-spatial-fields-v2.md](hybrid-spatial-fields-v2.md)
Format: spatial API version 2

## Boundary and staging

The unpublished `fenestra-ui-spatial` package exposes only a document-hidden
`prototype` module. Its normal dependency is exactly `fenestra-ui-layout`.
It has no IR, runtime, testkit, authoring, renderer, or candidate dependency.
All struct fields are private and read through typed getters; public enum
payloads remain available to exhaustive pattern matching.

The first `prototype` export set is:

```text
Affine2V2
SpatialAnchorComponentV2, SpatialAnchorTargetKindV2,
SpatialAnchorTargetV2, SpatialAnchorV2
SpatialAxisV2, SpatialExtentV2
SpatialContainerV2, SpatialFreePlacementV2, SpatialLayoutPlacementV2,
SpatialLocalTransformV2, SpatialNodeKeyV2, SpatialNodeV2,
SpatialOffsetV2, SpatialPlacementKindV2, SpatialPlacementV2,
SpatialPointV2, SpatialScalarV2, SpatialTopologyInputV2, SpatialViewportV2
SpatialContainerErrorKindV2, SpatialDependencyErrorKindV2,
SpatialInputErrorKindV2, SpatialLayoutDimensionErrorKindV2,
SpatialErrorLocationV2
SpatialLimitKindV2, SpatialLimitsV2, REGISTERED_SPATIAL_LIMITS_V2
SpatialNodeFieldV2
```

Every first-slice export preserves
`Send + Sync + Unpin + UnwindSafe + RefUnwindSafe`.

The first package slice defines topology and raw numeric storage. Scalar range,
rounding, affine composition, validation, island planning, and resolution enter
later RED/GREEN slices. This is not a temporary representation: the same value
types remain the final version-2 input surface.

Finite fieldless vocabularies expose `ALL` in the printed order. Closed error
vocabularies enumerate every finite nested combination. Payload enums containing
keys and indexed error locations do not expose an impossible `ALL`.

## Topology values

`SpatialNodeKeyV2` wraps `u32` and provides `new` and `get`. It is
`Clone + Copy + Debug + Eq + Ord + Hash`.

`SpatialViewportV2::new(width: i32, height: i32)` exposes `width` and `height`.
`SpatialScalarV2::new(raw: i64)` exposes `raw`; construction deliberately does
not validate the canonical subset. `SpatialPointV2::new(x, y)` and
`SpatialOffsetV2::new(x, y)` expose their two `SpatialScalarV2` components.

`Affine2V2::new(a, b, c, d, tx, ty)` stores six scalars in that getter order.
`SpatialLocalTransformV2::new(affine, origin)` exposes an `Affine2V2` and
`SpatialPointV2`. Arithmetic and identity constructors are not part of the
value-only first GREEN.

`SpatialAnchorComponentV2::ALL` is `Start, Center, End`.
`SpatialAnchorV2::new(horizontal, vertical)` exposes both components.
`SpatialAnchorTargetKindV2::ALL` is `Viewport, Parent, Node`.
`SpatialAnchorTargetV2` is the payload enum:

```text
Viewport
Parent
Node(SpatialNodeKeyV2)
```

`SpatialContainerV2::new(axis, padding, gap)` directly stores
`LayoutAxisV1`, `LayoutPaddingV1`, and `i32` gap from the neutral layout crate.
It exposes those three fields without reexporting their types.

`SpatialLayoutPlacementV2::new(width, height, transform)` stores two
`LayoutDimensionV1` values and one `SpatialLocalTransformV2`.

`SpatialFreePlacementV2::new` has the exact parameter order:

```text
width: i32
height: i32
self_anchor: SpatialAnchorV2
target: SpatialAnchorTargetV2
target_anchor: SpatialAnchorV2
offset: SpatialOffsetV2
transform: SpatialLocalTransformV2
```

It exposes one getter per field. `SpatialPlacementKindV2::ALL` is
`Root, Layout, Free`. `SpatialPlacementV2` is the payload enum:

```text
Root
Layout(SpatialLayoutPlacementV2)
Free(SpatialFreePlacementV2)
```

Consumers inspect public payload enums by exhaustive pattern matching. `Root`
contains no authored transform, so sentinel identity is structural.

`SpatialNodeV2::new(key, parent, placement, container)` exposes those fields.
`SpatialTopologyInputV2::new(viewport, nodes)` borrows a node slice and exposes
the viewport and slice. Topology values except the borrowed input implement
`Clone + Copy + Debug + Eq + PartialEq`; the input implements `Clone + Copy`.
All preserve `Send + Sync + Unpin + UnwindSafe + RefUnwindSafe`.

## Limits

`SpatialLimitKindV2` exposes the five phase arrays and `ALL` fixed by the
diagnostics contract. Their lengths are:

```text
DIRECT_ALL=12
TOPOLOGY_ALL=2
ISLAND_ALL=3
CONTENT_ALL=11
DEPENDENCY_ALL=2
ALL=30
```

`SpatialLimitsV2::new(values: [usize; 30])` stores one value per `ALL` entry;
`limit(kind)` reads it. `REGISTERED_SPATIAL_LIMITS_V2` contains the 30 exact
values printed by the core contract. The registered profile is evidence, not a
runtime default or product capacity.
Limit kinds and values implement `Clone + Copy + Debug + Eq + PartialEq`.

## First diagnostic surface

`SpatialInputErrorKindV2::ALL` is:

```text
EmptyInput
InvalidRootKey
RootHasParent
InvalidRootPlacement
NonDenseNodeKey
MissingSpatialParent
ForwardSpatialParent
InvalidPreorder
RootPlacementOnNonRoot
NegativeViewport(Width), NegativeViewport(Height)
NegativeFreeExtent(Width), NegativeFreeExtent(Height)
FreeOffsetOutOfDomain(X), FreeOffsetOutOfDomain(Y)
InvalidContainer(SpatialContainerErrorKindV2)
InvalidLayoutDimensions(SpatialLayoutDimensionErrorKindV2)
```

`SpatialContainerErrorKindV2::ALL` has negative padding left, right, top,
bottom; padding exceeds width, height; and negative gap. The dimension sub-enum
has negative width minimum, preferred, maximum; inverted width; then the same
height sequence. They map from the corresponding layout input failures but
cannot represent layout topology, viewport, or limit kinds.

Their exact payload forms are:

```rust
enum SpatialContainerErrorKindV2 {
    NegativePadding(LayoutPaddingSideV1),
    PaddingExceedsExtent(LayoutExtentV1),
    NegativeGap,
}

enum SpatialLayoutDimensionErrorKindV2 {
    NegativeConstraint {
        extent: LayoutExtentV1,
        field: LayoutConstraintFieldV1,
    },
    InvertedConstraint(LayoutExtentV1),
}
```

`SpatialDependencyErrorKindV2::ALL` is
`MissingTarget, SentinelNodeTarget, SelfTarget, Cycle`.
`SpatialNodeFieldV2::ALL` comes from the field contract.
The exact lengths are:

```text
SpatialContainerErrorKindV2::ALL=7
SpatialLayoutDimensionErrorKindV2::ALL=8
SpatialInputErrorKindV2::ALL=30
SpatialDependencyErrorKindV2::ALL=4
SpatialNodeFieldV2::ALL=33
```

`SpatialErrorLocationV2` initially exposes `Input`, `Viewport { extent }`,
`Node { index }`, `NodeField { index, field }`, `Island { index }`, and
`Dependency { ordinal }` with the typed payloads fixed there;
content and output locations are added only by their own RED/GREEN slices.
Missing, sentinel, and self targets use `NodeField { field: TargetKey }`; a
cycle uses `Dependency { ordinal }` for the stable ordinal fixed by the core
contract.

These diagnostic vocabularies and locations implement
`Clone + Copy + Debug + Eq + PartialEq`; the indexed location has no `ALL`.

The first slice exposes only closed diagnostic vocabulary and trusted
locations. The validation RED adds the stored error value, limit numbers,
`Error` implementation, redacted `Display` and `Debug`, and the runtime
auto-trait set without replacing these enums.

## Owned layout preparation seam

Layout 0.2 adds:

```rust
pub fn prepare_layout_v1(
    input: LayoutInputV1<'_>,
    limits: LayoutLimitsV1,
) -> Result<PreparedLayoutInputV1, LayoutErrorV1>;

pub fn compute_prepared_layout_v1<E: LayoutEngineV1 + ?Sized>(
    engine: &E,
    prepared: PreparedLayoutInputV1,
) -> Result<LayoutOutputV1, LayoutErrorV1>;
```

`PreparedLayoutInputV1`, `prepare_layout_v1`, and
`compute_prepared_layout_v1` are reexported through
`fenestra_ui_layout::prototype`; external tests never import a private module.

The opaque proof owns `LayoutViewportV1` and `Vec<LayoutNodeV1>`, has no public
constructor or getters, and does not implement `Clone`. Preparation validates
without invoking an engine. Prepared computation consumes the proof, invokes
the engine once, and validates output without repeating input validation.
`compute_layout_v1` delegates to the two functions.

The prepared proof preserves the runtime auto-trait set but deliberately has no
`Debug` implementation.

Spatial retains a trusted remap beside each proof. Layout input or output record
zero maps to the island host; record `n > 0` maps to real member `n - 1`.
For preflight, negative constraints and padding sides map to the corresponding
`NodeField`; inverted ranges and padding-fit failures use derived
`Node { index }`; negative gap maps to `NodeField { field: Gap }`. A singleton
maps directly to its node.

During execution, `InputNode` and `OutputRecord` use that same host/member map.
Layout `Input`, `Viewport`, and `Output` locations map to the trusted
`Island { index }`. An out-of-range engine-supplied record location also maps to
the island rather than retaining the supplied ordinal. `RecordCountMismatch`
is island-wide. Any preflight kind or location impossible for the constructed
layout input becomes `SpatialLayoutErrorKindV2::BridgeInvariant`; it is never
reclassified as authored input. Islands are indexed in stable-ordinal order.

After neutral output validation, the synthetic root is compared in
`X, Y, Width, Height` order with `(0,0,host width,host height)`. A mismatch is
`SyntheticRootMismatch(LayoutOutputFieldV1)` at `Island { index }`; no member
translation runs. `BridgeInvariant` remains reserved for an impossible bridge
kind or location and never represents candidate geometry.

## Later surface

The numeric RED adds checked scalar and affine operations without changing raw
storage. The content API RED adds the entity and payload records named by the
content and field contracts, then `SpatialInputV2` combines topology and content
views. Validation adds `SpatialResolveErrorV2`, which stores only closed kind,
trusted location, and optional limit numbers; it never retains or renders
authored keys, scalar values, or nested layout payloads. Resolution types appear
only when their exclusive REDs exist. Candidate types, logical identities, and
renderer handles never enter this module.
