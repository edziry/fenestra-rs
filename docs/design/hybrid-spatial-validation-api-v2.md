# WU-0013 aggregate input and diagnostics API contract

Parent API: [hybrid-spatial-api-v2.md](hybrid-spatial-api-v2.md)
Core semantics: [hybrid-spatial-reference-v2.md](hybrid-spatial-reference-v2.md)
Content semantics: [hybrid-spatial-content-reference-v2.md](hybrid-spatial-content-reference-v2.md)
Diagnostics: [hybrid-spatial-diagnostics-v2.md](hybrid-spatial-diagnostics-v2.md)
Fields: [hybrid-spatial-fields-v2.md](hybrid-spatial-fields-v2.md)
Private behavior:
[hybrid-spatial-input-validation-kernel-v2.md](hybrid-spatial-input-validation-kernel-v2.md)
Format: spatial validation API version 2

## Boundary and staging

This API-only slice composes the four existing borrowed raw views and completes
the closed spatial diagnostic vocabulary. It adds no validation behavior,
prepared proof, layout call, resolved record, snapshot, resolver function,
candidate-output validator, hit query, raster boundary, or artifact type.

The slice adds exactly these 30 `prototype` exports, taking the surface from 68
to 98 names:

```text
SpatialInputV2
SpatialColorChannelV2
SpatialPathFieldV2, SpatialPathVerbFieldV2
SpatialShapeFieldV2, SpatialPolygonPointFieldV2
SpatialBrushFieldV2, SpatialGradientStopFieldV2, SpatialImageFieldV2
SpatialClipFieldV2, SpatialPaintFieldV2, SpatialHitFieldV2,
SpatialSemanticFieldV2
SpatialOutputTableV2, SpatialOutputFieldV2
SpatialKeyedContentTableV2, SpatialPayloadTableV2,
SpatialContentReferenceV2, SpatialOrderedItemTableV2
SpatialPathGrammarErrorV2, SpatialShapeErrorV2, SpatialStrokeErrorV2,
SpatialGradientErrorV2, SpatialImageErrorV2, SpatialClipErrorV2,
SpatialContentErrorKindV2
SpatialLayoutErrorKindV2, SpatialOutputErrorKindV2,
SpatialResolveErrorKindV2, SpatialResolveErrorV2
```

This is exactly two structs and 28 enums. The existing
`SpatialErrorLocationV2` gains its final content and output variants without
adding another export. All struct fields remain private. The normal dependency
set remains exactly `fenestra-ui-layout`.

## Aggregate borrowed input

`SpatialInputV2` uses one common borrow lifetime and stores the four views in
topology, geometry, resources, and items order. Its exact public surface is:

```rust
impl<'a> SpatialInputV2<'a> {
    pub const fn new(
        topology: SpatialTopologyInputV2<'a>,
        geometry: SpatialGeometryInputV2<'a>,
        resources: SpatialResourceInputV2<'a>,
        items: SpatialItemInputV2<'a>,
    ) -> Self;
    pub const fn topology(self) -> SpatialTopologyInputV2<'a>;
    pub const fn geometry(self) -> SpatialGeometryInputV2<'a>;
    pub const fn resources(self) -> SpatialResourceInputV2<'a>;
    pub const fn items(self) -> SpatialItemInputV2<'a>;
}
```

Every method is `#[must_use]`. The view implements only `Clone + Copy` plus
`Send + Sync + Unpin + UnwindSafe + RefUnwindSafe`. It deliberately implements
no `Debug`, `Display`, `Eq`, `PartialEq`, `Hash`, `Ord`, or `PartialOrd` and owns,
formats, compares, hashes, or clones none of the supplied tables or image bytes.

## Field and table vocabularies

Every fieldless enum in this section implements
`Clone + Copy + Debug + Eq + PartialEq` and the runtime auto-trait set. Each
exposes only its `ALL` array, in the printed order below:

