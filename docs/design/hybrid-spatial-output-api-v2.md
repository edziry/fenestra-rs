# WU-0013 resolved spatial output and snapshot API contract

Parent API: [hybrid-spatial-api-v2.md](hybrid-spatial-api-v2.md)
Raw input: [hybrid-spatial-validation-api-v2.md](hybrid-spatial-validation-api-v2.md)
Reference semantics: [hybrid-spatial-reference-v2.md](hybrid-spatial-reference-v2.md)
Content semantics:
[hybrid-spatial-content-reference-v2.md](hybrid-spatial-content-reference-v2.md)
Diagnostics: [hybrid-spatial-diagnostics-v2.md](hybrid-spatial-diagnostics-v2.md)
Fields: [hybrid-spatial-fields-v2.md](hybrid-spatial-fields-v2.md)
Format: resolved spatial output API version 2

## Boundary and staging

This contract freezes the candidate-neutral records that cross the final
spatial-output validation boundary and the immutable snapshot produced after
that boundary. It does not add a renderer, candidate type, runtime generation,
logical identity, exact hit query, raster result, artifact, or public stable
facade.

Spatial node keys already equal preorder ordinals. Paint, hit, and semantic
input validation already proves ascending owners and dense owner-local item
ordinals. Output materialization therefore preserves those orders. It never
sorts, compacts, filters, reverses, or introduces a separate projection-order
proof.

The implementation sequence is:

1. raw output values and the borrowed five-table output view;
2. one immutable owner for raw spatial input;
3. an opaque prepared spatial proof and lifetime-free resolved snapshot;
4. infallible reference materialization;
5. candidate-output validation;
6. exact clip coverage, reverse hit selection, rasterization, and runtime
   publication in their own later RED/GREEN cuts.

The completed slice adds exactly 15 `prototype` exports, taking the exact
surface from 98 to 113 names: the eight raw output values below,
`SpatialOwnedInputV2`, `PreparedSpatialV2`, `SpatialResolvedSnapshotV2`, and
the four resolver functions. Granular RED/GREEN commits may add those names in
the staged order above but may not add aliases, traits, or convenience seams.

## Raw output AABB

Candidate output must be able to represent every malformed AABB that phase 11
rejects. It therefore cannot use `SpatialAabbV2`, whose constructors admit only
canonical values.

```rust
pub struct SpatialOutputAabbV2 {
    empty: bool,
    min_x: SpatialScalarV2,
    min_y: SpatialScalarV2,
    max_x: SpatialScalarV2,
    max_y: SpatialScalarV2,
}

impl SpatialOutputAabbV2 {
    pub const fn new(
        empty: bool,
        min_x: SpatialScalarV2,
        min_y: SpatialScalarV2,
        max_x: SpatialScalarV2,
        max_y: SpatialScalarV2,
    ) -> Self;
    pub const fn is_empty(self) -> bool;
    pub const fn min_x(self) -> SpatialScalarV2;
    pub const fn min_y(self) -> SpatialScalarV2;
    pub const fn max_x(self) -> SpatialScalarV2;
    pub const fn max_y(self) -> SpatialScalarV2;
}
```

Every AABB method is `#[must_use]`.

An accepted empty value has four zero edges. An accepted nonempty value has
canonical edges with both minima less than or equal to their maxima. Equality
is a nonempty closed line or point.

## Output records

All five record structs have private fields and one `const new` constructor in
their applicable `SpatialOutputFieldV2::ALL` order. Every constructor and
getter is `#[must_use]`; every getter is by-value and `const`.

```rust
pub enum SpatialPaintOutputReferenceV2 {
    Coverage {
        shape: SpatialShapeKeyV2,
        brush: SpatialBrushKeyV2,
    },
    Image {
        image: SpatialImageKeyV2,
    },
}

pub struct SpatialGeometryOutputRecordV2 {
    key: SpatialNodeKeyV2,
    base_x: SpatialScalarV2,
    base_y: SpatialScalarV2,
    base_width: SpatialScalarV2,
    base_height: SpatialScalarV2,
    world_from_local: Affine2V2,
    world_determinant: i128,
    world_aabb: SpatialOutputAabbV2,
}

pub struct SpatialClipOutputRecordV2 {
    key: SpatialClipKeyV2,
    world_from_local: Affine2V2,
    world_determinant: i128,
    primitive_world_aabb: SpatialOutputAabbV2,
    owner: SpatialNodeKeyV2,
    parent: Option<SpatialClipKeyV2>,
    shape: SpatialShapeKeyV2,
}

pub struct SpatialPaintOutputRecordV2 {
    key: u32,
    world_from_local: Affine2V2,
    world_determinant: i128,
    world_aabb: SpatialOutputAabbV2,
    owner: SpatialNodeKeyV2,
    reference: SpatialPaintOutputReferenceV2,
    clip: Option<SpatialClipKeyV2>,
    stack_ordinal: u32,
    item_ordinal: u32,
}

pub struct SpatialHitOutputRecordV2 {
    key: u32,
    world_from_local: Affine2V2,
    world_determinant: i128,
    world_aabb: SpatialOutputAabbV2,
    owner: SpatialNodeKeyV2,
    shape: SpatialShapeKeyV2,
    clip: Option<SpatialClipKeyV2>,
    stack_ordinal: u32,
    item_ordinal: u32,
}

pub struct SpatialSemanticOutputRecordV2 {
    key: u32,
    world_from_local: Affine2V2,
    world_determinant: i128,
    world_aabb: SpatialOutputAabbV2,
    owner: SpatialNodeKeyV2,
    shape: SpatialShapeKeyV2,
    clip: Option<SpatialClipKeyV2>,
    stack_ordinal: u32,
    item_ordinal: u32,
}
```

