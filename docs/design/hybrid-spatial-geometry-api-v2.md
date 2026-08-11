# WU-0013 raw geometry API contract

Parent API: [hybrid-spatial-api-v2.md](hybrid-spatial-api-v2.md)
Semantic contract:
[hybrid-spatial-content-reference-v2.md](hybrid-spatial-content-reference-v2.md)
Fields: [hybrid-spatial-fields-v2.md](hybrid-spatial-fields-v2.md)
Format: raw spatial geometry API version 2

## Purpose and staging

This contract fixes the Fenestra-owned raw value boundary for local paths,
shapes, fill and round-stroke coverage, and clips. It does not select a
geometry, tessellation, indexing, raster, or renderer candidate.

The first geometry API RED adds exactly the values in this document. It does
not add validation, flattening results, local-bound or containment functions,
resolved clip chains, paint, brushes, images, hit items, semantic items,
candidate output, or raster types. The reference geometry kernel remains
crate-private under the versioned
[geometry-kernel contract](hybrid-spatial-geometry-kernel-v2.md). Clip-chain
evaluation remains part of the later resolver because it requires topology
ancestry and world transforms. No temporary public geometry error or
validated-view type is introduced.

The prototype surface grows from 36 to exactly 50 reexports:

```text
SpatialPathKeyV2, SpatialShapeKeyV2, SpatialClipKeyV2,
SpatialPathVerbKindV2, SpatialPathVerbV2, SpatialPathV2,
SpatialShapeKindV2, SpatialShapeGeometryV2, SpatialShapeV2,
SpatialFillRuleV2, SpatialCoverageKindV2, SpatialCoverageV2,
SpatialClipV2, SpatialGeometryInputV2
```

## Typed keys and ranges

`SpatialPathKeyV2`, `SpatialShapeKeyV2`, and `SpatialClipKeyV2` each wrap one
raw `u32`. Each exposes only `new(value: u32) -> Self` and `get(self) -> u32`.
They do not validate density at construction time.

```rust
impl SpatialPathKeyV2 {
    pub const fn new(value: u32) -> Self;
    pub const fn get(self) -> u32;
}
impl SpatialShapeKeyV2 {
    pub const fn new(value: u32) -> Self;
    pub const fn get(self) -> u32;
}
impl SpatialClipKeyV2 {
    pub const fn new(value: u32) -> Self;
    pub const fn get(self) -> u32;
}
```

Every range start and length in this format is `u32`. Validation widens
`start + length` before converting to a host index. Raw records, keys, ranges,
and payloads never contain a `usize`, pointer, borrowed subslice, or candidate
handle; only `SpatialGeometryInputV2` is a borrowed aggregate view.

The three keys implement
`Clone + Copy + Debug + Eq + Hash + Ord + PartialEq + PartialOrd`. All other
owned values in this document implement `Clone + Copy + Debug + Eq + PartialEq`.
Every export preserves `Send + Sync + Unpin + UnwindSafe + RefUnwindSafe`.

## Paths

`SpatialPathVerbKindV2::ALL` is exactly
`MoveTo, LineTo, QuadraticTo, CubicTo, Close`.

```rust
impl SpatialPathVerbKindV2 {
    pub const ALL: [Self; 5];
}
```

`SpatialPathVerbV2` is the exhaustively matchable payload enum:

```text
MoveTo { to: SpatialPointV2 }
LineTo { to: SpatialPointV2 }
QuadraticTo { control: SpatialPointV2, to: SpatialPointV2 }
CubicTo {
    control1: SpatialPointV2,
    control2: SpatialPointV2,
    to: SpatialPointV2
}
Close
```

It has no `ALL`, constructor, kind getter, or accessor that hides payload
matching.

`SpatialPathV2::new(key, verb_start, verb_length)` stores a
`SpatialPathKeyV2` followed by two `u32` values. Its getters are `key`,
`verb_start`, and `verb_length` in that order.

```rust
impl SpatialPathV2 {
    pub const fn new(
        key: SpatialPathKeyV2,
        verb_start: u32,
        verb_length: u32,
    ) -> Self;
    pub const fn key(self) -> SpatialPathKeyV2;
    pub const fn verb_start(self) -> u32;
    pub const fn verb_length(self) -> u32;
}
```

## Shapes

`SpatialShapeKindV2::ALL` is exactly `Rect, Circle, Polygon, Path`.

```rust
impl SpatialShapeKindV2 {
    pub const ALL: [Self; 4];
}
```

`SpatialShapeGeometryV2` is the exhaustively matchable payload enum:

```text
Rect {
    origin: SpatialPointV2,
    width: SpatialScalarV2,
    height: SpatialScalarV2
}
Circle { center: SpatialPointV2, radius: SpatialScalarV2 }
Polygon { point_start: u32, point_length: u32 }
Path { path: SpatialPathKeyV2 }
```

It has no `ALL`, constructor, kind getter, or payload accessors.

