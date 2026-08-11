# WU-0013 hybrid spatial reference contract

Parent plan: [hybrid-spatial-composition.md](hybrid-spatial-composition.md)
Content contract: [hybrid-spatial-content-reference-v2.md](hybrid-spatial-content-reference-v2.md)
Diagnostics: [hybrid-spatial-diagnostics-v2.md](hybrid-spatial-diagnostics-v2.md)
Fields: [hybrid-spatial-fields-v2.md](hybrid-spatial-fields-v2.md)
Format: spatial contract version 2

## Scope and authority

Layout and free placement are closed peer choices on spatial relationships.
Neither is the universal root model. The same single-root spatial tree admits
an all-layout interface, an all-free interface, or either nesting direction.

Runtime owns logical identity, state, keyed structure, and lifetime. Key zero is
a synthetic viewport sentinel without logical identity. Authored nodes map to dense
pass-local `SpatialNodeKeyV2` values. Spatial keys, island ordinals, layout keys,
dependency vertices, and candidate IDs never persist or cross the boundary.

The spatial resolver owns placement, island orchestration, transforms, clips,
and geometry. Paint, hit, and semantic tables are independent consumers of
that geometry. A presentation frame is an immutable output. Pixels, paint
bounds, hit shapes, resource readiness, renderer state, and candidate output
never feed layout or free placement.

## Scalar and coordinate policy

`SpatialScalarV2` is a signed `i64` fixed-point value with 16 fractional bits.
Its real value is `raw / 65536`. The inclusive canonical raw domain is:

```text
minimum = -140737488289792
maximum =  140737488289792
```

This is exactly `+/-2147483647` logical units. Every accepted nonnegative
layout `i32` converts exactly. `i32::MIN` is outside the spatial domain so
negation remains representable. Extents are nonnegative scalars in the same
positive domain. Serialization writes the signed decimal raw value and an explicit
format scale of `65536`; it never writes a float.

Addition, subtraction, multiplication, division, affine composition, affine
application, inverse-point mapping, and bounds use `i128` intermediates. There is no
saturation or wrapping. A result outside the canonical raw domain is a typed
arithmetic failure. A ratio is rounded once to the nearest raw tick; an exact
half rounds away from zero. Dot products sum their unrounded products and round
only the complete component.

For `round_ratio(numerator, positive_denominator)`, the reference divides the
absolute numerator into quotient and remainder, increments the quotient when
`2 * remainder >= denominator`, then restores the sign. Directed floor and ceil
use Euclidean quotient and remainder and never pass through nearest rounding.

The contract distinguishes node-local, island-local, parent-local, untransformed
scene-placement, and scene-logical coordinates. Device-physical coordinates
exist only in a presenter adapter. Equal logical input produces equal spatial
bytes regardless of exterior display scale.

Rectangles use an origin plus nonnegative extent. Rect containment is half-open
on right and bottom. `SpatialAabbV2` instead stores closed conservative
`min_x,min_y,max_x,max_y` acceleration edges plus an explicit empty state. It
uses the closure of a local shape, including a rectangle's excluded far edges,
so reflection cannot lose an included near boundary. A transformed AABB applies
the already composed fixed matrix to every required local extremum as an exact
widened numerator. It selects rational minima and maxima first, then floors each
minimum and ceils each maximum directly to raw ticks. Nearest-rounded points are
not used to derive a bound. The AABB never replaces exact shape geometry.

## Spatial tree and placement

Input nodes are dense `u32` keys in spatial preorder; node zero is the only root.
It has no parent, uses `Root`, and counts toward node and depth limits; root-only
depth is one. It owns container axis, padding, and gap but no authored identity,
local transform, shape, clip, paint, hit, or semantic item. Every
later node declares an existing earlier parent and uses `Layout` or `Free`. The
active-ancestor-stack rule from layout validates preorder; a completed subtree
cannot be reopened.

Every node owns container policy for its children: `Row` or `Column`, integer
padding, and integer gap. Placement is separate:

- `Root` obtains `(0,0)` and the present logical viewport extent;
- `Layout` supplies integer minimum, preferred, and maximum width and height;
- `Free` supplies an integer width and height, self anchor, target, target
  anchor, and fixed-point offset.

Container padding and gap are preflighted through the shared layout validator
for every record, including an all-free leaf, so adding a later layout child
cannot expose previously unvalidated state.

