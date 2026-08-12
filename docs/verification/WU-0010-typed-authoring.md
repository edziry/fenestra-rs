# WU-0010 typed dual authoring verification

Status: complete
Result: pass
Date: 2026-08-10
Branch: `experiment/typed-authoring`
Evidence source commit: `c451645`
Windows CI source commit: `c2ed6f0`
Research baseline: `fenestra-research` commit
`176c42139776ed9f1ef879cd135bddadaf12a9da`

## Scope and version contract

The [typed dual authoring design](../design/typed-dual-authoring.md) and
[format-1 reference](../design/typed-authoring-reference.md) fix this work
unit's ownership, grammar, bounds, source mapping, dependency screen, runtime
script, exit criteria, and non-goals.

WU-0010 adds two unpublished host packages and one disposable probe. The
packages remain at `0.1.0`, Rust 2024, and `publish = false`. Package versioning
follows the ratified `MAJOR.MINOR.PATCH` policy: before 1.0, a later intentional
incompatible contract advances `MINOR` and may remove the old behavior without
a compatibility shim. The authoring, schema, construction, style, map,
semantic, and runtime artifact formats remain separate version-1 contracts.

This unit evaluates one low-level fixture language and macro spelling. It does
not publish, stabilize, or re-export either authoring package through the
facade.

## TDD and review evidence

The branch retains a focused RED before each new behavior and the corresponding
small GREEN implementation. The sequence covers:

- the bounded compiler vocabulary and preflight before the host compiler;
- the exact `.fen` fixture and 34 source anchors before the parser and lowerer;
- physical culprit origins and authored diagnostic order before their fixes;
- numeric domains, IR validator mapping, and invalid-span invariants;
- the UI token adapter, shared lexical boundary, numeric boundaries, limits,
  and opaque origins;
- typed token emission and canonical Rust before the one shared emitter;
- `OUT_DIR` generation, procedural expansion, the registered limit profile,
  and the thin `ui!` macro;
- expression-safe macro diagnostics before the semicolon-free
  `compile_error!` expansion;
- runtime equivalence before the generated macro lane entered the existing
  headless runtime;
- generated, map, semantic, runtime, and evidence artifacts before each
  canonical result;
- limit-priority and mutate-one-field controls before the final artifact
  encoders were accepted; and
- typed logical-state, receipt, surface, and projection faults before replacing
  string-level artifact mutations.

Independent read-only audits rejected incomplete contracts during development,
including borrowed native-style presentation seams, non-source-correct macro
offsets, eager diagnostic priority, incomplete semantic observation, and
string-level runtime artifact faults. The final runtime evidence applies 34
typed defects before the adapter and checks atomicity in both the typed lane log
and rendered model.

No test was weakened, skipped, or removed to admit the implementation.

## Shared compiler and dependency boundary

`fenestra-ui-authoring` owns two bounded adapters and one private compiler:

```text
.fen bytes -> text lexer --------+
                                  -> abstract tokens -> parser -> resolved model
proc_macro2 tokens -> UI adapter +
  -> one name/type/literal pass
  -> raw schema + construction + distinct style
  -> existing IR validators
  -> one resolved-model token emitter
```

Only token adaptation differs. Both routes share parsing, semantic priority,
name resolution, literal conversion, type checks, lowering, logical anchors,
IR validation, semantic observation, and emission. The private parsed and
resolved records are not public API.

The generated target expression uses absolute `fenestra_ui_ir` constructors.
It contains no parser, authoring, proc-macro, source-map, or build-time type.
The `fenestra-ui-macros` crate exposes only `ui!` and delegates once to the
shared expansion function. The `.fen` build script reads only the registered
fixture, writes only `OUT_DIR/layout_board_fen_v1.rs`, and invokes no network,
shell, or source-tree write.

The probe target tree with proc macros excluded contains only
`fenestra-ui-ir`. Authoring, proc-macro2, quote, and the macro implementation
remain in build or host graphs. Runtime and testkit are probe dev dependencies.

## Diagnostics and source maps

Every authored semantic record receives one unit byte in virtual source 0 and
one closed anchor kind. The logical catalog is exactly 34 `@` bytes. Equivalent
frontends therefore emit identical raw IR spans.

The `.fen` source map retains exact half-open UTF-8 byte ranges and an opaque
source ID. The UI source map retains the compiler span only in memory and never
serializes or invents Rust-file offsets. The two committed map artifacts have
the same six logical fields for all 34 rows; only the FEN rows add source ID and
physical byte range.

The CRLF contract compiles an equivalent in-memory fixture with every LF
replaced by CRLF. Its raw programs and canonical output remain identical, while
every later physical anchor advances by one byte per preceding carriage return.
The compiler therefore counts bytes as supplied and does not normalize source
coordinates. Generated Rust remains ASCII with one LF.

