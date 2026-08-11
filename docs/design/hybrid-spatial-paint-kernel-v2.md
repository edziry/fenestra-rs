# WU-0013 local paint kernel contract

Raw API: [hybrid-spatial-content-api-v2.md](hybrid-spatial-content-api-v2.md)
Semantic contract:
[hybrid-spatial-content-reference-v2.md](hybrid-spatial-content-reference-v2.md)
Fields: [hybrid-spatial-fields-v2.md](hybrid-spatial-fields-v2.md)
Diagnostics: [hybrid-spatial-diagnostics-v2.md](hybrid-spatial-diagnostics-v2.md)
Format: local paint kernel version 2

## Purpose and boundary

This contract freezes the candidate-neutral local arithmetic and preparation
needed by solid colors, linear gradients, and normalized images. The kernel is
crate-private. Its type names, proof layout, helper signatures, and allocation
strategy are not public API; its results, failure priority, locations, limits,
and rollback behavior are versioned.

The kernel does not validate dense global keys, range partition, owner order,
shape ownership, clip ancestry, or world transforms. It does not resolve
clips, select hits, join semantic descriptors, allocate a raster, decode an
encoded image, or call a renderer candidate. Those boundaries consume its
validated local proofs later.

Implementation proceeds through six exclusive RED/GREEN cuts:

```text
P1 color arithmetic
P2 gradient preparation
P3 gradient sampling
P4 normalized image validation
P5 image-paint preparation and local bounds
P6 nearest image sampling
```

No later cut weakens or replaces an earlier proof.

## P1 color arithmetic

A straight `SpatialRgba8V2 { r, g, b, a }` normalizes exactly to:

```text
r' = (r * a + 127) / 255
g' = (g * a + 127) / 255
b' = (b * a + 127) / 255
a' = a
```

Products widen before addition. This is the only straight-to-premultiplied
conversion. A raw normalized image is never passed through it.

Applying byte opacity `o` to an already premultiplied color computes all four
channels exactly once as `(channel * o + 127) / 255`. In particular, alpha is
not multiplied a second time by its original alpha.

For validated premultiplied source `s` and destination `d`, `SourceOver`
computes each channel independently as:

```text
out = s + (d * (255 - s.a) + 127) / 255
```

These operations are total and infallible over their stated byte domains.
They do not clamp or expose an arithmetic error. The results are in `0..=255`
and retain `r,g,b <= a`; this is a mathematical invariant checked by the RED.
A transparent validated source is exactly a no-op.

P1 tests every channel, alpha preservation, zero and full opacity, rounding
boundaries on both sides, transparent and opaque SourceOver, nontrivial alpha
composition, channel upper bounds, and premultiplication invariants. Literal
expected bytes never come from the implementation under test.

## P2 gradient preparation and priority

The caller has already validated dense brush keys, the global stop count, and
the complete gap-free stop-range partition. It supplies a trusted stop slice
for one brush. A solid brush only normalizes its straight color and cannot
fail.

One linear gradient completes these checks in order before a proof is
published:

1. `GradientStopsPerBrush` compares raw `stop_length` with its maximum;
2. scalar domain checks run for `GradientStartX`, `GradientStartY`,
   `GradientEndX`, then `GradientEndY`;
3. equal start and end returns `CoincidentEndpoints`;
4. fewer than two stops returns `TooFewStops`;
5. the first offset must be zero;
6. the last offset must be 65535;
7. table order scans for the first offset lower than its predecessor;
8. every retained straight stop color is normalized with P1.

The per-brush limit uses `LimitExceeded(GradientStopsPerBrush)` at
`Brush { index, field: GradientStopLength }`, with raw length as `observed`.
Scalar failures use the matching brush field. Coincident endpoints use
`Brush { index, field: GradientEndX }`. Too few stops uses
`Brush { index, field: GradientStopLength }`. First, last, and decreasing
offset failures use `InvalidGradient(FirstOffset)`,
`InvalidGradient(LastOffset)`, or `InvalidGradient(DecreasingOffset)` at
`GradientStop { brush, stop, field: Offset }`. Here `stop` is the zero-based
local ordinal within that brush's trusted range; decreasing names the later
stop that first violates order.

The limit wins over every scalar or semantic fault. Scalar fields complete
before coincidence, coincidence wins over too few stops, and the printed stop
checks then win in order. No error commits a proof. Equal adjacent offsets are
valid and retained verbatim.

