# WU-0013 authoring runtime equivalence format 2

Status: frozen for runtime-equivalence RED/GREEN
Work unit: WU-0013
Parent authoring contract:
[hybrid spatial authoring format 2](hybrid-spatial-authoring-v2.md)
Runtime adapter:
[runtime symbolic spatial adapter](hybrid-spatial-runtime-ir-api-v2.md)
Immutable snapshot:
[immutable spatial snapshot](hybrid-spatial-snapshot-api-v2.md)
Probe: `probes/exp-0007-typed-authoring`

## Boundary and compatibility

This cut proves that one manual raw format-2 quadruple, one `.fen` build lane,
and one `ui!` macro lane publish the same logical and spatial behavior. The
three lanes are candidates. None of them supplies expected runtime values.
A separate literal reconstruction is authoritative.

The existing `layout-board.fen`, `layout-board.ui`, every `LAYOUT_BOARD_*_V1`
item, generated V1 output name, V1 test, and `layout-board-*-v1.txt` artifact
in `exp-0007-typed-authoring` remain byte-for-byte unchanged. Format-2 files,
symbols, tests, support modules, and later artifacts use a `v2` suffix. No V1
artifact is regenerated or reinterpreted.

This cut adds no core-crate API, dependency, renderer, presenter, candidate
type, filesystem input at runtime, or stable public facade. It exercises the
already frozen authoring, IR, runtime, snapshot, hit, and raster APIs.

## Registered source and three candidate lanes

The probe owns these two additive fixtures:

```text
probes/exp-0007-typed-authoring/fixtures/hybrid-spatial-v2.fen
probes/exp-0007-typed-authoring/fixtures/hybrid-spatial-v2.ui
```

The `.fen` file is byte-for-byte equal to
`crates/fenestra-ui-authoring/tests/fixtures/hybrid_spatial_v2.fen` at entry to
this cut: 7,714 ASCII bytes, 250 LF-terminated lines, and exactly one final
LF. The `.ui` file wraps the same 1,610 abstract tokens once in `ui! { ... }`.
It adds no token inside the document and contains no second authored program.
Tests compare the extracted abstract token sequence, not whitespace or macro
spans. Both probe files are separately versioned so later evidence never reads
a test fixture through another package's source-tree path.

`build.rs` preserves its complete V1 route and additionally compiles the V2
`.fen` with `compile_fen_v2` and `REFERENCE_AUTHORING_LIMITS_V2`, then writes
`hybrid_spatial_fen_v2.rs` in `OUT_DIR` through `canonical_rust_v2`. Its error
mapping uses only the V2 typed diagnostic and the registered fixture-relative
byte range. A V2 failure cannot change the existing V1 diagnostic text.

The probe library adds exactly these disposable V2 observations:

```rust
pub const HYBRID_SPATIAL_FEN_V2: &[u8];
pub const HYBRID_SPATIAL_GENERATED_RUST_V2: &str;

pub fn generated_hybrid_spatial_v2() -> (
    SchemaManifest,
    ConstructionProgram,
    StyleProgram,
    SpatialProgramV2,
);

pub fn macro_hybrid_spatial_v2() -> (
    SchemaManifest,
    ConstructionProgram,
    StyleProgram,
    SpatialProgramV2,
);
```

The first function includes only `OUT_DIR/hybrid_spatial_fen_v2.rs`. The
second includes only `fixtures/hybrid-spatial-v2.ui`, whose `ui!` invocation
must dispatch through the version-neutral macro entry. Neither function calls
an authoring compiler at runtime.

The manual lane lives under
`tests/support/hybrid_spatial_v2/manual/`. It constructs every raw schema,
construction, style, spatial record, field, payload, and logical `SourceSpan`
directly. It does not parse either fixture, call authoring, include generated
Rust, invoke the macro, deserialize an artifact, or clone a candidate
quadruple. Cohesive builder files remain below 400 lines.

Before runtime execution, all three raw quadruples must be exactly equal.
Each lane is then independently validated in schema, construction, style, and
spatial order. Reusing one `ValidatedSpatialProgramV2` across lanes is
forbidden. The validation profile is exactly:

```text
ValidationLimits = (1, 8, 7, 1, 6, 19, 2, 4, 8)
StyleValidationLimits = 3
SpatialValidationLimitsV2 =
  [7, 5, 3, 3, 4, 4, 3, 1, 5, 3, 3, 1, 16]
```

