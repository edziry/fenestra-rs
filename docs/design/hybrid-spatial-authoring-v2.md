# WU-0013 hybrid spatial authoring format 2

Status: frozen for authoring RED/GREEN
Work unit: WU-0013
Plan: [hybrid spatial composition](hybrid-spatial-composition.md)
Symbolic IR: [symbolic spatial IR](hybrid-spatial-ir-v2.md)
IR API: [symbolic spatial IR API](hybrid-spatial-ir-api-v2.md)
Prior format: [typed authoring format 1](typed-authoring-reference.md)
Sources and diagnostics:
[format-2 source and diagnostic contract](hybrid-spatial-authoring-source-v2.md)
Reference fixture: [format-2 reference](hybrid-spatial-authoring-reference-v2.md)
Semantic artifact: [format-2 semantic artifact](hybrid-spatial-authoring-semantic-v2.md)

## Boundary and compatibility

Authoring format 2 adds one symbolic spatial program to the existing schema,
construction, and style programs. A successful compilation produces this exact
quadruple, in order:

```text
(SchemaManifest, ConstructionProgram, StyleProgram, SpatialProgramV2)
```

Format 1 and every `*V1` API, diagnostic, limit, source map, emitted triple,
fixture, and artifact remain byte-for-byte unchanged. `compile_fen_v1`,
`compile_ui_v1`, `emit_tokens_v1`, and `expand_ui_v1` accept only format 1.
They do not infer an empty spatial program and do not reinterpret format 2.

The `.fen` and `ui!` lanes still converge before parsing: each adapter produces
the same ordered abstract token stream, canonical token labels, and physical
origins. One shared format-2 parser, resolver, validator bridge, and emitter
consume that stream. The procedural macro reads only the leading format
declaration and dispatches to the corresponding complete V1 or V2 pipeline.
It contains no second grammar or lowering path.

Format 2 adds no file lookup, decoder, URI, renderer, layout, runtime, testkit,
candidate, or platform dependency. The authoring package continues to depend
only on `fenestra-ui-ir`, `proc-macro2`, and `quote`.

## Additive public surface

The following V2 items are added to the unpublished `prototype` surface:

```text
GeneratedRustV2, CompiledAuthoringV2, SourceMapEntryV2, SourceMapV2,
AuthoringDiagnosticKindV2, AuthoringDiagnosticV2,
AuthoringLimitKindV2, AuthoringLimitsV2,
SemanticArtifactLimitKindV2, SemanticArtifactLimitsV2,
SemanticArtifactErrorKindV2, SemanticArtifactErrorV2, SemanticArtifactV2,
FenSourceV2, PhysicalOriginV2, DiagnosticLocationV2,
AnchorKindV2, AuthoringFrontendV2,
SUPPORTED_AUTHORING_FORMAT_V2, REFERENCE_AUTHORING_LIMITS_V2,
REFERENCE_SEMANTIC_ARTIFACT_LIMITS_V2,
compile_fen_v2, compile_ui_v2, emit_tokens_v2, canonical_rust_v2,
canonical_semantics_v2, diagnostic_tokens_v2, expand_ui_v2, expand_ui
```

This is exactly 29 additive exports: the authoring registry grows from 29 to
58 names and from 12 to 23 public structs. No V1 item is removed or renamed.

`SUPPORTED_AUTHORING_FORMAT` remains 1.
`SUPPORTED_AUTHORING_FORMAT_V2` is `AuthoringFormatVersion::new(2)`.
`FenSourceV2`, `PhysicalOriginV2`, `DiagnosticLocationV2`, and
`AuthoringFrontendV2` have the same ownership, privacy, coordinate, and trait
contracts as their V1 counterparts. No conversion between V1 and V2 compiled
documents is public.