```text
SpatialColorChannelV2::ALL[4] = R, G, B, A

SpatialPathFieldV2::ALL[3] = Key, VerbStart, VerbLength
SpatialPathVerbFieldV2::ALL[9] =
  Kind, ControlX, ControlY, Control1X, Control1Y,
  Control2X, Control2Y, ToX, ToY
SpatialShapeFieldV2::ALL[13] =
  Key, Owner, Kind,
  RectX, RectY, RectWidth, RectHeight,
  CircleCenterX, CircleCenterY, CircleRadius,
  PolygonPointStart, PolygonPointLength, Path
SpatialPolygonPointFieldV2::ALL[2] = X, Y

SpatialBrushFieldV2::ALL[12] =
  Key, Kind, GradientStopStart, GradientStopLength,
  ColorR, ColorG, ColorB, ColorA,
  GradientStartX, GradientStartY, GradientEndX, GradientEndY
SpatialGradientStopFieldV2::ALL[5] = Offset, R, G, B, A
SpatialImageFieldV2::ALL[6] =
  Key, Width, Height, Stride, ByteLength, Pixel

SpatialClipFieldV2::ALL[5] = Key, Owner, Parent, Shape, FillRule
SpatialPaintFieldV2::ALL[19] =
  Owner, ItemOrdinal, Kind, Image,
  SourceX, SourceY, SourceWidth, SourceHeight,
  DestinationX, DestinationY, DestinationWidth, DestinationHeight,
  CoverageKind, Shape, FillRule, StrokeWidth, Brush, Opacity, Clip
SpatialHitFieldV2::ALL[8] =
  Owner, ItemOrdinal, CoverageKind, Shape, FillRule, StrokeWidth, Clip,
  InputPolicy
SpatialSemanticFieldV2::ALL[5] =
  Owner, ItemOrdinal, Shape, FillRule, Clip

SpatialOutputTableV2::ALL[5] = Geometry, Clip, Paint, Hit, Semantic
SpatialOutputFieldV2::ALL[25] =
  Key,
  BaseX, BaseY, BaseWidth, BaseHeight,
  AffineA, AffineB, AffineC, AffineD, AffineTx, AffineTy,
  Determinant,
  AabbEmpty, AabbMinX, AabbMinY, AabbMaxX, AabbMaxY,
  Owner, Parent, Shape, Brush, Image, Clip,
  StackOrdinal, ItemOrdinal
```

## Content diagnostic vocabularies

The content table and leaf-error enums have these exact `ALL` arrays:

```text
SpatialKeyedContentTableV2::ALL[5] = Path, Shape, Brush, Image, Clip
SpatialPayloadTableV2::ALL[3] = PathVerb, PolygonPoint, GradientStop
SpatialContentReferenceV2::ALL[6] = Path, Shape, Brush, Image, Clip, Owner
SpatialOrderedItemTableV2::ALL[3] = Paint, Hit, Semantic

SpatialPathGrammarErrorV2::ALL[6] =
  Empty, FirstNotMove, EmptySubpath, DrawingWithoutSubpath,
  CloseWithoutSegment, TrailingMove
SpatialShapeErrorV2::ALL[5] =
  NegativeExtent, NegativeRadius, PolygonTooShort,
  PolygonRepeatedFirst, PolygonAdjacentEqual
SpatialStrokeErrorV2::ALL[2] = NegativeWidth, ZeroWidth
SpatialGradientErrorV2::ALL[5] =
  CoincidentEndpoints, TooFewStops, FirstOffset, LastOffset, DecreasingOffset
SpatialImageErrorV2::ALL[9] =
  ZeroExtent, StrideMismatch, LengthMismatch, InvalidPremultipliedPixel,
  EmptySource, SourceOutOfBounds,
  NegativeDestinationExtent(Width), NegativeDestinationExtent(Height),
  EmptyDestination
SpatialClipErrorV2::ALL[4] =
  ForwardParent, ShapeOwnerMismatch, OwnerNotAncestor,
  ItemOwnerNotDescendant
```

