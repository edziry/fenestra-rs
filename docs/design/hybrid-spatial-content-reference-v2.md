# WU-0013 spatial content reference contract

Parent contract: [hybrid-spatial-reference-v2.md](hybrid-spatial-reference-v2.md)
Diagnostics: [hybrid-spatial-diagnostics-v2.md](hybrid-spatial-diagnostics-v2.md)
Fields: [hybrid-spatial-fields-v2.md](hybrid-spatial-fields-v2.md)
Format: spatial content contract version 2

## Purpose and tables

This contract freezes the minimum rich two-dimensional content required by
WU-0013 without selecting a geometry, raster, GPU, image, or indexing library.
Path, shape, brush, image, and clip records use dense pass-local global keys.
Paint, hit, and semantic items instead use an owner plus dense per-owner item
ordinal. Polygon points, path verbs, and gradient stops are ordinal payload rows
owned by canonical ranges. Every value is Fenestra-owned.

The raw input contains independent ordered tables for polygon points, path
verbs, paths, shapes, gradient stops, brushes, decoded images, clips, paint
items, hit items, and semantic geometry items. Ranges are contiguous
`start + length` slices into a validated table. Path ranges partition the verb
table in path-key order; polygon shapes partition the point table in shape-key
order; gradient brushes partition the stop table in brush-key order. There are
no gaps, overlaps, or unreferenced trailing records. Tables contain no borrowed
pointers, runtime identities, candidate handles, or host paths; images may own
bytes without making allocation identity part of the data.

Every shape names one authored spatial node greater than zero. Its coordinates
are node-local `SpatialScalarV2` values. A paint, hit, or semantic item may
reference only a shape owned by the same node. Reusing the same shape key is an
explicit choice; separate shapes remain independent even when their bytes are
equal.

## Shapes

The closed shape vocabulary is:

- `Rect`: origin plus nonnegative width and height;
- `Circle`: center plus nonnegative radius;
- `Polygon`: a contiguous range of three or more points;
- `Path`: one validated path key.

A zero-width or zero-height rectangle and a zero-radius circle use canonical
empty fill/clip AABBs. Round stroke retains their degenerate boundary: one zero
rectangle axis is a line, two zero axes are a point, and a zero-radius circle is
a point. A polygon does not repeat its first point at the end and has no equal
adjacent points. Collinear edges, winding direction, concavity, and
self-intersection are retained. Fill rules decide self-intersection.

Rect containment is left/top inclusive and right/bottom exclusive. Circle
containment uses widened exact `dx*dx + dy*dy <= radius*radius`; the zero-radius
special case remains empty. Polygon and flattened-path containment first treats
a point exactly on a nonzero boundary segment as inside. It then casts a ray in
positive x, counts only edges whose vertical half-open interval contains the
point, and evaluates either nonzero winding or even-odd parity. Horizontal and
zero-length segments do not add crossings.

Rect remains an analytic case. An adapter may not convert it to a closed path,
because path boundary inclusion would change its right and bottom edges.

For a directed segment from `p0` to `p1`, `edge = p1 - p0` and the crossing
cross product is `cross(edge, point - p0)`. An upward crossing requires
`p0.y <= point.y < p1.y` and a positive cross. A downward crossing requires
`p1.y <= point.y < p0.y` and a negative cross. Upward adds one to winding and
downward subtracts one. Boundary membership first requires exact zero cross and
a point coordinate inside both inclusive segment ranges.

Closed local base bounds use analytic rect/circle extrema, every polygon point,
and every path move, endpoint, and control point. Fill/clip make a zero rect
axis or zero-radius circle empty; stroke keeps its line or point. For positive
raw width `w`, stroke expands by `e = ceil(w / 2) = (w + 1) / 2` ticks widened,
subtracting `e` from minima and adding it to maxima. Image paint uses its
destination; world AABBs transform the four corners of the local box. Curves,
circles, and strokes retain exact coverage rather than promoting the AABB.
Every expanded local bound must remain inside the canonical scalar domain.
Failure is `LocalBoundsOutOfDomain`, with x checked before y, at the shape or
projection item whose expansion crossed the boundary.

