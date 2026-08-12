# WU-0013 private aggregate input validation kernel

Core semantics: [hybrid-spatial-reference-v2.md](hybrid-spatial-reference-v2.md)
Content semantics:
[hybrid-spatial-content-reference-v2.md](hybrid-spatial-content-reference-v2.md)
Diagnostics: [hybrid-spatial-diagnostics-v2.md](hybrid-spatial-diagnostics-v2.md)
Fields: [hybrid-spatial-fields-v2.md](hybrid-spatial-fields-v2.md)
Aggregate API:
[hybrid-spatial-validation-api-v2.md](hybrid-spatial-validation-api-v2.md)
Geometry kernel:
[hybrid-spatial-geometry-kernel-v2.md](hybrid-spatial-geometry-kernel-v2.md)
Paint kernel: [hybrid-spatial-paint-kernel-v2.md](hybrid-spatial-paint-kernel-v2.md)
Format: private spatial input validation kernel version 2

## Boundary and terminology

This contract freezes aggregate validation phases 1 through 7, ending after
local content bounds and before anchor-target, dependency-graph, layout-engine,
world-transform, output, hit-query, raster, presenter, or candidate work. It
adds no `prototype` export, normal dependency, public validated view, resolver
signature, output record, or candidate seam. Private Rust names, module
division, proof layout, and helper signatures are not versioned behavior.

A **global pass** completes its stated check over the entire relevant table or
derived set before the next pass starts. A **record-major pass** visits trusted
records by ascending input ordinal and completes the listed applicable checks
for one record before advancing. Table order always precedes record order, and
field order is the order in the field contract. A closed enum discriminant or
byte field with no invalid bit pattern is visited conceptually for mutation
evidence but cannot fail validation.

Every failure below constructs one `SpatialResolveErrorV2`. Only a
`LimitExceeded` kind stores `Some(observed)` and `Some(maximum)`; every other
kind stores `None, None`. Limit evidence is `u128`, equality succeeds, and the
first candidate strictly above an inclusive maximum fails. `Display`, `Debug`,
and `Error::source` follow the redacted aggregate API contract for errors
created in every phase here.

## P5 ownership precondition

Before the aggregate behavior RED, the private P5 image-paint representation is
refactored so a retained paint proof does not borrow the address of another
movable proof. It must remain tied to the exact P4-validated raw image and its
input lifetime even if a temporary image-proof collection moves or is dropped.
It may retain an equivalent validation token or the validated image reference;
it may not re-look up an unvalidated image later or clone image bytes.

This is an ownership correction only. P4/P5/P6 behavior, error priority,
locations, opacity, image identity, and allocation observability do not change.
No concrete private type name, field layout, lifetime spelling, or function
signature is fixed by this precondition.

## Trusted ordinal capacity and direct counts

Every globally indexed raw table is later addressed by a trusted `u32` record
ordinal. Therefore its mathematical row capacity is exactly:

```text
U32_ROW_CAPACITY = u32::MAX + 1 = 4294967296
effective_direct_maximum(kind) =
  min(caller_limit(kind) widened to u128, U32_ROW_CAPACITY)
```

The ceiling applies exactly to `Nodes, Shapes, Brushes, Clips, PaintItems,
HitItems, SemanticItems, Paths, Images`. It permits a final record at ordinal
`u32::MAX`; one more globally indexed record cannot receive a trusted
location. On a platform whose `usize` cannot represent the mathematical
ceiling, the caller's widened limit and possible slice lengths already provide
the tighter bound. A caller limit wider than `u32` does not narrow a small
valid input.

`PathVerbsTotal`, `PolygonPointsTotal`, and `GradientStopsTotal` instead use
their widened caller limits directly. Their diagnostics use an owner plus a
range-local `u32` ordinal, so multiple canonical `u32 start + u32 length`
ranges may validly partition a global payload beyond `U32_ROW_CAPACITY`.
Uncovered or structurally unaddressable payload is rejected later by the
range-partition rule, not reclassified as a representability limit.

Phase 1 is one global pass in exact `DIRECT_ALL` order:

```text
Nodes              topology.nodes
Shapes             geometry.shapes
Brushes            resources.brushes
Clips               geometry.clips
PaintItems          items.paint_items
HitItems            items.hit_items
SemanticItems       items.semantic_items
Paths               geometry.paths
PathVerbsTotal      geometry.path_verbs
PolygonPointsTotal  geometry.polygon_points
GradientStopsTotal  resources.gradient_stops
Images              resources.images
```

