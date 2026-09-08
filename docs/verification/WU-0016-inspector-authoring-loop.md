# WU-0016 inspector and authoring loop verification

Status: development-loop slice verified locally; broader product goal remains active
Branch: `feat/inspector-authoring-loop`
Code checkpoint: `5e914f1`
Verification date: 2026-09-07

## Implemented evidence

The layout inspector now accepts configurable raw authoring programs through
`LayoutInspector::from_programs`. Its default build compiles the registered
format-2 `.fen` fixture and expands the equivalent `ui!` fixture in `build.rs`.
The build fails closed when any of the four raw programs differ.

The default diagnostics expose:

```text
fen-bytes=Some(7714)
ui-bytes=Some(7722)
generated-bytes=Some(107789)
frontends-equivalent=true
selected-tone=Some([255, 192, 32, 255])
```

An externally supplied program quadruple reports unknown source metadata until
the application receives an explicit source metadata boundary.

## Verification commands

```text
cargo fmt --all -- --check
cargo test --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
RUSTDOCFLAGS='-D warnings -D missing-docs' cargo doc --workspace --all-features --no-deps --locked
cargo metadata --locked --no-deps --format-version 1
cargo tree --workspace --edges normal --locked
cargo check -p fenestra-layout-inspector --all-targets --target x86_64-pc-windows-msvc --locked
cargo run -p fenestra-layout-inspector --bin fenestra-layout-inspector --locked
cargo run -p fenestra-ui-authoring --bin fenestra-check --locked -- examples/hello-panel.fen
cargo run -p fenestra-layout-inspector --example hello-panel --locked
cargo run -p fenestra-layout-inspector --example hello-panel --locked -- --native-smoke
cargo run -p fenestra-layout-inspector --bin fenestra-layout-inspector-native --locked -- --smoke
```

All listed commands passed. The complete workspace test run passed 2,175 tests
across 179 suites with no ignored tests. Formatting, Clippy, documentation
with warnings and missing docs denied, manifests and dependency checks passed.

The existing registered WU-0015 artifact continues to pass its independent
verifier. It was preserved byte-for-byte; its Windows observations remain
historical evidence rather than a claim of a fresh Windows native execution.

## Added executable evidence

The small panel example compiles from 2,284 `.fen` bytes into 31,856 canonical
Rust bytes with 109 source anchors. Both frontends produce equal schema,
construction, style and spatial programs at build time and in an independent
authoring integration test.

Two application tests verify literal pixels and independent card hit targets,
selection color, keyed insertion, retained selection identity and resized
output. The headless executable reports:

```text
hello-panel|initial-generation=0|final-generation=3|nodes=4|keys=[10, 20, 30]|viewport=224x160|paints=4|hits=3|selected-tone=Some([255, 192, 32, 255])
```

Ten checker integration tests cover canonical Rust stdout, successful source
validation, command usage, missing files, CRLF locations, incomplete input,
invalid UTF-8, byte-limit crossings inside a multibyte character, unknown
symbols, detailed underlying IR limits and paths beginning with dashes.
The first six tests failed against the empty executable before implementation.

Seven native shell tests cover redraw scheduling, supported pointer input,
successive distinct insertion keys, zero-size presentation, preservation of
the evidence sequence and retention of a supplied inspector. Four regression
tests failed against the prior event handling before the correction.

## Local native evidence

Both native smoke commands above returned exit code 0 on Linux Wayland with
Rust `1.97.1` and kernel `7.1.13-100.fc43.x86_64`. Each created a real window,
formed its CPU reference raster, presented through softbuffer and exited
after the first successful presentation. No persistent native process remains
from these smoke checks.

The Windows MSVC target check succeeded for all inspector targets, including
the example. A cross-check proves compilation only; this increment did not
run the modified shell interactively on Windows.

## Limits

The result validates the application contract, build-time frontend parity,
checker behavior and local native presentation. It does not close EXP-0007,
establish authoring performance or reload budgets, or claim a public syntax or
platform support policy. Native smoke does not prove human usability, full
input coverage, idle CPU measurements or Linux Vulkan rendering. Those need
separate evidence.

Format 2 retains its registered source, depth, instance and generated-output
bounds. Repeated insertion remains subject to runtime capacity. The example
uses the inspector's tone-property and keyed-region conventions and imports
experimental programs; it is not yet a public application API. The
[completion goal](../project-completion-goal.md) records the remaining gates.
