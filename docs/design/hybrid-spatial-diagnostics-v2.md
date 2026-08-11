# WU-0013 spatial diagnostics contract

Core contract: [hybrid-spatial-reference-v2.md](hybrid-spatial-reference-v2.md)
Content contract: [hybrid-spatial-content-reference-v2.md](hybrid-spatial-content-reference-v2.md)
Fields: [hybrid-spatial-fields-v2.md](hybrid-spatial-fields-v2.md)
Format: spatial diagnostics version 2

## Boundary separation

Spatial resolution, encoded-image normalization, reference CPU rasterization,
and artifact encoding are separate bounded operations. Each has a closed error
type. A result from one boundary is never reclassified as a limit from another.
Finite vocabularies below expose `ALL` in printed order. Indexed locations and
payload enums containing authored keys do not.

Only the spatial resolver can return `SpatialResolveErrorV2`. It contains
`kind()` and `location()`, plus `observed()` and `maximum()` only for a limit.
Image decoding, rasterization, and artifact errors follow the same redacted
shape but use their own kind enumeration.

## Limit vocabularies

`SpatialLimitKindV2` is partitioned into phase arrays. `ALL` concatenates these
arrays in order; validation may place a non-limit phase between arrays.

```text
DIRECT_ALL =
  Nodes, Shapes, Brushes, Clips, PaintItems, HitItems, SemanticItems,
  Paths, PathVerbsTotal, PolygonPointsTotal, GradientStopsTotal, Images

TOPOLOGY_ALL = Depth, ChildrenPerNode

ISLAND_ALL =
  Islands, LayoutInputRecordsPerIsland, LayoutInputRecordsTotal

CONTENT_ALL =
  PathVerbsPerPath, PathSubpathsTotal,
  PolygonPointsPerShape,
  GradientStopsPerBrush,
  ImageEdge, ImagePixelsTotal,
  ClipDepth, PaintItemsPerNode, HitItemsPerNode,
  FlattenedSegmentsPerPath, FlattenedSegmentsTotal

DEPENDENCY_ALL = DependencyVertices, DependencyEdges
```

The encoded-image boundary has `EncodedBytes`; the raster boundary has
`Pixels`; the artifact boundary has `Records, LineBytes, ArtifactBytes`.
Canonical RGBA byte counts are checked arithmetic derived from pixel counts,
not duplicate limits.

Each registered limit has an exact accepted fixture and a valid one-over
fixture that reaches its named check before any earlier failure. Limits that
are mathematical consequences rather than rejectable capacities do not receive
an enum variant.

## Resolver error kinds

`SpatialResolveErrorKindV2` is:

```text
LimitExceeded(SpatialLimitKindV2)
Input(SpatialInputErrorKindV2)
Content(SpatialContentErrorKindV2)
Dependency(SpatialDependencyErrorKindV2)
Transform(SpatialTransformErrorKindV2)
Layout(SpatialLayoutErrorKindV2)
Arithmetic(SpatialArithmeticOperationV2)
Output(SpatialOutputErrorKindV2)
```

`SpatialLayoutErrorKindV2` is `Engine(LayoutEngineErrorKindV1) |
Output(LayoutOutputErrorKindV1) |
SyntheticRootMismatch(LayoutOutputFieldV1) | BridgeInvariant` in that order.
Prepared layout input failures map to the spatial input kinds below; an
impossible kind or location maps to `BridgeInvariant`. Candidate messages and
payloads never cross the spatial error. The API contract fixes
ordinal-to-spatial location mapping.

