# WU-0013 exact spatial hit-query API contract

Snapshot API: [hybrid-spatial-snapshot-api-v2.md](hybrid-spatial-snapshot-api-v2.md)
Reference semantics: [hybrid-spatial-reference-v2.md](hybrid-spatial-reference-v2.md)
Content semantics:
[hybrid-spatial-content-reference-v2.md](hybrid-spatial-content-reference-v2.md)
Geometry kernel:
[hybrid-spatial-geometry-kernel-v2.md](hybrid-spatial-geometry-kernel-v2.md)
Format: exact spatial hit-query API version 2

## Boundary

Hit testing is a total read-only query over one successfully resolved immutable
snapshot. It accepts one scene-logical point and returns the topmost exact hit,
if any. It does not accept raw input, a prepared proof, candidate output, a
layout engine, logical identity, runtime state, renderer state, or raster
pixels. `hit_test` performs no heap allocation, mutates no snapshot state, and
introduces no diagnostic or limit.

This cut adds exactly one `prototype` export and no free function. The public
surface advances from 113 to 114 names and from 47 to 48 structs. It adds only
the four result getters and `SpatialResolvedSnapshotV2::hit_test`; every other
public inherent method set remains unchanged. It adds no other public
constructor, method, associated constant, function, trait, alias, or module.

## Result and snapshot method

```rust
pub struct SpatialHitResultV2 {
    key: u32,
    owner: SpatialNodeKeyV2,
    item_ordinal: u32,
    local_point: SpatialPointV2,
}

impl SpatialHitResultV2 {
    pub const fn key(self) -> u32;
    pub const fn owner(self) -> SpatialNodeKeyV2;
    pub const fn item_ordinal(self) -> u32;
    pub const fn local_point(self) -> SpatialPointV2;
}

impl SpatialResolvedSnapshotV2 {
    pub fn hit_test(
        &self,
        scene_point: SpatialPointV2,
    ) -> Option<SpatialHitResultV2>;
}
```

The result has private fields and no public constructor. `key` is the dense
global key of the winning Hit output row. `owner` and `item_ordinal` are its
trusted spatial owner and owner-local hit ordinal. `local_point` is the query
point mapped through the winning Hit row's accepted world affine into the hit
owner's local coordinate system. It is never a clip-local point.

The result implements exactly `Clone + Copy + Debug + Eq + PartialEq` plus
`Send + Sync + Unpin + UnwindSafe + RefUnwindSafe + 'static`. It implements no
`Default`, `Display`, `Hash`, `Ord`, or `PartialOrd`. All four getters are
by-value, `const`, and `#[must_use]`. `hit_test` is non-const and `#[must_use]`.

The result deliberately omits a logical identifier, stack ordinal, shape,
clip, policy, transform, AABB, and source record. Spatial node keys are
pass-local; runtime owns any later mapping to stable logical identity. In
version 2 stack ordinal is already the owner key, while the global row key plus
owner-local ordinal identifies which of several accepting items won.

## Selection order and authority

The query scans the accepted Hit output table in exact reverse record order and
returns the first exact match. It never sorts, batches by coverage kind, or
uses node order in place of hit-row order. `Ignore` rows are skipped before
containment or selection. Paint, brush, image alpha, raster pixels, semantic
geometry, viewport bounds, and Geometry base bounds do not participate.

The accepted snapshot is authoritative for projection data:

- the Hit row supplies its world affine and conservative unclipped world AABB;
- each Clip row supplies its world affine and primitive conservative AABB;
- the snapshot's effective-clip vector supplies candidate-derived chain AABBs;
- retained trusted state supplies input policy, coverage kind, shape, fill
  rule, positive stroke token, terminal clip, clip ancestry, polygon ranges,
  and flattened paths.

Candidate validation permits Geometry values that differ from the reference,
so a query never substitutes the phase-10 reference transforms or AABBs still
retained privately by prepared state. AABB tests are reject-only accelerators:
passing one never accepts a hit without exact coverage.

## Exact clip and hit coverage

For a row with a terminal clip, an empty effective clip AABB rejects it. A
scene point outside that closed effective AABB also rejects it. Otherwise the
query examines every clip in the complete trusted parent chain. For each clip,
it maps the original scene point through that accepted Clip row's affine using
`Affine2V2::inverse_point` and requires exact K4 local fill coverage with the
clip's trusted shape and fill rule. Every ancestor must contain the point; an
AABB can never stand in for this exact test. Touching or point-valued effective
bounds therefore still require exact shape containment.

After the clip chain succeeds, the query reject-tests the Hit row's closed
world AABB, maps the original scene point through the accepted Hit row affine,
and dispatches its trusted coverage:

- Fill uses the exact K4 rect, circle, polygon, or flattened-path rule;
- RoundStroke uses the exact K5 rect, circle, polygon, or flattened-path rule
  with the retained validated positive width.

The existing half-open rect fill, closed circle boundary, polygon winding,
implicit path fill closure, explicit path stroke segments, round joins, round
ends, and zero-length segment rules remain unchanged. Polygon points come from
the exact trusted source range retained by the snapshot; paths use the owned K2
flattened representation. No raw K1 validation is repeated during a query.

An out-of-domain scene scalar is a miss. `inverse_point(None)`, including an
exact inverse result outside the canonical scalar domain, is also a miss rather
than an error. The query performs no viewport prefilter, clamping, saturation,
fallback transform, or epsilon test. A valid snapshot has already proved every
accepted affine nonsingular.

## Ownership, determinism, and deferrals

Repeated queries preserve the exact snapshot output tables, effective clip
allocation, retained source object graph, image bytes, and normalized private
state. The result owns only Copy values and borrows nothing from the snapshot.
The implementation uses no candidate library, unsafe code, cache, spatial
index, or new dependency.

Public clip-only queries, all-hit collection, logical identity joins, pointer
capture, semantic joins, paint frames, runtime generation
publication, invalidation, rollback, artifacts, and authoring remain separate
later RED/GREEN boundaries.

The snapshot-only CPU reference raster is fixed independently by
[hybrid-spatial-raster-api-v2.md](hybrid-spatial-raster-api-v2.md); it does not
participate in hit selection.