The construction values are components, properties, templates, regions,
child slots, initial properties, initial keys, template depth, and initial
instances. Spatial values are in the constructor's fixed 13-entry order.
Its first value bounds seven symbolic declarations before keyed expansion;
it does not bound the later sentinel-inclusive runtime geometry table.

## Runtime profile and operation script

Every lane constructs its own `UiRuntime::new_spatial_ir` with:

```text
initial viewport = SpatialViewportV2::new(192, 128)
spatial limits = REGISTERED_SPATIAL_LIMITS_V2
runtime capacity = RuntimeCapacity::new(4, 4, 12, 2, 96, 3)
reference raster pixel limit = 35,840
```

The runtime capacity values are operations, structural changes, live nodes,
live fragments, live property slots, and retained generations. They are a
probe profile, not defaults. The raster bound equals the largest registered
viewport below. Equality must pass and one under must fail in the existing
snapshot raster API; this cut does not add another raster limit.

The shared semantic script is applied one transaction at a time in this exact
order. Paths use the normalization grammar below.

```text
0 init at viewport 192x128
1 resize spatial viewport to 224x160
2 set root property span_x(0) from 180 to 176
3 set root property tone(4) from rgba8(96,72,48,255)
  to rgba8(80,40,24,255)
4 set root property policy(7) from ignore to accept
5 insert key 30 at final index 1 in root/s:0/s:0/r:1
6 move key 30 to final index 2 in root/s:0/s:0/r:1
7 update key 30 property span_y(1) from 12 to 14
8 remove key 20 from root/s:0/s:0/r:1
9 attempt to set root/s:0/s:0 property factor(3) from 1 to 0
```

Steps zero through eight yield nine observations at generations zero through
eight. Step nine fails and yields only one failure observation. Final keyed
order is `[10, 30]`.

Step nine must be exactly
`TransactionErrorKind::Spatial(RuntimeSpatialErrorV2::Ir(_))`; the IR kind is
`Resolve`, the resolver kind is `Transform(SingularTransform)`, the resolver
location is `Node { index: 3 }`, the IR span is logical bytes `226..227` in
source zero, and `operation_index()` is `None`. After failure, the committed
snapshot shares its exact outer state allocation with the pre-attempt handle;
generation, viewport, properties, logical identities, mapping, spatial
snapshot pointer, hit results, and raster bytes remain unchanged.

The factor binding that makes the determinant zero retains its own authored
field span at logical bytes `247..248`. The raw resolver reports a whole-node
singularity because the determinant depends on four affine coefficients, so
runtime maps that `Node` location to the containing node record rather than
attributing the failure to one contributing coefficient.

## Logical identity normalization

Runtime `NodeId` and `FragmentId` values never enter equality or artifacts.
The normalizer binds them by walking the retained construction in authored
slot order and keyed members in committed order. Its ASCII path grammar is:

```text
node       = root { /s:SLOT | /m:REGION_SLOT:KEY }
fragment   = node/r:REGION_SLOT
absent     = none
```

Slots and keys use unsigned base-10 spelling. A static segment is its authored
child slot. A member segment is its owner's authored region slot plus raw
`u64` key. Binding rejects missing or duplicate runtime identities, template
disagreement, child-order disagreement, extra children, and count mismatch.

Each successful observation owns generation, viewport, the normalized commit
receipt, and complete logical state. Logical state records nodes in
construction-guided depth-first order and includes path, parent path,
template, component, every property in schema order, authored child groups,
fragments, and members. Values retain their closed typed variant and payload;
formatting is not used for comparison.

The initial non-sentinel spatial mapping is exactly:

```text
1 root
2 root/s:0
3 root/s:0/s:0
4 root/s:0/s:0/s:0
5 root/s:0/s:0/m:1:10
6 root/s:0/s:0/m:1:20
7 root/s:1
8 root/s:1/s:0
```

Spatial key zero is recorded as `none`. Insert and move renumber the complete
dense pass-local suffix according to current member order; remove returns the
member portion to keys 5 and 6 and guide to key 7. The sentinel plus eight
initial mapped nodes produce nine geometry and mapping rows; insertion grows
both to ten. For every nonzero row the forward and reverse runtime lookups
must agree. A mapped path may not appear
twice, and no geometry key may be absent from the mapping except sentinel zero.

## Exact spatial projection normalization

Normalization reads one `RuntimeSpatialViewV2` and its exact immutable
snapshot once. It copies public values into private Eq records. It never reads
testkit, authoring source maps, generated Rust, candidate formatting, pointer
addresses, or private runtime state.

