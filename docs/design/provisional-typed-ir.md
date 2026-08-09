# Provisional typed IR plan

Status: active
Work unit: WU-0003
Branch: `feat/provisional-typed-ir`
Research baseline: `fenestra-research` commit `176c42139776ed9f1ef879cd135bddadaf12a9da`
Last updated: 2026-08-08

## Research

ADR-0001, the authoring and runtime boundary, and the feasibility spine already
fix the constraints required for this unit:

- construction and style programs are specialized representations linked by a
  shared typed schema, not a DOM or string property bag;
- the schema owns stable component and property symbols, source anchors,
  versioning, and declared invalidation metadata;
- construction owns template nodes, parentage, keys, and structural-region
  descriptors, while runtime identities and mutable state are created later;
- invalidation names are experimental instrumentation labels rather than a
  frozen public flag layout;
- production execution must not require a parser, browser, VM, or JavaScript
  runtime.

That baseline is sufficient for a hand-authored construction fixture. No new
external source or dependency is required. This unit exists separately from
transactions so runtime convenience cannot silently determine the upstream IR.

## Owned responsibility

`fenestra-ui-ir` owns the following unpublished prototype contracts:

- a supported schema format version plus authored schema namespace and revision;
- numeric `ComponentTypeId`, `PropertyId`, `TemplateNodeId`,
  `StructuralRegionId`, and `SourceId` values with explicit lookup scopes;
- half-open byte-range `SourceSpan` anchors with a distinct synthetic variant;
- a closed bootstrap value vocabulary: boolean, signed 32-bit fixture scalar,
  RGBA8, and the exact input-policy values `Accept` and `Ignore`;
- component property schemas containing a type, default value, source span,
  and declared invalidation set;
- one construction program containing template nodes and keyed structural
  regions with declared structural invalidation;
- validation into an immutable representation with deterministic iteration;
- typed validation errors that retain the source span responsible for failure.

The invalidation vocabulary contains `structure`, `style-match`, `intrinsic`,
`layout`, `semantics`, `hit-test`, `paint`, `composition`, and `surface` in one
stable iteration order for experiment artifacts. The representation of the set
is private and replaceable.

`SchemaFormatVersion` describes how to interpret a manifest; it is not schema
identity. A manifest declares a `SchemaNamespace` and `SchemaRevision`.
Component IDs are local to that manifest, and property IDs are local to one
component. Template-node and region IDs are local to one construction program.
`SourceId` is local to the source bundle supplied by the fixture. Numeric zero
is a valid ID, never a sentinel. Local IDs, revisions, and byte offsets use
`u32`; schema namespaces and keyed member keys use `u64`. None are registry
assignments, serialized ABI promises, runtime identities, or trace correlation
IDs.

Raw local IDs are references, not capabilities. `ValidatedSchema` receives a
private validation domain, and `ValidatedConstruction` retains that exact schema
through shared immutable ownership. Borrowed validated references resolve only
through their owning receiver. A raw ID from another manifest or program cannot
be used as a globally unique handle, even when its number is equal.

A concrete source span is the half-open byte range `[start, end)`, and an empty
range is valid. `SourceId` is only an opaque diagnostic namespace token in this
unit: validation does not receive source contents, resolve filenames, or check
an ID or byte offset against an external catalog.

The signed fixture scalar has ordinary `i32` equality and deliberately carries
no units or layout semantics. RGBA8 is four equality-comparable bytes without a
color-space or premultiplication promise. Input policy is fixture data and does
not imply a platform capability result.

## Construction fixture model

The provisional program uses explicit records rather than markup syntax:

```text
SchemaManifest
  -> ComponentSchema
       -> PropertySchema

ConstructionProgram
  -> TemplateNode
       -> initial typed property assignments
       -> ordered ChildSlot::Static or ChildSlot::Region entries
  -> StructuralRegion
       -> owner node
       -> repeat-body template root
       -> ordered initial u64 keys
       -> declared structural invalidation

validate_schema(manifest, limits)
  -> ValidatedSchema

validate_construction(validated_schema, construction, limits)
  -> ValidatedConstruction retaining that exact schema
```