All root, layout, and free base-box extents are nonnegative `i32` logical units.
They cross the Layout V1 bridge exactly as raw multiples of 65536; no
fixed-to-integer quantization occurs. Free offsets, anchors, shapes, and affine
values may use fractional fixed-point coordinates.

An anchor has independent horizontal `Start | Center | End` and vertical
`Start | Center | End` components. Center divides the integer extent exactly in
fixed point, so an odd extent produces one half unit. A target is `Viewport`,
`Parent`, or `Node(key)`. `Node` accepts only authored keys greater than zero;
the viewport target is the nonambiguous name for the sentinel box. Any non-self
authored node in the same computation may be a target, including an acyclic
forward reference.

Anchors read base border boxes in one global untransformed scene-placement
plane. For a free node:

```text
global origin = target anchor + offset - self anchor
local origin = global origin - parent global origin
```

The parent box is therefore an implicit dependency even when another target is
named. Parent and viewport anchors do not depend on transforms. A transform on
an ancestor still affects the final world geometry of its subtree, but never
changes an authored anchor or layout result.

## Layout islands

A layout island is one maximal nonempty set of `Layout` relationships. Its real
members are exactly the child endpoints of those relationships. Its host is the
already resolved sentinel or free-placed node from which the first relationship
leaves. The host appears only as a synthetic fixed root in the layout call; it
is not a second spatial node or real island output. Real members remain in
spatial preorder and `Free` subtrees are omitted.

The host layout style combines its resolved fixed extent with its node-owned
axis, padding, and gap. A layout participant combines the dimensions on its
incoming relationship with its own axis, padding, and gap. A `Free`
relationship is omitted from that island and may host a later island.

Each call remaps the synthetic host to layout key zero and real members to dense
keys starting at one. The layout engine receives the host extent as viewport.
Its synthetic root output must remain `(0,0,host width,host height)` or the
result is rejected by the closed bridge mismatch kind; that record is discarded.
Each real result converts from integer island-local coordinates to fixed point.
Members are then processed in spatial preorder:

```text
member global origin = host global origin + island result origin
member global far = member global origin + member extent
local placement origin = member global origin - spatial parent global origin
```

These use `IslandTranslationX,Y`, `BaseFarX,Y`, then `ParentDeltaX,Y` in that
order. The box becomes available only after all three stages pass.

A real member whose spatial parent is the host names layout parent zero; a nested
member names its already mapped parent in the same island. No other parent enters.
Per-island limits are `LayoutLimitsV1::new(count, count, count)`, including the
synthetic root, so the WU-0011 profile does not constrain runtime.
`LayoutInputRecordsTotal` sums only nonempty-island records, including synthetic
roots; singleton container proofs never reach an engine and do not count.

Layout 0.2 adds a validation-only preparation seam that owns one island input
and exposes no candidate call. Spatial builds and prepares every island before
materializing the dependency order. A failure in any container or island input
therefore leaves layout engine call count zero. Execution later passes each
opaque prepared island through the existing engine and output validator exactly
once; it does not repeat or bypass core layout validation.

Preparation items are ordered by the lowest real spatial key they cover. A
singleton sentinel uses ordinal zero, a singleton free node uses its key, and
an island uses its lowest member key. These ordinals are disjoint. Items scan
ascending, then use Layout V1's internal validation order.

A sentinel or free node that hosts no island is preflighted as a synthetic
single-root layout input and then discarded without an engine call. Thus every
container style is validated by the shared layout implementation exactly once,
including an all-free tree, without duplicating padding or gap rules in spatial.

Each nonempty island invokes `LayoutEngineV1` exactly once. An all-free tree
invokes it zero times. Invalid input, missing references, or a dependency cycle
invokes it zero times. Runtime derives limits from its preflighted draft.

## Dependency graph and evaluation

The closed graph has one unit for each non-root free placement and one for
each nonempty layout island. Available sources are the viewport and root box.
Edges point producer to consumer:

- the producer of a host box points to its island;
- parent and named-target box producers point to a free placement;
- a free placement points to an island that it hosts;
- an island points to free placements targeting one of its participants.

Only unit-to-unit edges count toward the dependency limit; viewport and sentinel
availability are sources, not vertices or edges. Duplicate edges are collapsed
before counting. Self targets and missing keys fail before graph construction;
self targets never enter cycle detection. Every unit produces a disjoint set of
authored keys: a free unit produces its node and
an island produces its real layout endpoints. Its stable ordinal is the lowest
key it produces.

