# WU-0013 deterministic CPU reference raster API contract

Snapshot API:
[hybrid-spatial-snapshot-api-v2.md](hybrid-spatial-snapshot-api-v2.md)
Reference semantics:
[hybrid-spatial-content-reference-v2.md](hybrid-spatial-content-reference-v2.md)
Diagnostics:
[hybrid-spatial-diagnostics-v2.md](hybrid-spatial-diagnostics-v2.md)
Paint kernel:
[hybrid-spatial-paint-kernel-v2.md](hybrid-spatial-paint-kernel-v2.md)
Format: deterministic CPU reference raster API version 2

## Boundary and exact public surface

The CPU reference raster is a bounded deterministic observation of one
successfully resolved immutable spatial snapshot. It accepts no raw spatial
input, prepared proof, candidate output, layout engine, logical identity,
runtime state, presenter, renderer handle, device scale, or surface state. It
does not mutate or publish the snapshot.

This cut adds exactly six `prototype` exports and one inherent snapshot method.
The public surface advances from 114 to 120 names and from 48 to 51 structs.
The six names are `ReferenceRasterLimitKindV2`, `ReferenceRasterLimitsV2`,
`REGISTERED_REFERENCE_RASTER_LIMITS_V2`, `ReferenceRasterErrorKindV2`,
`ReferenceRasterErrorV2`, and `ReferenceRasterV2`. The cut adds no free
function, public constructor for the error or raster, trait, alias, or module.
Existing public inherent method sets remain unchanged except for
`SpatialResolvedSnapshotV2::rasterize_reference`.

## Limits and errors

```rust
pub enum ReferenceRasterLimitKindV2 {
    Pixels,
}

impl ReferenceRasterLimitKindV2 {
    pub const ALL: [Self; 1] = [Self::Pixels];
}

pub struct ReferenceRasterLimitsV2 {
    values: [usize; 1],
}

impl ReferenceRasterLimitsV2 {
    pub const fn new(pixels: usize) -> Self;
    pub const fn limit(self, kind: ReferenceRasterLimitKindV2) -> usize;
}

pub const REGISTERED_REFERENCE_RASTER_LIMITS_V2: ReferenceRasterLimitsV2 =
    ReferenceRasterLimitsV2::new(4_194_304);

pub enum ReferenceRasterErrorKindV2 {
    LimitExceeded(ReferenceRasterLimitKindV2),
}

impl ReferenceRasterErrorKindV2 {
    pub const ALL: [Self; 1] = [
        Self::LimitExceeded(ReferenceRasterLimitKindV2::Pixels),
    ];
}

pub struct ReferenceRasterErrorV2 {
    kind: ReferenceRasterErrorKindV2,
    location: SpatialErrorLocationV2,
    observed: Option<u128>,
    maximum: Option<u128>,
}

impl ReferenceRasterErrorV2 {
    pub const fn kind(self) -> ReferenceRasterErrorKindV2;
    pub const fn location(self) -> SpatialErrorLocationV2;
    pub const fn observed(self) -> Option<u128>;
    pub const fn maximum(self) -> Option<u128>;
}
```

Every constructor, getter, and result in this contract is `#[must_use]`.
`ReferenceRasterLimitKindV2`, `ReferenceRasterLimitsV2`, and
`ReferenceRasterErrorKindV2` implement exactly `Clone + Copy + Debug + Eq +
PartialEq` plus `Send + Sync + Unpin + UnwindSafe + RefUnwindSafe + 'static`.
They do not implement `Default`, `Display`, `Hash`, `Ord`, or `PartialOrd`.

`ReferenceRasterErrorV2` has private fields and no public constructor. It
implements `Clone + Copy + Eq + PartialEq + Debug + Display + Error` plus
`Send + Sync + Unpin + UnwindSafe + RefUnwindSafe + 'static`, and no `Default`,
`Hash`, `Ord`, or `PartialOrd`. Its
location is always `SpatialErrorLocationV2::Input`. Its observed and maximum
values are always `Some` because its only kind is a limit. `Display` and
`Debug` expose only the closed limit label and never viewport values, pixels,
paint data, image bytes, source keys, pointers, or allocation details. The
exact strings are `reference-raster-error(limit-exceeded)` and
`ReferenceRasterErrorV2(reference-raster-error(limit-exceeded))`.

Raster preflight computes:

```text
observed_pixels = u128(viewport.width) * u128(viewport.height)
allocation_pixels = usize(isize::MAX) / 4
effective_maximum = min(caller Pixels, allocation_pixels)
```

If `observed_pixels` exceeds `effective_maximum`, rasterization returns
`LimitExceeded(Pixels)` at `Input` with those exact observed and maximum
values. This check occurs before allocating output bytes or evaluating a paint
item. Equality passes. The allocation ceiling is part of the effective pixel
maximum, not a second byte limit or a second error kind. At the registered
maximum the packed byte length is 16,777,216 and is representable on every
supported 32-bit or 64-bit target.