P2 covers solid normalization and infallibility, exact and one-over stop
limits, every scalar field, simultaneous limit/scalar/semantic faults, zero
and one stop, first/last offsets, the first of multiple decreases, local stop
ordinals, duplicates at zero/interior/65535, normalization of every stop, and
absence of a proof after failure.

## P3 total linear-gradient sampling

Sampling accepts only a P2 proof and one canonical point local to the paint
item owner. Let `delta = end - start`, `relative = point - start`, and
`denominator = dot(delta, delta)`. All differences, products, and sums use
widened signed arithmetic. The proof makes the denominator positive.

The quantized parameter is:

```text
t = round(dot(relative, delta) * 65535 / denominator)
t = clamp(t, 0, 65535)
```

The division uses nearest, ties-away rounding. Rounding happens before the
single clamp. Negative projection, positive overflow, horizontal, vertical,
diagonal, and canonical scalar-edge fixtures cover the complete parameter
domain; the placement of an integer clamp around nearest rounding is not
separately observable.

The lower stop is the greatest-ordinal stop whose offset is at most `t`; the
upper stop is the next strictly greater offset. If no upper stop exists, the
lower color is returned. Therefore the last zero-offset stop wins at `t=0`,
the last duplicate at any exact offset wins, and the final 65535 stop wins at
the far endpoint.

Between distinct stops, with `local = t - lower.offset` and
`span = upper.offset - lower.offset`, each premultiplied channel is:

```text
lower + round((upper - lower) * local / span)
```

The signed division is nearest with ties away from zero. Sampling is total,
allocates nothing, and cannot return an error. P3 covers padding, duplicate
transitions, increasing and decreasing channel deltas, signed half ties,
endpoint selection, shared brushes under different owner-local points, and
the premultiplied invariant of every sampled color.

## P4 normalized image validation and priority

The caller has already validated dense image keys and the direct image-table
count. It supplies the cumulative pixel count from prior valid images. One raw
image completes these checks in order:

1. zero width, then zero height;
2. `ImageEdge` for width, then height;
3. widened `width * height`, followed by cumulative `ImagePixelsTotal`;
4. stride equals widened `width * 4`;
5. byte length equals widened `stride * height`;
6. row-major pixels, then channels `R, G, B`, must be at most alpha.

Zero extent returns `InvalidImage(ZeroExtent)` at `Width` or `Height`.
Edge limits use the corresponding field and dimension as `observed`.
`ImagePixelsTotal` uses the cumulative count after including the current image
as `observed`, is located at `Image { index, field: Pixel }`, and commits that
count only on success. Stride and length mismatches use `Stride` and
`ByteLength` with `InvalidImage(StrideMismatch)` and
`InvalidImage(LengthMismatch)`. A premultiplication failure uses
`InvalidImage(InvalidPremultipliedPixel)` at
`ImagePixel { image, pixel, channel }` for the first row-major pixel and first
of `R, G, B` greater than alpha. Alpha itself has no invalid byte state.

All arithmetic widens before comparison. Successful validation borrows or
owns the exact original bytes without reordering, color conversion, implicit
alpha, or a second premultiplication. Failure publishes no proof and performs
no allocation or byte clone.

P4 covers every equality and one-over limit, simultaneous width/height faults,
cumulative rather than per-image pixels, stride-before-length priority, empty
and extra bytes, every offending color channel, row and pixel order, valid
transparent and opaque pixels, and unchanged cumulative state on failure. A
valid nontrivial premultiplied pixel such as `[64, 32, 0, 128]` remains the
exact same byte sequence in the proof; allocation identity is not observable.

## P5 image-paint preparation and local bounds

The later global item dispatcher scans the complete paint table once in item
order, then hit and semantic tables, dispatching the applicable helper at each
ordinal rather than batching by item kind. A round-stroke coverage item uses
geometry K1; an image paint uses P5; inapplicable helpers are skipped.

For an image paint, the caller supplies a P4 image proof after owner order,
item ordinal, and image reference are trusted. Before terminal clip ancestry
is checked, the item completes this raw-field sequence:

1. source width zero, then source height zero;
2. source x near, x far, y near, then y far;
3. destination scalar domain in x, y, width, height order;
4. negative destination width, then height;
5. zero destination width, then height.

After a nonempty source, near x fails when `source.x >= image.width`; widened
far x fails only when `source.x + source.width > image.width`, so equality
passes. Y is identical. Empty source returns `InvalidImage(EmptySource)` at
`SourceWidth` or `SourceHeight`; near failures use `SourceX` or `SourceY`, and
far failures use `SourceWidth` or `SourceHeight`. Near and far failures return
`InvalidImage(SourceOutOfBounds)`. Every P5 field location is
`Paint { index, field }`.