Numbers are retained as integers, not floats or debug strings. Every
`SpatialScalarV2` is its raw `i64`; determinants are exact `i128`; colors are
four bytes. An affine is `(a,b,c,d,tx,ty)` in that order. An AABB is
`(empty,min_x,min_y,max_x,max_y)`, including the required zero edges when
empty. Options retain `None | Some(key)`.

The normalized projection contains these tables in their supplied order:

- `mapping`: spatial key, optional logical path;
- `geometry`: key, optional logical path, base x, base y, base width, base
  height, affine, determinant, and world AABB;
- `clips`: key, owner key and path, parent key, shape key, affine,
  determinant, primitive world AABB, and the effective AABB at the same clip
  index;
- `paints`: global key, owner key and path, affine, determinant, world AABB,
  reference tagged as `coverage(shape,brush)` or `image(image)`, terminal
  clip, stack ordinal, and item ordinal;
- `hits`: global key, owner key and path, affine, determinant, world AABB,
  shape, terminal clip, stack ordinal, and item ordinal;
- `semantics`: the same record shape as hits, independently copied from the
  semantic table.

No table is sorted, filtered, deduplicated, clipped, or reconstructed. Exact
table counts at the largest keyed state are mapping 10 including sentinel,
geometry 10, clips 5, paints 6, hits 6, and semantics 5. Limits are checked
before each private allocation. Any count or mapping disagreement fails the
test instead of truncating an observation.

Hit normalization queries every logical pixel center in row-major order for
the observation viewport, followed by these four raw Fixed16 points:

```text
(-1, 0), (0, -1), (width * 65536, 0), (0, height * 65536)
```

For each query it records the raw scene point and either `none` or global hit
key, owner spatial key, owner logical path, item ordinal, and raw local point.
The maximum is 35,844 queries. It calls only `snapshot.hit_test`; paint and
raster pixels never infer a hit.

Raster normalization calls `snapshot.rasterize_reference` once and owns exact
width, height, `u64` stride, and every packed RGBA8 byte. The largest raster is
224 by 160, stride 896, 35,840 pixels, and 143,360 bytes. Runtime equivalence
compares the complete byte slice. A later bounded text artifact may encode a
measured digest, but a digest never replaces byte equality in this cut.

## Independent literal oracle

The authoritative expected log lives under
`tests/support/hybrid_spatial_v2/oracle/`. It starts from independently typed
literal schema, tree, property, placement, transform, shape, clip, paint, hit,
semantic, and image data, applies the semantic script, and fully reconstructs
every normalized observation and the typed rollback result.

Oracle modules may not import `fenestra-ui-authoring`, `fenestra-ui-macros`,
`fenestra-ui-runtime`, `fenestra-ui-spatial`, the manual raw builder, generated
Rust, either fixture, or candidate normalized records. They do not parse
authoring text or call validation, resolution, `hit_test`, or
`rasterize_reference`. Integer geometry, coverage, compositing, and sampling
helpers are local literal implementations. Only the operation script and
closed path grammar may be shared with lane runners.

All three complete logs compare field-by-field to the oracle before they are
compared with each other. Tests independently mutate each normalized field,
swap each ordered table, remove and duplicate a mapping row, change a hit
answer, and change each raster metadata field and one raster byte. Every
mutation must be detected. Cross-lane equality alone can never bless a golden.

No text artifact is committed until raw quadruple equality, all nine oracle
observations, mutation controls, typed failure, and rollback pass. Later
artifact encoding is printable ASCII with LF and explicit measured bounds; it
does not become the expected-value generator.

## RED/GREEN sequence

The implementation uses these focused commit pairs after this contract:

```text
test(authoring): specify spatial fixture lanes
feat(authoring): add spatial fixture lanes

test(authoring): specify spatial runtime equivalence
feat(authoring): prove spatial runtime equivalence

test(authoring): specify spatial rollback equivalence
feat(authoring): prove spatial rollback equivalence
```

The first RED fails only because the additive fixture lanes and independent
manual quadruple are absent. The second RED fixes normalization, exact limits,
the operation script, full literal oracle, mutation controls, and nine
successful observations. The third RED fixes the exact typed singular failure
and allocation-preserving rollback. Each GREEN implements only its preceding
contract. Existing V1 probe tests are mandatory controls after every pair.

## Nonclaims

This cut does not define a stable serialization, public testkit V2 API,
incremental resolver, renderer, presentation frame, native surface, density,
physical pixels, lifecycle, accessibility-platform bridge, or product
capacity. It proves one bounded versioned corpus through three authoring lanes
and one independent reference reconstruction.
