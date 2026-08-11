# WU-0013 private geometry kernel contract

Raw values: [hybrid-spatial-geometry-api-v2.md](hybrid-spatial-geometry-api-v2.md)
Semantics:
[hybrid-spatial-content-reference-v2.md](hybrid-spatial-content-reference-v2.md)
Diagnostics: [hybrid-spatial-diagnostics-v2.md](hybrid-spatial-diagnostics-v2.md)
Fields: [hybrid-spatial-fields-v2.md](hybrid-spatial-fields-v2.md)
Format: private spatial geometry kernel version 2

## Boundary and slices

This contract fixes the crate-private reference behavior needed by these
ordered geometry slices:

1. K1 validates local path, shape, and stroke geometry;
2. K2 flattens validated paths into bounded subpaths and segments;
3. K3 derives conservative local bounds;
4. K4 evaluates exact local fill coverage;
5. K5 evaluates exact local round-stroke coverage.

It adds no prototype reexport, public validated view, public error, dependency,
paint or resource model, output type, or candidate seam. Clip-chain traversal,
world transforms, and resolver orchestration remain outside this kernel. The
kernel may return private failure fragments, but their eventual public kind and
location are exactly those already registered by the diagnostics and field
contracts.

Rust identifiers, module division, concrete private structs, and crate-private
function signatures are not versioned behavior. Each behavioral RED introduces
only the smallest private seam it needs. The data and ordering invariants below
are versioned so different internal representations remain observationally
equivalent.

## K1 preflight and first failure

Table counts, dense keys, trusted ranges, and every `PathVerbsPerPath` check
complete before this path pass. That limit uses
`Path { index, field: VerbLength }`. For each path in key order, K1 performs
these complete passes:

1. scan every verb in local ordinal order and validate every applicable scalar
   in `SpatialPathVerbFieldV2::ALL` order;
2. scan the complete verb range for grammar;
3. only after valid grammar, add its subpaths to `PathSubpathsTotal` in `MoveTo`
   order.

Therefore a grammar failure later in one path wins over a subpath-total crossing
in an otherwise valid prefix of that same path. After grammar succeeds, the
first `MoveTo` whose accepted subpath makes the accumulated total exceed its
inclusive maximum reports `LimitExceeded(PathSubpathsTotal)` at that verb.

Path scalar fields apply as follows: move and line use `ToX, ToY`; quadratic
uses `ControlX, ControlY, ToX, ToY`; cubic uses `Control1X, Control1Y,
Control2X, Control2Y, ToX, ToY`; close has none. A scalar failure is
`ScalarOutOfDomain` at
`PathVerb { path, verb, field: <applicable field> }`.

Path grammar attribution is exact:

- `Empty` uses `Path { index, field: VerbLength }`;
- `FirstNotMove` uses the first verb's `PathVerb { field: Kind }`;
- `EmptySubpath` uses the offending later `MoveTo` and field `Kind`;
- `DrawingWithoutSubpath` uses the offending drawing verb and field `Kind`;
- `CloseWithoutSegment` uses the offending `Close` and field `Kind`;
- `TrailingMove` uses the final `MoveTo` and field `Kind`.

In `PathVerb`, `verb` is the zero-based ordinal inside the owning path, not the
global payload-table index. `PathSubpathsTotal`, both flattened-segment limits,
and `NonFlatAtMaximumDepth` use the source verb's `PathVerb { field: Kind }`.

Shapes run by key, completing every applicable check before the next shape.
After trusted ranges and references, rect and circle scalar-domain preflight
scans the applicable `SpatialShapeFieldV2::ALL` fields before signed semantics.
A polygon scans points by zero-based ordinal inside the shape and then `X, Y`;
next it checks `PolygonPointsPerShape`, `PolygonTooShort`,
`PolygonRepeatedFirst`, and `PolygonAdjacentEqual`, in that order. Scalar
failures use `Shape { index, field }` or
`PolygonPoint { shape, point, field }`. Semantic locations are:

- `NegativeExtent`: `RectWidth` before `RectHeight`;
- `NegativeRadius`: `CircleRadius`;
- `PolygonTooShort`: `PolygonPointLength`;
- `PolygonRepeatedFirst`: field `X` of the final polygon point;
- `PolygonAdjacentEqual`: field `X` of the later point in the first equal pair.

The first three use `Shape { index, field }`; the last two use
`PolygonPoint { shape, point, field }`, where `point` is local to that shape.
`PolygonPointsPerShape` also uses
`Shape { index, field: PolygonPointLength }`.

For paint and hit round stroke, `StrokeWidth` scalar-domain validation precedes
`NegativeWidth`, which precedes `ZeroWidth`. All three use the owning
`Paint { index, field: StrokeWidth }` or `Hit { index, field: StrokeWidth }`.
No grammar rule, signed comparison, flattening arithmetic, bound arithmetic, or
coverage operation may inspect a record before its applicable scalar-domain
preflight succeeds. K4 and K5 receive only canonical local query points; callers
do not clamp an out-of-domain inverse result into coverage.

## K2 flattened representation and limits

A successful validated path retains its canonical raw verb slice and subpath
count. A successful flattened path owns one point table plus ordered subpath
descriptors. Each descriptor carries `point_start`, `point_length`, and whether
the authored subpath ended with explicit `Close`. K3 derives bounds separately
from the validated raw geometry; K1 and K2 do not attach them.

The first point of each descriptor is its `MoveTo`. Every line or flat curve
leaf appends its endpoint. Explicit `Close` appends the first point, even when
that creates a zero-length segment. An implicit fill closure appends nothing
and does not count. Retained segments are adjacent point pairs within one
descriptor, so the total segment count is the sum of `point_length - 1`.

Every emitted line, curve leaf, or explicit close tentatively increments the
path count first. K2 checks `FlattenedSegmentsPerPath`, then tentatively
increments and checks `FlattenedSegmentsTotal`. The first crossing uses the
authored source verb. A recursive curve leaf retains the source ordinal of its
quadratic or cubic verb. Failure returns no partial flattened path and does not
commit a partial total. Depth, flatness, midpoint, and source-order behavior are
exactly those in the semantic contract; depth 16 and tolerance 256 are format
constants, not new public limits.

## K3 local bounds

The analytic rect and circle boundary boxes remain closed when one or both axes
degenerate. Polygon bounds include every point. Path bounds use the retained
raw move, endpoint, and control-point extrema rather than only flattened
endpoints. K3 derives this boundary box for every shape before choosing a
coverage-specific box; an empty fill does not bypass a base-bound failure.

Fill and clip bounds for a rectangle with either zero extent, and for a circle
with zero radius, are the canonical empty `SpatialAabbV2`. Round stroke instead
starts from the degenerate boundary box: one zero rectangle axis is a line, two
are a point, and a zero-radius circle is a point.

For a positive canonical stroke width with raw value `w`, the conservative
outward expansion in raw ticks is exactly:

```text
e = ceil(w / 2) = (w + 1) / 2
```

The addition and division are widened. K3 subtracts `e` from both minima and
adds `e` to both maxima, then requires every result to remain in the canonical
scalar domain. It completes x before y. A rect base-bound failure uses
`Shape { index, field: RectWidth }` or
`Shape { index, field: RectHeight }`; a circle uses
`Shape { index, field: CircleRadius }`. Stroke expansion uses the owning
paint or hit `StrokeWidth`. Failure is `LocalBoundsOutOfDomain(X)` before
`LocalBoundsOutOfDomain(Y)`.

## K4 and K5 coverage

K4 uses the exact rect, circle, polygon, and flattened-path fill rules from the
semantic contract. The canonical empty bounds above reject degenerate rect and
zero-radius circle fill and clip coverage. Open path subpaths close implicitly
for fill without mutating K2 storage or its segment count. A nonempty AABB may
reject a query but never accepts it without the exact shape test.

K5 uses validated positive width and the exact boundary of each shape. It keeps
the line and point cases above, every zero-length retained path segment, round
ends, and round joins. The rounded-up `e` is only the conservative AABB
expansion; exact coverage continues to use the widened inequalities registered
by the semantic contract. K4 and K5 allocate nothing and call no candidate.