For a rect the failure location is `RectWidth` or `RectHeight`; for a circle it
is `CircleRadius`. Polygon and path base bounds cannot cross after their points
pass scalar validation. Stroke expansion uses the paint or hit `StrokeWidth`;
an image destination uses `DestinationWidth` or `DestinationHeight`. Base
shapes scan by key first, followed by paint and then hit item order; inapplicable
checks are skipped.

## Paths and deterministic flattening

Each path record owns one canonical verb range. Verb payloads are inline:

- `MoveTo { to }` and `LineTo { to }` each contain one point;
- `QuadraticTo { control, to }` contains two points in that field order;
- `CubicTo { control1, control2, to }` contains three points in that order;
- `Close` contains no point.

Range `start + length` uses widened checked arithmetic before indexing. Paths
partition the complete verb table in key order. A path has one or more nonempty
subpaths:

1. zero verbs is `Empty`; a first verb other than `MoveTo` is `FirstNotMove`;
2. `MoveTo` starts a subpath; another before a drawing verb is `EmptySubpath`;
3. line and curve verbs require an active subpath, otherwise the failure is
   `DrawingWithoutSubpath`;
4. `Close` requires an active subpath with a drawing verb, otherwise the
   failure is `CloseWithoutSegment`, and a valid close ends that subpath;
5. ending immediately after `MoveTo` is `TrailingMove`;
6. another `MoveTo` after a drawing verb starts the next subpath, while an open
   final subpath is valid;
7. fill closes every open nonempty subpath implicitly, while stroke retains
   whether each subpath was explicitly closed.

Invalid grammar is rejected before any spatial evaluation or renderer call.
Duplicate curve controls and zero-length drawn segments are retained and later
treated as degenerate geometry rather than grammar errors.

The reference flattener is fixed and platform-independent. Lines emit their end
point. Quadratic and cubic curves recursively split at `t=1/2` with de
Casteljau arithmetic. Every midpoint is one widened sum divided once with the
registered nearest, ties-away rule.

A curve at depth `d` is tested for flatness before any depth decision. It is
flat when every control point satisfies this integer test against the chord:

```text
abs(cross(control - start, end - start))
    <= 256 * max(abs(end.x - start.x), abs(end.y - start.y))
```

Every control coordinate must also lie inside the endpoint bounding box
expanded by 256 raw ticks. If the chord is zero, every control coordinate must
be within 256 raw ticks of the start. A flat curve emits its end. The root has
depth zero. A nonflat curve at depth 16 fails with
`NonFlatAtMaximumDepth`; otherwise it splits and tests both children at depth
`d + 1`. It never emits a partial approximation. A line, curve leaf, and
explicit `Close` each count as one flattened segment, including a
zero-length segment. An implicit fill closure is evaluated without retaining an
extra segment and does not count. Splits preserve source order. Per-path count
is checked before adding it to the total; an overflow names the path and source
verb that attempted to exceed the limit.

## Fill and round stroke coverage

Paint and hit share one closed coverage vocabulary while retaining independent
records:

- `Fill { shape, rule }`, where rule is `NonZero | EvenOdd`;
- `RoundStroke { shape, width }`, where width is a positive spatial scalar.

Scalar-domain validation runs before stroke semantics. A representable
negative width is `NegativeWidth`, zero is `ZeroWidth`, and an expanded bound
outside the domain is `LocalBoundsOutOfDomain`.

Fill uses the exact shape rules above. Round stroke is the union of closed
radius disks swept along every boundary segment. Rect and polygon boundaries
use their exact line segments. Path stroke uses the registered flattened
subpaths. Open ends and joins are round because every segment endpoint
contributes the same disk; closed paths add the closing segment. Circle stroke
is the exact annulus satisfying
`max(2 * radius - width, 0)^2 <= 4 * distance_squared` and
`4 * distance_squared <= (2 * radius + width)^2`.

