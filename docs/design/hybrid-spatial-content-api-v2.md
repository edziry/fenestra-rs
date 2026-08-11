# WU-0013 raw paint and content API contract

Parent API: [hybrid-spatial-api-v2.md](hybrid-spatial-api-v2.md)
Geometry API:
[hybrid-spatial-geometry-api-v2.md](hybrid-spatial-geometry-api-v2.md)
Semantic contract:
[hybrid-spatial-content-reference-v2.md](hybrid-spatial-content-reference-v2.md)
Fields: [hybrid-spatial-fields-v2.md](hybrid-spatial-fields-v2.md)
Format: raw spatial paint and content API version 2

## Purpose and staging

This contract fixes the Fenestra-owned raw boundary for brushes, normalized
image records, paint items, hit items, and semantic geometry items. These
records are optional capabilities. They do not make a spatial node, subtree,
or complete interface scene-first, paint-first, or layout-first.

The raw content RED adds only the values in this document. It does not add
normalization, gradient sampling, image sampling, blending, validation,
resolved projections, hit selection, semantic joins, CPU raster output,
decoder types, candidate handles, or renderer types. Those behaviors require
their own exclusive REDs. In particular, `projection` means a later immutable
resolved output and is not another name for this raw input view.

The `prototype` surface grows from 50 to exactly 68 reexports:

```text
SpatialBrushKeyV2, SpatialImageKeyV2, SpatialRgba8V2,
SpatialGradientStopV2, SpatialBrushKindV2, SpatialBrushContentV2,
SpatialBrushV2, SpatialImageV2,
SpatialImageSourceRectV2, SpatialImageDestinationRectV2,
SpatialPaintKindV2, SpatialPaintContentV2, SpatialPaintV2,
SpatialInputPolicyV2, SpatialHitV2, SpatialSemanticGeometryV2,
SpatialResourceInputV2, SpatialItemInputV2
```

## Keys and colors

`SpatialBrushKeyV2` and `SpatialImageKeyV2` each wrap one raw `u32`. Each
exposes only `new(value: u32) -> Self` and `get(self) -> u32`. Construction
does not validate density.

```rust
impl SpatialBrushKeyV2 {
    pub const fn new(value: u32) -> Self;
    pub const fn get(self) -> u32;
}
impl SpatialImageKeyV2 {
    pub const fn new(value: u32) -> Self;
    pub const fn get(self) -> u32;
}
```

`SpatialRgba8V2::new(r, g, b, a)` stores four `u8` channels and exposes
`r`, `g`, `b`, and `a` getters in that order. The value itself does not claim
straight or premultiplied semantics; its containing field fixes that meaning.
Brush colors and gradient-stop colors are straight encoded sRGB. Validated
image bytes and resolved sample colors are premultiplied encoded sRGB.

```rust
impl SpatialRgba8V2 {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self;
    pub const fn r(self) -> u8;
    pub const fn g(self) -> u8;
    pub const fn b(self) -> u8;
    pub const fn a(self) -> u8;
}
```

The two keys implement
`Clone + Copy + Debug + Eq + Hash + Ord + PartialEq + PartialOrd`.
`SpatialRgba8V2` implements `Clone + Copy + Debug + Eq + PartialEq`.

## Gradient stops and brushes

`SpatialGradientStopV2::new(offset, color)` stores a `u16` offset and one
straight `SpatialRgba8V2`. Its getters are `offset` and `color`. Construction
does not validate endpoint coverage or ordering relative to other stops.

`SpatialBrushKindV2::ALL` is exactly `Solid, LinearGradient`.

```rust
impl SpatialBrushKindV2 {
    pub const ALL: [Self; 2];
}
```

`SpatialBrushContentV2` is the exhaustively matchable payload enum:

```text
Solid { color: SpatialRgba8V2 }
LinearGradient {
    stop_start: u32,
    stop_length: u32,
    start: SpatialPointV2,
    end: SpatialPointV2
}
```

The range fields precede coordinates so range partition is established before
any trusted stop location is formed. Every range start and length is raw `u32`
and validation widens their sum. The payload has no `ALL`, constructor, kind
getter, or payload accessors.

`SpatialBrushV2::new(key, content)` stores a `SpatialBrushKeyV2` and
`SpatialBrushContentV2`. Its getters are `key` and `content`. Construction
does not validate a stop range, color, coordinate domain, distinct endpoints,
or stop semantics.

```rust
impl SpatialGradientStopV2 {
    pub const fn new(offset: u16, color: SpatialRgba8V2) -> Self;
    pub const fn offset(self) -> u16;
    pub const fn color(self) -> SpatialRgba8V2;
}
impl SpatialBrushV2 {
    pub const fn new(
        key: SpatialBrushKeyV2,
        content: SpatialBrushContentV2,
    ) -> Self;
    pub const fn key(self) -> SpatialBrushKeyV2;
    pub const fn content(self) -> SpatialBrushContentV2;
}
```

