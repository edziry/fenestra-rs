# WU-0013 format-2 authoring sources and diagnostics

Status: frozen for authoring RED/GREEN
Work unit: WU-0013
Grammar: [hybrid spatial authoring format 2](hybrid-spatial-authoring-v2.md)
IR API: [symbolic spatial IR API](hybrid-spatial-ir-api-v2.md)
Format-1 source model: [typed authoring format 1](typed-authoring-reference.md)
Reference fixture: [format-2 reference](hybrid-spatial-authoring-reference-v2.md)
Semantic artifact: [format-2 semantic artifact](hybrid-spatial-authoring-semantic-v2.md)

## Limits

`AuthoringLimitKindV2::ALL` is exactly:

```text
FenSourceBytes, Tokens, IdentifierBytes, NestingDepth,
Components, Properties, Templates, Regions, ChildSlots,
InitialProperties, InitialKeys, StyleAssignments,
Images, ImageBytes, SpatialNodes, SpatialFields, Shapes, Paths, PathVerbs,
PolygonPoints, Brushes, GradientStops, Clips, PaintItems, HitItems,
SemanticItems, SourceAnchors, GeneratedRustBytes
```

`AuthoringLimitsV2` privately stores `[usize; 28]`, exposes only
`#[must_use] pub const fn new([usize; 28]) -> Self` and
`#[must_use] pub const fn limit(self, AuthoringLimitKindV2) -> usize`, and uses
inclusive limits. The versioned format-2 fixture fixes the first 27 entries
from its recorded source and authored counts before parser RED. Static record
limits equal or deliberately bound those counts, and every deliberate headroom
choice is recorded beside its measurement. The final `GeneratedRustBytes`
entry is the exact canonical-Rust byte length, including its one final LF. It
is measured only after the emitter and its mutation controls pass. The final
`REFERENCE_AUTHORING_LIMITS_V2` constant lands with that emitter cut; earlier
tests use explicit local limits. These are compiler safety limits, not runtime
expansion limits or product budgets.

The bridge constructs `SpatialValidationLimitsV2` in this exact order:

```text
SpatialNodes, Shapes, Brushes, Clips, PaintItems, HitItems, SemanticItems,
Paths, PathVerbs, PolygonPoints, GradientStops, Images, ImageBytes
```

Each value comes from the same-named V2 authoring limit. `SpatialFields` is
authoring-only and has no IR validation-limit slot.

The construction validator receives `Components` through `InitialKeys`
one-to-one. Template depth and initial expanded instances are graph-derived
validation resources rather than authored record counts. Format 2 uses
separate private `REGISTERED_TEMPLATE_DEPTH_V2` and
`REGISTERED_INITIAL_INSTANCES_V2` constants fixed by its versioned reference
fixture. It preserves failures for those resources as their exact
`IrValidation(LimitExceeded(...))` outcomes. It neither exposes duplicate
authoring-limit variants nor reuses the V1 private constants. The style
validator receives `StyleAssignments`.

No allocation is sized from a caller-supplied maximum. Counts are accumulated
with checked arithmetic while authored records are visited. `FenSourceBytes`
and `SourceAnchors` additionally use an effective maximum of
`min(caller_maximum, u32::MAX as usize)` because physical byte ends and logical
anchor ends are `u32`; the last accepted anchor has ordinal one below the
count. A checked overflow or first unrepresentable record reports the same
corresponding `LimitExceeded` kind and offending physical origin as a normal
one-over crossing. Dense node, image, shape, brush, and clip symbols are
checked `u32` conversions from authored order only after their category and
source-anchor claims pass. The source-anchor ceiling therefore also proves
every reachable dense symbol index representable. No sparse authored numeric
ID controls allocation.

## Spatial field accounting

`SpatialFields` counts every `SpatialFieldV2<T>` emitted into the raw program:

- viewport left, right, top, bottom, and gap;
- image symbol, width, height, and stride;
- node symbol, template, and each non-viewport parent reference;
- every bound padding, gap, dimension, free size, anchor target, offset,
  affine coefficient, and transform-origin coordinate;
- every shape, brush, and clip symbol and every spanned geometry, color,
  gradient offset, clip owner, clip name, or shape reference;
- every paint, hit, and semantic reference, bound width, destination
  coordinate, source rectangle value, opacity, color, and input policy.

A convenience transform still emits six affine fields. A nested node parent
emits one field even though nesting derives its value. A qualified clip address
emits its owner and clip fields independently. Record spans, enum
discriminants, `None`, anchor component literals, fill rules, raw image bytes,
and program spans do not count as fields.

Fields are claimed atomically with their `SpatialField` anchor while parsing in
physical source order. For several fields derived from one token, their tie
order is raw field order. The first field that would make the checked count
exceed the inclusive limit reports `LimitExceeded(SpatialFields)` at its shared
or direct physical origin. Checked count overflow selects the same field and
same diagnostic. Successful lowering must prove that this count equals the
number of `SpatialFieldV2<T>` values in the emitted raw program.