For a line segment, `delta = end - start`,
`dot = dot(point - start, delta)`, and
`length_squared = dot(delta, delta)`. The reference computes a fixed projection
parameter as `round(dot * 65536 / length_squared)`, clamped from zero through
65536. It
constructs each closest-point coordinate as
`start + round(delta * parameter / 65536)` and accepts when
`4 * distance_squared <= width_raw * width_raw` in widened arithmetic. A
zero-length segment is one radius disk. This algorithm avoids squared cross
products outside the widened domain.

Version 2 has no configurable cap, join, miter, dash, stroke alignment, or
non-scaling stroke. Those additions require a new content format; candidates
cannot silently reduce round-stroke semantics.

## Clips

One clip record contains a dense key, owner node, optional earlier parent clip,
shape, and fill rule. Clip is always filled coverage; stroke clips are absent.
The referenced shape must have the same owner as the clip. The clip owner must
be the item owner or one of its spatial ancestors. Along a chain, each parent
owner must be the same node or an ancestor of the child owner. The chain
therefore runs outer ancestor to inner descendant and cannot cycle.

Every item names its own optional terminal clip. To test a scene-logical point,
the resolver walks the complete chain, maps through each clip owner's forward
matrix with the registered direct inverse-point formula, and requires exact
local fill containment. A conservative world AABB may reject early but can
never accept a point. Empty clips remove all coverage. Visual primitives and
their unclipped bounds remain unchanged.

Resolved closed clip AABBs intersect in chain order: minima use maximum and
maxima use minimum. A maximum below its minimum sets the explicit empty state.
Equality retains a conservative point and still requires exact shape tests.
An empty AABB canonically stores zero in all four edge fields; no stale
pre-intersection edge survives.
Empty intersection preserves both the clip and item records; it only permits
the presenter to omit raster work.

## Brushes, alpha, and blending

The brush vocabulary is:

- `Solid { color }`;
- `LinearGradient { start, end, stops }`.

Authored brush colors are straight encoded sRGB `Rgba8`. They normalize to
premultiplied channels with `(channel * alpha + 127) / 255`. A paint item also
has byte opacity in the inclusive range zero through 255. Effective
premultiplied RGBA applies `(channel * opacity + 127) / 255` exactly once to
each of its four channels before blending; alpha is not multiplied twice.
The only blend mode is premultiplied `SourceOver` in sRGB byte space:

```text
out = source + (destination * (255 - source_alpha) + 127) / 255
```

Each output channel is range checked and must retain `r,g,b <= a`. Transparent
source is a no-op.

A linear gradient has distinct local start and end points and two or more
stops. Stop offsets are `u16` values from zero through 65535, nondecreasing in
table order; the first is zero and the last is 65535. Equal adjacent offsets
form a hard transition whose later color wins at that offset. Outside the line
the gradient pads to the endpoint color. Inside, the scalar projection is
`round(dot(local_point - start, end - start) * 65535 /
dot(end - start, end - start))`, clamped once from zero through 65535.
Between distinct stops, each normalized premultiplied channel uses
`left + round((right - left) * local_offset / stop_span)` with signed nearest,
ties-away rounding. Version 2 has no radial, conic, repeat, reflect, or
color-space interpolation choice.

Gradient coordinates are local to the owner of the paint item, even when a
brush record is shared. For a quantized parameter, the lower stop is the
greatest-ordinal stop whose offset is at most the parameter; the upper stop is
the next strictly greater offset. An exact duplicate therefore selects the
later duplicate. Endpoint parameters use the endpoint directly.

## Normalized images

One decoded image record contains dense key, `u32` width, `u32` height, `u32`
stride, and owned bytes. Width and height are positive and at most the
registered edge. The only format is premultiplied sRGB RGBA8. Canonical stride
is exactly `width * 4`, byte length is exactly `stride * height`, every color
channel is at most alpha, and all products use widened checked arithmetic before
allocation.

