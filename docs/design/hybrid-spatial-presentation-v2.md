# WU-0013 immutable spatial presentation contract

Status: frozen for presentation RED/GREEN
Work unit: WU-0013
Parent plan: [hybrid spatial composition](hybrid-spatial-composition.md)
Spatial snapshot:
[hybrid spatial snapshot API](hybrid-spatial-snapshot-api-v2.md)
Reference raster:
[hybrid spatial raster API](hybrid-spatial-raster-api-v2.md)
Runtime publication:
[hybrid spatial runtime API](hybrid-spatial-runtime-api-v2.md)
Candidate screen:
[hybrid spatial candidate screen](hybrid-spatial-candidate-screen.md)
Format: spatial presentation contract version 2

## Purpose and boundary

This cut publishes the paint-only part of one accepted spatial snapshot to one
renderer offer. The presenter can consume local geometry, resources, resolved
paint transforms, clips, order, or the deterministic CPU reference raster. It
cannot read logical state, layout input, runtime storage, testkit observations,
authoring records, or candidate-owned values through the frame.

The boundary remains split by ownership:

- `fenestra-ui-spatial` certifies the coherent paint projection because it owns
  the accepted source, normalized proof, resolved output, and resource bytes;
- `fenestra-ui-runtime` adds the exact committed generation and makes the view
  available only from renderer work;
- a probe-private adapter converts that view to reference CPU pixels or one
  candidate API and owns every native surface, candidate error, and staging
  allocation.

There is no public presenter or renderer trait. Runtime already has the closed
offer, accept, reject, completion, and renderer-loss protocol. A public trait
that runtime never calls would add candidate policy without adding an ownership
boundary.

## Exact spatial surface

The current spatial `prototype` has 121 explicit exports and 51 public structs.
This cut adds exactly one export and one struct, producing 122 exports and 52
structs:

```text
SpatialPaintFrameV2
```

The type and its only construction path are:

```rust
pub struct SpatialPaintFrameV2<'a> { /* private fields */ }

impl SpatialResolvedSnapshotV2 {
    #[must_use]
    pub fn paint_frame(&self) -> SpatialPaintFrameV2<'_>;
}
```

The frame has no public constructor. It implements exactly `Clone + Copy` plus
the applicable runtime auto traits. It implements no `Debug`, `Display`,
`Default`, equality, ordering, or hash trait. Every method below is
`#[must_use]`, takes the copyable view by value, and has the exact signature:

```rust
impl<'a> SpatialPaintFrameV2<'a> {
    pub fn viewport(self) -> SpatialViewportV2;
    pub fn polygon_points(self) -> &'a [SpatialPointV2];
    pub fn path_verbs(self) -> &'a [SpatialPathVerbV2];
    pub fn paths(self) -> &'a [SpatialPathV2];
    pub fn shapes(self) -> &'a [SpatialShapeV2];
    pub fn clip_primitives(self) -> &'a [SpatialClipV2];
    pub fn gradient_stops(self) -> &'a [SpatialGradientStopV2];
    pub fn brushes(self) -> &'a [SpatialBrushV2];
    pub fn images(self) -> &'a [SpatialImageV2];
    pub fn paint_items(self) -> &'a [SpatialPaintV2];
    pub fn resolved_clips(self) -> &'a [SpatialClipOutputRecordV2];
    pub fn effective_clip_aabbs(self) -> &'a [SpatialAabbV2];
    pub fn resolved_paints(self) -> &'a [SpatialPaintOutputRecordV2];
    pub fn rasterize_reference(
        self,
        limits: ReferenceRasterLimitsV2,
    ) -> Result<ReferenceRasterV2, ReferenceRasterErrorV2>;
}
```

No existing spatial method changes. The frame exposes no topology node,
placement, container, base geometry row, hit row, semantic row, prepared proof,
owned source, snapshot, layout key, logical identity, or candidate handle.

The frame is an unforgeable validation view, not a second validator. Its table
coherence is exact:

- clip primitives, resolved clips, and effective clip AABBs have equal length
  and pair by dense clip ordinal;
- paint items and resolved paints have equal length and pair by dense paint
  ordinal;
- resolved paints remain in accepted ascending painter order;
- shape, path, brush, gradient, and image references resolve in the exposed
  dense tables;
- unused global geometry or resource rows remain legal and do not become paint;
- candidates do not repeat raw validation or replace resolved transforms,
  AABBs, clip chains, or ordering with reconstructed values.

