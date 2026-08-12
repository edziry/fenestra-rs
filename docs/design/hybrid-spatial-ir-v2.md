# WU-0013 symbolic spatial IR contract

Status: frozen for IR and runtime-adapter RED/GREEN
Work unit: WU-0013
Parent plan: [hybrid spatial composition](hybrid-spatial-composition.md)
Raw spatial semantics: [hybrid spatial reference](hybrid-spatial-reference-v2.md)
Runtime publication:
[hybrid spatial runtime publication](hybrid-spatial-runtime-api-v2.md)
IR API: [symbolic spatial IR API](hybrid-spatial-ir-api-v2.md)
Runtime adapter:
[runtime symbolic spatial adapter](hybrid-spatial-runtime-ir-api-v2.md)
Package line: `0.2.0`
Format: symbolic spatial program version 2

## Purpose and authority

`fenestra-ui-ir` owns a dependency-free symbolic `SpatialProgramV2`. It links
authored spatial declarations to the existing construction and style domains
without importing layout, spatial, runtime, authoring, testkit, renderer, image
decoder, or candidate types.

The symbolic program is not a serialized raw `SpatialOwnedInputV2`. It contains
no `SpatialNodeKeyV2`, raw content key, range start, range length, item ordinal,
layout-island ordinal, dependency vertex, runtime `NodeId`, candidate handle, or
host path. Those values exist only while runtime materializes one logical
generation.

The existing ownership split remains authoritative:

- construction owns template factories, logical child structure, structural
  regions, initial keys, and construction initial values;
- style owns exact template-property replacements;
- symbolic spatial IR owns an explicit spatial tree, local spatial recipes,
  immutable normalized image resources, and typed owner-local property reads;
- runtime owns live logical identities, keyed members, current property slots,
  generation atomicity, and conversion to pass-local dense spatial tables;
- `fenestra-ui-spatial` owns raw spatial validation and resolution.

Successful symbolic validation proves linkage, scope, ordering, and bounded
static structure. It does not claim that every future live property value or
expanded keyed population will satisfy raw spatial semantics or capacities.

## Version and linkage

The supported spatial program format is exactly `SpatialFormatVersion(2)`.
Schema, construction, and style retain their existing supported format 1
contracts. A raw `SpatialProgramV2` carries the schema namespace and revision
and validates only against one exact `ValidatedStyleProgram` supplied by the
caller.

Validation rejects a spatial format other than 2 and a schema identity that
differs from the schema retained transitively by that style program. A
successful `ValidatedSpatialProgramV2` clones and retains that exact style
validation domain. Revalidating identical records creates a distinct spatial
validation domain; cloning the result shares it. Domain identity remains
private, process-local, nonserialized, and absent from diagnostics and debug
output.

The authoring document envelope advances to format 2 only in the later
authoring slice. Format 2 lowers schema format 1, construction format 1, style
format 1, and spatial format 2 as separate programs. The format-1 authoring
grammar has no implicit empty spatial section and is not reinterpreted as
format 2. Unsupported authoring formats continue to fail closed.

## Symbolic program model

One `SpatialProgramV2` contains, in order:

1. format, schema namespace, and schema revision;
2. one literal viewport-container declaration for the synthetic sentinel;
3. ordered symbolic node declarations;
4. ordered program-global normalized image declarations;
5. one program source span.

The viewport container contains only a literal row-or-column axis, four literal
`i32` padding values, and one literal `i32` gap. It cannot read a property
because the sentinel has no logical owner. Runtime derives its viewport and
`Root` placement from the supplied `SpatialViewportV2`; neither is authored in
IR.

Each symbolic node declaration contains, in order:

- one program-global node symbol and one target `TemplateNodeId`;
- a spatial parent address, either the viewport or another node symbol;
- one closed `Layout` or `Free` placement recipe;
- one child-container recipe;
- ordered node-local shapes, brushes, clips, paint items, hit items, and
  semantic geometry items;
- one node source span.

There is at most one spatial node declaration for a construction template. A
declaration applies to every live logical instance of that template. Templates
may be omitted, so a logical wrapper need not have a spatial node. An empty
node list is valid and materializes only the synthetic viewport sentinel.
Spatial parentage is explicit and independent from logical ownership.