An image paint item names an image, an in-bounds `u32` source rectangle, a
node-local Fixed16 destination rectangle with strictly positive extents,
opacity, `SourceOver`, and `Nearest` sampling. Destination containment is
half-open. A covered local point maps proportionally into the source rectangle;
integer source coordinates use
`source.x + floor((local.x - destination.x) * source.width /
destination.width)` and the corresponding y formula. The destination far edge
is excluded before sampling. Source near plus extent widens to `u64` and must
not exceed the image extent. A zero source width or height is `EmptySource`;
a widened source range beyond the image is `SourceOutOfBounds`. Destination scalars are
domain-checked first, then a negative width or height is
`NegativeDestinationExtent` and zero is `EmptyDestination`, in width-then-height
order. No image object, texture, atlas slot, decoder metadata, ICC payload, or
candidate handle enters output.

Image-paint validation orders source width then height emptiness; source x near
and far, then y near and far; destination scalar domain in x, y, width, height
order; negative width then height; zero width then height; and local bound x
then y. A source near failure names `SourceX` or `SourceY`; a far failure names
`SourceWidth` or `SourceHeight`. Destination local-bound failures name
`DestinationWidth` or `DestinationHeight`.

After nonempty source extents, x near fails when `source.x >= image.width`; x
far fails only when widened `source.x + source.width > image.width`, so equality
passes. Y uses the same predicates against image height. Near is checked before
far on each axis.

The decoder seam produces straight sRGB RGBA8 in top-left orientation; the
normalizer premultiplies it with the brush formula above. A different color
profile, animation, unnormalized orientation, or non-RGBA8 result is a closed
unsupported outcome in the first probe.

The image-resource probe accepts bounded encoded bytes separately from the
spatial input and compares an exact normalized record against literal expected
pixels. PNG is the first registered encoding. Its first profile admits only
noninterlaced 8-bit RGB or RGBA, with either no color-space chunk or an `sRGB`
chunk. RGB expands with alpha 255. Palette, grayscale, `tRNS`, `iCCP`, `gAMA`,
`cHRM`, animation, and embedded orientation metadata are closed unsupported
outcomes. Corrupt input, dimension or encoded-byte limits, stride overflow,
length mismatch, and decompression bombs are typed failures. Absence of a
color-space chunk means sRGB; no host color service participates.

The accepted chunk grammar is exactly PNG signature, `IHDR`, one optional
well-formed `sRGB` before image data, one or more contiguous `IDAT` chunks, and
`IEND`. CRC, duplicate singleton, missing, trailing-data, or ordering violations
are `Malformed`; repeated contiguous `IDAT` chunks are required image data. The
first other well-formed critical or ancillary chunk is `UnsupportedChunk` at
its trusted ordinal. A valid `sRGB` chunk has length one and rendering intent
zero through three; its intent does not alter normalized bytes.

## Paint, hit, and semantic records

A paint item is one of:

- `CoveragePaint { coverage, brush, opacity, clip }`;
- `ImagePaint { image, source, destination, opacity, clip }`.

A hit item stores independent coverage, clip, and `Accept | Ignore` input
policy. `Ignore` is skipped before containment or topmost selection. A hit item
never references a paint item, raster pixel, brush, or image alpha. A fully
transparent painted object may still have a hit item; an opaque painted object
may have none. Hit scans the normalized order in reverse and returns the first
exact match.

A semantic geometry item stores a neutral spatial key, local ordinal,
independent fill coverage, and clip. Runtime joins that record to its private
semantic descriptor after spatial resolution. The spatial crate never imports
a runtime type. Semantic geometry does not inherit paint coverage, hit policy,
painter order, or image metadata. Platform accessibility bridging is outside
WU-0013.

Paint, hit, and semantic tables are sorted by owner spatial key and dense
per-owner item ordinal. Within one node their ordinal sequences are independent.
Path, shape, brush, image, and clip tables use their own dense global keys and
the range rules above rather than owner sorting. The synthetic key-zero sentinel
owns no record in any content table.

