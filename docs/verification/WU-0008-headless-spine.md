# WU-0008 headless spine verification

Status: complete locally
Result: pass
Date: 2026-08-09
Branch: `feat/probe-headless-spine`
Research baseline: `fenestra-research` commit `176c42139776ed9f1ef879cd135bddadaf12a9da`

## Research and contract

The owned fixture, provisional style materialization, headless projection,
atomic publication rule, independent oracle, fixed scheduler script, bounded
traces, canonical artifact, probe boundary, and nonclaims are recorded in the
[headless EXP-0001 spine plan](../design/headless-exp-0001-spine.md). The fixed
research baseline requires a replaceable feasibility spine with one immutable
generation across logical state and projections. It leaves the native
environments, hardware, corpus, measurements, and thresholds of EXP-0001
unratified.

WU-0008 therefore closes only the deterministic synthetic headless boundary.
It does not execute EXP-0001 and does not satisfy a native feasibility gate.
The runtime and testkit APIs remain unpublished, documentation-hidden
prototype surfaces, and the product facade remains unchanged.

## TDD and review evidence

Each behavioral slice retained its failing contract tests before the matching
implementation. The versioned test evidence divides responsibility as follows:

- runtime tests for [style and insertion](../../crates/fenestra-ui-runtime/tests/runtime_headless_style.rs),
  [initial projection](../../crates/fenestra-ui-runtime/tests/runtime_headless_projection.rs),
  [hit testing and visibility](../../crates/fenestra-ui-runtime/tests/runtime_headless_hit_test.rs),
  [specification validation](../../crates/fenestra-ui-runtime/tests/runtime_headless_spec.rs),
  [projection limits and failures](../../crates/fenestra-ui-runtime/tests/runtime_headless_projection_failures.rs),
  [transactional rebuilds](../../crates/fenestra-ui-runtime/tests/runtime_headless_projection_transactions.rs),
  and [resize](../../crates/fenestra-ui-runtime/tests/runtime_headless_resize.rs)
  fix the materialization, geometry, ordering, capacity, diagnostic, rollback,
  invalidation, and immutable-snapshot contract; the separate
  [projection-limit suite](../../crates/fenestra-ui-runtime/tests/runtime_headless_projection_limits.rs)
  fixes every inclusive ceiling and their simultaneous priority;
- testkit tests for the [registered fixture](../../crates/fenestra-ui-testkit/tests/headless_fixture.rs),
  [independent oracle](../../crates/fenestra-ui-testkit/tests/headless_oracle.rs),
  [oracle faults](../../crates/fenestra-ui-testkit/tests/headless_oracle_faults.rs),
  [pointer delivery](../../crates/fenestra-ui-testkit/tests/headless_platform_pointer.rs),
  [resize delivery](../../crates/fenestra-ui-testkit/tests/headless_platform_resize.rs),
  [renderer resource derivation](../../crates/fenestra-ui-testkit/tests/headless_renderer.rs),
  and the [fixed runner](../../crates/fenestra-ui-testkit/tests/headless_runner.rs)
  fix clean-rebuild independence, committed-only input, callback deferral,
  stale-target rejection, resource lifetime, scheduler correlation, and trace
  identity;
- artifact tests cover [canonical encoding](../../crates/fenestra-ui-testkit/tests/headless_artifact_encode.rs),
  core [bounded decoding](../../crates/fenestra-ui-testkit/tests/headless_artifact_decode_core.rs),
  [scanner priority](../../crates/fenestra-ui-testkit/tests/headless_artifact_decode_scan.rs),
  [closed grammar](../../crates/fenestra-ui-testkit/tests/headless_artifact_decode_grammar.rs),
  [storage limits](../../crates/fenestra-ui-testkit/tests/headless_artifact_decode_limits.rs),
  [cross-record references](../../crates/fenestra-ui-testkit/tests/headless_artifact_decode_references.rs),
  [scheduler accounting](../../crates/fenestra-ui-testkit/tests/headless_artifact_decode_scheduler_stats.rs),
  [semantic verification](../../crates/fenestra-ui-testkit/tests/headless_artifact_verify_contract.rs),
  and first-difference checks for both [traces](../../crates/fenestra-ui-testkit/tests/headless_artifact_verify_trace.rs)
  and [projection families](../../crates/fenestra-ui-testkit/tests/headless_artifact_verify_projection.rs);
