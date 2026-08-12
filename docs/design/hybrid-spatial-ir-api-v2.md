# WU-0013 symbolic spatial IR API contract

Status: frozen for IR API and validation RED/GREEN
Work unit: WU-0013
Semantic contract: [symbolic spatial IR](hybrid-spatial-ir-v2.md)
Runtime adapter:
[runtime symbolic spatial adapter](hybrid-spatial-runtime-ir-api-v2.md)
Format: symbolic spatial IR API version 2

## Boundary and exact surface

This cut adds the dependency-free raw and validated symbolic spatial program
to `fenestra-ui-ir`. It imports no layout, spatial, runtime, authoring,
testkit, renderer, decoder, or candidate type. It reuses `PropertyId`,
`TemplateNodeId`, `StructuralRegionId`, `SchemaNamespace`, `SchemaRevision`,
`SourceSpan`, `InputPolicy`, `ValidatedStyleProgram`, `IrValidationError`,
`IrValidationErrorKind`, and `ValidationLimitKind`.

The `prototype` surface grows from 57 to exactly 99 exports and from 42 to 70
public structs. The 42 new exports, in registry order, are:

```text
SpatialFormatVersion, SUPPORTED_SPATIAL_FORMAT,
SpatialNodeSymbolV2, SpatialShapeSymbolV2, SpatialBrushSymbolV2,
SpatialClipSymbolV2, SpatialImageSymbolV2,
SpatialFieldV2, SpatialBindingV2, SpatialAxisV2,
SpatialAnchorComponentV2, SpatialFillRuleV2, SpatialNodeParentV2,
SpatialAnchorTargetRecipeV2, SpatialClipAddressV2,
SpatialPointRecipeV2, SpatialPaddingRecipeV2, SpatialDimensionRecipeV2,
SpatialTransformRecipeV2, SpatialViewportContainerV2,
SpatialContainerRecipeV2, SpatialLayoutPlacementRecipeV2,
SpatialFreePlacementRecipeV2, SpatialPlacementRecipeV2,
SpatialPathVerbRecipeV2, SpatialPolygonPointV2,
SpatialShapeGeometryV2, SpatialShapeDeclarationV2,
SpatialGradientStopV2, SpatialBrushContentV2,
SpatialBrushDeclarationV2, SpatialCoverageRecipeV2,
SpatialClipDeclarationV2, SpatialPaintRecipeV2, SpatialHitRecipeV2,
SpatialSemanticRecipeV2, SpatialImageDeclarationV2,
SpatialNodeDeclarationV2, SpatialProgramV2, SpatialValidationLimitsV2,
ValidatedSpatialProgramV2, validate_spatial
```

No public builder, mutable view, iterator type, dense key, raw range, item
ordinal, runtime identity, source map, validation-domain token, or module is
added. Existing public items and method sets otherwise remain unchanged.

## Versions, symbols, fields, and vocabularies

`SpatialFormatVersion` and the five symbol types each privately wrap one
`u32`, expose only `#[must_use] pub const fn new(u32) -> Self` and
`#[must_use] pub const fn get(self) -> u32`, and implement
`Clone + Copy + Debug + Eq + Hash + Ord + PartialEq + PartialOrd`.
`SUPPORTED_SPATIAL_FORMAT` is exactly `SpatialFormatVersion::new(2)`.

```rust
pub struct SpatialFieldV2<T> { /* value, span */ }
impl<T> SpatialFieldV2<T> {
    #[must_use] pub const fn new(value: T, span: SourceSpan) -> Self;
    #[must_use] pub const fn value(&self) -> &T;
    #[must_use] pub const fn span(&self) -> SourceSpan;
}

pub enum SpatialBindingV2<T> {
    Literal(T),
    Property(PropertyId),
}
```

Both generic types derive `Clone + Copy + Debug + Eq + PartialEq` when `T`
supports the corresponding trait. They have no other public item or standard
trait. `SpatialFieldV2` is the only source-bearing leaf wrapper. The four
allowed binding instantiations are `i32`, raw Fixed16 `i64`, `[u8; 4]`, and
`InputPolicy`. Version-2 recipe storage uses exactly those four generic
instantiations; the unconstrained generic enum itself remains usable with
other `T` values. No public aliases are added.