## Anchor vocabulary and logical spans

`AnchorKindV2::ALL` is the 13 V1 anchors in unchanged order followed by:

```text
Spatial, Resources, Image, SpatialNode, SpatialContainer,
SpatialPlacement, SpatialTransform, SpatialField, SpatialShape,
SpatialPathVerb, SpatialPolygonPoint, SpatialBrush, SpatialGradientStop,
SpatialClip, SpatialPaint, SpatialHit, SpatialSemantic
```

The parser appends anchors in physical source order. The 13 reused record kinds
follow their unchanged V1 rules. The spatial walk appends the `Spatial` record,
viewport container record and fields, `Resources`, each image record and
fields, then each top-level node subtree recursively in grammar order. Record
anchors precede any fields originating at the same token. Fields sharing one
token use raw constructor order, except that fields sourced by later explicit
operands are appended only when those operands are parsed. Raw constructor
fields may therefore reference logical spans whose ordinals are not adjacent.

Anchor ordinal `n` always lowers to
`SourceSpan::bytes(SourceId::new(0), n, n + 1)`. The logical source catalog is
exactly `[b'@'; anchor_count]`. Every raw record span and every emitted field
span is one registered non-document anchor. Punctuation has no anchor. Image
bytes share the image record span and add no per-byte anchor.

## Physical origins and canonical labels

Record anchors use these physical tokens and labels:

```text
Spatial=spatial, Resources=resources, Image=image name,
SpatialNode=node name, SpatialContainer=container,
SpatialPlacement=placement, SpatialTransform=transform,
SpatialShape=shape name, SpatialPathVerb=verb keyword,
SpatialPolygonPoint=point, SpatialBrush=brush name,
SpatialGradientStop=stop, SpatialClip=clip name,
SpatialPaint=paint, SpatialHit=hit, SpatialSemantic=semantic
```

A direct field uses the exact authored value or reference token and its
canonical spelling. A property binding uses the property name token and name.
A node, image, shape, brush, or clip symbol/reference uses its name token and
name. Each side of a qualified clip address uses its own name token and name.
An unsigned literal uses its decimal token and normalized decimal spelling. A
positive signed literal does the same. A negative signed literal uses the full
minus-through-decimal byte range in `.fen`, the minus token span in `ui!`, and
the normalized signed decimal spelling in both source maps. `fixed(...)` uses
the contained signed literal by the same rule. `rgba8` channels retain separate
field anchors only when the target IR stores separate fields; one bound IR
color field instead uses the `rgba8` keyword span and normalized complete
`rgba8(r,g,b,a)` label.

Point record anchors use the `point` token. Their x and y fields use their own
binding tokens. A nested-node parent field, derived from grammar nesting, uses
the child node-name token and label. Top-level viewport parentage emits no
field. An unspanned enum or kind uses its containing record span rather than a
new anchor.

Transform field origins are exact:

- affine operands map one-to-one to `a,b,c,d,tx,ty`;
- translate x and y map to `tx` and `ty`; its four matrix constants map to the
  `translate` token;
- scale x and y map to `a` and `d`; its four constants map to `scale`;
- all six identity constants map to `identity`;
- all six quarter-turn constants map to its decimal turn token;
- origin x and y map to their own binding tokens.

When several logical anchors share one physical token, they retain distinct
ordinals and identical frontend origin and label. This is intentional and does
not collapse source-map entries.

At a node-name token, anchors are appended as node record, node symbol, then a
derived parent field when nested; the template field is appended at the later
template-name token. At a transform convenience keyword, only coefficient
fields derived from that keyword are appended there in `a,b,c,d,tx,ty` order;
direct operand fields are appended at their later operand tokens. A
quarter-turn derives all coefficients from its later decimal token. These
rules make source order and raw field provenance simultaneously exact.

## Numeric failures

Malformed decimal spelling, primitive conversion outside `u8`, `u16`, `u32`,
`i32`, `i64`, or `u64`, and a quarter-turn value outside 0 through 3 produce
`InvalidLiteral` at the responsible field before raw IR validation. A
well-formed raw i64 fixed literal outside the canonical Fixed16 domain is
lowered unchanged and produces
`IrValidation(SpatialFixed16OutOfRange)` at that field. Property names and
types are resolved before IR construction: absence uses `UnknownPropertyName`
and a family mismatch uses `ValueTypeMismatch`. Dynamic ScalarI32-to-Fixed16
domain failure remains a runtime resolver diagnostic, not an authoring error.

Image bytes have no field anchors. An invalid byte reports `InvalidLiteral`
anchored to its containing `Image` record, with the exact bad byte token as
the diagnostic physical origin. `LimitExceeded(ImageBytes)` reports the same
image record and the first byte whose inclusion crosses the aggregate limit;
checked aggregate overflow selects that byte by the same rule. No byte anchor
is created.

