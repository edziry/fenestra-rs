# WU-0011 layout conformance plan

Status: implemented; verification complete
Work unit: WU-0011
Research baseline: `fenestra-research` commit
`176c42139776ed9f1ef879cd135bddadaf12a9da`
Package line at entry: `0.1.0`
Intended compatible package line: `0.1.1`

## Purpose

WU-0011 replaces the fixture-only vertical rectangle calculation with a small,
versioned, candidate-neutral layout boundary. It also evaluates one current
layout implementation against an independent oracle without selecting a final
engine or exposing that implementation through the IR or generic runtime.

The existing headless projection remains responsible for computed properties,
visibility, clipping, semantics, hit regions, scene rectangles, generations,
and transaction rollback. Layout owns only ordered logical bounds.

The reference contract is specified in
[layout-conformance-reference.md](layout-conformance-reference.md). The mobile
dependency decision is specified in
[mobile-lifecycle-preparation.md](mobile-lifecycle-preparation.md).

## Entry state and replacement seam

At entry, `fenestra-ui-runtime` performed layout inside
`runtime/headless/build.rs`. It traversed the committed tree in authored order,
placed every child at its parent's x coordinate, advanced one vertical cursor,
and clipped the result. The same projection then derived hit and scene records.
The testkit clean rebuild retained an independent copy of that fixture rule.

WU-0011 extracts only the bounds calculation:

1. Runtime materializes typed width and height properties as it does today.
2. Runtime assigns dense pass-local keys in authored preorder and constructs a
   candidate-neutral stack input.
3. An observationally deterministic `LayoutEngineV1` returns one owned bounds
   record per key without retaining correctness state across calls.
4. Runtime validates count, key order, coverage, rectangle domains, and checked
   arithmetic before using any record.
5. Runtime performs visibility, ancestor and viewport clipping, semantic, hit,
   and scene derivation itself.
6. Transaction publication remains after the complete draft rebuild, so an
   engine or projection failure leaves the previous generation untouched.

The default constructor uses the owned reference stack engine. A documentation-
hidden experimental constructor accepts a boxed engine with the same
call-local contract. This lets a disposable probe exercise a candidate through
the real runtime while preventing candidate types from entering generic APIs.

## Candidate-neutral ownership

A new unpublished `fenestra-ui-layout` package owns:

- dense pass-local keys;
- row and column stack axes;
- fixed preferred, minimum, and maximum logical dimensions;
- border-box padding and a nonnegative inter-child gap;
- a present logical viewport, including zero extents;
- explicit tree and record limits;
- ordered integer bounds output;
- a `Send + Sync + Unpin + UnwindSafe + RefUnwindSafe + 'static` engine trait
  whose results are observationally deterministic and call-local, plus the
  reference stack implementation;
- closed validation and failure vocabularies.

It owns no stylesheets, authored names, runtime identities, mutations,
visibility, clipping, hit testing, scene data, platform state, physical scale,
window or surface handles, text measurement, cache policy, or candidate types.

`fenestra-ui-runtime` depends on this package. `fenestra-ui-testkit` reaches it
only through runtime and retains an independent oracle. A disposable WU-0011
probe depends directly on layout, uses testkit only for runtime evidence, and
owns the Taffy adapter. The facade does not re-export the new prototype.

## Candidate screen

The screen was refreshed on 2026-08-10 from primary package and source records.

### Taffy 0.13.0