`SpatialInputErrorKindV2` vocabulary order is:

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
NegativeViewport(SpatialExtentV2)
NegativeFreeExtent(SpatialExtentV2)
FreeOffsetOutOfDomain(SpatialAxisV2)
InvalidContainer(SpatialContainerErrorKindV2)
InvalidLayoutDimensions(SpatialLayoutDimensionErrorKindV2)
```

`SpatialContainerErrorKindV2::ALL` is negative padding left, right, top, bottom;
padding exceeds width, height; then negative gap. The closed
`SpatialLayoutDimensionErrorKindV2::ALL` is negative width minimum, preferred,
maximum; inverted width; negative height minimum, preferred, maximum; inverted
height. Shared layout input failures map only into those two sub-enums. All
container and island inputs are preflighted exactly once in phase 5.

`SpatialDependencyErrorKindV2` vocabulary order is:

```text
MissingTarget
SentinelNodeTarget
SelfTarget
Cycle
```

Missing and sentinel targets fail by node and target-field order. Self target
then fails before graph allocation. Cycle names the smallest ordinal in the
first truly cyclic strongly connected component selected by the core rule.

`SpatialTransformErrorKindV2` vocabulary order is:

```text
ScalarOutOfDomain(SpatialTransformScalarFieldV2)
SingularTransform
ComposedTransformSingular(SpatialTransformStageV2)
```

The first two validate local matrices in node and field order before the graph
executes. `ComposedTransformSingular` is evaluated in node order after placement
and carries the first stage that rounded to zero determinant.

`SpatialArithmeticOperationV2` vocabulary order is:

```text
TargetOffsetX, TargetOffsetY,
SelfSubtractionX, SelfSubtractionY,
IslandTranslationX, IslandTranslationY,
BaseFarX, BaseFarY,
ParentDeltaX, ParentDeltaY,
Affine { stage: SpatialTransformStageV2,
         component: SpatialAffineComponentV2 },
