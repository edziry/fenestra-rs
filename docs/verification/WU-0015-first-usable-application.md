# WU-0015 first usable Fenestra application verification

Status: implementation in progress

## Current pure acceptance slice

The application package must pass the following from a clean worktree:

```text
cargo fmt --all -- --check
cargo test -p fenestra-layout-inspector --all-targets --locked
cargo clippy -p fenestra-layout-inspector --all-targets --locked -- -D warnings
cargo build --release -p fenestra-layout-inspector --bins --locked
```

The release binary runs a deterministic task equivalent to pointer move,
selection, keyed insertion, and resize. Its output is an observation summary
whose generation, node count, key order, viewport, and selection state are
reviewed as one tuple.

## Native acceptance slice

The pending native protocol will build the same release package, present the
runtime paint frame through a native window, exercise real pointer and resize
events, and record a bounded ASCII artifact. A passing native run will be
required before WU-0015 is marked complete.

## Nonclaims

A pure or single-machine native result does not establish broad platform,
renderer, accessibility, text, IME, packaging, or latency support.
