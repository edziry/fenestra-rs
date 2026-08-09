# WU-0001 workspace bootstrap verification

Status: active
Local result: pass
Remote result: pending
Date: 2026-08-08
Branch: `build/bootstrap-workspace`
Research baseline: `fenestra-research` commit `176c42139776ed9f1ef879cd135bddadaf12a9da`

## Research

- The base `fenestra` package is occupied, so the project selected the
  `fenestra-ui` package family.
- `cargo info` found no current package record for `fenestra-ui`,
  `fenestra-ui-ir`, `fenestra-ui-runtime`, `fenestra-ui-testkit`,
  `fenestra-ui-macros`, or `fenestra-ui-platform` on 2026-08-08. This is an
  observation, not a registry reservation.
- Rust 1.97.1 is the current stable point release recorded by the initial plan
  and is separate from the unresolved public MSRV.
- The repository has no license. Every package must remain unpublished and must
  not declare a license until that governance decision is versioned.
- The bootstrap uses only workspace path dependencies. No third-party runtime or
  build dependency is admitted.

## Planning

The package ownership, delivery protocol, safety boundary, dependency policy,
CI lanes, and exit criteria are defined by:

- [Initial implementation plan](../initial-implementation-plan.md)
- [Bootstrap work units](../bootstrap-work-units.md)

The bootstrap introduces configuration and package boundaries only. It does not
claim runtime behavior, a stable facade, platform support, or technical
feasibility.

## Implementation

- Rust 2024 virtual workspace with Cargo resolver 3.
- Exact Rust 1.97.1 toolchain with minimal profile, Rustfmt, and Clippy.
- Shared Rust and Clippy lints plus crate-level `forbid(unsafe_code)` for every
  current pure crate.
- Five `0.0.0`, `publish = false` packages:
  `fenestra-ui`, `fenestra-ui-ir`, `fenestra-ui-runtime`,
  `fenestra-ui-testkit`, and `fenestra-ui-exp-0001-spine`.
- One-way internal dependency graph: the probe depends on testkit, runtime, and
  IR; testkit depends on runtime and IR; runtime depends on IR; the facade is
  intentionally empty.
- Committed lockfile with no external packages.
- GitHub Actions quality lane on Ubuntu and test lanes on Ubuntu and Windows.

## Verification

The following commands passed locally with Rust 1.97.1:

```text
cargo metadata --format-version 1 --no-deps --locked
cargo tree --workspace --edges normal --locked
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps --locked
cargo run -p fenestra-ui-exp-0001-spine --quiet --locked
```

The metadata audit also proved:

- the package-name set is exact;
- every package has `publish = false`;
- every dependency is a local path dependency;
- the dependency graph has no cycle;
- the probe builds and exits successfully without claiming product behavior.

## Remaining limitations

- The Windows CI lane cannot be considered passed until the branch is published
  and GitHub Actions records its result.
- The selected names are not reserved on crates.io.
- License, MSRV, budgets, environment ownership, and hardware validation remain
  unresolved.
- Zero tests are expected in WU-0001 because it introduces no behavior. WU-0002
  begins with failing generational-identity tests.

WU-0001 remains active until its remote quality and Windows lanes pass. These
limitations do not block local TDD work on the headless runtime.