Closed diagnostics cover input, token, grammar, name, literal, type, limit, and
all forwarded IR validation failures. Display and Debug omit source contents,
paths, private maps, and compiler-span internals. Three pinned trybuild cases
verify the exact stable-toolchain caret for unsupported punctuation, an unknown
component, and the nesting-depth crossing. Valid macro expansion is an
expression, not a statement fragment.

## Semantic and runtime equivalence

The FEN build lane, UI macro lane, and independent manual fixture each validate
the exact same schema, construction, and distinct style programs. The semantic
artifact walks the retained resolved model, not the raw programs, and records
all 34 source-ordered declarations with explicit collection order, names, IDs,
types, values, invalidation, references, and logical spans. Exhaustive private
patterns and field-class faults prevent an incomplete observer from passing a
re-blessed golden.

Each of the three validated style programs then enters a fresh existing
headless runtime. Generation 0 is observed before five transactions:

1. control color changes from styled `rgba8(10,20,30,255)` to
   `rgba8(20,30,40,255)`;
2. key 30 is inserted at index 1;
3. key 30 moves from index 1 to index 2;
4. member 30 height changes from 12 to 14; and
5. key 20 is removed from index 1.

The three lanes independently produce generations 0 through 5, exact typed
receipts, complete normalized logical state, and all five headless projection
families. Every generation also matches a clean independent headless oracle.
Final keys are `[10,30]`. Runtime handles are resolved only inside one lane and
are never compared or serialized; artifacts use authored node and fragment
paths.

The runtime artifact contains 6 receipts, 5 mutations, 2 manifest entries, 33
node records, 165 properties, 18 child groups, 6 fragments, 15 members, 33
computed records, 33 geometries, 6 semantic records, 21 hit records, and 33
scene records. Typed faults cover receipt and mutation identity, all normalized
state field families, surface, and the five projection families before the
actual adapter encodes them.

## Versioned artifacts

All rows below are printable ASCII plus one final LF. SHA-256 identifies exact
versioned bytes; it is not an authenticity mechanism.

| Artifact | Bytes | LF lines | Max line | SHA-256 |
| --- | ---: | ---: | ---: | --- |
| `.fen` fixture | 1,350 | 47 | 77 | `8308bb19961812d8ec793469c3783c92e63489bbbcbc0e143d24dcac1c983ec6` |
| `ui!` fixture | 1,546 | 49 | 81 | `dd52b908919cb2eb5ddb61ecc10f43e5f43de744347652700700137f0ba3e2b8` |
| generated Rust | 13,793 | 1 | 13,792 | `b633d1f01c0da43827925ae245e095a02aa87a8229d870a1cbee9f2f5adf42de` |
| FEN map | 1,600 | 36 | 51 | `3f480eb8021dcca110e2b20bd715e7f2231cb9ea06f981b11859c5ab279a8323` |
| UI map | 1,250 | 36 | 47 | `baba40d517ee79dcc102c6562bfb9d2053924feee4d815c9b1e8f8b2fd5fc224` |
| semantic | 3,020 | 35 | 157 | `013d2c66d9858db5a5bcfb0b62fb060b373a081b20f19e27ea433864445a3871` |
| runtime | 26,400 | 478 | 111 | `8e3dc45ff29ed49ee9426cb0b907e11665fb7dc3335bd25f8a0179ec5325d9fa` |
| evidence summary | 4,723 | 36 | 250 | `a810b24d82b78e6dd93a9ac09e380bdcdc1bd72d888f789c0900b0fd8fe40050` |

The [evidence summary](../../probes/exp-0007-typed-authoring/tests/artifacts/layout-board-evidence-v1.txt)
also pins limits, counts, the three diagnostic snapshot hashes, `Cargo.lock`,
host dependency facts, and raw Linux workflow measurements. It deliberately
does not include its own hash.

## Dependencies, safety, and publication

Direct host admissions are proc-macro2 1.0.107 and quote 1.0.47, both MIT OR
Apache-2.0 with declared Rust 1.71. unicode-ident 1.0.24 is transitive,
`(MIT OR Apache-2.0) AND Unicode-3.0`, and declares Rust 1.71. trybuild 1.0.120
is macro-dev-only, MIT OR Apache-2.0, and declares Rust 1.88.

The trybuild closure adds glob, itoa, serde_json, serde_spanned,
target-triple, termcolor, TOML support, and zmij. Their exact versions,
licenses, declared Rust versions, and build scripts were inspected from locked
Cargo metadata. `syn 3.0.3` appears only through the dev-only serde derive
closure; it is not a direct or normal authoring dependency. The normal
authoring graph has no native or FFI package.