`rasterize_reference` delegates to the same retained snapshot operation. For
the same frame and limits it returns exactly the same metadata, bytes, and
typed error as `SpatialResolvedSnapshotV2::rasterize_reference`. It does not
rebuild a snapshot from the exposed tables.

## Exact runtime surface

The current runtime `prototype` has 74 exports and 52 public structs. This cut
adds exactly one export and one struct, producing 75 exports and 53 structs:

```text
RuntimePaintFrameV2
```

The exact surface is:

```rust
pub struct RuntimePaintFrameV2<'a> { /* private fields */ }

impl<'a> RuntimePaintFrameV2<'a> {
    #[must_use]
    pub const fn generation(self) -> RuntimeGeneration;
    #[must_use]
    pub const fn spatial(self) -> SpatialPaintFrameV2<'a>;
}

impl FrameWork {
    #[must_use]
    pub fn paint_frame(&self) -> Option<RuntimePaintFrameV2<'_>>;
}
```

`RuntimePaintFrameV2` has no public constructor. It implements exactly
`Clone + Copy` plus the applicable runtime auto traits and none of `Debug`,
`Display`, `Default`, comparison, ordering, or hash. No method is added to
`CommittedRuntimeSnapshot`: presentation consumes a scheduler offer rather
than turning general logical observation into a presenter API.

`FrameWork::paint_frame` returns `Some` only for spatial runtime mode. Ordinary
and headless modes return `None`. The generation is copied from the exact
committed snapshot retained by that `FrameWork`; it is never supplied by the
caller or inferred from a spatial key. An empty paint table still returns a
present frame.

This cut adds no public error, enum variant, constant, function, trait, alias,
or dependency. The scheduler still retains one `CommittedRuntimeSnapshot` as
visual work, and its protocol-accounted envelope remains exactly 40 bytes.

## Ownership, lifetime, and retention

The ownership chain remains:

```text
FrameWork
  -> CommittedRuntimeSnapshot Arc
  -> RuntimeState
  -> SpatialPublication Arc
  -> SpatialResolvedSnapshotV2 Arc
  -> accepted source Arc and image bytes
```

Both new frame types are borrowed views. Their lifetime is bounded by the
borrow of `FrameWork`, so a presenter cannot retain the frame after its call or
gain a hidden logical-state owner. A renderer that needs longer candidate
lifetimes must synchronously copy or upload candidate-owned resources before
acceptance. A future owned display-list handle requires a separate contract;
it is not smuggled in by cloning `FrameWork` or exposing an Arc.

A replaced runtime generation never mutates an earlier allocation. An old
retained `FrameWork` continues to report its old generation, viewport, table
contents, resource bytes, and raster result. A true runtime no-op preserves the
outer state and inner spatial allocations and creates no paint-frame
allocation. Frame formation itself is infallible and allocates nothing.

## Reference CPU and physical handoff

The first native lane uses the registered reference-raster limit. Staging runs
in this order before scheduler acceptance:

1. Obtain `RuntimePaintFrameV2` from the offered `FrameWork` and require equal
   offered and frame generations.
2. Require the frame viewport to equal the adapter-private logical surface.
3. Call `SpatialPaintFrameV2::rasterize_reference` and verify width, height,
   stride, and byte length against that viewport.
4. Preflight the adapter-private physical pixel and byte limits.
5. Reserve and fill one physical `u32` staging buffer.
6. Resize and acquire the Softbuffer surface, copy pixels, and call
   `Window::pre_present_notify`.
7. Invoke the supplied scheduler acceptance callback exactly once.
8. Present the acquired buffer.
9. On success, report scheduler completion outside the presenter call.

The spatial frame contains no device scale, physical extent, surface epoch,
window, or lifecycle state. Logical-to-physical conversion is probe-private.
For nonzero extents, physical pixel `(x, y)` chooses source indices with widened
unsigned arithmetic:

```text
source_x = min(((2*x + 1) * logical_width) /
               (2 * physical_width), logical_width - 1)
source_y = min(((2*y + 1) * logical_height) /
               (2 * physical_height), logical_height - 1)
```

This is edge-aligned nearest sampling by physical pixel center; an exact tie
selects the higher source index. The adapter composites the premultiplied RGBA8
reference pixel over opaque black. Therefore it packs the retained premultiplied
R, G, and B channels directly as Softbuffer `0x00RRGGBB`; alpha is not
unpremultiplied or multiplied again. Rows remain top-to-bottom.