AabbMinX, AabbMinY, AabbMaxX, AabbMaxY
```

Node or table order precedes operation order; affine stage precedes component.
Target, island, base-far, parent-delta, affine, and AABB arithmetic failures use
`Node { index }` for the owning spatial node. AABBs still scan output table and
record order before operation order; candidate-output validation instead uses
`OutputRecord`. There is no saturation fallback.

`SpatialOutputErrorKindV2` vocabulary order is:

```text
RecordCountMismatch
KeyMismatch
ScalarOutOfDomain
NegativeBaseExtent(SpatialExtentV2)
InvalidWorldDeterminant
InvalidAabb
InvalidClipChain
InvalidProjectionOrder
InvalidReference
```

Output validation runs in complete global passes in that order and in the
field contract's table order. It never sorts, clamps, deduplicates, or repairs
candidate output.

## Content error kinds

`SpatialContentErrorKindV2` is:

```text
NonDenseKey(SpatialKeyedContentTableV2)
InvalidRange(SpatialPayloadTableV2)
InvalidReference(SpatialContentReferenceV2)
InvalidOrder(SpatialOrderedItemTableV2)
ScalarOutOfDomain
InvalidPathGrammar(SpatialPathGrammarErrorV2)
InvalidShape(SpatialShapeErrorV2)
InvalidStroke(SpatialStrokeErrorV2)
InvalidGradient(SpatialGradientErrorV2)
InvalidImage(SpatialImageErrorV2)
InvalidClip(SpatialClipErrorV2)
NonFlatAtMaximumDepth
LocalBoundsOutOfDomain(SpatialAxisV2)
```

Keyed content tables are:

```text
Path, Shape, Brush, Image, Clip
```

Ordered item tables are `Paint, Hit, Semantic`. Payload tables are
`PathVerb, PolygonPoint, GradientStop`. Only keyed tables have supplied global
keys; ordered items use owner plus dense local ordinal.

References are `Path, Shape, Brush, Image, Clip, Owner`. Owner references are
checked against spatial node count before ancestry or local item order.

Path grammar kinds are:

```text
Empty
FirstNotMove
EmptySubpath
DrawingWithoutSubpath
CloseWithoutSegment
TrailingMove
```

Shape kinds are:

```text
NegativeExtent
NegativeRadius
PolygonTooShort
PolygonRepeatedFirst
PolygonAdjacentEqual
```

Stroke kinds are `NegativeWidth, ZeroWidth`. Gradient kinds are
`CoincidentEndpoints, TooFewStops, FirstOffset, LastOffset, DecreasingOffset`.
Image kinds are:

```text
ZeroExtent
StrideMismatch
LengthMismatch
InvalidPremultipliedPixel
EmptySource
SourceOutOfBounds
NegativeDestinationExtent(SpatialExtentV2)
EmptyDestination
```

Clip kinds are `ForwardParent, ShapeOwnerMismatch, OwnerNotAncestor,
ItemOwnerNotDescendant`. Per-path and total flatten count failures use
`LimitExceeded` with a trusted path/verb location.

`ImageDecodeLimitKindV2::ALL` is
`EncodedBytes, Edge(Width), Edge(Height), Pixels`.
`ImageDecodeErrorKindV2` is separate from spatial content:

```text
LimitExceeded(ImageDecodeLimitKindV2)
Malformed
UnsupportedBitDepth
UnsupportedColorType
UnsupportedInterlace
UnsupportedChunk
NormalizedOutputMismatch
```

Its `ALL` expands limits at their validation positions: encoded bytes first;
the five non-limit kinds through `UnsupportedChunk`; width edge, height edge,
and pixels; then `NormalizedOutputMismatch`. Edge observations are the decoded
width or height named by the extent.

`ReferenceRasterLimitKindV2::ALL` is `Pixels`.
`ReferenceRasterErrorKindV2` is:

```text
LimitExceeded(ReferenceRasterLimitKindV2)
```

`SpatialArtifactLimitKindV2::ALL` is `Records, LineBytes, ArtifactBytes`.
`SpatialArtifactErrorKindV2` is:

```text
LimitExceeded(SpatialArtifactLimitKindV2)
InvalidRecord
```

Raster `ALL` is `LimitExceeded(Pixels)`. Artifact `ALL` is
`LimitExceeded(Records), InvalidRecord, LimitExceeded(LineBytes),
LimitExceeded(ArtifactBytes)`.

Decoder locations are `Input`, `Chunk { index: u32 }`,
`Pixel { index: u32, channel: SpatialColorChannelV2 }`, and `Output`. Encoded
bytes use `Input`; edge and pixel limits and header-format failures use the
trusted IHDR chunk ordinal; an unsupported chunk uses its first trusted chunk
ordinal. Malformed structure uses `Input` until a chunk ordinal is trustworthy
and that `Chunk` afterward. A normalized record shape mismatch uses `Output`;
the first byte mismatch uses its pixel ordinal and `R, G, B, A` channel order.
Raster location is `Input`; artifact locations are `Input, Record(index)`.
Their indices are trusted ordinals.

Artifact record-count and total-byte limits use `Input`. `InvalidRecord` and
line-byte limits use the first `Record { index: u32 }` that fails.

Decode priority is encoded-byte limit; malformed container; header bit depth,
color type, and interlace; chunks in encoded order; width edge, height edge,
then pixel limits; decode and normalized-output validation. Raster receives
only a validated immutable spatial snapshot, so its sole typed preflight is the
pixel limit. Artifact priority is records, record grammar, line bytes in
record order, then total bytes. Only limit errors carry `observed/maximum`.

## Locations and redaction

`SpatialErrorLocationV2`, its field enums, transform stages, and output table
order are fixed by the field vocabulary.

All indices come from trusted enumeration, never malformed supplied keys.
`Display` and `Debug` render only closed labels, locations, and limit numbers.
They cannot contain authored scalar values, path payloads, colors, image bytes,
resource names, logical identities, candidate errors, pointers, or allocation
details. Error values remain `Send + Sync + Unpin + UnwindSafe + RefUnwindSafe`.

## Global enforcement priority

The complete first-failure order is:

1. `DIRECT_ALL` counts;
2. root and topology input kinds through `InvalidPreorder`;
3. `TOPOLOGY_ALL`;
4. remaining placement input kinds, integer free dimensions, and free-offset
   scalar fields;
5. `ISLAND_ALL`, then validation-only layout preflight for every container and
   island;
6. local transform scalar fields and determinant in node and field order;
7. path keys, ranges, and grammar; shape keys and records; brush keys and
   ranges, gradient payloads, and brush semantics; images; clips; paint, hit,
   and semantics; then flattening and local bounds, with `CONTENT_ALL` at the
   exact companion-contract points;
8. dependency target kinds, `DEPENDENCY_ALL`, and dry cycle detection;
9. stable dependency execution and layout errors;
10. world transform composition, closed AABBs, and clips;
11. output global passes.

Image decoding completes before a decoded image enters phase 1. Raster
preflight occurs after successful output and before allocation. Artifact
validation performs records, grammar, line bytes, and total bytes after all
typed evidence models pass. Simultaneous faults therefore have one observable
result across every implementation.
