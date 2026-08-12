# WU-0013 format-2 canonical semantic artifact

Status: frozen for authoring semantic-artifact RED/GREEN
Work unit: WU-0013
Grammar: [hybrid spatial authoring format 2](hybrid-spatial-authoring-v2.md)
Sources and anchors:
[format-2 source contract](hybrid-spatial-authoring-source-v2.md)
Reference fixture:
[format-2 authoring reference](hybrid-spatial-authoring-reference-v2.md)
Prior authoring format: [typed authoring format 1](typed-authoring-reference.md)

## Purpose and compatibility

The format-2 semantic artifact is the canonical byte observation of the one
retained resolved authoring model. It proves `.fen` and `ui!` frontend
equivalence independently of emitted Rust. It contains no frontend origin,
source path, token spelling, macro span, runtime identity, or canonical Rust.

The schema, construction, and style records reuse the format-1 collector and
line encodings byte-for-byte. Format 2 appends spatial records before the same
logical-anchor sort and density check. Every V1 type, function, artifact,
limit, fixture, and byte remains unchanged.

## Exact public surface

The semantic artifact contributes exactly seven additive exports: five types,
one reference constant, and one function. The five types are two enums and
three structs, so this slice contributes three public structs.

```text
SemanticArtifactLimitKindV2, SemanticArtifactLimitsV2,
SemanticArtifactErrorKindV2, SemanticArtifactErrorV2, SemanticArtifactV2,
REFERENCE_SEMANTIC_ARTIFACT_LIMITS_V2, canonical_semantics_v2
```

`SemanticArtifactLimitKindV2` derives `Clone, Copy, Debug, Eq, PartialEq` and
declares `ArtifactBytes, LineBytes, Records`. Its public `ALL` is exactly:

```text
[Records, LineBytes, ArtifactBytes]
```

`SemanticArtifactLimitsV2` derives `Clone, Copy, Debug, Eq, PartialEq`, stores
private `[usize; 3]` values in declaration order, and exposes only:

```rust
#[must_use]
pub const fn new(
    artifact_bytes: usize,
    line_bytes: usize,
    records: usize,
) -> Self;

#[must_use]
pub const fn limit(self, kind: SemanticArtifactLimitKindV2) -> usize;
```

`SemanticArtifactErrorKindV2` derives `Clone, Copy, Debug, Eq, PartialEq` and
declares `LimitExceeded(SemanticArtifactLimitKindV2)` followed by
`InvalidCompiledDocument`. Its public `ALL` is exactly:

```text
LimitExceeded(Records), LimitExceeded(LineBytes),
LimitExceeded(ArtifactBytes), InvalidCompiledDocument
```

`SemanticArtifactErrorV2` has private construction and exposes only
`#[must_use] pub const fn kind(&self) -> SemanticArtifactErrorKindV2`. It
implements `Display`, redacted `Debug`, and `std::error::Error`. Display is
exactly `limit-exceeded(records)`, `limit-exceeded(line-bytes)`,
`limit-exceeded(artifact-bytes)`, or `invalid-compiled-document`. Debug is
`SemanticArtifactErrorV2(<display>)` and contains no input data.

`SemanticArtifactV2` has private `Box<str>` storage and exposes only
`#[must_use] pub fn as_str(&self) -> &str` and
`#[must_use] pub fn as_bytes(&self) -> &[u8]`. Its redacted Debug is exactly
`SemanticArtifactV2 { bytes: N }`. It has no public constructor, mutation,
parser, `Clone`, `Display`, or frontend conversion.

All five types inherit the V1 auto-trait set: `Send`, `Sync`, `Unpin`,
`RefUnwindSafe`, and `UnwindSafe`. No type adds `Default`, `Hash`, or `Ord`.

```rust
pub fn canonical_semantics_v2(
    compiled: &CompiledAuthoringV2,
    limits: SemanticArtifactLimitsV2,
) -> Result<SemanticArtifactV2, SemanticArtifactErrorV2>;
```

The reference constant is measured only from the completed versioned fixture:
exact artifact bytes including its final LF, exact longest line bytes, and
exact record count.

## Byte grammar and envelope

The artifact is printable ASCII with LF endings and exactly one final LF. The
first physical line is the following single line:

```text
fenestra-authoring-semantics|2|authoring-format=2|schema-format=1|construction-format=1|style-format=1|spatial-format=2|records=N
```

Every remaining line is:

```text
record|A|K|span=A:E|PAYLOAD
```

`A` is the logical anchor ordinal and `E` is exactly `A + 1`. Unsigned values
use normalized base-10 spelling. Signed values have no plus sign, use one
minus sign when negative, and spell zero as `0`. Names are valid authoring
identifiers and therefore require no escaping. `N` excludes the header and is
the exact number of record lines.

