# WU-0007 provisional style IR verification

Status: complete locally
Result: pass
Date: 2026-08-09
Branch: `feat/provisional-style-ir`
Research baseline: `fenestra-research` commit `176c42139776ed9f1ef879cd135bddadaf12a9da`

## Research and contract

The owned style representation, exact replacement semantics, construction and
schema linking rules, validation priority, assignment limit, immutable output,
tests, and non-goals are recorded in the
[provisional style IR plan](../design/provisional-style-ir.md). The immutable
research baseline requires construction and style to remain specialized typed
programs linked through one schema, while leaving syntax, matching, precedence,
inheritance, animation, storage, and runtime computation to later experiments.

WU-0007 extends only `fenestra-ui-ir`. It adds no package dependency and changes
no manifest or lockfile. The runtime, testkit, product facade, and headless
probe remain unchanged. The IR crate remains unpublished, dependency-free,
unsafe-forbidden, and exposed only through its documentation-hidden
`prototype` surface.

## TDD and review evidence

The style contract tests were committed before the corresponding behavior.
The retained test artifacts divide responsibility as follows:

- [`valid_style.rs`](../../crates/fenestra-ui-ir/tests/valid_style.rs) verifies
  exact lookup, complete assignment views, schema defaults, style-only
  fallback, empty programs, and typed value origin;
- [`invalid_style.rs`](../../crates/fenestra-ui-ir/tests/invalid_style.rs)
  verifies every style failure and source anchor, exhaustive shared error
  coverage, global and within-assignment diagnostic priority, and opaque error
  formatting;
- [`style_limits_and_domains.rs`](../../crates/fenestra-ui-ir/tests/style_limits_and_domains.rs)
  verifies the inclusive assignment limit, zero capacity, crossing-record
  precedence, authored-order iteration, clone and revalidation domains, exact
  construction retention, component-local property scope, and sparse maximum
  symbols.

The existing malformed construction corpus remains intact and is combined with
the style corpus when checking `IrValidationErrorKind::ALL`. Review of the green
implementation confirmed that it did not weaken or remove a red assertion.
The only post-red test-support adjustment suppresses unused shared helpers in
test binaries that exercise different subsets of the same fixture.

The validator follows the documented order: program span, supported style
format, authored schema identity, assignment count, then assignments in
declaration order. Within one assignment it checks span, target, duplicate
identity, property, and value type. The second duplicate is reported even when
its value also has the wrong type.

## Linked style result

The raw `StyleProgram` carries a supported provisional format, schema namespace
and revision, and ordered exact assignments. Validation consumes that input and
retains the exact supplied `ValidatedConstruction`. A clone shares both private
domains; an independent validation receives a new style domain, and a
structurally equal construction retains its own construction domain.

Each assignment has the exact local key `(TemplateNodeId, PropertyId)`. Its
validated view exposes the resolved template and component-local property,
schema default, replacement, `ExactAssignment` origin, invalidation declaration,
and source span. Authored iteration reads the retained declaration vector. A
private map serves lookup only and cannot affect output order.

`linked_value` selects an exact assignment when present and otherwise selects
the component schema default with `SchemaDefault` origin. It deliberately does
not read a construction initial property. Missing targets or properties return
`None`, while an authored value equal to the schema default remains observable
as an exact assignment.

## Bounds, domains, and privacy

`StyleValidationLimits` has one inclusive `usize` assignment ceiling. Zero
accepts only an empty program. The raw count is checked before allocating the
lookup map or walking assignments, and the first record beyond the ceiling
supplies the error span even when that span is inverted. The validator stores
one retained record and at most one lookup entry per accepted assignment; it
does not allocate storage proportional to a numeric symbol.

The closed property vocabulary adds no variable-length selector, expression,
path, source text, or arbitrary payload. Assignment count bounds the records
introduced by this unit, but it is not an allocator-byte measurement or a
product budget.

Style and construction domains remain private, process-local, and
non-serialized. Public views expose no raw vectors, mutable slices, map entries,
dense indices, or domain token. `IrValidationError` exposes only its closed kind
and opaque source span. Error formatting and the opaque validated-program
summary contain no property value, private domain, lookup index, source text,
or host path.

## Verification

The following final commands passed locally with warnings denied where
required:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --workspace --all-features --no-deps --locked
cargo metadata --format-version 1 --no-deps --locked
cargo tree --workspace --edges normal --locked
git diff --check
```

The workspace gate passed 328 tests: 32 IR tests, 93 runtime tests, and 203
testkit tests. The facade and headless probe contain no tests. No executed test
was ignored or failed. Formatting, Clippy, and rustdoc completed with warnings
denied where applicable.

Metadata confirms that every package remains unpublished. The normal dependency
tree contains only workspace packages; `fenestra-ui-ir` remains dependency-free
and no dependency was added elsewhere. Package manifests and `Cargo.lock` did
not change during WU-0007. Source inspection confirms that no runtime or
testkit type, parser, selector system, mutable runtime state, or facade export
entered the IR.

The complete tracked-workspace ASCII scan and `git diff --check` passed. Every
Rust and Markdown file remains below 400 lines; the workspace maximum is 399.
The largest WU-0007 test is 235 lines, the changed validation error module is
246 lines, the linked-style implementation is 220 lines, and the design plan
is 318 lines. The five crate and probe targets retain `forbid(unsafe_code)`,
while workspace lints deny unsafe code and undocumented unsafe blocks.

## Result

Result: pass for WU-0007's local exact-target linking boundary.

Construction and style are distinct bounded hand-authored programs linked
through the exact supplied construction and its schema. Valid assignment views
and lookup results preserve typed values, provenance, invalidation, source
anchors, authored order, and private validation domains. Every malformed class
returns its documented typed failure and responsible span.

This result supplies the replaceable style input needed by the WU-0008 headless
integration. It is not a pass for EXP-0001, final styling semantics, a public
authoring model, or a framework feasibility gate.

## Limitations and nonclaims

- The style format and all `prototype` types remain unpublished, unstable,
  documentation-hidden, and absent from the product facade.
- Exact template assignments are not selectors, parts, states, variants,
  tokens, specificity, precedence, cascade, inheritance, animation,
  transitions, expressions, or environment-dependent style.
- The linked result is style-only. It does not decide precedence between
  construction initial values and style values.
- There is no parser, `.fen` or CSS syntax, macro lowering, source catalog,
  serialized artifact, reload protocol, compatibility hash, ABI, or migration
  policy.
- No runtime nodes, keyed-member targeting, computed-style cache, dependency
  graph, transaction, invalidation propagation, layout, scene, renderer, GPU,
  native window, or platform behavior is implemented or validated.
- One deterministic hand-authored fixture and malformed corpus are bounded
  correctness evidence, not exhaustive, fuzz, ergonomic, performance, or
  memory-consumption evidence.
- Verification ran on local Linux x86_64 with Rust and Cargo 1.97.1. The
  configured Windows CI lane was not observed for this unit and must pass
  before any cross-platform or Windows execution result is claimed.
- No mobile-target, Miri, benchmark, memory-profile, MSRV, performance, or
  product-support result is claimed.