`Layout` placement contains width and height minimum, preferred, and maximum
bindings plus a local transform. `Free` placement contains width and height
bindings, literal self and target anchors, a viewport, parent, or symbolic-node
anchor target, a fixed-point offset, and a local transform. Every node also
owns a literal axis plus bound padding and gap values for layout children.

The closed shape recipes are rectangle, circle, polygon, and path. Polygon
points are nested directly in their shape. Path verbs are nested directly in
their path shape. There is no path symbol or payload range in the IR.

Shapes, brushes, and clips have node-local symbols. Paint, hit, and semantic
items are ordered node-local declarations without authored ordinals. A shape
reference and a paint brush reference are implicitly scoped to the item owner.
Cross-owner shape and brush references are invalid and cannot be encoded by a
validated view.

Brush recipes are deliberately node-local even though the raw spatial brush
table uses global pass-local keys. Solid colors and linear-gradient coordinates
or stop colors may read properties on the owning logical node. Runtime emits a
separate derived brush for every live owner instance. Version 2 performs no
cross-owner sharing, interning, or equality-based deduplication.

A clip address is always owner-qualified as `(node symbol, local clip symbol)`.
This applies to a clip's optional parent and to every terminal clip reference,
so an item may name a clip declared by its own owner or a symbolic spatial
ancestor. A clip's shape remains local to the clip owner. The qualification is
symbolic; it never stores a raw clip key.

Paint retains the raw contract's `CoveragePaint` and `ImagePaint` choices. Hit
retains independent fill or round-stroke coverage plus input policy. Semantic
geometry retains an independent local shape, fill rule, and optional qualified
clip. Presence is structural: opacity zero and input `Ignore` do not remove a
record or make item ordinals sparse.

## Symbol scopes and derived raw tables

Numeric zero is valid for every IR symbol and is never a sentinel. Sparse
symbols, including `u32::MAX`, are valid when their declaration resolves within
the applicable bounded scope.

The exact scopes are:

- node symbols and image symbols are unique within one spatial program;
- a target template appears in at most one node declaration;
- shape, brush, and clip symbols are independently unique within one node;
- polygon points, path verbs, and gradient stops have no symbols outside their
  containing declaration;
- paint, hit, and semantic order is independent within one node.

Runtime derives dense tables only after symbolic validation and live expansion:

- node key zero is the synthetic viewport; non-sentinel node keys follow the
  expanded spatial preorder;
- the logical-node mapping uses that same non-sentinel order;
- shapes, brushes, and clips flatten by expanded node order, then local
  declaration order;
- path records and verbs flatten by the path-shape encounter order; polygon
  points flatten by polygon-shape encounter order; gradient stops flatten by
  gradient-brush encounter order;
- images flatten once in program declaration order;
- paint, hit, and semantic tables flatten by expanded owner order and use their
  independent local declaration positions as dense item ordinals.

Every dense key and every payload range is assigned from the checked flattened
cursor. Authored symbolic numbers do not influence a raw key, range, stack
ordinal, or item ordinal. Equal recipes remain separate records.

## Key contexts and live expansion

Construction validation gives every template one ordered repeat-region
signature. The root signature is empty. A static child retains its owner's
signature; a region repeat body appends that region's `StructuralRegionId`.
Nested regions append in outer-to-inner order. Because construction has one
definition owner per non-root template, this signature is unique.

A live logical instance carries the same signature paired positionally with
its concrete region keys. For a reference from source declaration `S` to node
declaration `T`, `T` is single-valued exactly when `T`'s region signature is a
prefix of, or equal to, `S`'s signature. Runtime resolves it by truncating the
source key context to `T`'s signature length. Equal signatures select the one
live target template instance in that exact context. The format has no key
override, so a reference cannot enter a deeper or divergent region and cannot
select another member of the same keyed region.

Every symbolic spatial parent reference and explicit node anchor target must
satisfy that rule. Parent references also form one tree rooted at the implicit
viewport and the node vector is its explicit preorder: a parent precedes its
children and a completed subtree cannot be reopened. Anchor targets may be
forward declarations. Symbolic validation rejects an explicit node target that
names its own declaration because that defect and its authored reference span
are already known. The spatial dependency validator remains responsible for
cycles involving distinct nodes. `Parent` and `Viewport` anchor forms need no
node-symbol lookup.