Owned crates use `forbid(unsafe_code)` and contain no unsafe block. proc-macro2,
quote, unicode-ident, and other upstream packages remain part of the host
supply-chain safety surface. Build scripts and procedural macros have build
machine authority. No security, sandbox, redistribution, or support conclusion
is made.

All nine workspace packages remain `publish = false`. The project still has no
selected license, public MSRV, reserved registry namespace, support matrix, or
ratified product capacity.

## Linux measurement and verification

The final Linux gate ran on Fedora 43 x86_64, kernel
`7.1.5-101.fc43.x86_64`, rustc and Cargo 1.97.1, LLVM 22.1.6, GNU Time 1.9,
and coreutils 9.7.

One detached worktree and dedicated empty target directory measured
`cargo build -p fenestra-ui-exp-0007-typed-authoring --locked` in debug mode.
GNU Time reported elapsed wall seconds and peak resident KiB. Target size is
the probe `.rlib`. A one-byte blank-line edit to the FEN fixture forced its
build script while preserving the generated-Rust hash.

| Case | Elapsed seconds | Peak RSS KiB | Probe rlib bytes |
| --- | ---: | ---: | ---: |
| clean | 1.27 | 195,320 | 285,646 |
| immediate no-op | 0.08 | 48,280 | 285,646 |
| one source edit | 0.11 | 86,264 | 285,646 |

These are environment-qualified observations, not pass/fail budgets or product
performance claims.

The following commands passed on evidence source commit `c451645`:

```text
cargo fmt --all -- --check
cargo test --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --workspace --all-features --no-deps --locked
cargo metadata --format-version 1 --no-deps --locked
cargo tree -p fenestra-ui-authoring --edges normal --locked
cargo tree -p fenestra-ui-macros --edges all --locked
cargo tree -p fenestra-ui-exp-0007-typed-authoring --edges normal,no-proc-macro --locked
cargo tree -p fenestra-ui-exp-0007-typed-authoring --target all --edges all --locked
git diff --check
```

The workspace ran 664 harness tests with no ignored or failed test. This count
includes 35 authoring tests, one macro harness that runs three compile-fail
cases, and 24 typed-authoring probe tests. Artifact hash, ASCII, CR, LF
attribute, final-LF, publication, dependency-direction, dirty-tree, and
Rust/Markdown file-size audits passed. The largest Rust or Markdown file is 399
lines.

## Windows verification

Status: pass on hosted Microsoft Windows Server 2025 `10.0.26100` with the
`1.97.1-x86_64-pc-windows-msvc` toolchain. The Windows lane defined by source
commit `c2ed6f0` checked the synthesized pull-request merge tree whose feature
head was that commit.

The locked workspace all-target/all-feature gate passed the same 664 tests as
Linux. A fail-fast Windows-only step then used a backslash-separated temporary
target path containing spaces and passed workspace format, Clippy with warnings
denied, rustdoc with warnings and missing documentation denied, metadata, the
four registered dependency-tree views, and all 24 typed-authoring probe tests.
Every generated `layout_board_fen_v1.rs` copy had the registered `b633d1f...`
SHA-256, `git diff --check` passed, and the checkout remained clean.

An authorized interactive Windows host was unavailable, so this evidence does
not include a native UI run or local Windows timing, RSS, or target-size
measurements. Those raw measurements are Linux-only observations and are not
correctness gates. The hosted lane proves the Windows build and pure authoring
contracts, not interactive Windows UI support.

## Result

```text
WU-0010 result: pass
EXP-0007 status: open; begun by WU-0010; no experiment result
```

The Linux and hosted Windows evidence satisfy the shared compiler, macro,
build, source-map, semantic, runtime, boundedness, privacy, and
deterministic-artifact contract for this work unit.

## Limitations and nonclaims

- No final `.fen` or `ui!` syntax, ergonomics, API, ABI, facade, compatibility,
  migration, registry reservation, publication, license, or MSRV is selected.
- No product budget, performance result, parser feasibility result, or full
  EXP-0007 feasibility result is established.
- There is no runtime parser, reload, serialization, VM, browser, DOM, CSSOM,
  selector, cascade, inheritance, animation, or computed-style system.
- There are no reactive expressions, conditions, text, events, slots, async,
  portals, custom widgets, layout selection, renderer selection, or native UI
  integration in this work unit.
- Stable Rust macro spans do not provide persistent original Rust byte offsets;
  nested expansion ancestry and renamed dependency discovery remain open.
- No Windows native UI, platform support, mobile support, parity, installer,
  redistribution, security, fuzzing, Miri, sanitizer, or production support
  claim is made.