- probe tests require the [versioned artifact](../../probes/exp-0001-spine/tests/headless_artifact.rs)
  to decode, re-encode, verify, and equal two fresh runs, while the
  [binary-output test](../../probes/exp-0001-spine/tests/headless_stdout.rs)
  requires exact stdout and empty stderr under distinct host and logging
  sentinel values.

Review confirmed that the runtime and testkit builders are independent, all
projection and trace comparisons are typed field-by-field, bounds are checked
before their corresponding storage allocation, errors expose only closed
kinds and optional indices, and the probe composes run, build, verify, and
encode without duplicating runtime or scheduler behavior.

All five registered oracle faults are exercised independently: computed style,
geometry order, semantic membership, hit order, and scene output. Each changes
only its named normalized family, and simultaneous faults follow the fixed
family and first-field priority without disclosing compared values.

## Runtime and projection result

The runtime materializes each property using exact style assignment before the
construction effective value. Direct mutation then owns the live value, while
a keyed insertion rematerializes the item template. Every effective logical or
surface commit rebuilds computed style, geometry, semantics, hit regions, and
scene rectangles before swapping one `Arc<RuntimeState>`. A failed rebuild
preserves the prior logical tree, projection, identities, generation, and
allocation; a true no-op preserves that exact allocation.

Generation zero contains five nodes in authored order and produces the manual
baseline of five computed-style records, five geometry records, one semantic
record, three hit regions, and five scene rectangles. Tests also fix raw bounds
versus clipped output, signed checked arithmetic, hidden-ancestor filtering,
zero-height retention, half-open hit edges, and reverse hit order.

The fixed script covers ticks `0` through `19` and publishes generations `0`
through `9`, with final generation `9`. Its publications are the initial build,
nested pointer mutation, insert, move, update, remove, resize, control hide,
root-width mutation, and root-color mutation. The repeated resize at tick `8`
is a true no-op and preserves generation `6`, its allocation, and the visual
request's latest tick `7`.

Frame IDs are `0` through `3`; accepted submission tokens are `0` and `1`; and
control sequences are `0` through `3`. Token `0` retains an older projection
whose pointer query still hits while the current projection misses. A failed
fresh frame is recoverable, renderer loss does not accept or retire its offered
frame, shutdown is idempotent, and completion of the final retained token takes
the scheduler from draining to stopped at tick `19`. At that terminal event,
deferred callback batches, controls, the visual request, in-flight frames, and
the renderer resource ledger are all empty. After every publication, the
candidate snapshot matches the independent clean-rebuild oracle.

The final surface is `90 x 70`. The final projection contains five computed
styles, five geometries, no semantic record, two hit regions, and four scene
rectangles. Immediately after resize, the root retains raw bounds
`(0,0,100,80)` and clip `(0,0,90,70)`; after the later root-width commit, the
final root bounds are `(0,0,84,80)` and its clip is `(0,0,84,70)`.

## Bounds, traces, and artifact

The registered IR capacities are `1/5/4/1/3/12/2/3/5` for components,
properties, templates, regions, child slots, initial properties, initial keys,
template depth, and initial instances. Style capacity is two assignments.
Runtime capacities are `8/8/8/2/40/3` for operations, structural work, live
nodes, live fragments, live properties, and retained generations. Projection
capacities are `8/8/1/8/8` for computed style, geometry, semantics, hit
regions, and scene rectangles.

The synthetic manifest is exactly platform `headless-fake`, clock
`scheduler`, scheduler domain `8001`, and projection choices
`full/vertical/rebuilt/reverse`. The script stays within the registered
scheduler capacities shown below. Each scheduler triple is items, accounted
bytes, and maximum residence ticks.

