# Versioning policy

Status: ratified for pre-1.0 development
Effective version: `0.1.0`
Last updated: 2026-08-09

## Scope

Fenestra package releases use three numeric Semantic Versioning components:
`MAJOR.MINOR.PATCH`. The policy follows
[Semantic Versioning 2.0.0](https://semver.org/) and Cargo's
[SemVer compatibility guidance](https://doc.rust-lang.org/cargo/reference/semver.html),
with the explicit pre-1.0 rules below.

This policy governs package versions and release communication. Versioned IR,
trace, artifact, and authoring formats retain their own closed format versions
because package and persisted-data compatibility are different contracts.

## Pre-1.0 development

The first pre-alpha workspace line is `0.1.0`. Before `1.0.0`, no public API or
backward-compatibility promise exists.

- `MINOR` advances for an intentional incompatible API, behavior, ownership,
  dependency, or supported-format change. The new line may remove the old
  behavior directly; a compatibility shim, deprecation window, or dual path is
  not required.
- `PATCH` advances for a correction or internal improvement intended to retain
  the current minor line's documented contract.
- An incompatible change discovered while preparing a patch advances `MINOR`
  instead. It is not hidden inside a patch release.
- Every known incompatibility records the affected surface, migration action,
  executable evidence, and replacement boundary in the owning work unit.

All workspace packages move in lockstep while their internal prototype surfaces
remain tightly coupled. Internal path dependencies use an exact package version
so a mixed workspace cannot silently combine incompatible prototype lines.

## Stable releases

Version `1.0.0` will define the first stable public API only after publication,
license, namespace, MSRV, support, and replacement-boundary gates are ratified.
At and after `1.0.0`:

- `MAJOR` advances for backward-incompatible public API changes;
- `MINOR` advances for backward-compatible public functionality; and
- `PATCH` advances for backward-compatible fixes.

The project may still choose a clean break instead of preserving compatibility,
but it must communicate that break through the required `MAJOR` increment.

## Format versions

Persisted and replayable formats do not infer compatibility from the Cargo
package version. A format change must do one of the following:

1. preserve the current canonical grammar and semantics;
2. introduce a new explicit format version and reject unsupported versions with
   a closed diagnostic; or
3. replace the old format when its owning experimental contract permits a clean
   break, with regenerated versioned fixtures and migration evidence.

Support for decoding older experimental formats is optional until a separate
retention policy requires it.

## Publication boundary

The workspace remains `publish = false`. The version is evidence and dependency
coordination, not a release or stability claim. Registry reservation, project
license, final MSRV, support environments, numeric budgets, and the public
facade remain separate governance gates.
