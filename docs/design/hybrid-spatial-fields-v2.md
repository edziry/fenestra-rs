# WU-0013 spatial field vocabulary

Core contract: [hybrid-spatial-reference-v2.md](hybrid-spatial-reference-v2.md)
Content contract: [hybrid-spatial-content-reference-v2.md](hybrid-spatial-content-reference-v2.md)
Diagnostics: [hybrid-spatial-diagnostics-v2.md](hybrid-spatial-diagnostics-v2.md)
API: [hybrid-spatial-api-v2.md](hybrid-spatial-api-v2.md)
Format: spatial field vocabulary version 2

## Purpose

Every validation error names a trusted record ordinal and, when more than one
field can fail in that record, one closed field. These names freeze multifault
priority without retaining an authored value. Finite field vocabularies expose
`ALL` in printed order; payload enums and indexed locations do not. A field that
does not apply to a record kind is skipped. Independent raw fields scan in that
order; an explicitly relational check may revisit fields in the semantic order
fixed by the companion content contract.

`SpatialAxisV2::ALL` is `X, Y`. `SpatialExtentV2::ALL` is `Width, Height`.
`SpatialTransformStageV2::ALL` is `About, Placed, World`.
`SpatialAffineComponentV2::ALL` is `A, B, C, D, Tx, Ty`.
`SpatialColorChannelV2::ALL` is `R, G, B, A`.

## Node fields

`SpatialNodeFieldV2::ALL` is:

```text
Key, Parent, Placement,
FreeWidth, FreeHeight, FreeOffsetX, FreeOffsetY,
LayoutWidthMinimum, LayoutWidthPreferred, LayoutWidthMaximum,
LayoutHeightMinimum, LayoutHeightPreferred, LayoutHeightMaximum,
ContainerAxis, PaddingLeft, PaddingRight, PaddingTop, PaddingBottom, Gap,
AffineA, AffineB, AffineC, AffineD, AffineTx, AffineTy,
TransformOriginX, TransformOriginY,
SelfAnchorHorizontal, SelfAnchorVertical,
TargetKind, TargetKey, TargetAnchorHorizontal, TargetAnchorVertical
```

Topology scans `Key, Parent`; phase 4 scans placement and free fields; shared
layout preflight scans layout dimensions and container fields in Layout V1
order; phase 6 scans affine fields then transform origin; dependency validation
scans target fields. Closed anchor and axis discriminants have no invalid raw
state, but remain named for artifact mutation and authoring diagnostics.

`SpatialTransformScalarFieldV2::ALL` is the affine and transform-origin subset
in the exact order above.

## Path and shape fields

`SpatialPathFieldV2::ALL` is `Key, VerbStart, VerbLength`.

`SpatialPathVerbFieldV2::ALL` is:

```text
Kind, ControlX, ControlY, Control1X, Control1Y,
Control2X, Control2Y, ToX, ToY
```

`SpatialShapeFieldV2::ALL` is:

```text
Key, Owner, Kind,
RectX, RectY, RectWidth, RectHeight,
CircleCenterX, CircleCenterY, CircleRadius,
PolygonPointStart, PolygonPointLength, Path
```

`SpatialPolygonPointFieldV2::ALL` is `X, Y`.

## Brush and image fields

`SpatialBrushFieldV2::ALL` is:

```text
Key, Kind, GradientStopStart, GradientStopLength,
ColorR, ColorG, ColorB, ColorA,
GradientStartX, GradientStartY, GradientEndX, GradientEndY
```

`SpatialGradientStopFieldV2::ALL` is `Offset, R, G, B, A`.

`SpatialImageFieldV2::ALL` is:

```text
Key, Width, Height, Stride, ByteLength, Pixel
```

Pixel location uses the trusted decoded pixel ordinal; it never renders channel
bytes. RGBA8 component types and enum discriminants are intrinsically valid.

## Clip and projection fields

`SpatialClipFieldV2::ALL` is:

```text
Key, Owner, Parent, Shape, FillRule
```

`SpatialPaintFieldV2::ALL` is:

```text
Owner, ItemOrdinal, Kind, Image,
SourceX, SourceY, SourceWidth, SourceHeight,
DestinationX, DestinationY, DestinationWidth, DestinationHeight,
CoverageKind, Shape, FillRule, StrokeWidth, Brush, Opacity, Clip
```

`SpatialHitFieldV2::ALL` is:

```text
Owner, ItemOrdinal, CoverageKind, Shape, FillRule, StrokeWidth, Clip,
InputPolicy
```

`SpatialSemanticFieldV2::ALL` is:

```text
Owner, ItemOrdinal, Shape, FillRule, Clip
```

## Output fields and table order

`SpatialOutputTableV2::ALL` is:

```text
Geometry, Clip, Paint, Hit, Semantic
```

Validated path, shape, brush, and image resources are owned input tables rather
than candidate-generated output tables. Candidate tessellation, textures, and
handles are never tables in the snapshot.

`SpatialOutputFieldV2::ALL` is:

```text
Key,
BaseX, BaseY, BaseWidth, BaseHeight,
AffineA, AffineB, AffineC, AffineD, AffineTx, AffineTy,
Determinant,
AabbEmpty, AabbMinX, AabbMinY, AabbMaxX, AabbMaxY,
Owner, Parent, Shape, Brush, Image, Clip,
StackOrdinal, ItemOrdinal
```

Output validation scans every table in `SpatialOutputTableV2::ALL` for record
count, then repeats that table order for keys, scalar fields, determinants,
AABBs, clip chains, projection order, and references. Within a record it uses
`SpatialOutputFieldV2::ALL`. A count mismatch uses the table-level `Output`
location because no supplied record ordinal is trusted.

## Trusted locations

The final version-2 `SpatialErrorLocationV2` is closed as:

```text
Input
Viewport { extent: SpatialExtentV2 }
Node { index: u32 }
NodeField { index: u32, field: SpatialNodeFieldV2 }
Island { index: u32 }
Dependency { ordinal: u32 }
Path { index, field }
PathVerb { path, verb, field }
Shape { index, field }
PolygonPoint { shape, point, field }
Brush { index, field }
GradientStop { brush, stop, field }
Image { index, field }
ImagePixel { image, pixel, channel }
Clip { index, field }
Paint { index, field }
Hit { index, field }
Semantic { index, field }
Output { table }
OutputRecord { table, index, field }
```

The first topology slice exposes only the prefix through `Dependency`.
Content and output variants enter only with their own exclusive REDs; the
completed unpublished 0.2.0 package exposes the full enum above.

`Dependency.ordinal` is the unit's stable ordinal: the lowest spatial key it
produces, not a dense graph index. Each input `field` has the enum belonging to
that record kind; output uses
`SpatialOutputFieldV2`. `Node { index }` is reserved for derived layout,
transform, and arithmetic failures, while raw input failures use `NodeField`.
Decoder chunks
and pixels and artifact records use boundary-specific locations; the raster
pixel-limit failure uses its whole-input location.