Their exact methods are:

```rust
impl SpatialGeometryOutputRecordV2 {
    pub const fn new(
        key: SpatialNodeKeyV2,
        base_x: SpatialScalarV2,
        base_y: SpatialScalarV2,
        base_width: SpatialScalarV2,
        base_height: SpatialScalarV2,
        world_from_local: Affine2V2,
        world_determinant: i128,
        world_aabb: SpatialOutputAabbV2,
    ) -> Self;
    pub const fn key(self) -> SpatialNodeKeyV2;
    pub const fn base_x(self) -> SpatialScalarV2;
    pub const fn base_y(self) -> SpatialScalarV2;
    pub const fn base_width(self) -> SpatialScalarV2;
    pub const fn base_height(self) -> SpatialScalarV2;
    pub const fn world_from_local(self) -> Affine2V2;
    pub const fn world_determinant(self) -> i128;
    pub const fn world_aabb(self) -> SpatialOutputAabbV2;
}

impl SpatialClipOutputRecordV2 {
    pub const fn new(
        key: SpatialClipKeyV2,
        world_from_local: Affine2V2,
        world_determinant: i128,
        primitive_world_aabb: SpatialOutputAabbV2,
        owner: SpatialNodeKeyV2,
        parent: Option<SpatialClipKeyV2>,
        shape: SpatialShapeKeyV2,
    ) -> Self;
    pub const fn key(self) -> SpatialClipKeyV2;
    pub const fn world_from_local(self) -> Affine2V2;
    pub const fn world_determinant(self) -> i128;
    pub const fn primitive_world_aabb(self) -> SpatialOutputAabbV2;
    pub const fn owner(self) -> SpatialNodeKeyV2;
    pub const fn parent(self) -> Option<SpatialClipKeyV2>;
    pub const fn shape(self) -> SpatialShapeKeyV2;
}

impl SpatialPaintOutputRecordV2 {
    pub const fn new(
        key: u32,
        world_from_local: Affine2V2,
        world_determinant: i128,
        world_aabb: SpatialOutputAabbV2,
        owner: SpatialNodeKeyV2,
        reference: SpatialPaintOutputReferenceV2,
        clip: Option<SpatialClipKeyV2>,
        stack_ordinal: u32,
        item_ordinal: u32,
    ) -> Self;
    pub const fn key(self) -> u32;
    pub const fn world_from_local(self) -> Affine2V2;
    pub const fn world_determinant(self) -> i128;
    pub const fn world_aabb(self) -> SpatialOutputAabbV2;
    pub const fn owner(self) -> SpatialNodeKeyV2;
    pub const fn reference(self) -> SpatialPaintOutputReferenceV2;
    pub const fn clip(self) -> Option<SpatialClipKeyV2>;
    pub const fn stack_ordinal(self) -> u32;
    pub const fn item_ordinal(self) -> u32;
}

impl SpatialHitOutputRecordV2 {
    pub const fn new(
        key: u32,
        world_from_local: Affine2V2,
        world_determinant: i128,
        world_aabb: SpatialOutputAabbV2,
        owner: SpatialNodeKeyV2,
        shape: SpatialShapeKeyV2,
        clip: Option<SpatialClipKeyV2>,
        stack_ordinal: u32,
        item_ordinal: u32,
    ) -> Self;
    pub const fn key(self) -> u32;
    pub const fn world_from_local(self) -> Affine2V2;
    pub const fn world_determinant(self) -> i128;
    pub const fn world_aabb(self) -> SpatialOutputAabbV2;
    pub const fn owner(self) -> SpatialNodeKeyV2;
    pub const fn shape(self) -> SpatialShapeKeyV2;
    pub const fn clip(self) -> Option<SpatialClipKeyV2>;
    pub const fn stack_ordinal(self) -> u32;
    pub const fn item_ordinal(self) -> u32;
}

impl SpatialSemanticOutputRecordV2 {
    pub const fn new(
        key: u32,
        world_from_local: Affine2V2,
        world_determinant: i128,
        world_aabb: SpatialOutputAabbV2,
        owner: SpatialNodeKeyV2,
        shape: SpatialShapeKeyV2,
        clip: Option<SpatialClipKeyV2>,
        stack_ordinal: u32,
        item_ordinal: u32,
    ) -> Self;
    pub const fn key(self) -> u32;
    pub const fn world_from_local(self) -> Affine2V2;
    pub const fn world_determinant(self) -> i128;
    pub const fn world_aabb(self) -> SpatialOutputAabbV2;
    pub const fn owner(self) -> SpatialNodeKeyV2;
    pub const fn shape(self) -> SpatialShapeKeyV2;
    pub const fn clip(self) -> Option<SpatialClipKeyV2>;
    pub const fn stack_ordinal(self) -> u32;
    pub const fn item_ordinal(self) -> u32;
}
```