## Owned raster result and snapshot method

```rust
pub struct ReferenceRasterV2 {
    width: u32,
    height: u32,
    stride: u64,
    bytes: Box<[u8]>,
}

impl ReferenceRasterV2 {
    pub const fn width(&self) -> u32;
    pub const fn height(&self) -> u32;
    pub const fn stride(&self) -> u64;
    pub fn bytes(&self) -> &[u8];
}

impl SpatialResolvedSnapshotV2 {
    pub fn rasterize_reference(
        &self,
        limits: ReferenceRasterLimitsV2,
    ) -> Result<ReferenceRasterV2, ReferenceRasterErrorV2>;
}
```

`ReferenceRasterV2` has private fields and no public constructor. It owns one
packed premultiplied encoded-sRGB RGBA8 byte sequence. `width` and `height` are
the accepted nonnegative logical viewport extents converted exactly to `u32`.
`stride` is exactly `u64(width) * 4`. The byte length is exactly the checked
widened product `width * height * 4` converted to `usize` after successful
preflight. Rows are top-to-bottom, pixels are left-to-right, channels are
`R, G, B, A`, and there is no padding.
The three metadata getters are `const`; `bytes` and rasterization are
non-const.

A zero width or height produces an empty byte sequence. Stride still records
`width * 4`, including when height is zero. The `u64` stride is deliberate: an
accepted viewport `(i32::MAX, 0)` has zero pixels and an empty byte sequence but
an exact stride of 8,589,934,588, which cannot be represented by `u32` or by
`usize` on every supported target.

The raster implements `Send + Sync + Unpin + UnwindSafe + RefUnwindSafe +
'static`. It deliberately
implements no `Clone`, `Copy`, `Debug`, `Default`, `Display`, equality,
ordering, or hash trait. Exact consumers compare its metadata and bytes. The
result owns its bytes and borrows nothing from the snapshot.

## Sampling and painter order

Rasterization initializes every sample to transparent premultiplied RGBA8. For
each output pixel `(x, y)`, it evaluates the Cartesian product of scene-logical
sample offsets `{1, 3, 5, 7} / 8`. Each of the 16 samples scans the complete
accepted Paint output table in ascending record order and applies source-over
after every covered paint. It never sorts,
reverses, batches by paint or resource kind, or derives painter order from
nodes.

The accepted snapshot remains authoritative for projection data:

- each Paint row supplies its world affine and conservative unclipped AABB;
- each Clip row supplies its world affine and primitive conservative AABB;
- the effective-clip slice supplies its accepted candidate-derived chain AABB;
- trusted retained state supplies paint kind, coverage, shape, fill rule,
  positive stroke token, brush or image key, opacity, terminal clip, clip
  ancestry, normalized resources, polygon ranges, and flattened paths.

Phase-10 reference transforms and AABBs retained privately by prepared state
never replace accepted candidate projection rows. AABBs are reject-only
accelerators and never establish exact coverage.

For a terminal clip, the sample must pass the effective AABB and every exact K4
shape in the complete trusted parent chain. Each link maps the original scene
sample through that accepted Clip row's own affine. An empty effective bound,
an out-of-domain inverse, or any exact miss rejects that paint for the sample.

A paint then reject-tests its accepted world AABB and maps the original scene
sample through its accepted Paint affine. `inverse_point(None)` is uncovered
for that paint rather than a raster error. Coverage fill uses exact K4;
round-stroke coverage uses exact K5 with the retained validated width. A solid
or linear-gradient brush is sampled in paint-owner local space from its
retained normalized premultiplied P2 representation, then opacity is applied
exactly once. An image paint applies its retained half-open destination and
nearest source mapping to the exact retained premultiplied P4 image bytes, then
applies opacity exactly once. No raw K1, P2, P4, or P5 validation is repeated.

Source-over uses the registered integer premultiplied formula after every
accepted paint. After all paints, each output channel is the average of the 16
final sample channels using `(sum + 8) / 16`. There is no adaptive sampling,
filtering, dithering, gamma conversion, SIMD-dependent path, font input, device
scale, or implicit surface background.

## Ownership, determinism, and deferrals

Rasterization allocates no per-pixel, per-sample, clip-chain, or paint-item
heap state and installs no cache or index. The owned output byte allocation is
created only after successful preflight. Repeated calls with the same snapshot
and limits produce identical metadata and bytes and preserve every snapshot
output table, effective-clip allocation, retained source object, image byte
allocation, and normalized private resource.

The reference raster is an evidence and conformance output, not the immutable
Fenestra paint frame and not a presenter contract. Candidate CPU comparison,
GPU output, semantic joins, runtime generation publication, invalidation,
rollback, logical identity, native presentation, artifacts, authoring, density,
physical pixels, safe areas, lifecycle, surface ownership, and multi-scene
behavior remain separate later boundaries.