| Storage | Registered capacity | Observed peak or final use |
| --- | --- | --- |
| Deferred callback batches | `1/80/8` | `1/80` |
| Controls | `4/128/8` | `2/64` |
| Replaceable visual request | `1/40/8` | `1/40` |
| In-flight frames | `2/80/8` | `2/80` |
| Fake renderer | `2/192/8` | `2/192` |
| Final projection | `8/8/1/8/8` | `5/5/0/2/4` |

`HeadlessTraceV1` contains exactly 55 events and accounts 8,800 bytes at 160
bytes per event, within its `128/20480` event/byte capacity.
`SchedulerTraceV1` contains exactly 41 events and accounts 3,936 bytes at 96
bytes per event, within its `256/24576` capacity. Both traces share the same
scheduler domain and correlate their tick, generation, frame, control, lane,
residence, and renderer observations.

The canonical evidence is
[`headless-spine-v1.txt`](../../probes/exp-0001-spine/tests/artifacts/headless-spine-v1.txt):

```text
bytes: 11227
LF-terminated lines: 144
SHA-256: f669d60f57efdcea7ccc797e65a6d40cbe75be5174ce1e200325a06e92113490
```

Every artifact byte is printable ASCII or LF, every line is at most 1,024
bytes, and the complete artifact is below the 65,536-byte and 512-line
ceilings. Before storing its borrowed line index, the scanner checks artifact
bytes, printable ASCII, line bytes, line count, and final LF. Closed grammar,
versions, canonical scalar forms, capacities, counts, and path depth are then
preflighted before reserving event and projection vectors. After bounded typed
records are parsed, the decoder validates scheduler accounting, residence, and
cross-record references. The semantic verifier reruns the fixed script and
compares fixture metadata, capacities, result, final generation, surface, both
traces, and all five projection families without comparing encoded bytes.

The decoded golden re-encodes byte-for-byte, verifies as `pass`, equals two
fresh runs, and is exactly the probe binary's stdout. The binary emits no
stderr, host identity, host-derived environment value, log record, source text,
native ID, runtime node ID, arena index, pointer value, or private validation
domain. Its closed synthetic environment manifest remains part of the artifact.
The repository attributes require LF checkout for this golden. The SHA-256
above identifies these exact versioned bytes; it is not an authenticity,
security, or runtime integrity mechanism.

## Dependencies, safety, and file boundaries

WU-0008 adds no package or external dependency and changes no manifest or
`Cargo.lock`. The normal dependency tree contains only workspace packages and
retains the documented direction: probe to IR/runtime/testkit, testkit to
IR/runtime, and runtime to IR. Metadata reports every package as unpublished.

The implementation adds no native handle, backend, async runtime, layout or
renderer library, platform API, or unsafe block. Every crate root and both
probe roots retain `forbid(unsafe_code)` where applicable, while workspace
lints deny unsafe code, unsafe operations in unsafe functions, and
undocumented unsafe blocks.

The complete Rust and Markdown tree remains below 400 lines per file. The
Markdown maximum and completed design plan are 399 lines, while the Rust
maximum is 398 lines. The full tracked-workspace ASCII and whitespace scans
pass. Canonical artifact output is independently restricted to printable ASCII
plus LF.

## Verification

The following commands passed locally with warnings denied where shown:

```text
cargo test -p fenestra-ui-exp-0001-spine --all-targets --all-features --locked
cargo run -p fenestra-ui-exp-0001-spine --quiet --locked | cmp - probes/exp-0001-spine/tests/artifacts/headless-spine-v1.txt
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --workspace --all-features --no-deps --locked
cargo metadata --format-version 1 --no-deps --locked
cargo tree --workspace --edges normal --locked
git check-attr text eol -- probes/exp-0001-spine/tests/artifacts/headless-spine-v1.txt
git diff --check
```