Taffy is the primary probe candidate. Its
[0.13.0 release](https://github.com/DioxusLabs/taffy/releases/tag/v0.13.0)
is the current stable release. Its aspect-ratio min/max correction is outside
the version 1 stack subset and is not used as evidence for this result. Its
[manifest](https://github.com/DioxusLabs/taffy/blob/v0.13.0/Cargo.toml) declares
MIT, Rust 1.71, a high-level tree, and independently selectable algorithms. The
probe pins exactly:

```text
taffy = { version = "=0.13.0", default-features = false, features = ["std", "taffy_tree", "flexbox"] }
```

This excludes Grid, Block, CSS float layout, parsing, Serde, detailed layout
data, and content-size output. The active normal/build closure is Taffy 0.13.0,
`arrayvec` 0.7.8, `slotmap` 1.1.1, and the latter's build dependency
`version_check` 0.9.5. Lock-only entries are reported separately from that
compiled tree. It has no native library, network runtime, or FFI edge. Taffy,
`arrayvec`, and `slotmap` contain dependency-owned unsafe code; Fenestra-owned
crates remain unsafe-forbidden. Taffy supports 32-bit and 64-bit pointer widths;
its optional strict-provenance path remains disabled.

The published-source inventory, including feature-gated modules, found 2 unsafe
blocks and 5 unsafe constructors in Taffy, 39 blocks, 11 unsafe functions, and 2
unsafe implementations in `arrayvec`, 67 blocks, 20 unsafe functions, 1 unsafe
trait, and 1 unsafe implementation in `slotmap`, and none in `version_check`.
These counts exclude examples embedded in documentation. They are
dependency-owned observations, not proof of soundness.

Version 0.13.0 was two days old when screened. Exact pinning, a disposable
adapter, an independent oracle, and deterministic artifacts contain that churn
risk. Version 0.12.2 is the lower-freshness fallback if 0.13.0 cannot build or
violates the contract; no functional difference between the two is claimed for
the bounded version 1 subset.

### Alternatives

[Morphorm 0.9.0](https://docs.rs/morphorm/0.9.0/morphorm/) is active, MIT,
Rust-native, tree-store neutral, and directly supports row, column, overlay,
grid, padding, gaps, and constraints. Its custom semantics and absent declared
MSRV make it a useful second candidate, not the first CSS-oriented comparison.

[Yoga](https://github.com/facebook/yoga) is mature and mobile-proven, but the
upstream implementation is C++20. Current Rust bindings add native build and
unsafe FFI costs, while the new pure-Rust `yoga-rs` 0.1.0 is too early for the
primary probe. [Stretch](https://github.com/vislyhq/stretch) is the historical
Rust Flexbox predecessor at 0.3.2; its age, older API, and unsafe tree storage
make it a fallback reference rather than a current candidate.

[`float-pigment-layout` 0.10.4](https://docs.rs/float-pigment-layout/0.10.4/float_pigment_layout/)
is held for a later comparison: it is MIT, Rust-native, generic over its
numeric representation, and supports Flexbox, Block, and Grid, but declares no
MSRV and brings a much larger trait and package closure including
dependency-owned unsafe. [`torin` 0.4.1](https://docs.rs/torin/0.4.1/torin/)
is also held: it is active and Rust-native but coupled to Freya's custom model,
uses edition 2024, and has a higher-churn 0.5.0 release candidate.

No candidate is selected for product use by WU-0011.

## Version decision

The implementation adds a new unpublished package, a documentation-hidden
runtime injection seam, and an internal bounds implementation while preserving
the existing `0.1.x` runtime behavior and public methods. Under the ratified
[versioning policy](../versioning-policy.md), this is a compatible `PATCH`
change from `0.1.0` to `0.1.1`. Every workspace package and exact internal path
dependency moves together on that lockstep line.

If implementation requires changing existing geometry, an existing public enum
shape, scheduler ownership, or the meaning of zero extent, work stops and the
workspace moves to `0.2.0` without a compatibility shim. The current plan does
not authorize such a break.

## TDD sequence

1. Workspace RED: new package and probe membership, exact `0.1.1` lockstep,
   dependency edges, publication flags, and candidate feature closure.
2. Contract RED: closed vocabularies, dense preorder tree rules, inclusive
   limits, invalid constraint priority, zero viewports, and privacy-safe errors.
3. Reference RED: row, column, nesting, min/max, padding, gap, overflow, zero,
   checked arithmetic, output order, and deterministic repeated runs.
4. Oracle RED: independently written normalized expected tables for the direct
   corpus, field-by-field comparison, and mutate-one-field controls; the runtime
   script continues to use its existing clean-rebuild oracle.
5. Candidate RED: Taffy adapter translation, finite/integer output conversion,
   corpus comparison, dependency privacy, and pass/adapt/stop classification.
6. Runtime RED: default behavior parity, injected candidate parity, malformed
   engine output, transaction rollback, and generation/invalidation correlation.
7. Projection RED: keyed insert, move, update, remove and resize correlate exact
   bounds with hit and scene records at every committed generation.
8. Evidence RED: one canonical corpus-and-runtime artifact, exact inclusive
   bounds, deterministic bytes, and dependency/environment manifests.

Each behavior begins with an expected failing test and receives a focused RED
commit before its minimum GREEN implementation when the split remains cohesive.

## Candidate result classification

- `pass`: every supported valid corpus case and the registered runtime script
  match the independent oracle exactly; core-invalid input is rejected before
  engine invocation, profile-invalid input before candidate backend or tree
  construction, and output conversion is bounded and deterministic.
- `adapt`: an additional documented corrective rule beyond the registered
  mapping is required and makes the complete admitted corpus pass without
  changing the neutral contract.
- `stop`: panic on admitted valid input, non-finite or unbounded output,
  candidate-type leakage, failure to preserve runtime atomicity, native
  dependency admission, or an unresolved correctness mismatch.

A pass selects only the bounded WU-0011 stack subset, not Taffy or Flexbox as the
final Fenestra layout model.

The adapter uses Flex display, border-box sizing, left-to-right direction,
relative positioning, visible overflow, no wrapping, zero grow and shrink,
automatic flex basis, start item/self/content alignment and justification,
zero margin, border, and inset, exact size/min/max lengths, and the
axis-specific gap: row maps horizontal gap and zero vertical gap, while column
maps zero horizontal gap and vertical gap. The logical viewport is passed as
definite available space even though fixed Stack bounds do not depend on it. It
disables Taffy's default rounding, accumulates parent-relative locations into
absolute edges with finite and edge-bound validation, then applies the contract
rounding once and emits records in authored-key order. It retains an
authored-key-to-candidate ID map only within the call and retains no Taffy tree
or cache between calls.
Runtime visibility never maps to `Display::None`.

For each dimension the adapter maps preferred to `size`, minimum to `min_size`,
and maximum to `max_size` as exact lengths; the candidate result must therefore
equal the contract clamp before cursor advancement.

## Verification gates

Required local gates are locked workspace tests, formatting, Clippy over every
target and feature with warnings denied, rustdoc with warnings and missing docs
denied, metadata and dependency trees, exact lock closure, publication and
license/MSRV/unsafe audits, ASCII and file-size checks, clean status, and two
fresh artifact runs. Pure tests must pass on Linux and Windows. Environment-
qualified timing, peak RSS, and output size are recorded as observations only;
there is no ratified performance budget.

Because WU-0010 intentionally records the workspace `Cargo.lock`, admitting
Taffy also refreshes that versioned evidence row and its hash without changing
the WU-0010 semantic or runtime artifact formats. The WU-0011 verification must
show both the active dependency tree and any lock-only entries.

## Nonclaims

WU-0011 does not define final layout, Flexbox, CSS, Grid, intrinsic or text
measurement, percentage units, baseline alignment, wrapping, absolute
positioning, scroll, damage, incremental layout, cache policy, safe area,
mobile lifecycle, platform support, public API, publication, license, MSRV, or
product capacity. It does not complete any open feasibility experiment.

Implementation logic is added in cohesive modules rather than appended to the
existing near-limit headless build, specification, transaction, or apply files.
The WU-0008 artifact and wire format remain unchanged; WU-0011 owns a separate
versioned corpus and evidence format.