Each node's child-slot vector defines exact order among static children and
regions. Each region's key vector defines exact initial member order. This
makes an empty region and multiple adjacent regions positionable without an
index inferred from unrelated node declarations. A region is placed exactly
once in its declared owner's child sequence and names one validated repeat-body
template root. Each initial key instantiates that template as a direct child of
the owner. A future keyed insert can therefore use the same validated factory
instead of inventing a parallel runtime construction path. Keys are `u64` and
unique only inside one region.

The repeat body may contain static descendants and nested regions. Its template
root belongs to exactly one region definition and is not an initially live node
by itself. Runtime `NodeId` values are minted separately for every instantiated
key.

An initial property assignment identifies a property in the target node's
component schema and supplies a typed value and span. Missing assignments use
the schema default. This is construction initialization, not a style rule or
runtime mutation.

Exactly one root is required by this bootstrap fixture. That is not a global
decision about fragments, portals, multiple windows, or future construction
programs.

## Invariants

Validation fails closed unless all of these conditions hold:

1. The manifest uses the supported `SchemaFormatVersion`. The construction
   header uses the supported `ConstructionFormatVersion` and references the
   manifest's exact authored namespace and revision.
2. Component IDs are unique; property IDs are unique within their component.
3. Every property default matches its declared bootstrap value type and has a
   non-empty invalidation set. A direct property cannot declare `structure` or
   `surface`; those transitions are owned by structural regions and runtime or
   platform inputs respectively.
4. Every initial assignment names a property owned by the node's component,
   has the declared type, and appears at most once per node.
5. Node and region IDs are unique, and every component, static child, region,
   owner, and repeat-body template reference resolves.
6. Every node except the root has exactly one owner through a static child slot
   or repeat-body declaration. There is exactly one program root, and template
   ownership is acyclic.
7. Every region is placed exactly once under its declared owner, and no two
   initial keys of one region are equal. Its invalidation is non-empty, contains
   `structure`, and does not contain `surface`.
8. Every byte span has `start <= end`; synthetic spans need no numeric sentinel.

Manifest and construction headers, component, property, node, initial-property,
child-slot, region, and initial-key records each carry a span. A duplicate
reports the second declaration; a bad reference reports the referencing record;
a root-count error reports the construction header; and a cycle reports the
edge that first closes it.

Validation reports the first error in this deterministic traversal: headers,
components and their properties in declaration order, nodes and their contents
in declaration order, then regions and initial keys in declaration order.
Multiple ownership registers all static edges in node and child-slot order,
then repeat-body ownership in region declaration order. Cycle, depth, and
expansion walks use child-slot order, with a placed region contributing one edge
to its repeat body. Initial keys never add traversal edges. Lookup tables may
accelerate validation but never choose the diagnostic.

Forward references are allowed. Validation may preindex the first declaration
of each ID, but duplicate and reference errors are still emitted only by the
documented declaration-order traversal. A region's repeat body contributes one
template edge regardless of its initial-key count. A missing repeat body or a
cycle through that edge uses the region record span; owner mismatch and repeated
placement use the responsible `ChildSlot::Region` span. Owner mismatch wins when
one slot is also a duplicate placement. Unplaced regions are reported before
node ownership, root count, and cycle validation.

The exact provisional error kinds are:

```text
unsupported-schema-format
unsupported-construction-format
schema-identity-mismatch
invalid-source-span
duplicate-component
duplicate-property
property-default-type-mismatch
empty-property-invalidation
invalid-property-invalidation
duplicate-node
duplicate-initial-property
unknown-initial-property
initial-property-type-mismatch
duplicate-region
missing-component
missing-static-child
missing-region
missing-region-owner
region-owner-mismatch
duplicate-region-placement
missing-region-template
unplaced-region
duplicate-node-owner
invalid-root-count
ownership-cycle
duplicate-region-key
invalid-region-invalidation
limit-exceeded
```