The probe package gate passes three library unit tests, three artifact
integration tests, and one binary-output integration test. No executed test is
ignored or failed. The workspace gate passes 500 tests: 32 IR integration
tests; 16 runtime unit and 129 runtime integration tests; 162 testkit unit and
154 testkit integration tests; and 3 probe unit and 4 probe integration tests.
The facade has no tests. Formatting, Clippy, rustdoc, canonical stdout
comparison, dependency inspection, LF attributes, ASCII, whitespace, and
file-limit checks pass.

Linux verification ran on Fedora 43 x86_64, kernel
`7.1.5-101.fc43.x86_64`, with Rust and Cargo `1.97.1`
(`x86_64-unknown-linux-gnu`, LLVM `22.1.6`).

## Windows verification

Status: pass in a fresh native checkout.

Verification also ran on 64-bit Windows 11 Core, reported as Windows
`10.0.26200`, with target `x86_64-pc-windows-gnu`, rustc `1.97.1`, and Cargo
`1.97.1`.
The same formatting, locked 500-test workspace gate, workspace Clippy,
workspace rustdoc including `missing-docs`, metadata, normal dependency tree,
diff, tracked ASCII, and file-limit checks passed. No test was ignored. The
package and unit/integration breakdown matched the Linux result above.

The Windows golden has 11,227 bytes, 144 LF bytes, zero CR bytes, and the same
SHA-256 recorded above. All three versioned text artifacts retained their
Linux-identical SHA-256 digests, and the explicit artifact attributes report
`text` set with `eol=lf`.

The first native checkout exposed a portability defect: two legacy fixture
files had been converted to CRLF, producing 55 codec test failures; separate
tracked Rust inputs had CRLF and failed the rustfmt check. The versioned
repository attributes now apply
`* text=auto eol=lf` and retain explicit LF rules for all three artifact and
fixture directories. A fresh clean checkout after that correction passed the
complete gate. This resolved defect is part of the verification evidence; the
initial failing checkout is not counted as a pass.

## Result

```text
WU-0008 result: pass
EXP-0001 status: open and not executed
```

WU-0008 passes its local deterministic synthetic headless contract: the fixed
runtime state, complete projections, scheduler lifecycle, independent oracle,
two correlated bounded traces, canonical artifact, semantic replay verifier,
and probe output agree on the exact versioned evidence above.

This result is not an EXP-0001 pass and does not establish native feasibility.

## Limitations and nonclaims

- All packages remain unpublished and replaceable. Runtime and testkit
  headless APIs remain documentation-hidden prototypes; the probe documents
  only its fixed run/error seam. None is product API or a compatibility
  promise.
- Exact provisional style assignment is not a final selector, cascade,
  inheritance, animation, transition, expression, or authoring result.
- The fixture-only vertical rebuild is not layout conformance, incremental
  layout, damage, pixel, text, color-space, accessibility, hosted-control,
  presentation, GPU, transparency, or export evidence.
- Fake pointer, resize, renderer, clock, scheduler resources, and logical
  rectangles are deterministic testkit models, not native input, windows,
  surfaces, rendering, synchronization, or resource accounting.
- Native Windows execution of the headless fake is not a native-window,
  backend, or platform-adapter result and does not establish product platform
  support.
- One fixed fixture, script, oracle, malformed corpus, and golden do not prove
  fuzz robustness, broad corpus correctness, scalability, or product support.
- The registered ceilings are correctness inputs, not allocator-byte or
  product budgets. `BUD-UI`, `BUD-INPUT`, `BUD-START`, `BUD-MEM`, `BUD-BIN`,
  `BUD-POWER`, `BUD-RECOVERY`, and the numeric product interpretation of
  `BUD-QUEUE` remain pending. No timing, memory, binary-size, startup, energy,
  queue-sizing, recovery, security, or performance conclusion is claimed.
- No mobile, native-window/backend integration, Miri, sanitizer, benchmark,
  memory-profile, MSRV, release-build, packaging, or deployment result is
  claimed.