Their payload forms are fixed by these declarations:

```rust
enum SpatialImageErrorV2 {
    ZeroExtent,
    StrideMismatch,
    LengthMismatch,
    InvalidPremultipliedPixel,
    EmptySource,
    SourceOutOfBounds,
    NegativeDestinationExtent(SpatialExtentV2),
    EmptyDestination,
}

enum SpatialContentErrorKindV2 {
    NonDenseKey(SpatialKeyedContentTableV2),
    InvalidRange(SpatialPayloadTableV2),
    InvalidReference(SpatialContentReferenceV2),
    InvalidOrder(SpatialOrderedItemTableV2),
    ScalarOutOfDomain,
    InvalidPathGrammar(SpatialPathGrammarErrorV2),
    InvalidShape(SpatialShapeErrorV2),
    InvalidStroke(SpatialStrokeErrorV2),
    InvalidGradient(SpatialGradientErrorV2),
    InvalidImage(SpatialImageErrorV2),
    InvalidClip(SpatialClipErrorV2),
    NonFlatAtMaximumDepth,
    LocalBoundsOutOfDomain(SpatialAxisV2),
}
```

`SpatialContentErrorKindV2::ALL` has 52 entries. It expands each nested `ALL`
at that variant's printed position, with `ScalarOutOfDomain` and
`NonFlatAtMaximumDepth` each occupying one entry and local bounds expanding
`X, Y` last. All content diagnostic enums implement the value traits and
runtime auto traits above.

## Layout, output, and resolver error kinds

The remaining payload enums are exactly:

```rust
enum SpatialLayoutErrorKindV2 {
    Engine(LayoutEngineErrorKindV1),
    Output(LayoutOutputErrorKindV1),
    SyntheticRootMismatch(LayoutOutputFieldV1),
    BridgeInvariant,
}

enum SpatialOutputErrorKindV2 {
    RecordCountMismatch,
    KeyMismatch,
    ScalarOutOfDomain,
    NegativeBaseExtent(SpatialExtentV2),
    InvalidWorldDeterminant,
    InvalidAabb,
    InvalidClipChain,
    InvalidProjectionOrder,
    InvalidReference,
}

enum SpatialResolveErrorKindV2 {
    LimitExceeded(SpatialLimitKindV2),
    Input(SpatialInputErrorKindV2),
    Content(SpatialContentErrorKindV2),
    Dependency(SpatialDependencyErrorKindV2),
    Transform(SpatialTransformErrorKindV2),
    Layout(SpatialLayoutErrorKindV2),
    Arithmetic(SpatialArithmeticOperationV2),
    Output(SpatialOutputErrorKindV2),
}
```

Their `ALL` lengths are 22, 10, and 192 respectively. Layout expands engine 9,
layout output 8, synthetic-root fields 4, then bridge invariant. Output expands
negative base extent as width then height. Resolver expands limits 30, input 30,
content 52, dependency 4, transform 12, layout 22, arithmetic 32, and output 10.

`SpatialResolveErrorKindV2::ALL` is a finite vocabulary inventory in top-level
variant order. It is not the global enforcement sequence: limit kinds remain
grouped there even though their phase arrays are interleaved with non-limit
validation by the diagnostics contract. All three enums implement the value
traits and runtime auto traits above.

## Trusted location surface

`SpatialErrorLocationV2` retains its existing prefix and gains the exact final
payload surface below:

