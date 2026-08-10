# WU-0011 layout conformance verification

Status: verification complete
Linux result: pass
Cross-platform result: pass
Date: 2026-08-10
Branch: `experiment/layout-conformance`
Evidence source commit: `a97533767672232843e3b06dbb9b40bc997a900a`
Windows verification source commit:
`6f2ce5651dfc185dbbc53d6d1fb9d83e805cf228`
Research baseline: `fenestra-research` commit
`176c42139776ed9f1ef879cd135bddadaf12a9da`

## Scope and version contract

The [layout conformance plan](../design/layout-conformance.md),
[version-1 reference](../design/layout-conformance-reference.md), and
[mobile lifecycle decision](../design/mobile-lifecycle-preparation.md) fix this
work unit's ownership, arithmetic, validation order, limits, corpus, runtime
mapping, candidate classification, and nonclaims.

WU-0011 adds one candidate-neutral package and one disposable probe. Every
workspace package advances in lockstep from `0.1.0` to `0.1.1`, remains Rust
2024, and has `publish = false`. This is the planned compatible `PATCH`: the
existing runtime constructors and observable default behavior remain intact.
No compatibility shim is required, and no incompatible public contract was
introduced.

The layout contract, corpus, and artifact are separate version-1 formats. The
facade does not re-export the new boundary or candidate. Nothing in this unit is
published, stabilized, or selected as final layout semantics.

## TDD and implementation evidence

The retained branch history begins each behavior with a focused failing test or
artifact requirement before its implementation. The sequence covers:

- workspace membership, the `0.1.1` lockstep line, and the unpublished package;
- closed input, output, error, location, trait, and registered-limit contracts;
- topology, scalar, padding, gap, arithmetic, output, and diagnostic priority;
- row and column reference geometry over the full admitted integer domain;
- an independent 23-case oracle with field and record mutation controls;
- candidate admission, explicit Taffy mapping, invocation, conversion, and
  dependency privacy before the bounded adapter;
- default and injected runtime construction, projection, error mapping,
  rollback, auto traits, and profile-independent runtime limits;
- real-runtime reference and candidate characterization over seven milestones;
- typed direct, runtime, metadata, and encoder faults before the canonical
  artifact; and
- exact and one-over record, line, byte, scalar, tree, and output-edge bounds.

No test was weakened, skipped, or removed to admit the implementation.

## Candidate-neutral layout boundary

[`fenestra-ui-layout`](../../crates/fenestra-ui-layout/src/lib.rs) owns dense
pass-local preorder keys, row and column axes, fixed preferred/minimum/maximum
dimensions, border-box padding, gap, logical viewport, tree limits, checked
integer bounds, closed errors, and `LayoutEngineV1`. The engine contract is
`Send + Sync + Unpin + UnwindSafe + RefUnwindSafe + 'static`, call-local, and
observationally deterministic. `ReferenceStackEngineV1` is the default owned
implementation.

The boundary does not own runtime identities, visibility, clipping, hit
testing, scene data, scale, surfaces, candidate IDs, platform state, or caches.
Input and output validation use trusted ordinals and stop at the documented
first error. Failed arithmetic or malformed output returns no partial layout.

The registered probe profile accepts exactly 32 nodes, depth 8, 16 children per
node, candidate input scalar 4096, and candidate output edge 524288. Equality is
accepted and each one-over value is rejected. The integer reference itself
accepts the nonnegative `i32` domain subject to padding fit and checked
arithmetic; the candidate ceilings are not core layout limits.

## Runtime integration and atomicity

The [headless layout adapter](../../crates/fenestra-ui-runtime/src/runtime/headless/layout.rs)
assigns fresh preorder keys and maps the existing materialized width and height
subset into zero-padding, zero-gap columns. A child's maximum width is the
already resolved parent width. The default constructor injects the reference
engine; a documentation-hidden constructor accepts a boxed neutral engine for
experiments.

Runtime derives node, depth, and child ceilings from the already preflighted
projected node count. Independent fixtures above 32 nodes, depth 8, and 16
children prove that the registered probe profile did not become a product
capacity limit. Core input validation occurs before the one engine invocation.
Output count, ordered keys, scalar domains, and far edges are validated before
any record enters the derived projection.

Runtime retains visibility, ancestor and viewport clipping, semantics, hit
regions, scene rectangles, generations, invalidation, and publication. Tests
feed distinct valid engine bounds and observe those bounds in geometry and the
correlated clips, hit records, and scene records. Initial and transactional
engine failures, including malformed output, map to the closed runtime errors
and preserve the prior allocation and generation. Injection preserves
`UiRuntime`'s `Send`, `Sync`, `Unpin`, `UnwindSafe`, and `RefUnwindSafe`
properties.

The registered runtime script records initial, color, insert, move, update,
remove, and resize steps. Reference and candidate runs have exact receipt and
projection generations, invalidation words, mutation counts, surfaces, and
projection families. Each milestone also matches the existing independent
clean-rebuild oracle. Color invalidates only paint; structural, dimension, and
surface changes retain their existing invalidation behavior.