Each check compares the widened slice length with its effective maximum: the
formula above for the nine globally indexed tables and the widened caller
limit for the three payload tables. A failure is `LimitExceeded(kind)` at
`Input`, with the exact length as `observed` and that effective maximum as
`maximum`. No record, key, range, image byte, or other payload is read until
every direct count passes. The ceiling is not a new limit kind or input error.

## Phase 2: root and topology

After counts pass, an empty node table returns `Input(EmptyInput)` at `Input`.
The root then checks `Key`, `Parent`, and `Placement` in that order. A nonzero
key, present parent, or non-`Root` placement returns `InvalidRootKey`,
`RootHasParent`, or `InvalidRootPlacement` at
`NodeField { index: 0, field: Key|Parent|Placement }`.

Later nodes use one record-major scan. At node ordinal `i`, checks are:

1. key equals `i`, else `NonDenseNodeKey` at `Key`;
2. a parent is present, indexes a supplied record, and equals that record's
   key, else `MissingSpatialParent` at `Parent`;
3. the parent ordinal is below `i`, else `ForwardSpatialParent` at `Parent`;
4. the parent remains on the active preorder ancestor stack, else
   `InvalidPreorder` at `Parent`.

All four are `Input(...)` at `NodeField { index: i, field }`. Thus a failure on
an earlier record wins over a different topology kind on a later record.
Successful scanning privately retains exact depth and child-count facts; it
does not enforce their limits yet. Parent lookup never trusts a malformed key.

## Phase 3: derived topology limits

Two global passes follow. Depth scans nodes in ordinal order, with the root at
depth one. `Depth` failure is at `Node { index }` for the first node whose
derived depth crosses the caller maximum. Children-per-node then scans owners
in node order. `ChildrenPerNode` failure is at `Node { index }` for the first
owner whose complete direct-child count crosses its caller maximum. Each stores
the derived count and caller maximum as widened evidence. No additional
representability ceiling applies because phase 1 already made every node
ordinal trusted.

## Phase 4: remaining placement input

Phase 4 performs these passes in order:

1. scan every non-root node for `Root` placement; the first returns
   `Input(RootPlacementOnNonRoot)` at `NodeField { field: Placement }`;
2. check viewport width then height; a negative value returns
   `Input(NegativeViewport(extent))` at `Viewport { extent }`;
3. scan nodes record-major, skipping non-`Free` placements; check free width,
   height, offset x, then offset y.

A negative free extent returns `Input(NegativeFreeExtent(Width|Height))` at
`FreeWidth|FreeHeight`. A noncanonical offset returns
`Input(FreeOffsetOutOfDomain(X|Y))` at `FreeOffsetX|FreeOffsetY`. Raw values are
never stored in the error. Layout dimensions and every container are deferred
to phase 5; local transforms are deferred to phase 6.

## Phase 5: islands, capacities, and layout preflight

The kernel derives every maximal nonempty layout island without invoking an
engine. Islands are ordered by their lowest real spatial key and receive dense
`u32` indices in that order. Each island's layout-record count is one synthetic
host plus its real members. Only these nonempty-island records contribute to
the total.

The three `ISLAND_ALL` checks are separate global passes:

1. `Islands` compares the complete island count and uses `Input`;
2. `LayoutInputRecordsPerIsland` scans island index order and uses the first
   `Island { index }` whose complete record count crosses the maximum;
3. `LayoutInputRecordsTotal` compares the complete sum and uses `Input`.

The observed value is respectively the complete island count, complete island
record count, or complete total. Maximum is the widened caller limit.
Per-island checks for all islands finish before the total check, so any
per-island crossing wins over the aggregate total. Phase-1 node capacity makes
island indices representable; layout-record arithmetic uses `u128` until it
passes the caller's `usize` limit.

The kernel next constructs validation-only preparation items. A root or free
node that hosts a nonempty island is that island's synthetic host and contributes
no singleton. A root or free node without an island contributes exactly one
singleton. Every layout node is a real member of exactly one island. Thus every
node-owned container reaches exactly one preparation item.

An item's stable key is zero for the sentinel singleton, the node key for a
free singleton, or the lowest real-member key for an island. These keys are
injective and disjoint. Items sort by stable key; `Island { index }` remains the
dense zero-based index after sorting islands and is not that key. Each complete
item is passed once through Layout V1 preparation before the next item starts.
The layout engine call count remains zero.

Prepared-input failures map exactly as follows:

```text
negative constraint   Input(InvalidLayoutDimensions(NegativeConstraint {
                        extent, field }))
                      NodeField at Layout{Width|Height}{Minimum|Preferred|Maximum}
inverted constraint   Input(InvalidLayoutDimensions(InvertedConstraint(extent)))
                      Node for the owning spatial node
negative padding      Input(InvalidContainer(NegativePadding(side)))
                      NodeField at PaddingLeft|Right|Top|Bottom
padding exceeds box   Input(InvalidContainer(PaddingExceedsExtent(extent)))
                      Node for the owning spatial node
negative gap           Input(InvalidContainer(NegativeGap)) at NodeField Gap
```

Within one item, Layout V1's complete internal priority applies: constraints,
negative padding, padding fit, then gap after its already-proved count,
topology, and viewport invariants. An earlier preparation item therefore wins
even when a later item has an earlier Layout V1 kind. A layout kind or location
impossible for constructed input becomes `Layout(BridgeInvariant)`. Its trusted
location is `Island { index }` for an island item and `Node { index }` for a
sentinel or free singleton. Candidate text and payload never cross the bridge.
All prepared proofs remain private and no singleton reaches execution.

## Phase 6: local transforms

Non-root nodes are scanned record-major. For each node, scalar domain checks
use `SpatialTransformScalarFieldV2::ALL`: affine A, B, C, D, Tx, Ty, then
transform-origin X and Y. Failure is
`Transform(ScalarOutOfDomain(transform_field))` at the exactly corresponding
`NodeField`. After all eight fields for that node pass, the widened exact
affine determinant is checked. Zero returns `Transform(SingularTransform)` at
`Node { index }`. The root has structural identity and is skipped. Placement,
world composition, and composed-transform singularity are outside this cut.

## Phase 7: content structure and local proofs

Phase 7 uses the table stages below as global priority. A stage completes
before the next starts. `InvalidRange` never carries limit evidence.

### Paths and path verbs

First, a global dense-key pass returns `Content(NonDenseKey(Path))` at
`Path { index, field: Key }`. A second pass partitions the verb table. Paths
scan by key; `verb_start` must equal the widened running cursor, then widened
`start + length` must not exceed the payload length. Start mismatch uses
`Content(InvalidRange(PathVerb))` at `VerbStart`; overflow or out-of-bounds end
uses the same kind at `VerbLength`. After the last path, any unconsumed trailing
verb, including payload with no paths, uses that kind at `Input`.

After the complete range is trusted, `PathVerbsPerPath` scans paths by key and
uses `Path { index, field: VerbLength }`, raw length as observed, and caller
maximum. K1 then runs record-major. Per path it scans applicable verb scalars,
validates complete grammar, then commits subpaths. K1 kinds, fields, local verb
ordinals, and the first `PathSubpathsTotal` crossing are exactly those in the
geometry-kernel contract. All path grammar completes before shape validation;
flattening remains deferred to the final local stage below.

### Shapes and polygon points

A global dense-key pass uses `Content(NonDenseKey(Shape))` at `Key`. One
record-major structural pass then checks each shape's authored non-sentinel
`Owner` before its variant. `Rect` and `Circle` have no structural payload.
`Polygon` checks `PolygonPointStart` against the running cursor, then checks the
widened `start + length` at `PolygonPointLength` before advancing the cursor.
`Path` validates its reference at `Path`. Missing or sentinel owners use
`Content(InvalidReference(Owner))`; a missing path uses
`InvalidReference(Path)`. Each uses the current `Shape { index, field }`.

Only after every shape record passes is the final polygon cursor compared with
the complete point-table length. Trailing or ownerless payload returns
`Content(InvalidRange(PolygonPoint))` at `Input`. A start mismatch or invalid
end returns that kind at `PolygonPointStart` or `PolygonPointLength`. Therefore
any record structural failure wins over leftover payload.

Only after that complete structure passes does K1 scan shapes record-major.
Rect, circle, polygon-point scalar order, signed semantics, local point
ordinals, `PolygonPointsPerShape`, and exact shape-error locations follow the
geometry-kernel contract. Path shapes add no scalar or grammar pass here.

### Brushes, stops, and images

Brush keys first form one global dense pass. Linear-gradient ranges then
partition the complete stop table in brush order. Start mismatch uses
`GradientStopStart`, invalid end uses `GradientStopLength`, and trailing or
ownerless stops use `Input`; all return `Content(InvalidRange(GradientStop))`.
P2 then scans brushes record-major, including its per-brush limit, scalar and
semantic priority, local stop ordinals, normalization, locations, and evidence.
Private P2 failures map exactly as fixed by the paint-kernel contract.