Runtime first enumerates the complete live logical tree with a
construction-guided depth-first walk and records each declared-template
instance with its key context. It starts from `BuildView.root()` paired with
the retained construction root factory. A static child slot consumes exactly
one matching live child. A region slot resolves the owner's fragment, consumes
its keyed members in order, appends each concrete key to the context, and
recurses through the repeat-body factory. Each owner must consume its live
children exactly. A missing fragment, template mismatch, parent mismatch, or
unconsumed child is a typed runtime materialization invariant failure. A plain
walk over `BuildView.children()` is insufficient because it loses region and
key boundaries.

Runtime then expands from one implicit viewport instance. For each concrete
spatial parent, it visits symbolic child declarations in program order. For
each child declaration, it selects matching live instances in the recorded
logical order whose resolved parent is that concrete parent; it emits one
child and its complete subtree before the next matching child instance.

Thus repeated parents expand as `parent(k1), subtree(k1), parent(k2),
subtree(k2)`, never as all parents followed by all children. Empty keyed regions
emit no instance. Insert, move, and remove operations change only the live
instances and logical-preorder portions implied by their construction regions;
they do not mutate or reinterpret the symbolic program.

Qualified clip addresses use the same context truncation, then add a spatial
ownership rule. The addressed node must be the current item or clip owner, or
one of its symbolic spatial ancestors. A same-owner parent clip must precede
the child clip locally. An ancestor clip is already earlier after spatial
preorder flattening. These rules make every derived raw clip parent and
terminal reference unambiguous and earlier where the raw contract requires it.

## Property bindings

Bindings are closed leaf values, not a general expression language. They read
only the current property slot of the live logical node represented by the
containing spatial declaration. They cannot read a parent, child, anchor
target, keyed sibling, viewport, resource, runtime identity, geometry result,
or another property indirectly.

The four binding families are:

- `I32`: an `i32` literal or an owner-local `PropertyId` whose schema type is
  `ScalarI32`;
- `Fixed16`: a signed raw `i64` fixed-point literal in the canonical spatial
  scalar domain or an owner-local `ScalarI32` property converted by exact
  multiplication by 65,536;
- `Rgba8`: a four-byte literal or an owner-local property whose schema type is
  `Rgba8`;
- `InputPolicy`: a literal `Accept | Ignore` or an owner-local property whose
  schema type is `InputPolicy`.

`I32` bindings are allowed for layout constraints, free width and height, and
node container padding and gap. `Fixed16` bindings are allowed for free
offsets, affine coefficients, transform origins, shape coordinates and sizes,
polygon points, path points, stroke width, gradient endpoints, and image
destination coordinates and sizes. `Rgba8` bindings are allowed for solid
colors and gradient-stop colors. `InputPolicy` bindings are allowed only on hit
items.

Axis, placement and shape discriminants, anchors, fill rules, opacity bytes,
gradient offsets, image source rectangles, image dimensions, stride, and image
bytes remain literals in version 2. There is no visibility or enabled binding.
There is no arithmetic beyond the exact `ScalarI32` to Fixed16 conversion and
no coercion between the four binding families.

The IR transform is already canonical: six `Fixed16` coefficient bindings in
`a, b, c, d, tx, ty` order plus a bound origin point. It contains no transform
operation list. Authoring conveniences may spell identity, translation, scale,
zero through three clockwise quarter turns, or one affine matrix. The frontend
must lower them immediately to the canonical coefficients, using only exact
zero, one, and sign changes for quarter turns. Property-bound matrix
composition and trigonometry would require a new expression contract and are
outside version 2.

At initialization, style's exact assignment wins, then construction's initial
value, then the schema default. Later builds read the current committed or
transaction-draft property slot produced from that same precedence. Symbolic
validation resolves every property through the declaration's target template
and component and requires the exact family above. Runtime never looks up a
property on the symbolic spatial parent.

Property invalidation metadata remains owned by the schema. Version 2 declares
no independent spatial invalidation set and no precise read-dependency cache;
the current runtime therefore rebuilds after every effective nonempty spatial
transaction as fixed by the runtime publication contract.

## Global image policy

Images are the only program-global spatial resources. Each image declaration
contains a program-global symbol, literal `u32` width, height, and stride, exact
owned bytes, and a source span. The bytes claim the spatial reference format:
top-to-bottom, left-to-right, premultiplied encoded-sRGB `R, G, B, A`.