The resolver first constructs and validates the entire graph and materializes
the complete stable Kahn order without executing a unit. If units remain after
the ready queue empties, it runs deterministic strongly connected components on
the unresolved subgraph, visiting units and adjacency in ascending ordinal. A
cyclic component has more than one unit or a self edge. The error names the
smallest ordinal in the cyclic component whose minimum ordinal is globally
smallest; a merely blocked dependent is never blamed. Layout call count stays
zero. Only a complete order may execute. Ready units are chosen by lowest stable
ordinal; ordinals are unique because produced key sets are disjoint.

Free evaluation orders target plus offset x then y, origin minus self anchor x
then y, base far x then y, and local origin minus parent x then y. Target and
self anchors are exact selections inside validated boxes. Island output is
validated before any box becomes available. All buffers remain private until
the complete graph resolves.

Input, content, island preparation, and dry-cycle failures make zero layout
calls. A free-unit arithmetic failure retains calls from earlier islands in the
materialized order. An island engine or output failure includes that island's
one call and makes no later call. World-transform or final-output failure occurs
after every scheduled island call. None of these cases publishes partial state.

## Affine transforms

`Affine2V2` stores six `SpatialScalarV2` coefficients using column vectors:

```text
[ a c tx ]
[ b d ty ]
[ 0 0  1 ]
```

The product `compose(left, right)` applies `right` first and rounds each complete
result component once. Fixed composition is not associative. Every non-sentinel
node stores a local affine and transform origin. Its final transform is:

```text
about = compose(translate(transform_origin),
                compose(local_affine, translate(-transform_origin)))
placed = compose(translate(local_placement_origin), about)
world_from_local = compose(parent_world, placed)
```

With scale `S=65536`, composition computes:

```text
a  = round_ratio(la*ra + lc*rb, S)
b  = round_ratio(lb*ra + ld*rb, S)
c  = round_ratio(la*rc + lc*rd, S)
d  = round_ratio(lb*rc + ld*rd, S)
tx = round_ratio(la*rtx + lc*rty + ltx*S, S)
ty = round_ratio(lb*rtx + ld*rty + lty*S, S)
```

Point application uses the same translation formula with `x,y` in place of
`rtx,rty`. No product is rounded before the component sum.

The sentinel transform is identity. Identity, translation, scale, and
quarter-turn rotation have canonical
constructors. A complete affine matrix supports other precomputed rotations;
version 2 does not accept an angle or define trigonometry. This still proves
rotation without making host math libraries or floating point authoritative.

The exact determinant is `a*d - c*b` in widened raw arithmetic. A zero
determinant is `SingularTransform`; there is no epsilon. Every node transform
must be invertible even if the node currently has no hit item. Version 2 does
not quantize or publish an inverse matrix because a rounded reciprocal may be
singular or fail to compose to exact identity.

For a scene point `(px,py)` and raw determinant `det`, inverse application uses
the forward matrix directly:

```text
dx = px - tx
dy = py - ty
local_x = round_ratio((d*dx - c*dy)*S, det)
local_y = round_ratio((a*dy - b*dx)*S, det)
```

If `det` is negative, numerator and denominator signs are normalized before
the positive-denominator ratio rule. An exact result outside the canonical
local domain is conclusively outside every valid local shape and returns no
coverage rather than an arithmetic error.

Raw matrix fields and singularity are validated globally before graph
evaluation or layout calls. World composition and placement translation occur
after base boxes resolve, in spatial preorder. Every `about`, `placed`, and
world result is range checked; a rounded zero determinant is
`ComposedTransformSingular`. These failures may occur after earlier island
calls, but no partial geometry escapes.

## Geometry and projection order

Successful output has one geometry record per input node in key order. `BaseX`
and `BaseY` are its Fixed16 global origin in the untransformed scene-placement
plane; `BaseWidth` and `BaseHeight` are nonnegative integer local extents embedded
exactly in Fixed16. `world_from_local` and every output AABB use scene-logical
space. No record contains a logical `NodeId`, generation, pixels, or candidate data.

Resolved clip records retain their exact local shape, owner transform, parent
clip, and conservative world AABB. The primitive is never replaced by its
intersection. Paint, hit, and semantic records each name their own terminal
clip, so sharing is explicit rather than assumed.