Image keys next form one global dense pass. P4 scans images record-major with a
single cumulative accepted-pixel count. Its zero extent, edge limits, widened
pixel total, stride, length, premultiplication, `u128` pixel ordinal, channel
order, locations, commit behavior, and evidence follow the paint-kernel
contract. A caller edge limit above `u32::MAX` does not narrow a raw `u32`
dimension; no extra image representability limit exists.

### Clips

Clip keys first form one global dense pass. Clips then scan record-major:
authored owner reference; parent reference; earlier-parent rule; shape
reference; equal shape owner; parent-owner ancestry; then effective chain
depth. Missing references use `InvalidReference(Owner|Clip|Shape)` at
`Owner|Parent|Shape`. `ForwardParent` uses `Parent`, `ShapeOwnerMismatch` uses
`Shape`, and `OwnerNotAncestor` uses `Parent`. `ClipDepth` uses `Parent`, the
complete chain depth as observed, and caller maximum. All are located at the
current `Clip { index, field }`; a root owner is an invalid owner reference.

### Paint, hit, and semantic items

Tables run completely in `Paint, Hit, Semantic` order. Each is record-major.
For one record, an authored owner reference is checked before sorted owner
order and dense owner-local item ordinal. A missing or sentinel owner is
`InvalidReference(Owner)` at `Owner`; decreasing/reopened owner order is
`InvalidOrder(table)` at `Owner`; a wrong local ordinal is the same kind at
`ItemOrdinal`. Paint and hit then check their per-node limit at
`ItemOrdinal`, using the candidate owner count and caller maximum.

Coverage paint next visits `CoverageKind`, validates a same-owner `Shape`,
visits `FillRule` or dispatches `StrokeWidth` to K1, validates `Brush`, visits
`Opacity`, then validates the terminal `Clip`. Image paint validates `Image`,
then dispatches to P5 in its relational order: empty source width then height;
source x near then far, y near then far; destination scalar x, y, width, height;
negative destination width then height; zero destination width then height.
It then visits `Opacity` and validates the terminal `Clip`.

Hit visits `CoverageKind`, validates a same-owner `Shape`, visits `FillRule` or
dispatches `StrokeWidth` to K1, validates terminal `Clip`, then visits
`InputPolicy`. Semantic validates a same-owner `Shape`, visits `FillRule`, then
validates terminal `Clip`. A missing or wrong-owner shape is
`InvalidReference(Shape)` at `Shape`. Brush, image, and missing terminal clip
keys use their matching `InvalidReference` at `Brush`, `Image`, or `Clip`. A
terminal clip whose owner is not the item owner or an ancestor returns
`InvalidClip(ItemOwnerNotDescendant)` at `Clip`. Opacity, fill rule, input
policy, and closed discriminants have no invalid raw state.

P5 retains only its pre-clip proof during this pass. The entire paint table,
then hit table, then semantic table, including terminal clips, must succeed
before any image destination far edge is checked. Thus any later item-phase
failure wins over an earlier deferred P5 bounds failure.

### Flattening and local bounds

After every item passes, K2 flattens paths in key and source-verb order. At
each emitted segment it checks `FlattenedSegmentsPerPath` before the cumulative
`FlattenedSegmentsTotal`; nonflat depth, local source attribution, widened
arithmetic, and no-partial-path commit follow the geometry kernel.

Local bounds use one shared scan and never batch by kind. Base shapes run first
in key order. Rect derives x far at `RectWidth` before y far at `RectHeight`;
circle derives x then y with `CircleRadius`; polygon and path base derivation is
infallible after prior scalar validation. Paint then runs in input order:
`RoundStroke` dispatches to K3, image paint dispatches to P5, and fill skips.
Hit finally runs in input order: `RoundStroke` dispatches to K3 and fill skips.
Clips and semantics use validated base bounds and add no local-bound error.
Exact x-before-y `LocalBoundsOutOfDomain`, canonical empty/closed AABBs, and
failure fields follow the two local kernel contracts. A failure discards every
flattened or local proof.

## Proof atomicity and nonclaims

Success produces one crate-private validation proof containing only trusted
topology, island preparation, local transforms, normalized resources, validated
items, flattened paths, and local bounds needed by later phases. No partial
proof is observable. On any failure, all temporary and prepared values are
dropped; caller input, layout call count, prior runtime snapshot, generation,
properties, projections, operation attribution, and presented frame remain
unchanged.

The proof may borrow immutable raw tables and image bytes or own derived data,
but it never owns logical identity, clones image bytes, exposes allocation
identity, or permits unchecked reconstruction. Dependency targets and limits,
cycle detection, unit execution, world geometry, output validation, public
resolution, and publication require later exclusive RED/GREEN cuts.
