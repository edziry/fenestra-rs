# WU-0005 deterministic runtime oracles verification

Status: complete locally
Result: pass
Date: 2026-08-09
Branch: `test/runtime-oracles`
Research baseline: `fenestra-research` commit `176c42139776ed9f1ef879cd135bddadaf12a9da`

## Research and contracts

The owned fixture, generator, clean model, replay boundary, observation order,
identity ledger, logical trace, failure artifact, and local exit criteria are
recorded in the [runtime oracle plan](../design/runtime-oracles.md). The exact
wire grammar, limits, precedence, and semantic verification order are recorded
in the [V1 wire and generation contract](../design/runtime-oracle-v1.md). The
[artifact contract](../design/runtime-oracle-artifacts.md) owns persistence,
privacy, and retention, while the
[reduction contract](../design/runtime-oracle-reduction-v1.md) fixes the metric,
transform order, budget, and completion claims.

The immutable research baseline supplied the authoring/runtime boundary and
the feasibility constraints. The implementation adds no external dependency,
uses no network or environment input, and does not expand the result into a
framework, renderer, scheduler, or EXP telemetry claim.

## TDD and review evidence

The fixture and clean-state tests were written before the testkit surface
existed. Generator, replay, canonical case and trace, known-fault, observation,
identity, artifact codec, reducer, verifier, and committed-golden behavior each
received focused tests before their corresponding implementation. Missing
surfaces first failed at compilation; behavioral regressions then failed at
their typed boundary before the minimum implementation made them pass.

Review strengthened the first green in several places:

- commit shape is validated before post-commit state observation;
- snapshot diagnostics preserve raw structure and reported counts, apply the
  documented global limit priority before bounded collection, and retain
  coherent aliases for the identity ledger;
- identity transitions are atomic and distinguish state mismatch from alias or
  lifecycle mismatch without serializing physical handles;
- artifact decoding completes grammar and canonicality checks before global
  limits, counts, references, fingerprint legality, and trailing data;
- artifact verification checks all semantic paths before all operations, uses
  base-snapshot incarnation rules, and orders provenance, replay, trace,
  fault-free, and reduction failures exactly as documented;
- all eleven semantic verification classes have typed error evidence; the
  defensive fault-free class is exercised at its private replay-phase
  boundary, while the committed artifact exercises that phase's success path;
- reduction compares the complete first `ReplayFailureV1`, not merely the fact
  that a candidate failed.

The final reducer regression materializes a strictly smaller index transform
that still fails at the same transaction but with different expected keys. A
24-evaluation fixed-point run rejects that candidate, proving that an arbitrary
failure does not satisfy the target.

## Committed synthetic artifact

The durable regression fixture is
[`known-move-omission-v1.txt`](../../crates/fenestra-ui-testkit/tests/artifacts/known-move-omission-v1.txt).
It is owned by `fenestra-ui-testkit` and retained in Git for failure-format V1.
It contains 1,741 ASCII bytes, 69 newline-terminated records, and SHA-256
`5496f101bbabfdfb82ce6b511ac28d65e56d69c8b3cf6f24bbee345ac022464e`.

The registered seed `1592614637` and generator config `16/4/12` reproduce an
original case of 16 transactions, 32 operations, and 1,096 canonical bytes.
`OmitMoveV1` targets operation 4 and first fails at transaction 2 with no
operation-local rejection. Its physical-identity-free fingerprint is
`StateMismatch` at fragment `root/r:1`, field `keyed-order`, with expected keys
`9,7,8` and observed keys `7,8,9`.

The 4,096-evaluation reducer reaches a fixed point after 35 evaluations. The
stored minimum is transaction 2 with operations 3 and 4, totaling one
transaction, two operations, and 55 canonical case bytes. Its trace is one
59-byte terminal commit event. A second reduction consumes 23 evaluations and
returns the identical fixed point. Replays of both stored cases reproduce the
exact failure, while the minimized case passes without the injected fault.

The internal golden builder independently regenerates, replays, reduces, and
encodes the artifact twice and requires both byte sequences to equal the
committed file. The public regression decodes, re-encodes, and semantically
verifies the committed bytes. The structural codec fixture remains separate
and is not treated as verified provenance.

The schema is closed and has no raw message, local filesystem path, physical
handle, address, clock, process, machine, text, asset, or user-payload field.
Debug output exposes only bounded counts and closed enum values. Production
code accepts and returns caller-owned bytes; it performs no artifact file I/O,
upload, or automatic retention.

## Verification

The following commands passed locally with warnings denied where required:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p fenestra-ui-testkit --all-targets --all-features --locked
cargo test -p fenestra-ui-runtime --all-targets --all-features --locked
cargo test --workspace --all-targets --all-features --locked
RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --workspace --no-deps --locked
cargo metadata --format-version 1 --no-deps --locked
cargo tree --workspace --edges normal --locked
git diff --check
```

The focused testkit gate passed 180 tests: 140 unit tests and 40 integration
tests. The runtime gate passed 68 tests: 16 unit tests and 52 integration tests.
The workspace gate passed 270 tests in total: 22 IR tests, 68 runtime tests,
and 180 testkit tests. No executed test was ignored or failed.

The replay corpus covered seeds `0..=31` at 64 transactions with one retained
runtime generation. Separate generator regressions require equal inputs to
produce equal cases and seeds 0 and 1 to produce distinct canonical seeded
tails.

Metadata confirms that every workspace package remains unpublished. The normal
dependency tree remains local: runtime depends only on IR, testkit depends only
on IR and runtime, and the EXP spine depends only on those three packages. The
workspace manifest, package manifests, and lockfile did not change during
WU-0005, and no external crate was added.

The ASCII scan passed over the testkit source, tests, artifact, and WU-0005
design and verification documents. The 400-line check passed: the largest
testkit Rust file is the 396-line fixture, and the largest WU-0005 design file
is the 399-line runtime oracle plan. The testkit, runtime, and IR crates retain
`forbid(unsafe_code)`; workspace lints deny unsafe code and undocumented unsafe
blocks. Compiler, Clippy, rustdoc, metadata, dependency-tree, and diff checks
all completed successfully.

## Result

Result: pass for WU-0005's local deterministic-oracle boundary.

One injected incremental correctness defect is reproducible from a committed,
bounded synthetic artifact. Its original and minimized cases regenerate and
replay deterministically, retain the same semantic fingerprint, and remain
distinguishable from both fault-free behavior and a different failing
candidate. The result is sufficient to begin WU-0006 scheduler and retained
work semantics without treating this logical trace as full telemetry.

## Limitations

- The `prototype` surface remains documentation-hidden, unpublished, absent
  from the product facade, and unstable.
- The V1 envelope embeds no digest or signature, and the verifier authenticates
  neither. The SHA-256 above is an informational identity for the committed
  bytes, not cryptographic integrity or hostile-input trust.
- Fixed point means only that no remaining ordered V1 transform preserved the
  exact failure. It does not establish global or academic minimality.
- The generated corpus is deterministic and bounded, not exhaustive, fuzzed,
  statistically representative, or a property-testing quality result.
- Logical generations, mutation records, and invalidation classes do not prove
  final layout, style, accessibility, scene, renderer, GPU, or platform work.
- No scheduler, queue, reentrancy, callback, async, cross-thread, frame, or
  headless EXP envelope behavior is claimed by this unit.
- Verification ran on the local Linux environment. Windows CI must pass before
  any cross-platform byte-determinism claim.
- No Miri, benchmark, memory profile, MSRV, mobile-target, performance, or
  product-support result is claimed.