Spatial preorder defines `stack_ordinal`. Paint, hit, and semantic projection
tables are ordered by `(stack_ordinal,item_ordinal)`, with dense per-node item
ordinals. Resource tables retain their global-key order from the content
contract. Paint is consumed ascending. Hit is queried descending and returns
the first exact match. Semantics remains independently ordered. Version 2 has
no authored z-index or stacking context.

## Registered conformance limits

These inclusive limits are experiment evidence, not product capacities:

```text
nodes=256                     depth=32
children-per-node=64          islands=64
layout-input-records-per-island=128
layout-input-records-total=192
dependency-vertices=192       dependency-edges=256
shapes=1024                   brushes=256
clip-records=512              clip-depth=32
paint-items=1024              hit-items=512
paint-items-per-node=64       hit-items-per-node=64
semantic-items=256            paths=256
path-verbs-per-path=256       path-verbs-total=4096
path-subpaths-total=1024      flattened-segments=65536
flattened-segments-per-path=4096
polygon-points-per-shape=256  polygon-points-total=4096
gradient-stops-per-brush=32   gradient-stops-total=2048
images=64                     image-edge=4096
image-pixels-total=4194304
encoded-image-bytes=16777216
reference-raster-pixels=4194304
artifact-records=4096         artifact-line-bytes=1024
artifact-bytes=1048576
```

Direct table-count limits are checked before reading records. Topology-derived
limits are checked only after topology exists. Equality passes; one over fails
without retaining the next record. The corpus independently exceeds the old
layout profile of 32 nodes, depth 8, and 16 children to prove those values do
not become runtime capacities.

The evidence artifact does not duplicate every row of a maximum-size synthetic
input. Small semantic cases are recorded field by field. Exact and one-over
capacity fixtures record the typed limit, observed count, canonical input byte
count, and digest, which the test recomputes from an independently constructed
fixture. Artifact limits therefore bound evidence encoding rather than spatial
input cardinality.

## Closed validation priority

Validation returns the first failure in these phases:

1. the diagnostics contract's `DIRECT_ALL` limits;
2. root sentinel, dense key, parent existence, forward parent, and preorder;
3. depth then children-per-node;
4. sentinel restrictions, placement discriminant, integer free dimensions,
   and free-offset scalar fields;
5. island derivation, island count, layout records per island and in total,
   then the shared layout preflight for every container and island;
6. local transform domain and determinant in node and field order;
7. every content table phase in the exact companion-contract order;
8. anchor target scope, dependency vertices and edges, then stable cycle check;
9. graph evaluation in stable vertex order, including layout and arithmetic;
10. world transforms in node order, clips in clip order, then bounds;
11. output count, keys, scalar domains, world determinants, closed conservative
    AABBs, clip chains, and projection table order.

Artifact limits are enforced separately as records, record grammar, line
bytes, then total bytes. A simultaneous crossing returns records before line
bytes and line bytes before artifact bytes.

Diagnostics expose only the closed kinds, fields, stages, and trusted ordinal
locations fixed by the diagnostics contract, plus `observed/maximum` for
limits. `Display` and `Debug` never include authored values, malformed keys,
image bytes, logical identities, pointers, or candidate errors.

## Atomicity, corpus, and evidence

Resolution builds owned temporary geometry and projection tables. Runtime
publishes one immutable allocation only after every check passes. A failure
preserves the previous allocation, generation, properties, projections,
operation attribution, and presented frame.

The registered corpus proves all-layout, all-free, both nesting directions,
mixed siblings, transparent wrappers, independent geometry, forward anchors,
cycles, transforms, clips, reverse hit, keyed mutations, resize, failures, and
rollback. The content contract fixes its rich cases.

Manual raw input, `.fen`, and `ui!` produce independently lowered but identical
normalized semantics. A literal oracle, the owned reference resolver, and each
candidate lane are built separately and compared field by field. Per-field
faults must change only their typed slice and be detected. Two fresh runs must
produce the same bounded ASCII/LF artifact on Linux and Windows.

EXP-0008 remains open. WU-0011 records `layout-v1`; WU-0013 records
`spatial-v2` and candidate tuples `lane + crate version + features + target +
corpus`. `Pass` requires every required case; `Adapt` retains them through one
closed rule; `Stop` rejects. Reduced output never passes or closes EXP-0008.

The contract consumes one present logical viewport. It has no density, physical
pixel, safe-area, lifecycle, surface epoch, mobile, or multi-scene vocabulary.
Zero extent is geometry, not absence. WU-0012 remains deferred.
