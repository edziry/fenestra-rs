# WU-0013 hybrid spatial composition verification

Status: Linux and Windows GNU verification complete; Windows MSVC gate pending
Linux result: pass
Windows GNU result: pass
Required target-pair result: pending Windows MSVC
Date: 2026-08-12
Branch: `feat/hybrid-spatial-composition`
Linux evidence source commit: `0ee2ebe7a2a6873fa98670846d225c32cd2c3543`
Windows GNU evidence source commit: `52234d2ba09fff2f282f618ff346e039689ef458`
Research baseline: `fenestra-research` commit
`176c42139776ed9f1ef879cd135bddadaf12a9da`

## Scope and contracts

The [composition plan](../design/hybrid-spatial-composition.md),
[reference contract](../design/hybrid-spatial-reference-v2.md),
[authoring equivalence contract](../design/hybrid-spatial-authoring-runtime-equivalence-v2.md),
[presentation contract](../design/hybrid-spatial-presentation-v2.md), and
[evidence contract](../design/hybrid-spatial-evidence-v2.md) define the
versioned WU-0013 boundary.

The workspace advances in lockstep to unpublished version `0.2.0`. Format 1
authoring remains byte stable. Format 2 is additive and emits the spatial
program as the fourth member of the authored result. No compatibility shim,
candidate type, renderer type, or probe surface enters the facade.

WU-0012 remains deferred. The spatial boundary owns no density, safe area,
surface epoch, lifecycle, Android, iOS, or multi-scene state.

## Implemented boundary

The implementation separates logical construction, layout-island allocation,
spatial parentage, placement, visual geometry, hit geometry, semantic bounds,
and paint. Layout and free placement are peer modes. Nested layout-in-free and
free-in-layout trees are expanded through construction-guided preorder without
making layout keys spatial identities.

The Fenestra-owned contracts include:

- checked Fixed16 coordinates, affine translate, rotate, scale, inverse, and
  singularity handling;
- viewport, parent, and explicit-node anchors with keyed-context prefix and
  cycle validation;
- rectangle, circle, polygon, and bounded path geometry with fill and stroke;
- owner-local brushes, gradients, images, clips, paint, hit, and semantic
  records with independent bounds;
- reverse-painter-order transformed hit testing independent of raster output;
- immutable resolved snapshots and borrowed paint frames;
- atomic runtime materialization, publication, retained generations, no-op
  identity, typed failure mapping, and rollback; and
- a private native reference presenter that consumes only the runtime paint
  frame and preserves pre-accept versus post-accept failure semantics.

Manual, `.fen`, and `ui!` format-2 lanes build equal raw programs, validate
separately, and produce equal nine-observation runtime logs. The independent
literal oracle compares complete state, receipts, mappings, geometry, clips,
paint, hits, semantics, every registered hit query, and every raster byte. Its
mutation controls cover records, fields, orders, options, queries, raster
metadata, and raster bytes.

## TDD and failure evidence

The retained branch history introduces focused failing contracts before each
implementation slice. It covers raw and symbolic spatial APIs, aggregate
validation, geometry and paint kernels, hit testing, rasterization, runtime
publication, authoring, macro dispatch, immutable presentation, native
staging, baseline evidence, five candidate lanes, and lane artifacts.

Typed controls exercise direct and derived limits, invalid geometry, malformed
paths and images, missing resources, anchor cycles, singular transforms,
candidate conversion faults, output-table faults, raster limits, failed
runtime rebuilds, presenter rejection, renderer loss, and exact rollback.
No test is skipped or weakened to admit a candidate result.

## Candidate dispositions

All candidate dependencies are exact, optional, private to
`fenestra-ui-exp-0008-hybrid-spatial`, and disabled by default. The
[candidate screen](../design/hybrid-spatial-candidate-screen.md) records their
licenses, declared Rust versions, direct unsafe and build-script facts, and
replacement constraints. The lockfile-derived transitive closure of each
registered tuple is hashed into its lane artifact.

The observed bounded-corpus dispositions are:

- `numeric-spatial`: Euclid 0.22.14, Kurbo 0.13.1, and Fixed 1.30.0 pass;
- `path-hit`: Kurbo 0.13.1 adapts by the closed `edge-rounding` rule and Lyon
  Tessellation 1.0.20 passes;
- `cpu-reference`: Tiny-Skia 0.12.0 and Raqote 0.8.5 stop on exact raster
  mismatch, leaving the Fenestra reference authoritative;
- `native-renderer`: Vello 0.9.0 with wgpu 29.0.3 builds the registered scene
  but stops as `target-unavailable`; no GPU pixel execution is claimed; and
- `image-resource`: PNG 0.18.1 adapts by closed orientation normalization,
  while Image 0.25.10 stops on an exact metadata mismatch.

`pass` is only a result for the registered corpus. It does not select a final
numeric, geometry, raster, image, path, or renderer dependency.

## Canonical artifacts

Every artifact is ASCII, uses LF with one final LF, stays within 4096 records,
1024 bytes per line, and 1 MiB total, and reconstructs freshly in the test
gate. Lane verification removes only candidate and classification rows and
then proves that the remaining baseline bytes are exact.