## Deterministic CPU reference

The owned CPU reference raster consumes only a successfully validated immutable
spatial snapshot; it has no raw scene-input seam. It is not the existing
opaque-rectangle raster, which remains a control subset. The new reference
allocates one transparent premultiplied RGBA8 buffer for the integer logical
viewport. Each pixel uses the Cartesian product of sample offsets
`{1,3,5,7}/8`, for 16 scene-logical samples. It visits paint items in normalized
ascending order for each sample.

The output origin is top-left. Rows ascend by y, pixels ascend by x, channels
are stored `R, G, B, A`, and stride is exactly `viewport.width * 4`; there is no
row padding. Pixel `(x,y)` uses scene points `(x + sx, y + sy)` for the
registered sample offsets.

Viewport width times height is widened and checked against the raster-pixel
limit before allocation. At the registered maximum, multiplication by four is
at most 16777216 bytes and therefore representable on every supported 32- or
64-bit target; it is a checked derivation, not another error or capacity.

For coverage paint it applies the direct inverse-point formula, exact coverage,
and all clips, then samples the local brush and blends. For image paint it applies
the same transform and clips, samples nearest normalized image data, applies
opacity, and blends. The 16 final premultiplied sample results are averaged per
channel with `(sum + 8) / 16`. There is no adaptive antialiasing, filtering,
dithering, host font, SIMD-dependent path, or device scale. A native adapter may
composite this transparent buffer over its configured surface background.

Reference bytes must be identical on Linux and Windows before WU-0013 exits.
Candidate CPU pixels are exact only if the registered profile proves equality
on both systems; otherwise the artifact records a closed mismatch class and
versioned tolerance. GPU output is
never a universal byte oracle.

## Validation order and corpus

After spatial topology, node transforms, and table-count limits, content
validation orders:

1. dense path keys and verb ranges; then, per path, applicable verb scalar
   domains, complete path grammar, and `PathSubpathsTotal`, in that order;
2. dense shape keys, owner, kind, ranges, scalar domains, and per-record limits;
3. dense brush keys, kinds, and gradient-range partition; gradient stops by
   trusted brush ordinal; then color, coordinate, stop-order, and normalization
   semantics;
4. dense image keys, dimensions, edge, pixel count, stride, bytes, then channel
   normalization;
5. dense clip keys, owner ancestry, earlier parent, shape ownership, chain depth;
6. paint, hit, and semantic owner order, item ordinal, kind-specific fields in
   their declared order, opacity, and terminal clip ancestry;
7. local flatten counts, local containment auxiliaries, and local bounds.

Every error location uses a trusted table and ordinal, never a malformed key or
payload. Independent raw fields scan in the companion field-vocabulary order;
range, image-rectangle, and other relational checks use the explicit semantic
suborders above. These raw and local phases complete before dependency
evaluation or the first layout, raster, presenter, or spatial candidate call.
Image decoding is a separate earlier candidate boundary that completes before
spatial input exists. Transformed
bounds and clip intersections occur later in the core output phase after base
placement and world transforms resolve.

The required corpus includes empty and degenerate shapes; rect and circle edge
points; convex, concave, reversed, collinear, and self-intersecting polygons;
holed multi-subpath fills under both rules; open and closed line, quadratic, and
cubic paths; flatten threshold and depth endpoints; round stroke ends, joins, and zero
segments; nested transformed clips; misses inside AABBs; independent paint,
hit, and semantic shapes; hard and interpolated gradient stops; alpha and
SourceOver; image corners, crop, nearest scaling, transparency, malformed input,
and exact resource limits.

Every field has a typed mutation control. Literal shape, hit, normalized image,
and CPU pixel oracles are independent from candidate output. Structural scene,
hit transcript, normalized resources, and reference CPU pixels enter the
canonical bounded artifact; dependency-owned tessellation and GPU buffers do
not.
