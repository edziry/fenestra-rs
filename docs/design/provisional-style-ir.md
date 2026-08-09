# Provisional style IR plan

Status: complete locally
Work unit: WU-0007
Branch: `feat/provisional-style-ir`
Research baseline: `fenestra-research` commit `176c42139776ed9f1ef879cd135bddadaf12a9da`
Last updated: 2026-08-09

## Research

The ratified
[authoring and runtime boundary](https://github.com/edziry/fenestra-research/blob/176c42139776ed9f1ef879cd135bddadaf12a9da/init/architecture/authoring-style-runtime-boundary.md),
[feasibility spine contract](https://github.com/edziry/fenestra-research/blob/176c42139776ed9f1ef879cd135bddadaf12a9da/init/architecture/feasibility-spine-contract.md),
and [ADR-0001](https://github.com/edziry/fenestra-research/blob/176c42139776ed9f1ef879cd135bddadaf12a9da/init/decisions/ADR-0001-initial-product-direction.md)
require construction and style to remain specialized typed programs linked
through one schema. They also leave matching, precedence, inheritance,
animation, tokens, syntax, storage, and runtime computation open to later
experiments.

WU-0003 already supplies the minimum shared vocabulary: schema namespace and
revision, component-local typed properties, fixed-size values, opaque source
spans, declared invalidation sets, template-local construction targets, and
private immutable validation domains. That vocabulary is sufficient to test
the linker boundary without adding another dependency or selecting final style
semantics.

The smallest honest WU-0007 lane is an exact-template assignment program. It
can prove that a separately validated style artifact links to one exact
construction, resolves registered typed properties, and produces deterministic
replacement values. It cannot prove component matching, user-facing syntax, a
cascade, inheritance, or computed-style performance.

## Owned responsibility

`fenestra-ui-ir` owns these additional unpublished prototype contracts:

- `StyleFormatVersion` and one supported provisional format;
- an unvalidated `StyleProgram` with authored schema identity and ordered exact
  assignments;
- one `StyleAssignment` naming a construction-local `TemplateNodeId`, a
  component-local `PropertyId`, a typed value, and a `SourceSpan`;
- an explicit inclusive assignment-count limit;
- validation against one exact `ValidatedConstruction` domain;
- an immutable linked `ValidatedStyleProgram` with deterministic authored
  iteration and exact lookup;
- receiver-scoped views of the target, property, schema default, replacement
  value, typed value origin, invalidation declaration, and source anchor;
- typed failures that use the offending source anchor and disclose no value or
  validation-domain data.

The construction program remains the owner of template definitions, initial
property values, child order, and structural regions. The schema remains the
owner of property type, default, and invalidation metadata. Style owns only the
ordered exact replacements declared by its own program.

No runtime crate type enters this representation. A style target is a template
factory, not a runtime node, keyed member, fragment, generation, or native
resource.

## Provisional program model

The raw and linked flow is:

```text
ValidatedSchema
  -> ValidatedConstruction

StyleProgram
  -> ordered StyleAssignment records

validate_style(&ValidatedConstruction, StyleProgram, StyleValidationLimits)
  -> ValidatedStyleProgram retaining that exact construction domain
```

Each assignment has one exact key:

```text
(TemplateNodeId, PropertyId) -> PropertyValue
```

`TemplateNodeId` is local to the construction program. An assignment to a
repeat-body template applies to every runtime instance later built from that
factory. V1 cannot address one keyed runtime member independently and does not
interpret ancestry, descendants, parts, states, variants, tokens, or
environmental conditions.

An assignment is valid only when its target template exists, the target's
component declares the property, and the replacement value has the declared
closed type. A target-property pair may appear at most once. Numeric zero and
sparse `u32::MAX` symbols remain valid when they resolve inside their owning
domains.

An empty style program is valid. It still links to the exact construction and
provides the linked schema defaults for every resolved target property.

## Exact replacement semantics

For a resolved target and property, V1 selects the linked style value in this
fixed order:

1. the exact style assignment, when present;
2. the component schema default.

This is one experiment-local replacement layer, not cascade precedence or
specificity. It is also a style-only result: it does not read or override a
construction initial property. The precedence between construction
initialization and style remains open for the headless integration experiment.
Equal replacement and default values are valid and remain observable as an
authored assignment. Validation never mutates the construction, schema, or
runtime state.

Every linked assignment exposes:

- the resolved `TemplateFactory` and `PropertySchemaView`;
- the immutable schema default;
- the immutable style replacement value;
- the property's already-validated invalidation set;
- the authored assignment span.

`ValidatedStyleProgram::linked_value(target, property)` returns a typed linked
view containing the value, invalidation, and a closed `StyleValueOrigin` of
`ExactAssignment` or `SchemaDefault`. Missing targets or properties return
`None`. Assignment iteration preserves authored order. Exact lookup may use a
private map, but map order never becomes output order.

These views are the WU-0007 equivalence surface. Tests can compare complete
typed tuples and linked values with a manual expected result without raw
storage, parser syntax, textual formatting, or runtime state.

## Linking and validation domains

The style header declares the supported style format plus the exact schema
namespace and revision. Both authored identity values must match the schema
retained by the supplied construction.

Successful validation clones and retains the exact `ValidatedConstruction`.
Cloning a validated style output shares its private style domain and the
retained construction domain. Revalidating identical style input against the
same construction creates a new style domain. Validating it against a
separately validated but structurally equal construction links to that other
construction domain.

The raw header does not invent a construction UUID or globally unique template
symbol. Domain identity stays private, process-local, non-serialized, and
absent from diagnostics and traces.

## Invariants and diagnostics

Validation fails closed unless all of these conditions hold:

1. The program span is valid and its format is the supported
   `StyleFormatVersion`.
2. The authored schema namespace and revision match the schema retained by the
   supplied construction.
3. The assignment count is at or below the explicit inclusive limit.
4. Every assignment span is valid.
5. Every assignment target resolves through the supplied construction.
6. Every property resolves through the target template's component schema.
7. Every replacement value matches the resolved property type.
8. No target-property pair appears more than once.

Existing shared failures retain their meaning for style validation:

```text
schema-identity-mismatch
invalid-source-span
```

WU-0007 adds these closed failures to `IrValidationErrorKind`:

```text
unsupported-style-format
missing-style-target
unknown-style-property
style-property-type-mismatch
duplicate-style-assignment
limit-exceeded(style-assignments)
```

`IrValidationErrorKind::ALL` remains exhaustive across all provisional
validators. The malformed corpus must cover every new concrete outcome and the
responsible span instead of weakening the existing exhaustive assertion.

Global validation priority is:

1. program span;
2. supported style format;
3. schema namespace and revision identity;
4. assignment-count limit;
5. assignments in authored declaration order.

Within one assignment, checks use span, target reference, duplicate identity,
property reference, then value type. A duplicate reports the second assignment.
The first authored failing assignment wins even when private lookup tables use
a different physical order.

`IrValidationError` continues to expose only `kind()` and `span()`. Its manual
`Debug` and `Display` must not include property values, private domains, map
indices, source text, host paths, or traversal state.

## Bounded validation

`StyleValidationLimits` contains one explicit `usize` ceiling for total
assignments. The maximum is inclusive. Zero accepts only an empty program.

Validation checks the raw count before allocating the exact lookup table or
walking records. The first record beyond the limit supplies the diagnostic
span even when that record's source range is inverted, matching the existing
construction preflight rule. The input vector is already caller-owned; the
validator does not allocate storage proportional to a numeric ID.

The closed V1 values contain no variable-length string, path, selector, or
expression payload. The assignment count therefore bounds every new validated
record and lookup entry introduced by this unit. These values are experiment
inputs, not ratified product budgets or allocator-memory measurements.

## Immutable output boundary

The provisional entry point consumes its raw program:

```text
validate_style(
  &ValidatedConstruction,
  StyleProgram,
  StyleValidationLimits,
) -> Result<ValidatedStyleProgram, IrValidationError>
```

Validated output retains private immutable shared storage. Public prototype
views do not expose raw input vectors, mutable slices, map entries, unchecked
constructors, dense indices, a domain token, or `Deref` to validated storage.
`Debug` for the validated program is an opaque summary and cannot expose
values or internal ownership.

The API remains documentation-hidden under `prototype`, absent from the
`fenestra-ui` facade, and `publish = false`.

## Tests written before behavior

1. A linked program styles two exact templates and preserves authored
   assignment order.
2. Linked lookup returns a style replacement or schema default with exact typed
   provenance and never treats a construction initial value as style data.
3. Assignment views expose the resolved target and property, default and
   replacement values, origin, invalidation, and source span needed for a manual
   equivalence result.
4. An empty program validates and returns linked schema defaults without using
   construction initial values.
5. Unsupported style format and mismatched schema namespace or revision return
   typed header failures.
6. Invalid program and assignment spans return `invalid-source-span` with the
   exact anchor.
7. Missing targets, unknown properties, wrong value types, and duplicate
   target-property pairs return their typed failures; duplicates use the second
   assignment span.
8. Multiple invalid assignments select the first authored failure, and
   within-record checks follow the documented order.
9. The assignment limit is inclusive, zero accepts an empty program, and the
   first crossing record wins over its invalid span.
10. The same local property ID on different component targets resolves in its
    owning component scope.
11. Sparse maximum target and property IDs validate without proportional
    allocation when the construction and schema declare them.
12. A style clone shares its style and construction domains, while repeated
    validation creates a new style domain and separately validated
    constructions remain distinct.
13. The exhaustive malformed corpus includes every style error kind and limit
    category without removing construction coverage.
14. Error and validated-output formatting expose no values, internal indices,
    source text, host paths, or domain tokens.
15. The crate dependency tree remains empty, and source inspection confirms no
    parser, selector, runtime identity, mutable runtime state, or external
    dependency entered the IR.

## Non-goals and replacement boundary

This unit does not add:

- selector syntax or matching, component scopes, named parts, states, variants,
  tokens, precedence, specificity, cascade, inheritance, animation, transitions,
  interpolation, expressions, or environment dependencies;
- a `.fen` or CSS parser, `ui!` lowering, proc macro, source catalog, code
  generator, serialized wire artifact, reload protocol, or compatibility hash;
- a final declaration of which component properties are publicly stylable;
- runtime nodes, keyed-member targeting, computed-style caches, dependency
  graphs, mutation, invalidation propagation, transactions, layout, scene, or
  renderer state;
- precedence between construction initial properties and style values;
- browser recovery, unknown-property tolerance, arbitrary strings, CSSOM, DOM,
  JavaScript, or a VM;
- a public API, stable format, migration policy, ABI, registry assignment, or
  product capacity.

The exact-target assignment representation is deliberately replaceable. Later
experiments may add scoped match plans and richer schema metadata, but they must
preserve typed linking, deterministic diagnostics, explicit invalidation, and
the specialized construction/style boundary.

## Verification and exit

The unit passes locally only after:

```text
cargo fmt --all -- --check
cargo clippy -p fenestra-ui-ir --all-targets --all-features --locked -- -D warnings
cargo test -p fenestra-ui-ir --all-targets --all-features --locked
RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc -p fenestra-ui-ir --no-deps --locked
```

Workspace tests, dependency direction, `publish = false`, `forbid(unsafe_code)`,
ASCII, diff cleanliness, and the practical 400-line file limit are also
required.

Passing WU-0007 proves only that construction and style are distinct linked,
bounded, hand-authored typed programs and that one exact-target fixture has a
deterministic manual result. It does not prove authoring ergonomics, final style
semantics, runtime computed style, incremental matching, compatibility,
performance, memory consumption, or a feasibility gate.