Stops, brush kinds, payloads, and brush records implement
`Clone + Copy + Debug + Eq + PartialEq`.

## Owned normalized image records

`SpatialImageV2` stores, in order, a `SpatialImageKeyV2`, `u32` width,
`u32` height, `u32` stride, and one `Box<[u8]>`. The box is the exact owned
byte sequence; no spare capacity, decoder object, texture, profile, or
candidate handle is retained. The constructor accepts bytes without
validating dimensions, stride, length, or premultiplication.

Bytes are top-to-bottom row-major, pixels are left-to-right, and each pixel is
stored `R, G, B, A`; the trusted pixel ordinal is `y * width + x`. A raw image
already claims premultiplied sRGB. Validation checks that claim, including
`r,g,b <= a`, and never premultiplies these bytes. Only straight authored
brush colors, gradient stops, and straight decoder output are normalized.

```rust
impl SpatialImageV2 {
    pub fn new(
        key: SpatialImageKeyV2,
        width: u32,
        height: u32,
        stride: u32,
        bytes: Box<[u8]>,
    ) -> Self;
    pub const fn key(&self) -> SpatialImageKeyV2;
    pub const fn width(&self) -> u32;
    pub const fn height(&self) -> u32;
    pub const fn stride(&self) -> u32;
    pub fn bytes(&self) -> &[u8];
}
```

The constructor and all getters are `#[must_use]`. The constructor and byte
getter are not required to be `const`; the other four getters are `const`.
`SpatialImageV2` implements `Clone + Debug + Eq + PartialEq`, but not `Copy`,
`Hash`, or ordering traits. Its private owned storage is the deliberate
exception to the pointer-free raw record rule used by geometry values.

`SpatialImageSourceRectV2::new(x, y, width, height)` stores four raw `u32`
values and exposes those getters in that order.
`SpatialImageDestinationRectV2::new(x, y, width, height)` stores four
`SpatialScalarV2` values and exposes the same getter order. Neither constructor
validates emptiness, sign, bounds, or the scalar domain.

```rust
impl SpatialImageSourceRectV2 {
    pub const fn new(x: u32, y: u32, width: u32, height: u32) -> Self;
    pub const fn x(self) -> u32;
    pub const fn y(self) -> u32;
    pub const fn width(self) -> u32;
    pub const fn height(self) -> u32;
}
impl SpatialImageDestinationRectV2 {
    pub const fn new(
        x: SpatialScalarV2,
        y: SpatialScalarV2,
        width: SpatialScalarV2,
        height: SpatialScalarV2,
    ) -> Self;
    pub const fn x(self) -> SpatialScalarV2;
    pub const fn y(self) -> SpatialScalarV2;
    pub const fn width(self) -> SpatialScalarV2;
    pub const fn height(self) -> SpatialScalarV2;
}
```

## Paint items

`SpatialPaintKindV2::ALL` is exactly `CoveragePaint, ImagePaint`.

```rust
impl SpatialPaintKindV2 {
    pub const ALL: [Self; 2];
}
```

`SpatialPaintContentV2` is the exhaustively matchable payload enum:

```text
CoveragePaint {
    coverage: SpatialCoverageV2,
    brush: SpatialBrushKeyV2,
    opacity: u8,
    clip: Option<SpatialClipKeyV2>
}
ImagePaint {
    image: SpatialImageKeyV2,
    source: SpatialImageSourceRectV2,
    destination: SpatialImageDestinationRectV2,
    opacity: u8,
    clip: Option<SpatialClipKeyV2>
}
```

The fixed `SourceOver` blend and `Nearest` sampler are format behavior, not
authored choices, so no blend or sampling enum enters the raw API.
`SpatialPaintContentV2` has no `ALL`, constructor, kind getter, or payload
accessors.

`SpatialPaintV2::new(owner, item_ordinal, content)` stores a
`SpatialNodeKeyV2`, raw `u32` local ordinal, and `SpatialPaintContentV2`.
Its getters are `owner`, `item_ordinal`, and `content`. Construction does not
validate the sentinel owner, owner order, dense ordinal, references, source
rectangle, destination rectangle, or clip ancestry. Opacity is a `u8`, so its
complete raw domain is already the accepted inclusive range.

```rust
impl SpatialPaintV2 {
    pub const fn new(
        owner: SpatialNodeKeyV2,
        item_ordinal: u32,
        content: SpatialPaintContentV2,
    ) -> Self;
    pub const fn owner(self) -> SpatialNodeKeyV2;
    pub const fn item_ordinal(self) -> u32;
    pub const fn content(self) -> SpatialPaintContentV2;
}
```

## Hit and semantic items

`SpatialInputPolicyV2::ALL` is exactly `Accept, Ignore`.

```rust
impl SpatialInputPolicyV2 {
    pub const ALL: [Self; 2];
}
```

`SpatialHitV2::new(owner, item_ordinal, coverage, clip, input_policy)` stores
those exact typed fields in order. Its getters use the same names and order.
The coverage remains independent from every paint item and image alpha.