## Candidate result

The disposable probe maps every admitted field explicitly to Taffy 0.13.0:
Flex display, border-box sizing, relative positioning, visible overflow, no
wrap, zero grow and shrink, automatic basis, start alignment and justification,
zero margin, border, and inset, exact dimensions, padding, and axis-specific
gap. Available viewport space is definite and candidate pixel rounding is
disabled.

Core validation and the 4096 candidate admission scan complete before a Taffy
tree is constructed. Each admitted call creates one tree, retains its ID map
only for that call, computes once, converts relative locations to cumulative
absolute edges, validates finite/nonnegative/524288-bounded raw edges, applies
the contract rounding once, and emits authored-key order. Candidate types do
not enter the layout package, runtime, testkit, IR, facade, or artifact.

All 23 admitted direct cases match independently written expected records in
the oracle, integer reference, and candidate lanes. The seven-step real-runtime
script also matches all three lanes. Candidate classification for this bounded
version-1 stack subset is therefore `pass`.

No corrective semantic rule beyond the predeclared profile admission, direct
field mapping, and output conversion was required. Those screening mechanics
are the planned disposable adapter boundary, not an `adapt` result.

This does not select Taffy, Flexbox, or the stack subset as Fenestra's final
layout engine or authored model. EXP-0008 remains open and bounded; WU-0011
does not claim a complete experiment result.

## Versioned artifact

The canonical
[layout conformance artifact](../../probes/exp-0008-layout-conformance/tests/artifacts/layout-conformance-v1.txt)
is printable ASCII plus LF with exactly one final LF.

| Bytes | LF lines | Max line | SHA-256 |
| ---: | ---: | ---: | --- |
| 50,835 | 412 | 385 | `0e7a0d7c7133b80a9ab0375c9adfbdb2395c9fd0decd01968f4b29bfcb768959` |

The artifact stays within the inclusive 512-record, 512-byte-line, and
65,536-byte limits. It contains 15 metadata rows, 286 direct rows, 109 runtime
rows, and 2 closing rows. The direct section contains 23 cases, 120 input rows,
120 combined oracle/reference/candidate output rows, and 23 result rows. The
runtime section contains one header, 7 generation rows, 38 geometry rows, 24 hit
rows, 38 scene rows, and one result row.

Rows use normalized authored paths rather than runtime or Taffy IDs. They
contain no host path, clock, username, pointer, source payload, or `Debug`
output. Typed mutate-one-field controls cover the modeled direct, runtime, and
metadata fields. Encoding validates the complete model and record count before
rendering, then every line before the accumulated bytes, preserving the
`records`, `line-bytes`, `artifact-bytes` priority. Two fresh constructions
produce the same bytes and match the versioned golden.

## Dependencies, safety, and publication

The layout package has no external dependency. Runtime's normal tree adds only
the layout package beside the existing IR package. Taffy is exact-pinned and
private to the disposable probe:

```text
taffy = { version = "=0.13.0", default-features = false, features = ["std", "taffy_tree", "flexbox"] }
```

The active candidate normal/build closure is Taffy 0.13.0, arrayvec 0.7.8,
slotmap 1.1.1, and slotmap's build dependency version_check 0.9.5. Taffy is MIT
and declares Rust 1.71. arrayvec is MIT OR Apache-2.0 and declares Rust 1.51.
slotmap is Zlib and declares Rust 1.58.0. version_check reports MIT/Apache-2.0
and declares no Rust version. Runtime, testkit, and IR are probe dev
dependencies only.

Relative to the active probe tree, `Cargo.lock` also records inactive
`taffy -> serde`, `serde -> serde_core`, `serde -> serde_derive`, and
`serde_core -> serde_derive` edges. serde, serde_core, and serde_derive are
1.0.229 and MIT OR Apache-2.0; the first two declare Rust 1.56 and serde_derive
declares Rust 1.71. Their procedural-macro closure was already locked elsewhere
in the workspace. The artifact intentionally summarizes the candidate feature
edge and derive edge; it is not an exhaustive lock graph.

Fenestra-owned WU-0011 code forbids unsafe code and contains no unsafe block.
The published-source inventory, excluding documentation examples, records 2
unsafe blocks and 5 unsafe functions in Taffy; 39 blocks, 11 functions, and 2
unsafe implementations in arrayvec; 67 blocks, 20 functions, 1 unsafe trait,
and 1 unsafe implementation in slotmap; and none in version_check. Those
observations do not prove soundness. slotmap owns the only active custom build
and uses version_check to classify rustc; Taffy, arrayvec, and version_check
have no custom build target. The active closure has no native library, network
runtime, `links` declaration, or FFI edge.

All eleven workspace packages remain unpublished. The project still has no
selected license, public MSRV, reserved registry namespace, support matrix, or
ratified product capacity. The WU-0010 evidence refresh at versioned commit
`c3514f6` updates only its locked Cargo evidence and summary hash; its semantic
and runtime goldens and measurements are unchanged.

