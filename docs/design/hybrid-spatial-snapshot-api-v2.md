# WU-0013 immutable spatial snapshot API contract

Output records: [hybrid-spatial-output-api-v2.md](hybrid-spatial-output-api-v2.md)
Raw input: [hybrid-spatial-validation-api-v2.md](hybrid-spatial-validation-api-v2.md)
Reference semantics: [hybrid-spatial-reference-v2.md](hybrid-spatial-reference-v2.md)
Reference raster: [hybrid-spatial-raster-api-v2.md](hybrid-spatial-raster-api-v2.md)
Format: immutable spatial snapshot API version 2

## Immutable input ownership

The borrowed `SpatialInputV2` remains the validation view. Runtime publication
needs one owner that outlives every proof without putting an owner beside a
proof that borrows it. The exact owner is:

```rust
pub struct SpatialOwnedInputV2 { /* private fields */ }

impl SpatialOwnedInputV2 {
    pub fn new(
        viewport: SpatialViewportV2,
        nodes: Box<[SpatialNodeV2]>,
        polygon_points: Box<[SpatialPointV2]>,
        path_verbs: Box<[SpatialPathVerbV2]>,
        paths: Box<[SpatialPathV2]>,
        shapes: Box<[SpatialShapeV2]>,
        clips: Box<[SpatialClipV2]>,
        gradient_stops: Box<[SpatialGradientStopV2]>,
        brushes: Box<[SpatialBrushV2]>,
        images: Box<[SpatialImageV2]>,
        paint_items: Box<[SpatialPaintV2]>,
        hit_items: Box<[SpatialHitV2]>,
        semantic_items: Box<[SpatialSemanticGeometryV2]>,
    ) -> Self;
    pub fn as_input(&self) -> SpatialInputV2<'_>;
}
```

Both methods are `#[must_use]`. The owner implements the runtime auto-trait set
and no `Clone`, `Copy`, `Debug`, `Default`, comparison, ordering, hash, or
display trait. Sharing occurs only through `Arc<SpatialOwnedInputV2>`. In
particular, resolution never deep-clones `SpatialImageV2` byte storage.

## Prepared proof and resolved snapshot

```rust
pub struct PreparedSpatialV2 { /* private owned type-state */ }
pub struct SpatialResolvedSnapshotV2 { /* private owned state */ }

impl SpatialResolvedSnapshotV2 {
    pub const fn viewport(&self) -> SpatialViewportV2;
    pub fn output(&self) -> SpatialOutputV2<'_>;
    pub fn effective_clip_aabbs(&self) -> &[SpatialAabbV2];
    pub fn hit_test(&self, scene_point: SpatialPointV2)
        -> Option<SpatialHitResultV2>;
    pub fn rasterize_reference(
        &self,
        limits: ReferenceRasterLimitsV2,
    ) -> Result<ReferenceRasterV2, ReferenceRasterErrorV2>;
}
```

The three snapshot getters, hit query, and reference raster result are
`#[must_use]`. The hit result, selection, and exact-coverage contract is fixed
separately in [hybrid-spatial-hit-api-v2.md](hybrid-spatial-hit-api-v2.md).
The bounded raster result and sampling contract are fixed separately in
[hybrid-spatial-raster-api-v2.md](hybrid-spatial-raster-api-v2.md).
`PreparedSpatialV2` owns the
immutable source `Arc` and private unforgeable validated keys, ranges, tokens,
and derived phase-10 results. It contains no borrow into that source and no
candidate handle. Preparation borrows the source only inside its call, consumes
the final private proof into that owned representation, ends every borrow, and
then moves the `Arc` into the prepared value. Later stages never reconstruct a
validated image, path, shape, brush, clip, or item from raw input alone.

`SpatialResolvedSnapshotV2` consumes the prepared value, transfers that exact
source `Arc`, and owns the accepted five output tables, effective clip bounds,
flattened paths, normalized resources, and validated item data needed by later
exact hit and raster consumers. Image-backed values retain an image key and
validated metadata; the unchanged bytes remain in the same retained Arc-owned
source object graph. The source Arc remains owned by the snapshot, which has no
lifetime parameter or self-reference.

Reference materialization transfers the prepared effective clip vector.
Candidate validation instead stores the vector derived from the supplied
primitive Clip rows. In both cases its length and order exactly match the Clip
output table.

Both opaque values implement the runtime auto-trait set and deliberately
implement no `Clone`, `Copy`, `Debug`, `Default`, comparison, ordering, hash, or
display trait. Runtime shares the completed snapshot with `Arc`; no partial
value is observable.

## Resolver seams

```rust
pub fn prepare_spatial_v2<E: LayoutEngineV1 + ?Sized>(
    engine: &E,
    input: Arc<SpatialOwnedInputV2>,
    limits: SpatialLimitsV2,
) -> Result<PreparedSpatialV2, SpatialResolveErrorV2>;

pub fn materialize_reference_spatial_v2(
    prepared: PreparedSpatialV2,
) -> SpatialResolvedSnapshotV2;

pub fn validate_spatial_output_v2(
    prepared: PreparedSpatialV2,
    supplied: SpatialOutputV2<'_>,
) -> Result<SpatialResolvedSnapshotV2, SpatialResolveErrorV2>;

pub fn resolve_spatial_v2<E: LayoutEngineV1 + ?Sized>(
    engine: &E,
    input: Arc<SpatialOwnedInputV2>,
    limits: SpatialLimitsV2,
) -> Result<SpatialResolvedSnapshotV2, SpatialResolveErrorV2>;
```

All four results are `#[must_use]`. Preparation completes phases 1 through 10,
including layout execution. Reference materialization is direct and
infallible. The one-shot resolver is preparation followed by reference
materialization. Candidate validation consumes the same prepared state and
returns the structurally validated snapshot directly; there is no separately
reusable validated-candidate token.

A reference snapshot is immediately eligible for runtime publication. A
candidate snapshot is eligible only for a previously admitted candidate lane;
admission includes the independent exact field-by-field comparison required by
the evidence contract. Structural output validation alone never admits a lane.

## Runtime ownership and nonclaims

Runtime retains `Arc<SpatialResolvedSnapshotV2>` inside its immutable generation
and exposes only borrowed snapshot access. A no-op may share the same Arc; a
spatial rebuild allocates a complete replacement before the outer runtime
generation is published. Failure drops that replacement and preserves the
prior source, snapshot, generation, and presented frame. This requires no
`ArcSwap`, new dependency, unsafe code, pinning, or self-referential storage.

The snapshot does not expose its raw owner or private normalized-resource
store. The exact hit and raster APIs consume only the snapshot; they
do not accept raw input, prepared proofs, candidate output, logical identities,
or renderer handles.