The closed fieldless vocabularies and their exact `ALL` arrays are:

```text
SpatialAxisV2: Row, Column
SpatialAnchorComponentV2: Start, Center, End
SpatialFillRuleV2: NonZero, EvenOdd
```

Each implements `Clone + Copy + Debug + Eq + PartialEq`. The payload enums
`SpatialNodeParentV2`, `SpatialAnchorTargetRecipeV2`,
`SpatialPlacementRecipeV2`, `SpatialPathVerbRecipeV2`,
`SpatialShapeGeometryV2`, `SpatialBrushContentV2`,
`SpatialCoverageRecipeV2`, and `SpatialPaintRecipeV2` have no `ALL`, kind
getter, parallel kind enum, or public constructor. Their variants are
exhaustively matchable.

## Exact recipe storage

In the following inventory, `I` means
`SpatialFieldV2<SpatialBindingV2<i32>>`, `F` means the same over `i64`, `C`
means the same over `[u8; 4]`, and `P` means the same over `InputPolicy`.
These abbreviations are documentation notation, not public aliases. Every
listed struct stores private fields in printed order. It exposes one
`#[must_use] new` with arguments in that order and one `#[must_use]` getter per
field. A struct whose complete storage is Copy has a `pub const fn new`, and
each getter is `pub const fn field(self)`. A struct containing `Vec` or `Box`
has a non-const `pub fn new`; every getter takes `&self`, Copy fields return by
value, non-Copy non-collection fields return `&T`, and `Vec<T>` or `Box<[T]>`
fields return `&[T]`. Scalar, reference, and span getters on those owned
records are `const`; slice getters are non-const. Every `span` getter returns
`SourceSpan`.

```text
SpatialClipAddressV2:
  owner: SpatialFieldV2<SpatialNodeSymbolV2>,
  clip: SpatialFieldV2<SpatialClipSymbolV2>
SpatialPointRecipeV2: x: F, y: F
SpatialPaddingRecipeV2: left: I, right: I, top: I, bottom: I
SpatialDimensionRecipeV2: minimum: I, preferred: I, maximum: I
SpatialTransformRecipeV2:
  a: F, b: F, c: F, d: F, tx: F, ty: F, origin: SpatialPointRecipeV2
SpatialViewportContainerV2:
  axis: SpatialAxisV2,
  left/right/top/bottom/gap: SpatialFieldV2<i32>, span: SourceSpan
SpatialContainerRecipeV2:
  axis: SpatialAxisV2, padding: SpatialPaddingRecipeV2, gap: I
SpatialLayoutPlacementRecipeV2:
  width: SpatialDimensionRecipeV2, height: SpatialDimensionRecipeV2,
  transform: SpatialTransformRecipeV2
SpatialFreePlacementRecipeV2:
  width: I, height: I, self_anchor: [SpatialAnchorComponentV2; 2],
  target: SpatialAnchorTargetRecipeV2,
  target_anchor: [SpatialAnchorComponentV2; 2],
  offset: SpatialPointRecipeV2, transform: SpatialTransformRecipeV2
SpatialPolygonPointV2: point: SpatialPointRecipeV2, span: SourceSpan
SpatialShapeDeclarationV2:
  symbol: SpatialFieldV2<SpatialShapeSymbolV2>,
  geometry: SpatialShapeGeometryV2, span: SourceSpan
SpatialGradientStopV2:
  offset: SpatialFieldV2<u16>, color: C, span: SourceSpan
SpatialBrushDeclarationV2:
  symbol: SpatialFieldV2<SpatialBrushSymbolV2>,
  content: SpatialBrushContentV2, span: SourceSpan
SpatialClipDeclarationV2:
  symbol: SpatialFieldV2<SpatialClipSymbolV2>,
  parent: Option<SpatialClipAddressV2>,
  shape: SpatialFieldV2<SpatialShapeSymbolV2>,
  fill_rule: SpatialFillRuleV2, span: SourceSpan
SpatialHitRecipeV2:
  coverage: SpatialCoverageRecipeV2,
  clip: Option<SpatialClipAddressV2>, input_policy: P, span: SourceSpan
SpatialSemanticRecipeV2:
  shape: SpatialFieldV2<SpatialShapeSymbolV2>,
  fill_rule: SpatialFillRuleV2,
  clip: Option<SpatialClipAddressV2>, span: SourceSpan
SpatialImageDeclarationV2:
  symbol: SpatialFieldV2<SpatialImageSymbolV2>,
  width/height/stride: SpatialFieldV2<u32>, bytes: Box<[u8]>,
  span: SourceSpan
```

