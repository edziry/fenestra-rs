# Typed dual authoring plan

Status: active
Work unit: WU-0010
Branch: `experiment/typed-authoring`
Reference fixture: [typed authoring reference](typed-authoring-reference.md)
Research baseline: `fenestra-research` commit `176c42139776ed9f1ef879cd135bddadaf12a9da`
Last updated: 2026-08-09

## Goal

WU-0010 tests whether one `.fen` file and one `ui!` invocation can share one
typed authoring implementation and lower into the same existing schema,
construction program, and distinct style program. Passing means the two routes
share tokens, parsing, name resolution, diagnostics, bounds, lowering, code
generation, and observable runtime behavior. Similar output from two separate
implementations is not sufficient.

The unit remains an unpublished experiment. It does not select final syntax,
stabilize an API, or add authoring behavior to the runtime.

Governing research is the immutable
[authoring and runtime boundary](https://github.com/edziry/fenestra-research/blob/176c42139776ed9f1ef879cd135bddadaf12a9da/init/architecture/authoring-style-runtime-boundary.md),
[EXP-0007 contract](https://github.com/edziry/fenestra-research/blob/176c42139776ed9f1ef879cd135bddadaf12a9da/init/experiments/EXP-0007-typed-dsl-reactive-runtime.md),
and
[authoring alternatives study](https://github.com/edziry/fenestra-research/blob/176c42139776ed9f1ef879cd135bddadaf12a9da/init/architecture/authoring-alternatives-study.md).
The current IR owns format-1 schema, construction, style, spans, and validation;
the runtime consumes validated programs; and the testkit owns the fixture and
oracle. WU-0010 evolves none of those target representations for parsing.

## Version contract

All new workspace packages use package version `0.1.0`, Rust 2024, and
`publish = false`. WU-0010 does not bump the package minor merely because it
adds unpublished packages. Package versions continue to follow
`MAJOR.MINOR.PATCH`: before 1.0, a later intentional compatibility break moves
to the next minor without a backward-compatibility shim, while a compatible
fix uses patch.

Package versioning is independent from authoring, schema, construction, and
style formats, which are each format 1. None is a stability promise; format and
package changes are recorded independently.

## Owned packages and dependency direction

WU-0010 adds these unpublished packages only when their failing tests exist:

```text
probes/exp-0007-typed-authoring build.rs
  -> fenestra-ui-authoring
       -> fenestra-ui-ir
       -> proc-macro2
       -> quote

ui! consumer target
  -> fenestra-ui-macros [host proc macro]
       -> fenestra-ui-authoring [host only]
  -> fenestra-ui-ir [generated constructors]
  -> fenestra-ui-runtime [existing validated execution]
```

`fenestra-ui-authoring` owns both adapters, shared tokens, parser, source map,
lowerer, validation bridge, and emitter, and is host-only. `fenestra-ui-macros`
is a thin `proc-macro = true` wrapper whose sole unpublished surface is `ui!`;
it delegates once and returns tokens or one spanned `compile_error!`.
`fenestra-ui-exp-0007-typed-authoring` owns only disposable fixtures, comparison,
measurements, and artifacts.

No new package is re-exported by `fenestra-ui`. The normal target dependency
graph, inspected with `cargo tree -e normal,no-proc-macro`, must not contain
`fenestra-ui-authoring`, `proc-macro2`, or `quote`. The separate build and
proc-macro graph must show them and no unexpected native or FFI dependency.

## One parser, two token adapters

The implementation flow is fixed:

```text
UTF-8 .fen bytes -> bounded text lexer ---------+
                                                   -> AbstractTokenV1 stream
proc_macro2 TokenStream -> bounded token adapter -+
  -> shared parser
  -> shared name resolution and private ResolvedDocumentV1
  -> raw schema, construction, and style programs
  -> existing IR validators
  -> one ResolvedDocumentV1-to-TokenStream emitter
```

Only the text lexer and `proc_macro2` token adapter are frontend-specific.
They must produce the same closed abstract tokens and canonical labels for
equivalent input. The parser, syntax recovery policy, diagnostic priority,
name tables, type checks, limits, lowerer, and emitter are single
implementations.

`AbstractTokenKindV1` contains only identifiers, unsigned decimal integers,
and the punctuation used by format 1. A leading minus is its own punctuation
token. Keywords are recognized by the shared parser from identifier tokens.
The procedural adapter recursively converts brace, bracket, and parenthesis
groups while checking nesting depth. It rejects delimiter-none groups, raw
identifiers, suffixed or non-decimal literals, strings, characters, floats,
and unsupported punctuation. The text lexer rejects their textual equivalents.
Flattened opening and closing delimiters both count as tokens; root depth is 0.

This narrow token layer makes `syn` unnecessary. There is no second Rust-like
grammar to parse and no Rust syntax tree to retain.

## Technical API boundary

The cross-package API is documentation-hidden under
`fenestra_ui_authoring::prototype`. Fields remain private. The intended shape is:

```text
compile_fen_v1(FenSourceV1, AuthoringLimitsV1)
  -> Result<CompiledAuthoringV1, AuthoringDiagnosticV1>
compile_ui_v1(proc_macro2::TokenStream, AuthoringLimitsV1)
  -> Result<CompiledAuthoringV1, AuthoringDiagnosticV1>
emit_tokens_v1(&CompiledAuthoringV1, AuthoringLimitsV1)
  -> Result<proc_macro2::TokenStream, AuthoringDiagnosticV1>
canonical_rust_v1(&CompiledAuthoringV1, AuthoringLimitsV1)
  -> Result<GeneratedRustV1, AuthoringDiagnosticV1>
diagnostic_tokens_v1(AuthoringDiagnosticV1)
  -> proc_macro2::TokenStream
expand_ui_v1(proc_macro2::TokenStream, AuthoringLimitsV1)
  -> proc_macro2::TokenStream
```

`FenSourceV1` borrows bytes and carries one opaque source ID. Neither compiler
entry point reads files or process state. `CompiledAuthoringV1` retains one
private `ResolvedDocumentV1`, the raw triple constructed from it, one host-only
source map, its 34-byte virtual catalog, and closed evidence counts. The
resolved document is the sole authority used to construct the raw triple, emit
tokens, and produce the exhaustive semantic observation. Raw IR plus source map
is not treated as inspectable because current raw fields are private.

The one emitter accepts the retained resolved document and returns
`proc_macro2::TokenStream`. The macro returns those tokens directly. The `.fen`
lane passes the compiled document to `canonical_rust_v1`, which invokes that
emitter and wraps its canonical spelling plus exactly one LF. It does not
accept an origin-free token stream or add a second formatter. `GeneratedRustV1`
exposes borrowed text while its debug form reports only the byte count.

Generated target code is one expression that constructs the three raw IR
programs through absolute `::fenestra_ui_ir::prototype` paths. It names no
authoring, token, parser, or source-map type. The existing IR validators link
the raw programs before the existing runtime consumes them. There is no target
parser, target lowerer, second style engine, or duplicated mutation path.

After compilation, a probe-local canonical observer combines the retained
resolved observation with current validated and runtime views to inspect every
generated field and behavior. The manually constructed
`HeadlessFixtureV1::build()` is an independent third oracle; neither generated
frontend is the expected result for the other.

The hard-coded IR crate path is an experiment constraint. Dependency rename
discovery is deferred instead of adding another dependency.

## Source spans and host-only source maps

WU-0010 preserves the existing IR `SourceSpan` V1. It does not add line and
column coordinates or change the IR format.

Every semantic record receives an `AnchorKindV1` and source-order ordinal. The
registered logical catalog is exactly `[b'@'; 34]`: one real byte per anchor in
virtual `SourceId::new(0)`. Ordinal `n` addresses that byte:

```text
SourceSpan::Bytes {
    source: SourceId::new(0),
    start: ordinal,
    end: ordinal + 1,
}
```

The inclusive anchor count is checked before `usize` to `u32` conversion.
Equivalent inputs therefore lower to identical valid byte ranges in the same
virtual catalog, not invented ranges in either physical file.

`SourceMapV1` stays on the host and has one entry per logical anchor:

```text
logical SourceSpan + AnchorKindV1 + PhysicalOriginV1
```

For `.fen`, `PhysicalOriginV1` stores the exact half-open UTF-8 byte range in
the supplied source. For `ui!`, it stores the opaque `proc_macro2::Span` used
to emit the compiler diagnostic plus a bounded canonical token label. The
opaque compiler span is neither serialized nor compared for equality.

Anchor token and label selection is exact: document=`format`, schema=`schema`,
component/property/template/region=declared name, construction=`construction`,
initial property=property after `set`, either child=referenced name, initial
key=key literal, style=`style`, and style assignment=property after the dot.

Semantic program comparison erases physical origins. Source-map comparison
checks logical span, anchor kind, authored order, and canonical token label;
frontend tests separately prove exact `.fen` byte ranges and `ui!` compiler
caret placement.

Stable macro spans provide no dependable persistent original byte range.
WU-0010 neither enables `span-locations`, reads Rust sources, derives offsets
from line/column pairs, nor claims nested-expansion ancestry.

## Registered bounds

Every bound is inclusive and has no unbounded default. Exact fixture limits are:

```text
fen_source_bytes=8192, tokens=1024, identifier_bytes=32, nesting_depth=8
components=1, properties=5, templates=4, regions=1, child_slots=3
initial_properties=12, initial_keys=2, style_assignments=2
source_anchors=34, generated_rust_bytes=32768
```

The lowerer also uses existing IR limits
`ValidationLimits::new(1, 5, 4, 1, 3, 12, 2, 3, 5)` and
`StyleValidationLimits::new(2)`. The last two IR limits are template depth 3
and initial instances 5.

All totals use checked arithmetic. `fen_source_bytes` and invalid UTF-8 apply only
to `.fen`; the shared `tokens` limit applies to both frontends. Tokens,
identifiers, and nesting are checked while adapting input.
Record totals and anchors are preflighted before name-table or output
allocation. Sparse numeric IDs use maps; no allocation is sized by the largest
ID. Generated length, including the final LF, is checked before return. Caller-owned
input and allocator failure are not typed retained-memory outcomes.

These are reproducible experiment capacities, not product budgets.

## Closed diagnostics and priority

`AuthoringDiagnosticV1` exposes frontend, closed kind, and this private-field
location enum:

```text
enum DiagnosticLocationV1 {
    Physical(PhysicalOriginV1),
    Anchored { logical: SourceSpan, anchor_kind: AnchorKindV1,
               physical: PhysicalOriginV1 },
}
```

Byte, UTF-8, lexical, limit, and EOF failures before anchor assignment are
`Physical`; later record/semantic/IR failures are `Anchored`. No failure fakes a
logical span. ASCII formatting omits values, absolute paths, and compiler debug.

Closed kinds are `invalid-utf8`, `unsupported-token`,
`unsupported-authoring-format`, `unexpected-token`, `unexpected-eof`,
`invalid-identifier`, `invalid-literal`, `duplicate-{component,property,
template,region}-name`, `unknown-{component,property,template,region}-name`,
`value-type-mismatch`, `limit-exceeded(AuthoringLimitKindV1)`, and
`ir-validation(IrValidationErrorKind)`.

The IR wrapper retains all 42 current concrete validation outcomes and the
responsible logical span. It does not reduce them to a string.

One first error is selected in this order:

1. `.fen` source-byte limit, then UTF-8 validity; neither applies to `ui!`;
2. at each next physical token: unsupported lexical form, shared token-count
   crossing, identifier-byte crossing, then nesting-depth crossing, before
   adapting or examining another token;
3. authoring format;
4. at each record-opening token, components, properties, templates, regions,
   child slots, initial properties, initial keys, style assignments, then
   anchor-count crossing before that payload grammar; otherwise grammar follows
   physical order;
5. duplicate names, unknown names, literals, and value types in authored
   declaration order;
6. schema validation, construction validation, then style validation;
7. generated Rust byte limit including final LF.

A record-count crossing uses the first record beyond the inclusive limit.
Invalid fixtures compare kind plus logical anchor only for `Anchored`; physical
locations are verified per frontend. Format 1 has no multi-error recovery.

## Build integration

The build script joins only `fixtures/layout-board.fen` below
`CARGO_MANIFEST_DIR`, declares that relative `rerun-if-changed`, and writes
canonical token spelling plus one LF only to `OUT_DIR/layout_board_fen_v1.rs`.
Target code includes that fixed output. A failure is rendered only as
`fixtures/layout-board.fen:<start>..<end>:<code>`; no absolute path is exposed.
The build script does not write the source tree, invoke a shell, access the
network, or inspect environment beyond `CARGO_MANIFEST_DIR` and `OUT_DIR`. This
follows Cargo's
[build-script output contract](https://doc.rust-lang.org/cargo/reference/build-scripts.html).

The `ui!` lane performs no file I/O. Procedural macros and build scripts run
with build-machine authority, so their exact dependency graph is evidence and
a supply-chain boundary.

## Dependency screen

Direct versions are exact, not ranges:

| Package | Use | License | Declared Rust | Decision |
| --- | --- | --- | --- | --- |
| `proc-macro2 = 1.0.107` | host token adapter/emitter | MIT OR Apache-2.0 | 1.71 | admit, without `span-locations` |
| `quote = 1.0.47` | spanned diagnostics only | MIT OR Apache-2.0 | 1.71 | admit |
| `trybuild = 1.0.120` | macro diagnostics only | MIT OR Apache-2.0 | 1.88 | gated dev candidate |
| `syn = 3.0.3` | alternative Rust parser | MIT OR Apache-2.0 | 1.71 | reject for WU-0010 |

Authoring pins `proc-macro2` and uses pinned `quote` only for spanned
`compile_error!`; it does not create a second program emitter. Macros has no
`syn`. Exact `trybuild` enters dev-dependencies only after Cargo records and the
plan accepts its added locked closure, features, build scripts, licenses, and
declared Rust version.

`unicode-ident = 1.0.24` is `(MIT OR Apache-2.0) AND Unicode-3.0` and Rust 1.71.
All locked transitives, duplicates, features, scripts, and licenses are gates.

`syn` remains an alternative only if later grammar embeds Rust expressions;
its Rust AST would duplicate this narrow parser today.

All admitted code is host-side and pure Rust, but dependencies may contain
internal unsafe code. Workspace unsafe lints govern owned packages, not their
dependencies. The repository still has no selected project license and no
ratified public MSRV. Rust 1.97.1 remains the development toolchain; this screen
does not make an MSRV or publication claim.

## Test-driven implementation slices

Each behavior slice begins with a focused failing test and records the expected
failure before implementation:

1. Contract and tokens: membership/API smoke tests and paired adapter tests fail
   before packages and equivalent bounded abstract tokens exist.
2. Parser and lowerer: tests fail until one private resolved document produces
   the raw triple, exhaustive observation, logical catalog, and emitter input.
3. `.fen`: UTF-8, syntax, name, type, count, and exact-byte tests fail before
   the bounded text lexer and source map are complete.
4. `ui!`: equivalent faults and `trybuild` caret snapshots fail before the
   token adapter and `compile_error!` mapping are complete.
5. Emission and build: the single token output, canonical LF string,
   clean/no-op/edited `OUT_DIR` builds, and target graph tests fail first.
6. Equivalence: generated validated views, resolved observation, source maps,
   runtime observer, and manual fixture oracle fail first; mutate-one-field
   controls guard against circular or incomplete observation.
7. Style and runtime: tests fail until style stays distinct and paired direct,
   insert, move, update, and remove behavior has identical committed results.
8. Evidence: byte-exact artifacts and Linux/Windows gates fail until the result
   is independently reproducible.

Refactoring follows only while all completed slices stay green. Test support is
split by responsibility; no Rust or Markdown file should exceed 400 lines.

## Evidence and verification

Versioned evidence contains both fixtures, resolved/validated/manual-oracle
observations, both source maps, normalized path/key logs, pinned `trybuild`
stderr, a token-spelling golden checked against `OUT_DIR`, versions and hashes.
It also records environment-qualified clean/no-op/edit build time, host peak
RSS, and target artifact size.

Correctness capacities and exact artifacts are gates. Build time, RSS, and size
remain raw measurements until budgets and environments are ratified.

Linux and Windows fresh checkouts run workspace format, locked Clippy with
warnings denied, all-target/all-feature tests, rustdoc with warnings and missing
docs denied, metadata, target-all tree, and probe `normal,no-proc-macro` tree.

The audit also checks ASCII source, LF artifacts, exact hashes, package
publication locks, dependency direction, dirty diff, and the 400-line limit.

Windows verification specifically covers `PathBuf` joins under `OUT_DIR`,
spaces and backslashes in paths, `include!` path construction, UTF-8 byte
offsets, deterministic LF generated output, normalized `trybuild` stderr, and
the same logical-program artifact hash as Linux. Input byte offsets are measured
as supplied; the compiler does not silently rewrite CRLF before mapping.

Passing these lanes proves build and authoring reproducibility on the named
machines. It does not establish Windows native UI support.

## Non-goals and replacement boundary

WU-0010 does not establish final syntax or ergonomics; public API, ABI,
publication, compatibility, migration, license, MSRV, product capacity,
performance budget, or a feasibility result. It adds no runtime parser, reload,
serialization, VM, browser, DOM, CSSOM, general tree, text, events, slots,
conditions, reactive expressions, async, portals, or custom widgets. It adds no
selector, matching, cascade, precedence, inheritance, token, state, part,
animation, transition, or computed-style system. The mutation script remains a
direct transaction oracle. It does not claim original Rust byte offsets,
nested origins, source discovery, line/column coordinates, renamed IR dependency
support, native backend selection, Windows UI support, mobile, or parity.

The abstract token vocabulary, name-bearing host records, numeric IDs, logical
anchor encoding, exact-target style syntax, and generated constructor shape are
replaceable experiment choices. Any replacement must retain shared frontend
semantics, specialized construction and style programs, closed typed
diagnostics, deterministic source mapping, bounded work, and no target parser.

## Exit

WU-0010 passes only when both fixtures traverse the same parser, lowerer, and
emitter; lower to the exact registered programs; retain distinct typed style;
produce frontend-correct source diagnostics; run the same direct and keyed
behavior; keep compiler dependencies out of target execution; and satisfy the
versioned Linux and Windows evidence contract.