```text
artifact                      bytes  lines  max-line  sha256
spatial-v2.txt                24508    327       214  bc71d3f9167808984abf083613ea86a81eced60d8670d9b3133821dbb34d21a1
numeric-spatial-v2.txt        26436    339       275  011a9bb2b7ff7a724052acb679dfa13f292c3e57d5f590d22a93c718645883b1
path-hit-v2.txt               25814    335       278  81ea1e72d41aa360d41d5346b9ce3e7140d5929124c3a55e255c2701de0c6134
cpu-reference-v2.txt          25816    335       275  615204cc98084da54626dc140f10ae6d7bd764887505faae7e55f5e0122adf92
native-renderer-v2.txt        25288    331       328  cd979393ba2686f44003145f89e7d3f979b9d0b1c0def9aabc17ea89dc56971f
image-resource-v2.txt         25846    335       273  4c3977c8fb79f5f9f539e101eab7211b481603fb82ea0226350c33a651f68af1
```

The baseline SHA-256 appears in every candidate row. Candidate rows pair the
required Linux and Windows target labels and contain the same lock-closure
digest for each pair. The Windows GNU run below confirms that all six artifact
files remain byte identical on a native Windows host. It does not convert the
registered `x86_64-pc-windows-msvc` rows into executed MSVC evidence.

## Linux verification

The local host reported Fedora Linux kernel `7.1.5-101.fc43.x86_64`,
`x86_64-unknown-linux-gnu`, rustc and Cargo 1.97.1, and LLVM 22.1.6. Commands
were run serially with one Cargo build job and one test thread.

These commands passed on the source commit above:

```text
cargo fmt --all -- --check
cargo metadata --format-version 1 --no-deps --locked
cargo tree -p fenestra-ui-exp-0008-hybrid-spatial --depth 1 --all-features --locked
CARGO_BUILD_JOBS=1 CARGO_PROFILE_TEST_DEBUG=0 RUSTFLAGS="-D warnings" cargo test --workspace --all-targets --all-features --locked -- --test-threads=1
CARGO_BUILD_JOBS=1 CARGO_PROFILE_DEV_DEBUG=0 cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
CARGO_BUILD_JOBS=1 CARGO_PROFILE_DEV_DEBUG=0 RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --workspace --all-features --no-deps --locked
git diff --check
```

The hybrid evidence probe passed all 62 tests in its full-feature profile. Its
five candidate modules passed 25 focused tests, and its lane artifact module
passed four fresh encoding and verification tests. The workspace gate also
passed all retained historical suites after refreshing the versioned EXP-0007
lockfile fingerprint for the new optional dependencies.

## Windows verification

Status: Windows GNU pass; Windows MSVC pending.

An isolated `git archive` of the Windows evidence source commit was copied to
a temporary directory on a Windows 11 Home host. The host reported build
`10.0.26200`, `x86_64-pc-windows-gnu`, rustc and Cargo 1.97.1, and LLVM 22.1.6.
Commands were run serially with one Cargo build job and one test thread. The
build target directory was outside the exported source tree.

These commands passed:

```text
cargo fmt --all -- --check
cargo metadata --format-version 1 --no-deps --locked
cargo tree -p fenestra-ui-exp-0008-hybrid-spatial --depth 1 --all-features --locked
CARGO_BUILD_JOBS=1 CARGO_PROFILE_TEST_DEBUG=0 RUSTFLAGS="-D warnings" cargo test --workspace --all-targets --all-features --locked -- --test-threads=1
CARGO_BUILD_JOBS=1 CARGO_PROFILE_DEV_DEBUG=0 cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
CARGO_BUILD_JOBS=1 CARGO_PROFILE_DEV_DEBUG=0 RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --workspace --all-features --no-deps --locked
```

The complete workspace and the 62-test hybrid evidence probe passed. The six
artifact measurements matched the canonical table exactly, including zero CR
bytes. A sorted manifest of every exported source file matched before and
after the gate: 1,491 files, 8,201,038 bytes, and SHA-256
`d8a623e8b5b1fa843f6858783981c467cc93acc6fa8de5c91f7049398e9fca7e`.
The temporary source and target directories were then removed.

The host has only the GNU Rust toolchain installed. This run therefore makes
no MSVC, DX12 execution, or GPU pixel claim. It establishes native Windows GNU
build, test, lint, documentation, and artifact reproducibility only.

The versioned [CI workflow](../../.github/workflows/ci.yml) already runs the
locked workspace all-target/all-feature test gate on `windows-2025`, followed
by workspace Clippy, rustdoc with missing documentation denied, metadata,
dependency trees, diff checks, and a clean-tree assertion. WU-0013 remains
open until its MSVC lane passes on the exact source tree. The GNU result above
does not substitute for that required target tuple.

## Result and nonclaims

```text
WU-0013 Linux result: pass
WU-0013 Windows GNU result: pass
WU-0013 required target-pair result: pending Windows MSVC gate
EXP-0008 hybrid spatial status: open pending Windows MSVC verification
```

The result does not publish crates, expose the probe, select a final
dependency, promise incremental performance, stabilize the API or MSRV, or
claim GPU pixels, interactive Wayland, interactive Win32, accessibility
platform integration, mobile lifecycle, installer behavior, or product
support. The branch is verified on Linux and Windows GNU, but it is not ready
for the authorized squash merge until the required Windows MSVC gate is
versioned.
