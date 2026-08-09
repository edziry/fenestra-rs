# WU-0003 provisional typed IR verification

Status: complete locally
Result: pass
Date: 2026-08-08
Branch: `feat/provisional-typed-ir`
Research baseline: `fenestra-research` commit `176c42139776ed9f1ef879cd135bddadaf12a9da`

## Research and planning

The baseline mapping, owned schema and construction responsibilities, limits,
diagnostic precedence, invariants, non-goals, and replacement boundary are
recorded in the
[provisional typed IR plan](../design/provisional-typed-ir.md). The existing
research was sufficient for this unit, and no external dependency was added.

## TDD evidence

The first integration corpus was written before the crate exposed behavior.
The initial run failed as expected:

```text
cargo test -p fenestra-ui-ir --all-targets --all-features --locked
error[E0432]: unresolved import `fenestra_ui_ir::prototype`
error[E0433]: could not find `prototype` in `fenestra_ui_ir`
```

The first implementation validated the corpus, but review found that its
immutable views did not expose enough linked data for a later runtime to
materialize templates. Tests for component properties, effective defaults and
overrides, and property and region invalidation failed before the view surface
was added:

```text
error[E0599]: no method named `component` found for `TemplateFactory`
error[E0599]: no method named `effective_value` found for `TemplateFactory`
error[E0599]: no method named `invalidation` found for `RegionFactory`
```

A separate diagnostic-order review found that sibling depth and mixed static
and repeat-body ownership were processed in the wrong order. Both regressions
failed with the later source span before the iterative walkers and two-stage
ownership registration were corrected:

```text
depth_reports_the_first_authored_sibling_edge ... FAILED
static_owners_are_registered_before_repeat_body_owners ... FAILED
test result: FAILED. 3 passed; 2 failed
```

The final IR suite has 22 integration tests covering:

- all 36 concrete typed error outcomes and their responsible source spans;
- closed property types, defaults, initial overrides, and invalidation rules;
- exact authored namespace and revision linking;
- static and keyed-region ownership, placement, order, roots, and cycles;
- schema and construction validation-domain separation and clone sharing;
- receiver-scoped linked views without raw validated storage access;
- all nine inclusive validation-limit classes and preflight precedence;
- sparse maximum numeric IDs without ID-sized allocation;
- deterministic invalidation and error-kind iteration;
- a 2,048-template iterative ownership chain;
- checked multiplicative expansion whose first authored overflowing edge wins;
- opaque error output without values, internal indices, or domain tokens.

## Verification

The following commands passed locally with Rust 1.97.1:

```text
cargo fmt --all -- --check
cargo clippy -p fenestra-ui-ir --all-targets --all-features --locked -- -D warnings
cargo test -p fenestra-ui-ir --all-targets --all-features --locked
RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc -p fenestra-ui-ir --no-deps --locked
cargo test --workspace --all-targets --all-features --locked
cargo metadata --format-version 1 --no-deps --locked
cargo tree --workspace --locked
```

Observed result: all 22 IR tests and all 42 workspace tests passed. Formatting,
Clippy, and rustdoc completed with warnings denied. Metadata confirms that every
package remains unpublished. The dependency tree confirms that
`fenestra-ui-ir` has no dependencies and does not depend on the runtime.

`git diff --check` and the ASCII scan over the changed code and documents also
passed. `forbid(unsafe_code)` and the compiler gates reject unsafe code. The
400-line file check passed: the largest changed file is the 384-line design
plan, and the largest Rust or test file is the 374-line malformed-fixture
corpus.

## Limitations

- The `prototype` module is documentation-hidden, unpublished, and not
  re-exported by the `fenestra-ui` facade. It is not a stable public API.
- Numeric symbols remain local references. Receiver-scoped views and lifetimes
  constrain access; private `Arc` storage retains and distinguishes domains but
  is not a serialized identity, ABI value, or trace correlation token.
- `SourceId` and byte spans are opaque anchors; this unit does not resolve a
  source catalog, filenames, source contents, or external byte bounds.
- Validation limits are explicit experiment inputs, not ratified product
  budgets. They bound validation preflight, linking, indexes, depth, and initial
  expansion after the caller has already materialized the input vectors. They
  type configured and arithmetic exhaustion, not allocator OOM or process abort
  behavior.
- The scalar, RGBA8, input-policy, key, and invalidation vocabularies are closed
  bootstrap fixtures. They do not establish layout units, color semantics,
  platform capability, final CSS semantics, or stable serialized formats.
- The construction IR has no parser, macro lowering, style program, reactive
  slots, runtime node identities, transactions, scene projection, renderer, or
  platform integration.
- Verification was executed on the local Linux environment. The branch has not
  been pushed, so no Windows CI result is claimed for this unit.
- No performance, memory-consumption, MSRV, mobile-target, or feasibility-gate
  result is claimed.

This pass proves only that one bounded, hand-authored typed construction program
can be validated and consumed through immutable linked views. WU-0004 must
still prove atomic runtime mutation, keyed lifecycle, invalidation accumulation,
and committed-generation behavior.