The list is exhaustive but does not define global priority. Within one record,
checks use span, identity, references, then payload order;
the taxonomy above breaks ties inside one category. Tests depend on the typed
error kind and span, not prose formatting. Reachability
is not a separate error: with resolved references and exactly one owner for
every non-root node, a disconnected component is either another root or an
ownership cycle.

`limit-exceeded` carries one of these typed limit kinds: components,
properties, templates, regions, child slots, initial properties, initial keys,
template depth, or expanded initial instances.

## Validation limits

Both validation stages require an explicit `ValidationLimits`; there is no
unbounded default. Every field is `usize`, and a value equal to the limit is
accepted. Limits independently cap these global totals:

- components in one manifest;
- properties summed across all components;
- templates and regions in one construction program;
- child slots and initial properties summed across all templates;
- initial keys summed across all regions;
- template depth and expanded initial runtime instances.

The validator:

- uses maps keyed by declared IDs and never allocates a vector sized by the
  largest numeric ID;
- accepts sparse IDs, including `u32::MAX`, within record limits;
- uses iterative ownership traversal rather than recursive DFS;
- computes nested repeat expansion with checked addition and multiplication;
- reports a typed limit error before exposing a validated output.

Global validation priority is:

1. header spans, supported formats, and authored schema identity;
2. count limits in components, properties, templates, regions, child slots,
   initial properties, then initial keys order;
3. record spans and semantic validation in the declared stage order;
4. region placement, including unplaced regions;
5. node ownership, root count, and cycles;
6. template depth;
7. expanded initial instances.

Count preflight uses only lengths and checked sums before allocating lookup
tables. A count error uses the provided span of the first record that crosses
the inclusive limit, even if that untrusted range is itself inverted; therefore
the limit error wins for that record. Checked count overflow is the same typed
limit error and uses the record whose addition overflowed.

Root template depth is one. Every static-child or repeat-body edge adds one.
The first edge entering depth `limit + 1` supplies the error span; if a zero
limit rejects the root, the construction header supplies it. Traversal is
iterative.

Initial expansion starts with root multiplicity one. A static child inherits its
owner multiplicity. A region repeat body receives `owner multiplicity * initial
key count`. Each instantiated template contributes its multiplicity once, and
the total is the checked sum across the template graph. The maximum is
inclusive. Root overflow uses the construction header, static expansion uses
the responsible `ChildSlot::Static`, and repeat multiplication or addition uses
the region record. Empty regions contribute zero body instances. All arithmetic
uses `checked_add` and `checked_mul` before comparing the limit.

The limits used by tests are reproducible harness inputs, not ratified product
budgets. WU-0004 may impose a smaller runtime transaction capacity without
changing this IR validation contract.

## Non-goals and replacement boundary

This unit does not add:

- runtime `NodeId` or future `FragmentId`, mutable property slots, mutation
  logs, transactions, generations, rollback, or projection state;
- a style program, matching, selectors, precedence, inheritance, tokens,
  animation, or computed style;
- `.fen`, `ui!`, a parser, proc macros, code generation, reactive expressions,
  development reload, or serialized artifacts;
- arbitrary property names, `Any` values, a universal node, DOM mutation, or
  browser recovery behavior;
- component child-type constraints, events, portals, semantics, geometry,
  layout, scene, renderer, GPU, platform, or native-resource types;
- a stable API, ABI, version migration policy, or facade re-export.

The exact prototype entry points consume their unvalidated inputs:

```text
validate_schema(SchemaManifest, ValidationLimits) -> ValidatedSchema
validate_construction(&ValidatedSchema, ConstructionProgram, ValidationLimits)
  -> ValidatedConstruction
```