## Linux measurement and verification

The local gate ran on Fedora 43 x86_64, kernel `7.1.5-101.fc43.x86_64`, rustc
and Cargo 1.97.1, LLVM 22.1.6, and GNU Time 1.9.

A detached worktree and dedicated initially empty target directory measured
`cargo build -p fenestra-ui-exp-0008-layout-conformance --locked` in debug mode.
GNU Time reported elapsed wall seconds and peak resident KiB. Target size is the
probe `.rlib`. The one-source-edit run forced recompilation without changing
the canonical artifact hash.

| Case | Elapsed seconds | Peak RSS KiB | Probe rlib bytes |
| --- | ---: | ---: | ---: |
| clean | 1.15 | 180,924 | 5,392,818 |
| immediate no-op | 0.07 | 48,276 | 5,392,818 |
| one source edit | 0.16 | 137,736 | 5,392,818 |

These are environment-qualified observations, not correctness budgets or
product performance claims.

The following commands passed against evidence source commit `a975337`:

```text
cargo fmt --all -- --check
cargo test --workspace --all-targets --all-features --locked --no-fail-fast
cargo test -p fenestra-ui-exp-0008-layout-conformance --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --workspace --all-features --no-deps --locked
cargo metadata --format-version 1 --no-deps --locked
cargo tree -p fenestra-ui-layout --edges all --locked
cargo tree -p fenestra-ui-runtime --edges normal --locked
cargo tree -p fenestra-ui-exp-0008-layout-conformance --edges normal,no-proc-macro --locked
cargo tree -p fenestra-ui-exp-0008-layout-conformance --target all --edges all --locked
git diff --check
```

The workspace ran 821 harness tests with no failed or ignored test. The layout
probe ran 64 tests. Artifact hash, deterministic rebuild, printable ASCII, CR,
LF attribute, final-LF, publication, version lockstep, dependency-direction,
candidate-privacy, dirty-tree, and Rust/Markdown file-size audits passed. Every
Rust and Markdown file remains below 400 lines.

## Windows verification

Status: pass.

The pure gate ran on an authorized SSH Windows host: Microsoft Windows 11 Home
10.0.26200, 64-bit, with rustc and Cargo 1.97.1, LLVM 22.1.6, and host target
`x86_64-pc-windows-gnu`. A `git archive` of clean source commit `6f2ce565` was
expanded into a temporary staging directory without repository metadata. A
post-gate comparison of relative path, size, and SHA-256 matched all 623 tracked
files against tree `554eedc0ef2a7d1d0e29d705224f4c4892ef81ab`; no source file
changed. Build output used a dedicated `CARGO_TARGET_DIR` whose path contained
spaces.

The following commands passed on that exact staged tree:

```text
cargo fmt --all -- --check
cargo test --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
set "RUSTDOCFLAGS=-D warnings -D missing-docs" && cargo doc --workspace --all-features --no-deps --locked
```

The workspace listed and ran 821 harness tests; the layout probe listed and ran
64. The Windows artifact independently measured 50,835 bytes, 412 LF lines, no
CR, maximum line length 385, one final LF, and SHA-256
`0e7a0d7c7133b80a9ab0375c9adfbdb2395c9fd0decd01968f4b29bfcb768959`.
These observations close the Windows pure gate without a Git push, pull
request, or merge.

This was a target-native SSH verification, not a hosted CI run. It does not
claim MSVC coverage, interactive Win32 presentation, installer behavior,
platform parity, or Windows product support. The versioned
[CI workflow](../../.github/workflows/ci.yml) remains the integration gate when
the branch is later reviewed or merged remotely.

## Mobile preparation decision

The WU-0012 audit remains `decision complete; implementation deferred`. Layout
uses a present logical viewport and introduces no physical scale, safe area,
surface, lifecycle, platform thread, cache, or resource-ownership vocabulary.
Zero extent remains geometry rather than absence. All documented stop-the-line
triggers remain false, so WU-0012 was not required to implement WU-0011. No
mobile compilation, lifecycle, input, safe-area, or support claim is made.

## Result and limitations

```text
WU-0011 implementation status: complete
WU-0011 Linux result: pass
WU-0011 cross-platform result: pass
EXP-0008 status: open; bounded stack subset pass; no candidate selected
WU-0012 implementation status: deferred; no trigger entered
```

- No final layout semantics, renderer, engine, facade API, ABI, stable
  compatibility promise, migration, publication, license, MSRV, or product
  capacity is selected.
- Row, padding, gap, and general min/max remain direct-corpus semantics; the
  provisional authored IR exposes only the preserved vertical fixed-size
  runtime subset.
- There is no CSS, Grid, intrinsic or text measurement, percentage, baseline,
  wrap, absolute positioning, scroll, damage, incremental layout, or cache
  policy.
- There is no native UI, Windows native UI, mobile, safe-area, lifecycle,
  platform parity, installer, redistribution, security, fuzzing, Miri,
  sanitizer, or production support claim.