`SpatialSemanticGeometryV2::new(owner, item_ordinal, shape, fill_rule, clip)`
stores a
`SpatialNodeKeyV2`, raw `u32` local ordinal, `SpatialShapeKeyV2`,
`SpatialFillRuleV2`, and optional `SpatialClipKeyV2`. Its getters use those
same names. The spatial record deliberately contains no role, label, action,
runtime descriptor, logical identity, or platform accessibility type.

```rust
impl SpatialHitV2 {
    pub const fn new(
        owner: SpatialNodeKeyV2,
        item_ordinal: u32,
        coverage: SpatialCoverageV2,
        clip: Option<SpatialClipKeyV2>,
        input_policy: SpatialInputPolicyV2,
    ) -> Self;
    pub const fn owner(self) -> SpatialNodeKeyV2;
    pub const fn item_ordinal(self) -> u32;
    pub const fn coverage(self) -> SpatialCoverageV2;
    pub const fn clip(self) -> Option<SpatialClipKeyV2>;
    pub const fn input_policy(self) -> SpatialInputPolicyV2;
}
impl SpatialSemanticGeometryV2 {
    pub const fn new(
        owner: SpatialNodeKeyV2,
        item_ordinal: u32,
        shape: SpatialShapeKeyV2,
        fill_rule: SpatialFillRuleV2,
        clip: Option<SpatialClipKeyV2>,
    ) -> Self;
    pub const fn owner(self) -> SpatialNodeKeyV2;
    pub const fn item_ordinal(self) -> u32;
    pub const fn shape(self) -> SpatialShapeKeyV2;
    pub const fn fill_rule(self) -> SpatialFillRuleV2;
    pub const fn clip(self) -> Option<SpatialClipKeyV2>;
}
```

Each type exposes one by-value `pub const` getter per constructor field.
Source and destination rectangles, paint kinds and payloads, paint records,
input policy, hit records, and semantic records implement
`Clone + Copy + Debug + Eq + PartialEq`.

## Borrowed resource and item inputs

`SpatialResourceInputV2<'a>` has the exact constructor order:

```text
gradient_stops: &'a [SpatialGradientStopV2]
brushes: &'a [SpatialBrushV2]
images: &'a [SpatialImageV2]
```

`SpatialItemInputV2<'a>` independently has the exact constructor order:

```text
paint_items: &'a [SpatialPaintV2]
hit_items: &'a [SpatialHitV2]
semantic_items: &'a [SpatialSemanticGeometryV2]
```

```rust
impl<'a> SpatialResourceInputV2<'a> {
    pub const fn new(
        gradient_stops: &'a [SpatialGradientStopV2],
        brushes: &'a [SpatialBrushV2],
        images: &'a [SpatialImageV2],
    ) -> Self;
    pub const fn gradient_stops(self) -> &'a [SpatialGradientStopV2];
    pub const fn brushes(self) -> &'a [SpatialBrushV2];
    pub const fn images(self) -> &'a [SpatialImageV2];
}
impl<'a> SpatialItemInputV2<'a> {
    pub const fn new(
        paint_items: &'a [SpatialPaintV2],
        hit_items: &'a [SpatialHitV2],
        semantic_items: &'a [SpatialSemanticGeometryV2],
    ) -> Self;
    pub const fn paint_items(self) -> &'a [SpatialPaintV2];
    pub const fn hit_items(self) -> &'a [SpatialHitV2];
    pub const fn semantic_items(self) -> &'a [SpatialSemanticGeometryV2];
}
```

Each view exposes by-value getters with the same names and slice types as its
constructor. Each implements only `Clone + Copy` plus the runtime auto-trait
set. Neither owns, formats, compares, hashes, or mutates supplied tables.
Image bytes remain owned by each borrowed image record and outlive the
resource view.

The later full `SpatialInputV2` composes topology, geometry, resource, and item
views. The two content views are separate so resources may exist without any
projection item and paint, hit, and semantic tables remain independent. This
does not merge logical parentage with spatial parentage and does not require
any node to paint, hit, expose semantics, use layout, or use free placement.

## Surface discipline and TDD

Every struct field is private. Except for the deliberately owned image API,
every constructor and getter is `pub const`; all are `#[must_use]`. The three
fieldless vocabularies expose only their registered `ALL` arrays. The two
payload enums expose no `ALL` and remain exhaustively matchable.

Every export preserves `Send + Sync + Unpin + UnwindSafe + RefUnwindSafe`.
The API RED proves exact storage, round trips, enum order, payload matching,
owned image lifetime, traits, borrowed-view lifetime, private fields, and the
absence of IR, runtime, testkit, candidate, decoder, resolved-output, and
renderer types. The GREEN adds no normal dependency beyond
`fenestra-ui-layout` and no validation or sampling behavior.

Straight-color normalization, gradient preparation and sampling, normalized
image validation and nearest sampling, opacity and blending, clip-chain
projection, hit selection, semantic projection, CPU rasterization, and closed
diagnostics each require their own later RED before implementation.