Every two-component anchor array is exactly `[horizontal, vertical]`.

`SpatialNodeParentV2` is
`Viewport | Node(SpatialFieldV2<SpatialNodeSymbolV2>)`.
`SpatialAnchorTargetRecipeV2` is
`Viewport | Parent | Node(SpatialFieldV2<SpatialNodeSymbolV2>)`.
`SpatialPlacementRecipeV2` is
`Layout(SpatialLayoutPlacementRecipeV2) |
Free(SpatialFreePlacementRecipeV2)`.

`SpatialPathVerbRecipeV2` is
`MoveTo { to: SpatialPointRecipeV2, span: SourceSpan } |
LineTo { to: SpatialPointRecipeV2, span: SourceSpan } |
QuadraticTo { control: SpatialPointRecipeV2, to: SpatialPointRecipeV2,
span: SourceSpan } | CubicTo { control1: SpatialPointRecipeV2,
control2: SpatialPointRecipeV2, to: SpatialPointRecipeV2,
span: SourceSpan } | Close { span: SourceSpan }` and exposes only
`#[must_use] pub const fn span(&self) -> SourceSpan`.

`SpatialShapeGeometryV2` is
`Rect { origin: SpatialPointRecipeV2, width: F, height: F } |
Circle { center: SpatialPointRecipeV2, radius: F } |
Polygon { points: Vec<SpatialPolygonPointV2> } |
Path { verbs: Vec<SpatialPathVerbRecipeV2> }`.

`SpatialBrushContentV2` is
`Solid { color: C } |
LinearGradient { start: SpatialPointRecipeV2,
end: SpatialPointRecipeV2, stops: Vec<SpatialGradientStopV2> }`.
`SpatialCoverageRecipeV2` is
`Fill { shape: SpatialFieldV2<SpatialShapeSymbolV2>,
rule: SpatialFillRuleV2 } |
RoundStroke { shape: SpatialFieldV2<SpatialShapeSymbolV2>, width: F }`.

`SpatialPaintRecipeV2` is exhaustively:

```text
CoveragePaint {
  coverage: SpatialCoverageRecipeV2,
  brush: SpatialFieldV2<SpatialBrushSymbolV2>,
  opacity: SpatialFieldV2<u8>, clip: Option<SpatialClipAddressV2>,
  span: SourceSpan
}
ImagePaint {
  image: SpatialFieldV2<SpatialImageSymbolV2>,
  source_x/source_y/source_width/source_height: SpatialFieldV2<u32>,
  destination_origin: SpatialPointRecipeV2,
  destination_width/destination_height: F,
  opacity: SpatialFieldV2<u8>, clip: Option<SpatialClipAddressV2>,
  span: SourceSpan
}
```

It exposes only `#[must_use] pub const fn span(&self) -> SourceSpan`. Payload
enum fields use the public visibility implied by an exhaustively matchable
public variant.

`SpatialNodeDeclarationV2` stores, in order,
`symbol: SpatialFieldV2<SpatialNodeSymbolV2>`,
`template: SpatialFieldV2<TemplateNodeId>`, `parent: SpatialNodeParentV2`,
`placement: SpatialPlacementRecipeV2`,
`container: SpatialContainerRecipeV2`,
`shapes: Vec<SpatialShapeDeclarationV2>`,
`brushes: Vec<SpatialBrushDeclarationV2>`,
`clips: Vec<SpatialClipDeclarationV2>`,
`paint_items: Vec<SpatialPaintRecipeV2>`,
`hit_items: Vec<SpatialHitRecipeV2>`,
`semantic_items: Vec<SpatialSemanticRecipeV2>`, then `span: SourceSpan`. Its
constructor and same-named getters follow the rule above.