The composite affine and AABB arguments represent their consecutive closed
field groups. The Clip getter is named `primitive_world_aabb` to keep its
meaning explicit.

The applicable output fields are exactly:

```text
Geometry = Key, BaseX, BaseY, BaseWidth, BaseHeight,
           AffineA, AffineB, AffineC, AffineD, AffineTx, AffineTy,
           Determinant, AabbEmpty, AabbMinX, AabbMinY, AabbMaxX, AabbMaxY
Clip = Key, AffineA, AffineB, AffineC, AffineD, AffineTx, AffineTy,
       Determinant, AabbEmpty, AabbMinX, AabbMinY, AabbMaxX, AabbMaxY,
       Owner, Parent, Shape
Paint = Key, AffineA, AffineB, AffineC, AffineD, AffineTx, AffineTy,
        Determinant, AabbEmpty, AabbMinX, AabbMinY, AabbMaxX, AabbMaxY,
        Owner, Shape and Brush for Coverage or Image for Image, Clip,
        StackOrdinal, ItemOrdinal
Hit = Key, AffineA, AffineB, AffineC, AffineD, AffineTx, AffineTy,
      Determinant, AabbEmpty, AabbMinX, AabbMinY, AabbMaxX, AabbMaxY,
      Owner, Shape, Clip, StackOrdinal, ItemOrdinal
Semantic = Key, AffineA, AffineB, AffineC, AffineD, AffineTx, AffineTy,
           Determinant, AabbEmpty, AabbMinX, AabbMinY, AabbMaxX, AabbMaxY,
           Owner, Shape, Clip, StackOrdinal, ItemOrdinal
```

Geometry keys are dense node keys and Clip keys are dense input clip keys.
Paint, Hit, and Semantic keys are dense global ordinals in their own source
tables. For those three tables, `stack_ordinal` equals the owner node key,
because version-2 node keys are preorder ordinals. `item_ordinal` is dense only
within that owner and restarts independently in each table.

Every row carries its owning world affine and the exact widened raw determinant
as a separately supplied field. This makes a stale nonzero determinant
observable. Clip carries its primitive owner-transformed K3 bound. Paint, Hit,
and Semantic carry their unclipped world bounds. The snapshot separately owns
the effective parent-intersected clip bounds; the primitive is never replaced.

Paint references expose only the fields applicable to the validated source
variant. Fill rule, stroke width, opacity, source and destination rectangles,
input policy, and exact normalized resources remain in snapshot-owned source
data and are not candidate-mutable duplicates.

The raw AABB, paint reference, and all five rows implement
`Clone + Copy + Debug + Eq + PartialEq` and the runtime auto-trait set. They do
not implement `Default`, ordering, hashing, or formatting beyond derived
`Debug`.

## Borrowed output view

```rust
pub struct SpatialOutputV2<'a> {
    geometry: &'a [SpatialGeometryOutputRecordV2],
    clips: &'a [SpatialClipOutputRecordV2],
    paints: &'a [SpatialPaintOutputRecordV2],
    hits: &'a [SpatialHitOutputRecordV2],
    semantics: &'a [SpatialSemanticOutputRecordV2],
}

impl<'a> SpatialOutputV2<'a> {
    pub const fn new(
        geometry: &'a [SpatialGeometryOutputRecordV2],
        clips: &'a [SpatialClipOutputRecordV2],
        paints: &'a [SpatialPaintOutputRecordV2],
        hits: &'a [SpatialHitOutputRecordV2],
        semantics: &'a [SpatialSemanticOutputRecordV2],
    ) -> Self;
    pub const fn geometry(self) -> &'a [SpatialGeometryOutputRecordV2];
    pub const fn clips(self) -> &'a [SpatialClipOutputRecordV2];
    pub const fn paints(self) -> &'a [SpatialPaintOutputRecordV2];
    pub const fn hits(self) -> &'a [SpatialHitOutputRecordV2];
    pub const fn semantics(self) -> &'a [SpatialSemanticOutputRecordV2];
}
```

Every view method is `#[must_use]`. The getters return the exact supplied
slices. The view implements only `Clone + Copy` and the runtime auto-trait set.
It owns, clones, formats, compares, hashes, sorts, or defaults no record.

## Companion contracts and nonclaims

Immutable source ownership, prepared type-state, the resolved snapshot, and
the resolver signatures are fixed in
[hybrid-spatial-snapshot-api-v2.md](hybrid-spatial-snapshot-api-v2.md).
The candidate-output global passes and exact diagnostic attribution are fixed
in [hybrid-spatial-output-validation-v2.md](hybrid-spatial-output-validation-v2.md).

Exact clip containment, hit result type, reverse hit selection, CPU
rasterization, semantic runtime joins, candidate adapters, runtime generations,
invalidation, rollback, presentation, artifacts, and authoring remain later
boundaries.