Validated outputs use private immutable shared storage. A construction retains
the exact `ValidatedSchema` supplied to its linker. Receiver-scoped read-only
views expose component properties, defaults and invalidation, template
components and resolved initial overrides, effective values, ordered child
factories, region owner and repeat-body factories, structural invalidation, and
initial keys. They do not expose raw input records, mutable slices, unchecked
constructors, a validation-domain token, or `Deref` to input storage.

Each successful schema validation creates one private `Arc`-backed schema
domain. Clones share it; revalidating identical input creates another. Each
successful construction validation likewise creates a separate construction
domain and retains a clone of the exact schema `Arc`. Construction clones share
both domains, while a second validation against the same schema creates a new
construction domain. Domains are never public tokens, serialized data, debug
fields, or trace IDs.

`IrValidationError` has private fields plus typed `kind()` and `span()` accessors.
Its manual `Debug` and `Display` show only the semantic error kind and opaque
source anchor, never validation domains, dense indices, map layout, traversal
state, property values, source text, or host paths.

EXP-0007 must validate and evolve these schemas before either authoring route is
accepted. WU-0007 adds a distinct provisional style program before the headless
spine; it does not expand this unit. The closed value vocabulary and numeric IDs
are disposable experiment choices. Non-empty invalidation is required only for
the mutable fixture properties in this unit, not every future schema record.

## Tests written before behavior

1. A minimal root, static child, initial property overrides, and empty and
   populated keyed regions validate into immutable, deterministic records.
2. Unsupported formats and mismatched schema namespace or revision fail with
   typed errors.
3. A table-driven malformed-fixture corpus covers every listed validation kind
   and asserts the offending span, including duplicate and unknown initial
   properties, duplicate region placement, and an unplaced region.
4. Duplicate component, property, node, and region IDs fail.
5. Missing component, static-child, region, owner, and repeat-body references
   fail.
6. Defaults and initial properties with the wrong type fail.
7. Empty invalidation and inverted source spans fail.
8. Missing roots, multiple roots, multiple owners, and ownership cycles fail.
9. Missing, repeated, or mismatched region placement and duplicate keys fail.
10. Invalidation union and iteration cover all nine labels deterministically.
11. Diagnostics retain the offending source anchor.
12. One populated region and one adjacent empty region among static children
    preserve exact placement.
13. Same local property IDs in different components and same template and region
    IDs in separately validated programs do not collide across their scopes.
14. Numeric zero is accepted for every ID type.
15. Two failures in one validation stage select the first declaration-order
    error.
16. Construction rejects a namespace or revision mismatch; separately validated
    schemas with equal local IDs have distinct domains, and construction retains
    the exact schema domain supplied to validation.
17. Sparse maximum numeric IDs validate without proportional allocation.
18. Every validation-limit kind has a failing boundary case; deep or
    multiplicative repeat expansion returns a typed error instead of panicking.
19. Cloning a validated output shares its validation domain and remains
    immutable; a separately validated output receives another domain.
20. Property invalidation rejects `structure` and `surface`; region invalidation
    requires `structure` and rejects `surface`.
21. Error formatting exposes no validation domain, internal index, property
    value, or source text.
22. Depth and expansion diagnostics select the first authored edge, including
    when later siblings would also fail or overflow.
23. Deep static ownership and multiplicative repeat chains complete iteratively
    or return their typed bounded error without recursion or arithmetic wrap.

## Verification and exit

The unit passes locally only after:

```text
cargo fmt --all -- --check
cargo clippy -p fenestra-ui-ir --all-targets --all-features --locked -- -D warnings
cargo test -p fenestra-ui-ir --all-targets --all-features --locked
RUSTDOCFLAGS="-D warnings" cargo doc -p fenestra-ui-ir --no-deps --locked
```

Workspace tests, dependency direction, `publish = false`, `forbid(unsafe_code)`,
ASCII, diff cleanliness, and the 400-line file limit are also required. Passing
this unit proves only that one provisional hand-authored construction fixture
can be validated. It does not prove authoring ergonomics, runtime correctness,
style semantics, compatibility, performance, or any feasibility gate.