The symbolic IR contains no encoded file, decoder, URI, filesystem path,
profile, animation, texture, atlas slot, candidate handle, lazy loader, or
readiness state. Decoding and straight-to-premultiplied normalization happen
before construction of this program. Materialization preserves declaration and
byte order without deduplication. The spatial resolver remains the authority
for nonzero extent, canonical stride and length, total pixel limits, and the
premultiplied-channel invariant.

Images are emitted once per program, not once per live node. An image paint
uses its global image symbol. Unreferenced valid image declarations are allowed;
resource presence does not imply a paint item.

## Checked runtime materialization

Materialization has a count pass before a value or allocation pass. The count
pass walks the live expansion and computes, with checked `u128` arithmetic,
the sentinel-inclusive node count and every expanded shape, brush, clip,
paint, hit, semantic, path, verb, point, and stop count. It also checks each
owner-local item count, each nested payload length, global image count and
bytes, and all raw `u32` key, ordinal, and range representability ceilings. It
checks the 12 direct limits in exact `SpatialLimitKindV2::DIRECT_ALL` order by
reusing the spatial package's effective direct-count rule. Runtime does not
duplicate its caller-versus-`u32` maximum calculation.

Topology depth and children, island and layout-record counts, per-path and
per-shape content limits, image edge and pixels, clip depth, per-node item
limits, flattened segments, and dependency cardinality remain in the raw
resolver's existing order. They do not determine how much raw table storage
the mapper allocates, and prechecking them here would change multifault
priority. Static symbolic limits bound unexpanded authored structure only and
never replace runtime `SpatialLimitsV2`.

No raw table, logical mapping, payload buffer, or image copy is allocated until
the shared direct-count preflight and raw representation checks pass. The value
pass then evaluates bindings against the current live owner, assigns dense keys
and cursors in the fixed order above, constructs one owned raw input and
matching logical mapping, and invokes the reference spatial resolver. Every
`i32 * 65,536` product fits `i64`; a dynamic property result outside the
canonical spatial domain is emitted unchanged so the resolver reports its
field-specific typed failure. Raw spatial semantic failures, including
negative extents, invalid transforms, paths, images, clips, anchors, or
dependency cycles, remain typed resolver failures.

Materialization and resolution complete before runtime publishes a generation.
Any count, binding, invariant, or resolver failure preserves the prior logical
and spatial generation exactly.

The current `RuntimeSpatialProgramV2::build` callback is intentionally
infallible and cannot represent materialization preflight failure. Therefore
`ValidatedSpatialProgramV2` does not implement that trait and is not hidden
inside it. The manual callback remains available for raw fixtures and fault
injection. The additive fallible constructor, source attribution, shared direct
count preflight, and runtime error composition are fixed by the
[runtime adapter contract](hybrid-spatial-runtime-ir-api-v2.md).

## Bounded symbolic validation

Validation requires explicit inclusive limits over unexpanded authored
structure. It checks source spans, format and schema linkage, symbols,
construction templates and keyed contexts, explicit spatial preorder, typed
bindings, local references, clip ancestry and ordering, and self anchor
targets. It does not validate live multiplicity or property-dependent spatial
semantics.

The exact raw and validated public values, 13 static limit categories, closed
diagnostic vocabulary, traits, source attribution, and ten-phase first-failure
order are fixed by the
[symbolic spatial IR API contract](hybrid-spatial-ir-api-v2.md). That contract
is authoritative when a mechanical API name is not repeated here.

## Implementation staging

1. Add an API-only RED for dependency freedom, exact symbolic storage, scopes,
   private fields, traits, and absence of raw spatial identities.
2. Add validation REDs for each phase above, including multifault priority,
   exact/one-over limits, sparse symbols, private linked domains, and spans.
3. Implement the minimum raw values and validator in `fenestra-ui-ir` without a
   new dependency.
4. Implement the fallible runtime materializer and connect a validated spatial
   program to atomic runtime publication.
5. Freeze authoring format 2 and lower `.fen` and `ui!` through one shared
   grammar to byte-identical raw schema, construction, style, and spatial IR.
6. Compare manual raw input and both authored lanes after every initial, keyed,
   property, and resize mutation, then record bounded Linux and Windows
   evidence.

The IR API and runtime adapter each begin with an exclusive reviewed RED before
their corresponding GREEN implementation.