Physical zero extent is the existing native suspension case and invokes no
presenter. A nonzero physical extent paired with a zero logical raster, a
viewport mismatch, or inconsistent raster metadata is a private invariant
failure before acceptance. The pure corpus fixes conversions at scale 1.0,
1.25, and 2.0. These rules are adapter evidence, not spatial semantics.

## Private presenter protocol and failures

The native lane owns a private `SpatialPresenterPortV2`. Its semantic call
takes only `RuntimePaintFrameV2<'_>`, the accepted adapter-private surface
tuple, and one `FnOnce` scheduler-acceptance callback. Candidate contexts,
surfaces, buffers, and errors remain below that port. The core crates neither
implement nor name the port.

A reference-raster limit, staging preflight, reservation, resize, acquire,
copy, pre-present notification, or acceptance failure occurs before a
successful `AcceptFrame`. The driver rejects the outstanding offer and does
not replace its last successful presentation record. A `present` failure after
acceptance never rejects the accepted frame; it reports `RendererLost` for the
accepted epoch and follows existing ordered retirement. A successful present
is followed by same-turn `Complete` in the CPU lane.

Presentation failure does not roll back a `RuntimeGeneration` that was already
committed before `OfferFrame`. It preserves the last successfully presented
frame and adapter-owned presentation evidence. Runtime build, validation, or
spatial resolution failures still occur before publication and preserve the
prior runtime generation. Requiring an OS presentation failure to unpublish a
runtime generation would require a different two-phase runtime protocol and is
outside this contract.

Softbuffer allocation identity is opaque and carries no retention claim. The
fake port retains an explicit last-successful staging digest so replacement
and failure behavior are observable without claiming compositor scanout.

## Fake presenter and native controls

The exclusive presenter RED covers:

- the port signature contains no `FrameWork`, committed snapshot, logical
  node, layout, testkit, authoring, IR, or candidate type above the adapter;
- a frame reports the offered generation and exact spatial viewport and table
  counts;
- successful phase order is raster, stage, resize, acquire, copy, notify,
  accept, present, then scheduler completion;
- reference RGBA bytes and physical `0x00RRGGBB` bytes are checked against
  independent literals, including transparent and partial-alpha pixels;
- 1.0, 1.25, and 2.0 conversion, physical limits, zero suspension, and
  metadata mismatch are independent cases;
- every pre-accept fault rejects once, never accepts, and retains the prior
  successful digest;
- acceptance followed by present failure reports loss, never rejection, and
  retains the accepted work until ordered retirement;
- an earlier offered frame remains readable after a later runtime commit;
- ordinary and headless offers expose no paint frame;
- source scans keep the spatial V2 adapter free of testkit, authoring, layout,
  runtime-private, and upstream candidate vocabulary.

The WU-0009 V1 native lane remains a control. New code lives in a parallel
`spatial_v2` module in `exp-0001-native-spine`; `run_native_probe_v1`, its
default binary, V1 trace grammar, exact Fedora artifact bytes, tests, and
candidate versions remain unchanged. The package may retain its existing
testkit dependency for V1, but no `spatial_v2` source file may import it.

## Docs-first RED/GREEN sequence

1. Commit this frozen contract and its parent-plan link.
2. Add an external spatial API RED for the exact export, traits, signatures,
   table pairing, allocation identity, and raster delegation.
3. Implement the paint view without copying or revalidating tables.
4. Add a runtime API and behavior RED for the exact export, mode matrix,
   generation seal, retained old work, no-op identity, rollback, and unchanged
   scheduler accounting.
5. Implement `FrameWork::paint_frame` by borrowing its retained publication.
6. Add the private fake-presenter, reference handoff, conversion, failure, and
   V1 non-regression REDs.
7. Implement the parallel spatial V2 native lane and Softbuffer adapter.
8. Record bounded native evidence and dependency facts separately; do not
   bless a candidate pixel result as the reference oracle.

Recommended focused commits are:

```text
docs(spatial): freeze paint frame publication
test(spatial): specify immutable paint frame
feat(spatial): expose immutable paint frame
test(runtime): specify offered paint frames
feat(runtime): publish paint frames to scheduler
test(native): specify spatial reference presentation
feat(native): present spatial reference frames
```

## Deferrals and nonclaims

This contract does not select Softbuffer, Vello, a GPU API, a scene cache, an
owned-surface export path, damage tracking, asynchronous frame ownership, or a
public renderer interface. It makes no density, physical-pixel, safe-area,
surface-lifecycle, multi-scene, compositor-completion, or scanout claim.
Physical conversion exists only in the disposable native adapter, so WU-0012
remains deferred.
