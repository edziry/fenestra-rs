# Typed authoring format-1 reference

Status: active
Work unit: WU-0010
Design: [typed dual authoring plan](typed-dual-authoring.md)
Last updated: 2026-08-09

## Purpose

This document fixes the one grammar and fixture used to test WU-0010. It is a
versioned experiment input, not final product syntax. The `.fen` file contains
`document`. The Rust lane contains the same tokens inside `ui! { document }`.
The wrapper is not part of the shared grammar.

## Lexical contract

`.fen` input must be UTF-8 and otherwise uses only ASCII grammar characters.
There is no byte-order mark, comment, string, character, raw identifier, float,
numeric suffix, numeric base prefix, or underscore inside a number.

Whitespace is ASCII space, tab, carriage return, or line feed and is ignored
between tokens. Physical byte offsets still count every supplied byte,
including CRLF as two bytes.

Identifiers match:

```text
[A-Za-z_][A-Za-z0-9_]*
```

They are at most 32 bytes. Reserved words are the literal words shown in the
grammar. Integer spelling is `0` or a nonzero decimal digit followed by decimal
digits. A signed scalar may have one leading `-`. Range conversion is checked
after parsing the spelling.

The punctuation vocabulary is:

```text
{ } [ ] ( ) : ; , = . -
```

The `proc_macro2` adapter maps delimited groups to the same opening and closing
abstract punctuation and preserves the opaque span of each physical token. It
rejects `Delimiter::None` and any Rust token that the text lexer cannot produce.
Both adapters attach the same canonical label to an equivalent abstract token.

## Grammar

The following EBNF is normative for authoring format 1:

```text
document       = "format" uint ";" schema construction style EOF ;

schema         = "schema" "namespace" uint "revision" uint
                 "{" component { component } "}" ;
component      = "component" ident "=" uint
                 "{" property { property } "}" ;
property       = "property" ident "=" uint ":" value_type "=" value
                 "invalidates" invalidation_set ";" ;

construction   = "construction" "{"
                 template { template } region { region } "}" ;
template       = "template" ident "=" uint ":" ident
                 "{" { initial | child } "}" ;
initial        = "set" ident "=" value ";" ;
child          = "child" ("template" | "region") ident ";" ;
region         = "region" ident "=" uint
                 "owner" ident "repeat" ident
                 "keys" "[" [ uint { "," uint } ] "]"
                 "invalidates" invalidation_set ";" ;

style          = "style" "{" { style_assignment } "}" ;
style_assignment
               = "set" ident "." ident "=" value ";" ;

value_type     = "bool" | "scalar_i32" | "rgba8" | "input_policy" ;
value          = "true" | "false" | [ "-" ] uint
               | "rgba8" "(" byte "," byte "," byte "," byte ")"
               | "accept" | "ignore" ;
invalidation_set
               = "[" invalidation { "," invalidation } "]" ;
invalidation   = "structure" | "style_match" | "intrinsic" | "layout"
               | "semantics" | "hit_test" | "paint" | "composition"
               | "surface" ;

ident          = identifier token not equal to a reserved word ;
uint           = checked unsuffixed base-10 integer token ;
byte           = uint in 0..=255 ;
EOF            = end of the shared abstract token stream ;
```

Context selects the closed value type. A property default must match its
declared `value_type`. Template initial and style values resolve their property
through the named component/template before type checking. Schema namespace is
`u64`; revision and all component, property, template, and region IDs are
`u32`; region keys are `u64`; scalar values are `i32`.

Names are compile-time only. Component names are unique per document, property
names per component, and template and region names in their respective
construction scopes. Numeric IDs retain the current IR scopes and validation.

Declaration order maps directly to existing IR authored order. Template child
statements preserve their position among other child statements. Initial
properties and children become their two existing specialized vectors; their
relative interleaving is not retained or assigned runtime meaning.

## Registered headless fixture

The exact `.fen` source is:

```text
format 1;
schema namespace 8001 revision 1 {
  component fixture = 0 {
    property width = 0: scalar_i32 = 40
      invalidates [layout, semantics, hit_test, paint, composition];
    property height = 1: scalar_i32 = 10
      invalidates [layout, semantics, hit_test, paint, composition];
    property color = 2: rgba8 = rgba8(32, 32, 32, 255)
      invalidates [paint];
    property visible = 3: bool = true
      invalidates [semantics, hit_test, paint];
    property input = 4: input_policy = ignore
      invalidates [hit_test];
  }
}
construction {
  template root = 0: fixture {
    set width = 100;
    set height = 80;
    set color = rgba8(1, 1, 1, 255);
    child template container;
  }
  template container = 1: fixture {
    set width = 80;
    set height = 50;
    set color = rgba8(2, 2, 2, 255);
    child template control;
    child region items;
  }
  template control = 2: fixture {
    set width = 30;
    set color = rgba8(3, 3, 3, 255);
    set input = accept;
  }
  template item = 3: fixture {
    set height = 12;
    set color = rgba8(4, 4, 4, 255);
    set input = accept;
  }
  region items = 0 owner container repeat item
    keys [10, 20]
    invalidates [structure, layout, semantics, hit_test, paint, composition];
}
style {
  set control.color = rgba8(10, 20, 30, 255);
  set item.color = rgba8(80, 90, 100, 255);
}
```

The Rust test fixture copies every token in the complete document above
verbatim as the body of `ui! { ... }`. It contains no comment, include, string
literal, or token-generating helper.

## Exact lowered schema

The schema uses format 1, namespace 8001, revision 1, and component ID 0. Its
five component-local properties are:

| Name | ID | Type | Default | Invalidation |
| --- | ---: | --- | --- | --- |
| width | 0 | scalar_i32 | 40 | layout, semantics, hit_test, paint, composition |
| height | 1 | scalar_i32 | 10 | layout, semantics, hit_test, paint, composition |
| color | 2 | rgba8 | 32,32,32,255 | paint |
| visible | 3 | bool | true | semantics, hit_test, paint |
| input | 4 | input_policy | ignore | hit_test |

## Exact lowered construction

The construction uses format 1 and the same authored schema identity. It has
four templates, one region, three child slots, twelve initial properties, and
two initial keys:

| Template | ID | Initial properties | Ordered children |
| --- | ---: | --- | --- |
| root | 0 | width=100, height=80, color=1,1,1,255 | template container |
| container | 1 | width=80, height=50, color=2,2,2,255 | template control, region items |
| control | 2 | width=30, color=3,3,3,255, input=accept | none |
| item | 3 | height=12, color=4,4,4,255, input=accept | none |

Region `items` has ID 0, owner `container`, repeat body `item`, keys `[10, 20]`,
and invalidation `structure, layout, semantics, hit_test, paint, composition`.

The construction limit is exactly:

```text
ValidationLimits::new(1, 5, 4, 1, 3, 12, 2, 3, 5)
```

## Exact lowered style

The distinct style program uses format 1 and the same schema identity. Its two
authored assignments are ordered:

1. `control.color = rgba8(10, 20, 30, 255)`;
2. `item.color = rgba8(80, 90, 100, 255)`.

Its limit is `StyleValidationLimits::new(2)`. Style lookup retains the existing
`ExactAssignment` versus `SchemaDefault` origin. It does not use a construction
initial value as style fallback and does not move either assignment into the
construction program.

## Logical anchors

The shared parser assigns 34 semantic anchors in source order:

| Ordinals | Anchor kind | Count |
| --- | --- | ---: |
| 0 | document | 1 |
| 1 | schema | 1 |
| 2 | component | 1 |
| 3..7 | property | 5 |
| 8 | construction | 1 |
| 9 | root template | 1 |
| 10..12 | root initial_property | 3 |
| 13 | root static_child | 1 |
| 14 | container template | 1 |
| 15..17 | container initial_property | 3 |
| 18 | container static_child | 1 |
| 19 | container region_child | 1 |
| 20 | control template | 1 |
| 21..23 | control initial_property | 3 |
| 24 | item template | 1 |
| 25..27 | item initial_property | 3 |
| 28 | region | 1 |
| 29..30 | initial_key | 2 |
| 31 | style | 1 |
| 32..33 | style_assignment | 2 |

The virtual source catalog is exactly `[b'@'; 34]` under `SourceId::new(0)`, one
real byte per logical anchor. Ordinal `n` lowers to
`SourceSpan::Bytes(SourceId::new(0), n, n + 1)` and addresses catalog byte `n`.

Exact anchor tokens and canonical labels are: Document=`format`;
Schema=`schema`; Component=declared name after `component`; Property=declared
name after `property`; Construction=`construction`; Template=declared name
after `template`; InitialProperty=property name after `set`; StaticChild and
RegionChild=referenced name; Region=declared name after `region`; InitialKey=key
literal; Style=`style`; StyleAssignment=property name after the dot. Both
frontends record these choices before building raw IR.

The `.fen` source map associates every logical anchor with the exact byte range
of its defining keyword or name token. The macro source map associates it with
the corresponding opaque token span and canonical label. Physical origins do
not enter semantic equality or target output.