The parser validates a `quarter_turn` decimal as a primitive `u32` and then as
the closed range 0 through 3 before claiming any derived coefficient field.
Either failure is `InvalidLiteral` anchored to the containing
`SpatialTransform` record with the decimal token as physical origin. A valid
turn then claims all six `a,b,c,d,tx,ty` fields in order. Field or source-anchor
limit failure during that claim reports the first crossing coefficient, so an
invalid turn never leaves a partial coefficient-anchor sequence.

## Diagnostic vocabulary and order

`AuthoringDiagnosticKindV2` has, in order, the first 16 non-payload V1 kinds,
then `DuplicateSpatialNodeName`, `DuplicateSpatialImageName`,
`DuplicateSpatialShapeName`, `DuplicateSpatialBrushName`,
`DuplicateSpatialClipName`, `UnknownSpatialNodeName`,
`UnknownSpatialImageName`, `UnknownSpatialShapeName`,
`UnknownSpatialBrushName`, and `UnknownSpatialClipName`, followed by every
`LimitExceeded(AuthoringLimitKindV2::ALL)` and every
`IrValidation(IrValidationErrorKind::ALL)`. Its `ALL` contains exactly
`16 + 10 + 28 + 80 = 134` concrete outcomes in that order.

The exact unchanged 16-kind prefix is:

```text
InvalidUtf8, UnsupportedToken, UnsupportedAuthoringFormat, UnexpectedToken,
UnexpectedEof, InvalidIdentifier, InvalidLiteral, DuplicateComponentName,
DuplicatePropertyName, DuplicateTemplateName, DuplicateRegionName,
UnknownComponentName, UnknownPropertyName, UnknownTemplateName,
UnknownRegionName, ValueTypeMismatch
```

Every V2 authoring-limit label is the lowercase hyphenated spelling of its
printed variant: `fen-source-bytes`, `tokens`, `identifier-bytes`,
`nesting-depth`, `components`, `properties`, `templates`, `regions`,
`child-slots`, `initial-properties`, `initial-keys`, `style-assignments`,
`images`, `image-bytes`, `spatial-nodes`, `spatial-fields`, `shapes`, `paths`,
`path-verbs`, `polygon-points`, `brushes`, `gradient-stops`, `clips`,
`paint-items`, `hit-items`, `semantic-items`, `source-anchors`, and
`generated-rust-bytes`.

The ten new name diagnostics use these exact labels in printed order:

```text
duplicate-spatial-node-name, duplicate-spatial-image-name,
duplicate-spatial-shape-name, duplicate-spatial-brush-name,
duplicate-spatial-clip-name, unknown-spatial-node-name,
unknown-spatial-image-name, unknown-spatial-shape-name,
unknown-spatial-brush-name, unknown-spatial-clip-name
```

A duplicate spatial node, image, shape, brush, or clip name reports the later
declaration's symbol `SpatialField` anchor, not its containing record anchor.
An unknown name reports the exact reference field: explicit anchor target or
qualified clip owner for `UnknownSpatialNodeName`, image-paint image for
`UnknownSpatialImageName`, clip/coverage/semantic shape for
`UnknownSpatialShapeName`, coverage-paint brush for
`UnknownSpatialBrushName`, and the clip half of a qualified address for
`UnknownSpatialClipName`. In a qualified clip address, owner resolution runs
first: a missing owner reports the owner field and prevents clip-name lookup;
an existing owner with a missing clip reports the clip field. Existing
`UnknownTemplateName` reports the node's template field.

V2 diagnostic storage, getters, exact labels, redaction, `Debug`, `Display`,
`Error`, and source behavior mirror V1 with V2 vocabulary and locations.
Compilation selects the first failure in this order:

1. `.fen` source bytes, UTF-8, or macro token adaptation;
2. token, identifier, and delimiter-depth limits in physical order;
3. envelope format, grammar, authoring record/field limits, and source anchors
   in authored order, delaying numeric conversion until its semantic field;
4. schema name resolution and schema validation;
5. construction name resolution and construction validation;
6. style name resolution and style validation;
7. images in declaration and printed field order;
8. spatial node, template, parent, and keyed-context resolution in preorder;
9. each node's container, placement, transform, shapes, brushes, clips, paint,
   hit, semantic, and nested-node fields in printed order;
10. `validate_spatial` with the exact 13 mapped spatial limits;
11. generated Rust bytes during emission.

IR failures retain their exact kind and map the returned logical span to its
registered V2 anchor and frontend physical origin. The bridge accepts only a
registered one-byte source-0 non-document span; disagreement is a private
compiler invariant, never fabricated user input.

## Canonical semantic artifact

The exact surface, byte grammar, spatial record and field encodings, bounded
failure priority, reference measurement, and mutation controls are fixed by
the [format-2 semantic artifact contract](hybrid-spatial-authoring-semantic-v2.md).