The 13 unchanged V1 labels are:

```text
document, schema, component, property, construction, template,
initial-property, static-child, region-child, region, initial-key,
style, style-assignment
```

The 17 appended labels, in anchor-vocabulary order, are:

```text
spatial, resources, image, spatial-node, spatial-container,
spatial-placement, spatial-transform, spatial-field, spatial-shape,
spatial-path-verb, spatial-polygon-point, spatial-brush,
spatial-gradient-stop, spatial-clip, spatial-paint, spatial-hit,
spatial-semantic
```

This is an exact closed vocabulary of 30 record labels.

## Spatial record payloads

The following payload alternatives are exact. `O` is a zero-based table
order. `N`, `S`, and `B` are resolved node, shape, and brush symbols. `H` and
`V` are `start|center|end`. `R` is `non-zero|even-odd`. `C` is
`none|qualified`.

```text
spatial:
format=2|namespace=U64|revision=U32|nodes=COUNT|images=COUNT

resources:
images=COUNT

image:
order=O|name=IDENT|bytes=hex:LEN:HEX

spatial-node:
order=O|name=IDENT|parent=viewport
order=O|name=IDENT|parent=node

spatial-container:
owner=viewport|axis=row
owner=viewport|axis=column
owner=node:N|axis=row
owner=node:N|axis=column

spatial-placement:
node=N|kind=layout
node=N|kind=free|self-anchor=H,V|target=viewport|target-anchor=H,V
node=N|kind=free|self-anchor=H,V|target=parent|target-anchor=H,V
node=N|kind=free|self-anchor=H,V|target=node|target-anchor=H,V

spatial-transform:
node=N

spatial-shape:
node=N|order=O|name=IDENT|kind=rect
node=N|order=O|name=IDENT|kind=circle
node=N|order=O|name=IDENT|kind=polygon
node=N|order=O|name=IDENT|kind=path

spatial-path-verb:
node=N|shape=S|order=O|kind=move-to
node=N|shape=S|order=O|kind=line-to
node=N|shape=S|order=O|kind=quadratic-to
node=N|shape=S|order=O|kind=cubic-to
node=N|shape=S|order=O|kind=close

spatial-polygon-point:
node=N|shape=S|order=O

spatial-brush:
node=N|order=O|name=IDENT|kind=solid
node=N|order=O|name=IDENT|kind=linear-gradient

spatial-gradient-stop:
node=N|brush=B|order=O

spatial-clip:
node=N|order=O|name=IDENT|parent=C|fill-rule=R

spatial-paint:
node=N|order=O|kind=coverage|coverage=fill|fill-rule=R|clip=C
node=N|order=O|kind=coverage|coverage=round-stroke|clip=C
node=N|order=O|kind=image|clip=C

spatial-hit:
node=N|order=O|coverage=fill|fill-rule=R|clip=C
node=N|order=O|coverage=round-stroke|clip=C

spatial-semantic:
node=N|order=O|fill-rule=R|clip=C

spatial-field:
owner=A|role=ROLE|value=VALUE
```

`HEX` contains exactly two lowercase hexadecimal digits per stored byte in
order. For example, `[0, 15, 255]` is `bytes=hex:3:000fff`; empty bytes are
`bytes=hex:0:`. Declaration names occur only on declaration records. Resolved
references use numeric symbols.

## Spatial field values and roles

Every emitted `SpatialFieldV2<T>` produces exactly one `spatial-field` line.
`owner` is the nearest containing spatial record anchor. The closed value-tag
grammar is:

```text
node:N, image:N, shape:N, brush:N, clip:N, template:N,
u8:N, u16:N, u32:N, i32:N,
i32-literal:N, i32-property:N,
fixed16-literal:N, fixed16-property:N,
rgba8-literal:RED,GREEN,BLUE,ALPHA, rgba8-property:N,
input-policy-literal:accept, input-policy-literal:ignore,
input-policy-property:N
```

An `i32-binding`, `fixed16-binding`, `rgba8-binding`, or
`input-policy-binding` means exactly the corresponding literal or property tag
above. Allowed roles and value families are:

```text
image:
  symbol=image; width,height,stride=u32
spatial-node:
  symbol=node; template=template; parent=node
viewport spatial-container:
  left,right,top,bottom,gap=i32
node spatial-container:
  left,right,top,bottom,gap=i32-binding
layout spatial-placement:
  width-minimum,width-preferred,width-maximum=i32-binding
  height-minimum,height-preferred,height-maximum=i32-binding
free spatial-placement:
  width,height=i32-binding; target=node
  offset-x,offset-y=fixed16-binding
spatial-transform:
  a,b,c,d,tx,ty,origin-x,origin-y=fixed16-binding
spatial-shape:
  symbol=shape
  rect: origin-x,origin-y,width,height=fixed16-binding
  circle: center-x,center-y,radius=fixed16-binding
spatial-polygon-point:
  x,y=fixed16-binding
spatial-path-verb:
  move-to,line-to: to-x,to-y=fixed16-binding
  quadratic-to: control-x,control-y,to-x,to-y=fixed16-binding
  cubic-to: control1-x,control1-y,control2-x,control2-y,
            to-x,to-y=fixed16-binding
spatial-brush:
  symbol=brush; solid: color=rgba8-binding
  linear-gradient: start-x,start-y,end-x,end-y=fixed16-binding
spatial-gradient-stop:
  offset=u16; color=rgba8-binding
spatial-clip:
  symbol=clip; parent-owner=node; parent-clip=clip; shape=shape
coverage spatial-paint:
  shape=shape; stroke-width=fixed16-binding; brush=brush; opacity=u8
  clip-owner=node; clip=clip
image spatial-paint:
  image=image; source-x,source-y,source-width,source-height=u32
  destination-x,destination-y,destination-width,destination-height=fixed16-binding
  opacity=u8; clip-owner=node; clip=clip
spatial-hit:
  shape=shape; stroke-width=fixed16-binding
  clip-owner=node; clip=clip; input-policy=input-policy-binding
spatial-semantic:
  shape=shape; clip-owner=node; clip=clip
```

Optional roles appear only when the record discriminant requires them.
Unspanned axes, parent and target kinds, anchors, recipe kinds, fill rules, and
option presence remain on their containing record. The artifact observes the
canonical eight transform fields, not the authored transform convenience.

## Bounds and failure priority

Limits are inclusive. Generation performs these steps in exact order:

1. Compute the record count with checked arithmetic; overflow is
   `InvalidCompiledDocument`.
2. Check `Records` before collecting or allocating record lines.
3. Collect records; reject invalid names, count disagreement, anchor overflow,
   duplicate or gapped anchors, and non-ASCII output as
   `InvalidCompiledDocument`.
4. Sort by anchor and require `record.anchor == ordinal` for every record.
5. Before allocating or appending each line, compute its checked byte length,
   then check `LineBytes`, then cumulative `ArtifactBytes` including its LF.
6. Append exactly one LF per line and finish after the final record.

The checked image line calculation includes `2 * image.bytes().len()`. Exact
limits pass. One under fails. Simultaneous valid-document crossings select
`Records`, then `LineBytes`, then `ArtifactBytes`.

The completed encoder must prove that `records=N`, logical source catalog
bytes, source-map entries, and dense anchor count are equal. The reference
constant uses the exact completed artifact values and is an experiment safety
profile, not a product budget.

## Mutation and fixture controls

The private resolved-model suite must:

- run every existing V1 logical mutation unchanged through V2;
- change each spatial record payload key and closed discriminant independently;
- change every allowed owner-kind, role, and value-tag occurrence;
- change literal payload, property ID, and literal/property choice for all four
  binding families;
- mutate every image byte position, then insertion and removal;
- swap entries in every ordered table: images, nodes, shapes, polygon points,
  path verbs, brushes, stops, clips, paint, hit, and semantic items;
- change logical spans and reject duplicate or gapped anchors;
- reject invalid, non-ASCII, or pipe-containing retained names without
  serializing them into errors;
- compile the committed `.fen` and `ui!` fixture twice each and require all
  four artifacts to equal one committed golden byte-for-byte; and
- prove exact limits pass, each one-under limit fails, record labels and field
  combinations are closed, output is ASCII with one final LF, and privacy
  strings are absent.

Canonical Rust is never used as the semantic oracle.

## Implementation reuse and nonclaims

The existing V1 encoder file is already near the repository file-size limit.
Implementation extracts a neutral logical-record collector, checked record
type, and bounded ASCII writer for both versions. The V1 encoder first moves to
those helpers under its existing golden. V2 invokes the same logical collector
and appends a separate spatial collector; it does not copy the V1 logical
formatting rules.

The V2 surface and orchestration belong in `semantic/v2.rs`; spatial record
collection and value tags belong in focused `semantic/v2/` modules. Private
unit tests own mutation access. Public contract and committed-golden tests stay
in integration tests. Files remain below 400 lines.

This artifact is not an interchange format, cache key, stable ABI,
deserializer input, cryptographic commitment, runtime trace, render oracle, or
replacement for raw-program validation. A future change to any retained
semantic field, record grammar, or line byte is a new semantic artifact
version.
