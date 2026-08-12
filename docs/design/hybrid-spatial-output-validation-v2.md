# WU-0013 candidate spatial output validation contract

Output records: [hybrid-spatial-output-api-v2.md](hybrid-spatial-output-api-v2.md)
Snapshot API: [hybrid-spatial-snapshot-api-v2.md](hybrid-spatial-snapshot-api-v2.md)
Diagnostics: [hybrid-spatial-diagnostics-v2.md](hybrid-spatial-diagnostics-v2.md)
Fields: [hybrid-spatial-fields-v2.md](hybrid-spatial-fields-v2.md)
Format: candidate spatial output validation version 2

## Boundary

Validation is structural and relational, not the independent
candidate-versus-reference correctness oracle. That oracle compares two
completed outputs field by field in versioned evidence. Structural validation
never sorts, clamps, repairs, deduplicates, or publishes a partial table.

## Global passes

`validate_spatial_output_v2` performs these complete global passes:

1. record count by `SpatialOutputTableV2::ALL`;
2. dense key by table and record order;
3. scalar domain by the applicable `SpatialOutputFieldV2::ALL` subset,
   including exact integer embedding for Geometry width and height;
4. nonnegative Geometry width then height;
5. stored determinant equality with the row affine's exact determinant and a
   nonzero determinant;
6. canonical AABB shape and exact AABB derivation;
7. Clip parent-chain order while deriving effective bounds from the supplied
   primitive bounds;
8. Paint, Hit, and Semantic stack and owner-local item order;
9. source and cross-output references.

Expected counts are Geometry = source nodes including the sentinel, Clip =
source clips, Paint = source paint items, Hit = source hit items, and Semantic =
source semantic items.

Geometry derives its expected AABB from the closed local
`[0,0]..[base_width,base_height]` box and its supplied affine. Clip, Paint, Hit,
and Semantic derive their expected AABB from the retained source-row local
bound and supplied affine. These derivations use the trusted source row selected
by the dense output key, never an unvalidated candidate reference. The Clip
source is its retained K3 clip bound; Paint and Hit use their retained local
bounds; Semantic uses its retained shape fill bound.

The AABB pass first derives one expected canonical value and then compares
`AabbEmpty, AabbMinX, AabbMinY, AabbMaxX, AabbMaxY`. This one comparison order
covers noncanonical empty encoding, inverted edges, and a wrong but canonical
candidate bound.

The Clip-chain pass keeps one optional scratch effective bound per row and
range-checks every supplied key without dereferencing it first. A missing parent
or a parent whose scratch bound is unavailable makes the current scratch bound
unavailable and reaches the final pass without a phase-7 error. An in-range self
or forward parent is `InvalidClipChain`. For valid candidate Owner keys, the
parent owner must be the same as or a spatial ancestor of the current owner;
otherwise the result is `InvalidClipChain`. Invalid Owner keys are deferred to
the final reference pass. A strictly earlier parent with an available bound
derives the current effective bound by intersection. Every accepted row is
available, and the snapshot stores this candidate-derived vector rather than
the reference vector retained by preparation.

Projection order compares each keyed row's `(stack_ordinal, item_ordinal)` with
the trusted source row's `(owner.get(), item_ordinal)`, independently in Paint,
Hit, and Semantic. Exact source tuples imply nondecreasing stacks and dense
owner-local ordinals. The pass does not trust the candidate Owner field and
does not reorder the supplied rows.

The final reference pass uses the trusted keyed source row. Before reading a
candidate reference, it safely range-checks it. Every non-Geometry row affine
must equal the supplied Geometry row for the trusted source owner, component by
component. Its stored determinant already matched that affine in pass 5. The
candidate Owner, Parent, Shape, Brush, Image, and terminal Clip fields must then
equal the source row and name existing applicable records.

For Paint, a source Coverage row requires a candidate Coverage reference and
checks Shape then Brush. A source Image row requires a candidate Image reference
and checks Image. A variant mismatch is attributed to Shape for an expected
Coverage row or Image for an expected Image row; no Kind field is invented.

Candidate Geometry base values, world transform, and their consistently
derived AABB need not equal the independently materialized reference values.
Such structurally valid differences are exact evidence mismatches, not
validation errors.

## Exact diagnostic attribution

A count mismatch uses `Output { table }`. Every other failure uses
`OutputRecord { table, index, field }` with no observed or maximum evidence.
Within each pass, table order precedes record order and applicable field order.

The exact field attribution is:

```text
KeyMismatch -> Key
ScalarOutOfDomain -> first applicable scalar field
ScalarOutOfDomain for fractional Geometry extent -> BaseWidth, then BaseHeight
NegativeBaseExtent(Width) -> BaseWidth
NegativeBaseExtent(Height) -> BaseHeight
InvalidWorldDeterminant -> Determinant
InvalidAabb ->
  first of AabbEmpty, AabbMinX, AabbMinY, AabbMaxX, AabbMaxY
InvalidClipChain -> Parent
InvalidProjectionOrder for a stack tuple -> StackOrdinal
InvalidProjectionOrder for a local ordinal -> ItemOrdinal
InvalidReference for owner-world mismatch ->
  first of AffineA, AffineB, AffineC, AffineD, AffineTx, AffineTy
InvalidReference for source/reference mismatch ->
  first applicable of Owner, Parent, Shape, Brush, Image, Clip
```

If exact AABB derivation itself cannot represent an edge, the failure is
`InvalidAabb` at that edge field rather than a resolver `Arithmetic` error.
Phase 10 already proved the reference path; this output pass classifies only
the supplied candidate record.

## Evidence and deferrals

Independent evidence compares every accepted candidate field with a fresh
reference materialization. A mismatch never changes the closed resolver error
vocabulary and never silently admits the candidate lane.

Exact clip containment, reverse hit selection, rasterization, semantic joins,
runtime publication, and artifact comparison remain later boundaries. This
validator invokes none of them.