```rust
pub fn compile_fen_v2(
    source: FenSourceV2<'_>,
    limits: AuthoringLimitsV2,
) -> Result<CompiledAuthoringV2, AuthoringDiagnosticV2>;

pub fn compile_ui_v2(
    tokens: proc_macro2::TokenStream,
    limits: AuthoringLimitsV2,
) -> Result<CompiledAuthoringV2, AuthoringDiagnosticV2>;

pub fn emit_tokens_v2(
    compiled: &CompiledAuthoringV2,
    limits: AuthoringLimitsV2,
) -> Result<proc_macro2::TokenStream, AuthoringDiagnosticV2>;

pub fn canonical_rust_v2(
    compiled: &CompiledAuthoringV2,
    limits: AuthoringLimitsV2,
) -> Result<GeneratedRustV2, AuthoringDiagnosticV2>;

pub fn canonical_semantics_v2(
    compiled: &CompiledAuthoringV2,
    limits: SemanticArtifactLimitsV2,
) -> Result<SemanticArtifactV2, SemanticArtifactErrorV2>;

pub fn diagnostic_tokens_v2(
    error: AuthoringDiagnosticV2,
) -> proc_macro2::TokenStream;

pub fn expand_ui_v2(
    input: proc_macro2::TokenStream,
    limits: AuthoringLimitsV2,
) -> proc_macro2::TokenStream;

pub fn expand_ui(
    input: proc_macro2::TokenStream,
    v1_limits: AuthoringLimitsV1,
    v2_limits: AuthoringLimitsV2,
) -> proc_macro2::TokenStream;
```

`diagnostic_tokens_v2` mirrors the V1 signature over
`AuthoringDiagnosticV2`. `CompiledAuthoringV2` privately retains the resolved
format-2 model and exposes only borrowed `schema`, `construction`, `style`,
`spatial`, `logical_source_catalog`, and `source_map` getters. `GeneratedRustV2`
exposes only `as_str`. V2 source-map records expose the same four observations
as V1, with `AnchorKindV2` and `PhysicalOriginV2`.

The exact seven-item semantic surface, byte grammar, observation, limits, and
mutation controls are fixed by the
[format-2 semantic artifact contract](hybrid-spatial-authoring-semantic-v2.md).

`AuthoringFrontendV2` is exactly `Fen | UiMacro`. Its `ALL` order is printed
order. V2 errors, maps, opaque generated output, and compiled output retain the
same redaction and auto-trait policy as their V1 equivalents.

## Lexical contract

Format 2 reuses the format-1 lexical contract unchanged. Input is UTF-8 and
the grammar itself is ASCII. Identifiers, unsigned decimal tokens, signed
decimal composition, whitespace, punctuation, group adaptation, opaque macro
spans, and rejection of strings, floats, comments, suffixes, raw identifiers,
base prefixes, and numeric underscores remain exactly as specified by the
format-1 reference.

No encoded image literal or file path is added. Image bytes are an explicit
bounded list of decimal bytes.

The V1 reserved-word set remains byte-for-byte unchanged. The V2 reserved-word
set is the union of that V1 set and every quoted identifier literal in the
format-2 productions below. An identifier used as a name may not equal any
member of the set for its selected format.

## Complete grammar

The format-1 productions `schema`, `construction`, `style`, `value_type`,
`value`, `invalidation_set`, `ident`, `uint`, `byte`, and `EOF` are reused
without change. Format 2 adds these normative productions:

```text
document_v2 = "format" "2" ";" schema construction style spatial EOF ;

spatial = "spatial" "format" "2" "{"
          viewport resources { node } "}" ;
viewport = "viewport" "container" axis
           "padding" "(" signed_i32 "," signed_i32 ","
                             signed_i32 "," signed_i32 ")"
           "gap" signed_i32 ";" ;
resources = "resources" "{" { image } "}" ;
image = "image" ident "{"
        "width" uint ";" "height" uint ";" "stride" uint ";"
        "bytes" "[" [ byte { "," byte } ] "]" ";" "}" ;

node = "node" ident ":" ident "{"
       container ";" placement ";" transform ";"
       { shape } { brush } { clip }
       { paint } { hit } { semantic } { node } "}" ;

container = "container" axis
            "padding" "(" i32_binding "," i32_binding ","
                              i32_binding "," i32_binding ")"
            "gap" i32_binding ;
axis = "row" | "column" ;

placement = "placement" "layout"
            "width" dimension "height" dimension
          | "placement" "free"
            "width" i32_binding "height" i32_binding
            "self_anchor" anchor_pair
            "target" anchor_target
            "target_anchor" anchor_pair "offset" point ;
dimension = "dimension" "(" i32_binding "," i32_binding ","
                               i32_binding ")" ;
anchor_pair = "anchor" "(" anchor_component "," anchor_component ")" ;
anchor_component = "start" | "center" | "end" ;
anchor_target = "viewport" | "parent" | "node" ident ;

transform = "transform" transform_kind "origin" point ;
transform_kind = "identity"
               | "translate" "(" point ")"
               | "scale" "(" fixed_binding "," fixed_binding ")"
               | "quarter_turn" "(" uint ")"
               | "affine" "(" fixed_binding "," fixed_binding ","
                                  fixed_binding "," fixed_binding ","
                                  fixed_binding "," fixed_binding ")" ;
point = "point" "(" fixed_binding "," fixed_binding ")" ;

shape = "shape" ident shape_body ;
shape_body = "rect" "{"
             "origin" point ";" "width" fixed_binding ";"
             "height" fixed_binding ";" "}"
           | "circle" "{"
             "center" point ";" "radius" fixed_binding ";" "}"
           | "polygon" "{" { point_record } "}"
           | "path" "{" { path_verb } "}" ;
point_record = point ";" ;
path_verb = "move_to" point ";" | "line_to" point ";"
          | "quadratic_to" point point ";"
          | "cubic_to" point point point ";" | "close" ";" ;

brush = "brush" ident brush_body ;
brush_body = "solid" "{" "color" color_binding ";" "}"
           | "linear_gradient" "{"
             "start" point ";" "end" point ";"
             { gradient_stop } "}" ;
gradient_stop = "stop" uint color_binding ";" ;

clip = "clip" ident "{"
       "parent" optional_clip ";" "shape" ident ";"
       "fill_rule" fill_rule ";" "}" ;
optional_clip = "none" | clip_address ;
clip_address = ident "." ident ;
fill_rule = "non_zero" | "even_odd" ;

coverage = "fill" ident "rule" fill_rule
         | "round_stroke" ident "width" fixed_binding ;
paint = "paint" "coverage" "{"
        coverage ";" "brush" ident ";" "opacity" byte ";"
        "clip" optional_clip ";" "}"
      | "paint" "image" "{"
        "image" ident ";"
        "source" "(" uint "," uint "," uint "," uint ")" ";"
        "destination" point fixed_binding fixed_binding ";"
        "opacity" byte ";" "clip" optional_clip ";" "}" ;
hit = "hit" "{" coverage ";" "clip" optional_clip ";"
      "input" input_binding ";" "}" ;
semantic = "semantic" "{" "shape" ident ";"
           "fill_rule" fill_rule ";" "clip" optional_clip ";" "}" ;

i32_binding = signed_i32 | "property" ident ;
fixed_binding = "fixed" "(" signed_i64 ")" | "property" ident ;
color_binding = "rgba8" "(" byte "," byte "," byte "," byte ")"
              | "property" ident ;
input_binding = "accept" | "ignore" | "property" ident ;
signed_i32 = [ "-" ] uint checked as i32 ;
signed_i64 = [ "-" ] uint checked as i64 ;
```

The four padding bindings are left, right, top, and bottom. The two anchor
components are horizontal then vertical. Dimensions are minimum, preferred,
and maximum. `quadratic_to` takes control then destination. `cubic_to` takes
control 1, control 2, then destination. Image source values are x, y, width,
and height. Image destination values are origin, width, and height.

`quarter_turn` accepts only 0 through 3 clockwise. Authoring immediately
lowers transform conveniences to the canonical affine coefficients and origin.
It uses exact raw Fixed16 zero, 65,536, and sign changes; it performs no float,
trigonometric, or property-bound matrix composition.

For `S = 65,536`, the exact `(a,b,c,d,tx,ty)` lowering is:

```text
identity:          ( S, 0, 0, S, 0, 0)
translate(x,y):    ( S, 0, 0, S, x, y)
scale(x,y):        ( x, 0, 0, y, 0, 0)
quarter_turn(0):   ( S, 0, 0, S, 0, 0)
quarter_turn(1):   ( 0, S,-S, 0, 0, 0)
quarter_turn(2):   (-S, 0, 0,-S, 0, 0)
quarter_turn(3):   ( 0,-S, S, 0, 0, 0)
affine(a,b,c,d,x,y):( a, b, c, d, x, y)
```

Coordinates are y-down and positive quarter turns are clockwise. Direct
translate, scale, and affine operands preserve their binding. Derived constant
coefficient spans and labels follow the source contract linked above.