```rust
enum SpatialErrorLocationV2 {
    Input,
    Viewport { extent: SpatialExtentV2 },
    Node { index: u32 },
    NodeField { index: u32, field: SpatialNodeFieldV2 },
    Island { index: u32 },
    Dependency { ordinal: u32 },
    Path { index: u32, field: SpatialPathFieldV2 },
    PathVerb { path: u32, verb: u32, field: SpatialPathVerbFieldV2 },
    Shape { index: u32, field: SpatialShapeFieldV2 },
    PolygonPoint {
        shape: u32,
        point: u32,
        field: SpatialPolygonPointFieldV2,
    },
    Brush { index: u32, field: SpatialBrushFieldV2 },
    GradientStop {
        brush: u32,
        stop: u32,
        field: SpatialGradientStopFieldV2,
    },
    Image { index: u32, field: SpatialImageFieldV2 },
    ImagePixel {
        image: u32,
        pixel: u128,
        channel: SpatialColorChannelV2,
    },
    Clip { index: u32, field: SpatialClipFieldV2 },
    Paint { index: u32, field: SpatialPaintFieldV2 },
    Hit { index: u32, field: SpatialHitFieldV2 },
    Semantic { index: u32, field: SpatialSemanticFieldV2 },
    Output { table: SpatialOutputTableV2 },
    OutputRecord {
        table: SpatialOutputTableV2,
        index: u32,
        field: SpatialOutputFieldV2,
    },
}
```

All ordinals are trusted `u32` enumeration results except an image pixel
ordinal, which is `u128` because the image kernel fixes widened product and
pixel-ordinal evidence in that type. Pre-rejection products may exceed `usize`.
The location remains `Clone + Copy + Debug + Eq + PartialEq`, preserves the
runtime auto-trait set, and exposes no impossible `ALL` array.

This value slice does not decide how aggregate validation rejects a supplied
table whose length cannot produce a trusted `u32` ordinal. Totality for that
case belongs to the later validation-kernel contract; it does not change the
location payload types fixed here.

## Stored resolver error

`SpatialResolveErrorV2` stores, in order, a `SpatialResolveErrorKindV2`, a
`SpatialErrorLocationV2`, `Option<u128>` observed, and `Option<u128>` maximum.
Its fields and constructors are private. Only resolver-owned code may construct
it. Its exact public methods are:

```rust
impl SpatialResolveErrorV2 {
    pub const fn kind(self) -> SpatialResolveErrorKindV2;
    pub const fn location(self) -> SpatialErrorLocationV2;
    pub const fn observed(self) -> Option<u128>;
    pub const fn maximum(self) -> Option<u128>;
}
```

Every getter is `#[must_use]`. A `LimitExceeded` value has both numbers and
every other kind has neither. `u128` is required because the image-pixel
candidate may be exactly `usize::MAX + 1` before rejection. `maximum` is the
effective checked maximum: normally the widened caller limit, or a later
representability ceiling when the validation-kernel contract explicitly places
one before that caller limit.

The value implements `Clone + Copy + Eq + PartialEq`, custom `Display` and
`Debug`, `std::error::Error`, and the runtime auto-trait set. `Display` renders
`spatial-resolve-error(<category>)`, where category is exactly one of
`limit-exceeded`, `input`, `content`, `dependency`, `transform`, `layout`,
`arithmetic`, or `output`. `Debug` renders
`SpatialResolveErrorV2(<Display>)`. `Error::source` returns `None`. Formatting
never contains authored values, malformed keys, image bytes, resource names,
logical or candidate identities, pointers, allocation details, or candidate
messages. Exact kinds, locations, and limit evidence remain available through
typed getters rather than formatting.

## Surface discipline and TDD

The API RED imports all 30 names through one grouped import and fails only with
the unresolved-import diagnostic. It fixes the 98-name export set, 37 private
structs, exact methods, one-lifetime signatures, enum payload matching, every
`ALL` length and order, exhaustive location matching, traits, and the absence
of a public error constructor. It also proves that the aggregate view does not
copy underlying slices or image bytes. The first validation-behavior RED
constructs resolver-owned errors and tests their exact redacted formatting.

The GREEN adds values only. It updates every legacy exact-export guard and the
old exhaustive location match without weakening either. It adds no output
record, resolver signature, validation algorithm, dependency, public alias,
glob, macro, candidate type, runtime type, or testkit type. Aggregate validation
and resolved projection behavior require their own exclusive REDs.