`SpatialShapeV2::new(key, owner, geometry)` stores a `SpatialShapeKeyV2`, a
`SpatialNodeKeyV2`, and a `SpatialShapeGeometryV2`. Its getters are `key`,
`owner`, and `geometry`. Construction does not validate the sentinel owner,
scalar domain, extent signs, point range, or path reference.

```rust
impl SpatialShapeV2 {
    pub const fn new(
        key: SpatialShapeKeyV2,
        owner: SpatialNodeKeyV2,
        geometry: SpatialShapeGeometryV2,
    ) -> Self;
    pub const fn key(self) -> SpatialShapeKeyV2;
    pub const fn owner(self) -> SpatialNodeKeyV2;
    pub const fn geometry(self) -> SpatialShapeGeometryV2;
}
```

## Coverage and clips

`SpatialFillRuleV2::ALL` is exactly `NonZero, EvenOdd`.
`SpatialCoverageKindV2::ALL` is exactly `Fill, RoundStroke`.

```rust
impl SpatialFillRuleV2 {
    pub const ALL: [Self; 2];
}
impl SpatialCoverageKindV2 {
    pub const ALL: [Self; 2];
}
```

`SpatialCoverageV2` is the exhaustively matchable payload enum:

```text
Fill { shape: SpatialShapeKeyV2, rule: SpatialFillRuleV2 }
RoundStroke { shape: SpatialShapeKeyV2, width: SpatialScalarV2 }
```

It has no `ALL`, constructor, kind getter, or payload accessors. Raw variants
do not validate the shape reference or stroke width.

`SpatialClipV2::new(key, owner, parent, shape, fill_rule)` stores, in that
order, a `SpatialClipKeyV2`, `SpatialNodeKeyV2`, optional earlier
`SpatialClipKeyV2`, `SpatialShapeKeyV2`, and `SpatialFillRuleV2`. Its getters
are `key`, `owner`, `parent`, `shape`, and `fill_rule`. Construction does not
validate density, ancestry, parent order, shape ownership, or chain depth.

```rust
impl SpatialClipV2 {
    pub const fn new(
        key: SpatialClipKeyV2,
        owner: SpatialNodeKeyV2,
        parent: Option<SpatialClipKeyV2>,
        shape: SpatialShapeKeyV2,
        fill_rule: SpatialFillRuleV2,
    ) -> Self;
    pub const fn key(self) -> SpatialClipKeyV2;
    pub const fn owner(self) -> SpatialNodeKeyV2;
    pub const fn parent(self) -> Option<SpatialClipKeyV2>;
    pub const fn shape(self) -> SpatialShapeKeyV2;
    pub const fn fill_rule(self) -> SpatialFillRuleV2;
}
```

## Borrowed raw input

`SpatialGeometryInputV2<'a>` has the exact constructor order:

```text
polygon_points: &'a [SpatialPointV2]
path_verbs: &'a [SpatialPathVerbV2]
paths: &'a [SpatialPathV2]
shapes: &'a [SpatialShapeV2]
clips: &'a [SpatialClipV2]
```

It exposes getters with those same names and types. This table order follows
the raw content format; validation may scan the tables in the phase order fixed
by the semantic contract. The view implements only `Clone + Copy` plus the
runtime auto-trait set. It does not own, format, compare, hash, or mutate the
supplied rows.

```rust
impl<'a> SpatialGeometryInputV2<'a> {
    pub const fn new(
        polygon_points: &'a [SpatialPointV2],
        path_verbs: &'a [SpatialPathVerbV2],
        paths: &'a [SpatialPathV2],
        shapes: &'a [SpatialShapeV2],
        clips: &'a [SpatialClipV2],
    ) -> Self;
    pub const fn polygon_points(self) -> &'a [SpatialPointV2];
    pub const fn path_verbs(self) -> &'a [SpatialPathVerbV2];
    pub const fn paths(self) -> &'a [SpatialPathV2];
    pub const fn shapes(self) -> &'a [SpatialShapeV2];
    pub const fn clips(self) -> &'a [SpatialClipV2];
}
```

The later full `SpatialInputV2` composes the existing topology view, this
geometry view, and separately versioned paint/resource views. It does not
replace or mutate this constructor.

## Surface discipline and TDD

Every struct field is private. Every constructor and getter is `pub const`,
`#[must_use]`, and returns or stores the exact types above. The four fieldless
vocabularies expose only their registered `ALL` arrays. The three payload enums
do not expose `ALL`; consumers match them exhaustively.

The API RED proves exact round trips with distinct values, enum order and
payload matching, key traits, value traits, borrowed-view behavior, runtime
auto traits, and absence of candidate/runtime/testkit types. The GREEN adds no
normal dependency beyond `fenestra-ui-layout`, no validator, and no geometry
algorithm. Path grammar, deterministic flattening, exact fill and round-stroke
coverage, local bounds, and clip-chain behavior each require their applicable
later validation, geometry-kernel, or resolver RED before implementation.