Destination scalar failures use `ScalarOutOfDomain` at the matching paint
field. Signed failures use `NegativeDestinationExtent(Width|Height)` and zero
uses `EmptyDestination`, each at its extent field. This pre-clip stage binds
the exact P4 image used for source validation; it must be impossible to pair
the prepared fields with a different image.

The caller next retains this pre-clip proof while it completes the entire
global paint, hit, and semantic item phase, including every intrinsic field,
reference, and terminal clip check in table order. Only after that whole phase
succeeds may the shared local-bounds phase run: it scans every base shape by
key, then the complete paint table once in item order, then the hit table. A
round-stroke coverage item dispatches to geometry K3, an image paint dispatches
to P5, and an inapplicable fill check is skipped. P5 adds destination x plus
width, then y plus height. A far-edge failure uses
`LocalBoundsOutOfDomain(X|Y)` at `DestinationWidth` or `DestinationHeight`.
Success retains the validated source, positive destination, byte opacity,
bound image proof, and a closed conservative local AABB from destination near
to far. Coverage remains half-open; the closed far edge is bounds data only.

No error publishes a final proof or partial bounds. Any later paint, hit, or
semantic item-phase failure wins over an earlier prepared image paint's
destination far-edge failure; this includes a terminal clip failure on the
same item.

The local P5 RED tests the full pre-clip relational order with multifault
suffixes, source equality, `u32` widened far edges, all four scalar fields,
signed and zero extents, canonical Fixed16 far edges, and width-before-height
priority. It also tests the far-edge operation after a trusted whole-item-phase
gate. The later aggregate-validation RED, not this local cut, must prove that a
later paint, hit, semantic, or same-item terminal-clip failure wins over an
earlier prepared image far-edge failure. For raw item preflight and again for
shared bounds, mixed-kind controls cover both `ImagePaint[0]` versus
`RoundStroke[1]` and `RoundStroke[0]` versus `ImagePaint[1]`; the bounds corpus
also fixes base-shape-before-paint and paint-before-hit priority. Opacity zero
and 255 remain valid without an impossible opacity error.

## P6 total nearest image sampling

Sampling accepts only one P5 proof, including its bound P4 image, and one
canonical owner-local point. It returns no color outside the half-open
destination. For a covered point:

```text
sx = source.x + floor(
    (point.x - destination.x) * source.width / destination.width
)
sy = source.y + floor(
    (point.y - destination.y) * source.height / destination.height
)
```

The differences and products use widened raw Fixed16 arithmetic. The positive
destination extent is the denominator; division is mathematical floor before
adding the source near coordinate. The proof guarantees the resulting source
coordinates and RGBA byte ordinal are in bounds.

P6 reads bytes in `R, G, B, A` order, applies P1 opacity exactly once, and
returns the premultiplied result. It allocates nothing and is infallible over
its proofs. Tests cover every destination edge, crop corners, upscaling,
downscaling, noninteger Fixed16 points, floor versus nearest, nonzero source
origins, transparent pixels, opacity endpoints, and the final included sample
immediately before each far edge. A nontrivial intermediate opacity over four
distinct premultiplied channels has literal expected bytes so applying opacity
twice or changing channel order fails.

## Staging and nonclaims

P1 through P6 may use private validated values but add no `prototype` export.
They add no normal dependency and make no candidate call. Global validation
later maps P2 and P4 limit failures to
`SpatialResolveErrorKindV2::LimitExceeded`; it maps semantic failures through
`SpatialResolveErrorKindV2::Content(SpatialContentErrorKindV2)` and preserves
the trusted locations and priority stated here.

The nested mappings are exact. P2 scalar failures use `Content(ScalarOutOfDomain)`;
its other failures use `Content(InvalidGradient(...))`. P4 zero extent,
stride, length, and pixel failures use `Content(InvalidImage(...))`. P5 source,
negative-destination, and empty-destination failures use
`Content(InvalidImage(...))`; destination scalar failures use
`Content(ScalarOutOfDomain)`, and far-edge failures use
`Content(LocalBoundsOutOfDomain(...))`. The `...` is the exact printed inner
kind named in P2, P4, or P5, never a new wrapper or reclassification.

World transforms, clip-chain intersections, painter order, reverse hit
selection, semantic projection, 16-sample CPU rasterization, decoder probes,
and native presentation remain later independent RED/GREEN cuts. No local
paint-kernel result makes any interface or subtree scene-first.