## Registered authoring capacities

The inclusive limits for this fixture are:

```text
fen_source_bytes=8192
tokens=1024
identifier_bytes=32
nesting_depth=8
components=1
properties=5
templates=4
regions=1
child_slots=3
initial_properties=12
initial_keys=2
style_assignments=2
source_anchors=34
generated_rust_bytes=32768
```

The source, token, and generated byte ceilings deliberately leave measurement
headroom. The semantic record ceilings are exact. All are experiment inputs,
not product budgets.

## Equivalence observations

`CompiledAuthoringV1` privately retains the authoritative
`ResolvedDocumentV1`. That model constructs the raw triple, drives the sole
token emitter, and produces the exhaustive expected observation. Raw triple
plus map is not an observer because current raw fields are private.

The probe compiles and validates both generated expressions. Its canonical
observer walks the retained resolved model while querying all corresponding
current validated and runtime views. It records formats, identity, order, IDs,
types, values, invalidation, child kinds, region links and keys, style targets,
logical spans, and behavior. `HeadlessFixtureV1::build()` supplies a third,
manual oracle independent of both generated routes. Tests mutate every field
class to prove the combined observer detects emitter or view disagreement.

Fields unavailable through current validated views are covered by the retained
resolved observation plus the byte-exact emitted-token golden. Validators,
views, and runtime cover accepted semantics. `.fen` and `ui!` compare logical
spans exactly; comparison with the manual fixture erases its synthetic spans.

Source-map tests compare anchor kind, ordinal, logical span, and canonical
label. `.fen` tests additionally slice the original bytes at every range.
`trybuild` tests prove `ui!` diagnostics point to the expected token. Opaque
procedural spans are not serialized or compared.

## Registered runtime sequence

Both validated outputs and the manual fixture initialize separate runtimes with
the existing projection spec, surface `120 x 90`, and registered capacity. The
existing observer assigns `NodePathV1` from authored child-slot ordinals:
`Static(authored_slot)` and `Member(region_slot, key)`. `FragmentPathV1` is
`(owner_path, region_slot)`. It never compares process-local `NodeId` or
`FragmentId` values.

The normalized mutation log names paths, keys, indices, values, manifests, and
invalidation. Projection records use paths and stable typed fields. Define
`P=[paint]`, `D=[layout, semantics, hit_test, paint, composition]`, and
`R=[structure, layout, semantics, hit_test, paint, composition]`. The exact
generation and one-record receipt tuples are:

```text
0: []
1: PropertyChanged(root/s:0/s:0, color, rgba8(3,3,3,255), rgba8(20,30,40,255), P)
2: KeyInserted(root/s:0/r:1, 30, final=1, [root/s:0/m:1:30], R)
3: KeyMoved(root/s:0/r:1, 30, old=1, final=2, R)
4: PropertyChanged(root/s:0/m:1:30, height, 12, 14, D)
5: KeyRemoved(root/s:0/r:1, 20, old=1, [root/s:0/m:1:20], R)
```

Generation 0 is initial state; each successful operation publishes generations
1 through 5 with exactly one effective record. The tuple paths use existing
`NodePathV1`/`FragmentPathV1` canonical spelling, never runtime IDs.

After every publication, all three runtimes have equal normalized receipt and
projection logs, relationships, properties, computed style, geometry,
semantics, hit tests, and scene data. The final keys are `[10, 30]`.

This sequence proves equivalent consumption of already compiled programs. It
does not add or prove compiled reactive expressions.

## Deterministic artifacts

The reference outputs are ASCII with LF endings and one final newline:

- span-origin-free canonical program text;
- `.fen` logical-to-byte source map;
- macro logical-anchor and canonical-label map;
- pinned compiler diagnostic stderr cases;
- generated Rust expression golden;
- evidence summary with versions, limits, measured counts, byte lengths,
  dependency facts, and content hashes.

`canonical_rust_v1` accepts the compiled document so output-limit diagnostics
retain its frontend and document origin. Its opaque output exposes borrowed
text; debug reports only the byte count and never the generated expression.

Artifacts contain no absolute path, username, time, source literal disclosure
beyond the checked fixture, opaque span debug output, or process environment.

## Reference nonclaims

Format 1 has no imports, includes, user strings, expressions, Rust blocks,
events, text, slots, conditions, selectors, inheritance, precedence, tokens,
animation, recovery, reload, or serialization. Names and explicit numeric IDs
are fixture devices, not an ergonomics decision. The grammar and anchor order
may be replaced after WU-0010 without a compatibility shim while the package
remains pre-1.0 and unpublished.