`SpatialProgramV2` stores, in order,
`format: SpatialFormatVersion`, `schema_namespace: SchemaNamespace`,
`schema_revision: SchemaRevision`,
`viewport_container: SpatialViewportContainerV2`,
`nodes: Vec<SpatialNodeDeclarationV2>`,
`images: Vec<SpatialImageDeclarationV2>`, and `span: SourceSpan`. It exposes
only the constructor and same-named immutable getters. There is no implicit
empty program constructor.

## Traits and ownership

Every recipe struct and payload enum whose complete storage is Copy implements
`Clone + Copy + Debug + Eq + PartialEq`. `SpatialShapeGeometryV2`,
`SpatialShapeDeclarationV2`, `SpatialBrushContentV2`,
`SpatialBrushDeclarationV2`, `SpatialImageDeclarationV2`,
`SpatialNodeDeclarationV2`, and `SpatialProgramV2` contain `Vec` or `Box` and
implement `Clone + Debug + Eq + PartialEq` but not `Copy`. No raw recipe
implements `Default`, `Display`, `Error`, `Hash`, or ordering; the format
version and five symbols are the only exceptions for hash and order. All
values have the normal runtime auto-trait set and are `'static` when their
generic payload is `'static`.

`SpatialValidationLimitsV2` stores one private `[usize; 13]`, implements
`Clone + Copy + Debug + Eq + PartialEq`, and exposes only
`#[must_use] pub const fn new(values: [usize; 13]) -> Self`. The array follows
the limit order below. There is no registered default.

`ValidatedSpatialProgramV2` has private Arc-owned program, lookup, signature,
and domain state. It implements only `Clone`, manual redacted `Debug` as
`ValidatedSpatialProgramV2(..)`, and the runtime auto traits. It exposes:

```rust
#[must_use] pub fn program(&self) -> &SpatialProgramV2;
#[must_use] pub fn style(&self) -> &ValidatedStyleProgram;
#[must_use] pub fn node(&self, symbol: SpatialNodeSymbolV2)
    -> Option<&SpatialNodeDeclarationV2>;
#[must_use] pub fn node_for_template(&self, template: TemplateNodeId)
    -> Option<&SpatialNodeDeclarationV2>;
#[must_use] pub fn region_signature(&self, symbol: SpatialNodeSymbolV2)
    -> Option<&[StructuralRegionId]>;
#[must_use] pub fn shares_domain_with(&self, other: &Self) -> bool;
```

No getter exposes a map, allocation identity, dense key, or source-provenance
table. Cloning shares the exact program and retained style domain;
revalidation creates a distinct spatial domain.

## Validation limits and errors

`ValidationLimitKind` gains, after `StyleAssignments`, exactly:

```text
SpatialNodes, SpatialShapes, SpatialBrushes, SpatialClips,
SpatialPaintItems, SpatialHitItems, SpatialSemanticItems, SpatialPaths,
SpatialPathVerbs, SpatialPolygonPoints, SpatialGradientStops,
SpatialImages, SpatialImageBytes
```

The 13 static counts exclude the synthetic viewport and live keyed
multiplicity. Paths count path-shaped declarations; verbs, polygon points,
gradient stops, and image bytes are checked totals. Equality passes. The first
declaration or nested record that crosses a maximum supplies the error span;
an image-byte failure uses that image record. Checked total overflow is the
same `LimitExceeded` failure. No lookup allocation precedes all 13 checks.

`IrValidationErrorKind` gains these 25 non-limit variants immediately before
the existing generic `LimitExceeded` variant, in this declaration order:

```text
UnsupportedSpatialFormat,
DuplicateSpatialNode, DuplicateSpatialTemplate, MissingSpatialTemplate,
MissingSpatialParent, SpatialParentContextMismatch,
SpatialParentNotEarlier, InvalidSpatialPreorder,
UnknownSpatialProperty, SpatialPropertyTypeMismatch,
SpatialFixed16OutOfRange,
DuplicateSpatialShape, DuplicateSpatialBrush, DuplicateSpatialClip,
DuplicateSpatialImage, MissingSpatialShape, MissingSpatialBrush,
MissingSpatialImage, MissingSpatialClipOwner, MissingSpatialClip,
SpatialClipOwnerNotAncestor, SpatialClipParentNotEarlier,
MissingSpatialAnchorTarget, SelfAnchorTarget,
SpatialAnchorContextMismatch
```

Existing `SchemaIdentityMismatch`, `InvalidSourceSpan`, and
`LimitExceeded(ValidationLimitKind)` are reused. `IrValidationErrorKind::ALL`
grows from 42 to exactly 80 entries. Its existing 42 entries remain the exact
prefix. The appended entries are `UnsupportedSpatialFormat`, the 13 new
`LimitExceeded` values in the limit order above, then the remaining 24
non-limit variants in the declaration order above. Display remains the
existing lowercase hyphenated variant name, with the existing typed limit
suffix and source span; Debug and evidence redaction remain unchanged.

## Validation entry point and priority

```rust
#[must_use = "spatial IR validation errors must be handled"]
pub fn validate_spatial(
    style: &ValidatedStyleProgram,
    program: SpatialProgramV2,
    limits: SpatialValidationLimitsV2,
) -> Result<ValidatedSpatialProgramV2, IrValidationError>;
```

The first failure is selected by these complete phases:

1. program span, format, and schema identity;
2. the 13 counts in printed order;
3. nodes in program order: record and symbol spans, duplicate symbols and
   templates, template resolution, parent resolution and context, earlier
   parent, then exact active-subtree preorder;
4. viewport container, then every node placement, transform, and container
   field or binding in stored order;
5. all node-local shapes, nested polygon points, and path verbs in stored
   order, including local-symbol uniqueness;
6. all node-local brushes and nested stops in stored order;
7. global images in declaration and field order;
8. all clips: symbol, parent owner and clip, ancestry and same-owner order,
   shape, and remaining fields;
9. all paint tables, then all hit tables, then all semantic tables, resolving
   local shapes and brushes, images, qualified clips, and bindings in stored
   order;
10. free-placement explicit node anchor targets in node order: existence,
    self target, then keyed-context compatibility.

An invalid record span precedes its fields. An invalid field span precedes the
field's semantic check. A duplicate node, shape, brush, clip, or image symbol
reports the later symbol field; a duplicate template reports the later
template field. A missing template reports its template field. Parent
existence, context, earlier-order, and preorder failures report the current
node's parent-symbol field. Property lookup, type, and Fixed16 failures report
the containing leaf field. Missing local shape, brush, or image references
report that reference field. A missing clip owner or clip reports the
respective owner or clip field; clip ancestry reports the owner field and
same-owner clip ordering reports the clip field. Missing, self, and
context-incompatible anchor targets report the explicit target-node field.
Fixed16 literals are checked against the canonical raw spatial scalar domain.
Property references resolve through the node's target template and require the
exact binding family.

Negative extents, layout ranges, transform singularity, path and gradient
semantics, image stride, length, pixels and premultiplication, clip depth,
dependency cycles, and every live property value remain raw resolver
authority after runtime expansion.

## RED/GREEN obligations

The API RED freezes the 99/70 registries, private fields, constructors,
getters, enum exhaustiveness, traits, dependency direction, and absence of raw
spatial identities. Validation REDs cover every phase, all limit equality and
one-over cases, sparse symbols, forward anchors, nested keyed signatures,
multifault priority, source spans, private domain sharing, and exact retained
style ownership. Both reviewed RED cuts precede one cohesive minimum IR GREEN;
no placeholder `validate_spatial` implementation is an intermediate GREEN.