## Names, ownership, and symbolic expansion

The first node identifier is a program-global spatial node name. The identifier
after `:` is a construction template name. A format-2 program may select a
subset of templates, may be empty, and has at most one spatial declaration per
selected template. Top-level nodes have viewport parentage. Nested nodes have
their containing node as spatial parent. Nesting order is the authored symbolic
preorder and is lowered to the flat IR parent references in that order.

Names do not enter the IR. Node and image symbols are assigned dense zero-based
`u32` values in their respective authored preorder and resource declaration
order. Shape, brush, and clip symbols are assigned dense zero-based values in
declaration order within each owning node. These assignments are deterministic
compiler data, not persistent spatial identities.

Images and node names are program-global. Shapes, brushes, and clips are local
to one node declaration; the same local name may appear under another node.
Shape and brush references are unqualified and resolve in the current node.
Every clip reference is owner-qualified, including a same-owner reference.
Clip owners must be the current node or a symbolic spatial ancestor. A
same-owner clip parent must have been declared earlier.

Each selected template has the ordered structural-region signature fixed by
the construction validator. A source-to-target node reference is single-valued
only when the target signature is a prefix of the source signature or the two
signatures are equal. Spatial parent signatures must have that same relation.
Runtime expands the nested symbolic forest once per matching keyed context,
recursing through a parent's complete subtree before the next matching parent.
Empty regions emit no instance. Authoring never emits raw keys, ranges,
ordinals, or runtime identities.

## Binding and literal rules

Bindings resolve relative to the current node's construction template and
component, never its spatial parent. An `i32_binding` property must be
`ScalarI32`. A `fixed_binding` property must be `ScalarI32` and runtime converts
it by exact checked multiplication by 65,536. A color property must be `Rgba8`.
An input property must be `InputPolicy`. `Bool` is unused by spatial format 2.

Fixed literals are raw signed i64 ticks and must lie in the canonical
`SpatialScalarV2` domain. Literal dimensions, coordinates, colors, policies,
enums, opacity, image metadata, and resource bytes are not reactive. There is
no visibility or enabled binding. Present construction instances and authored
items remain structurally present even at zero opacity or input `ignore`.

## Limits, sources, and diagnostics

The exact 28 compiler limits, 30 anchor roles, 134 concrete diagnostics,
`SpatialFields` accounting, logical-to-physical mapping, numeric failure split,
IR-limit bridge, and first-failure order are fixed by the
[format-2 source and diagnostic contract](hybrid-spatial-authoring-source-v2.md).

## Emission, macro dispatch, and acceptance

The emitter walks the one retained resolved V2 model and constructs all four
raw programs. It emits fully qualified `fenestra_ui_ir::prototype` paths,
owned vectors/boxed byte slices, explicit enum variants, exact signed integer
spellings including `i64::MIN`, deterministic commas, and no frontend origin.
`.fen` and `ui!` emission must be token-identical and canonical Rust must be
ASCII with LF endings and exactly one final newline.

The version-neutral `expand_ui` recognizes only the exact initial abstract
tokens `format`, unsuffixed decimal `2`, and `;` as the V2 route. It sends that
complete stream to `expand_ui_v2`. Every other stream, including empty,
malformed, suffixed, incomplete, format 1, and an unsupported version, is sent
unchanged to `expand_ui_v1`. This preserves every existing macro diagnostic and
does not trial-compile one grammar after another.

The authoring RED/GREEN sequence is:

1. freeze the exact V2 surface, traits, limit vocabulary and derivation,
   diagnostics, and V1 non-change;
2. freeze the complete grammar, literal bounds, source anchors, and failure
   priority across both frontends;
3. freeze semantic lowering for every recipe branch and owner-local scope;
4. freeze deterministic quadruple emission and macro format dispatch;
5. add the separate versioned reference fixture, semantic artifact, and final
   measured authoring and artifact limit constants;
6. prove manual, `.fen`, and `ui!` quadruples and runtime behavior equivalent
   across initial state, resize, property writes, keyed insert/move/remove,
   typed failure, and rollback.

Format 2 makes no claim about encoded assets, expressions beyond the four
bindings, conditional participation, intrinsic sizing, renderer choice,
filesystem build policy, mobile lifecycle, density, safe areas, surface
ownership, or multi-scene behavior.
