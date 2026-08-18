# WU-0015 first usable Fenestra application verification

Status: native automation verified

Registered native artifact: [windows-native-v1.txt](../../apps/fenestra-layout-inspector/tests/artifacts/windows-native-v1.txt)

Source commit: `95a5f35290e6b8574fe2aadcf5cfc333d38821b1`

Windows result: `pass|records=10|bytes=608|generation=6`

Artifact SHA-256: `641d34244cc3e16cc9ad618d999b3ab093e87c714c2b0ea903a58ab9fd677e29`

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

The native protocol builds the same release package, presents the runtime paint
frame through a native window, exercises real pointer, keyboard, and resize
events, and records a bounded ASCII artifact. The registered native run passed
the independent verifier.

The ordered artifact milestones are:

```text
initial-present -> pointer-move -> pointer-press -> keyed-insert
-> mutation-present -> resize -> resize-present -> close
```

The native operator uses `SendInput` for the pointer and `Space` key, then
`SetWindowPos` and a real `Alt+F4` close input. The standalone verifier
replays the artifact independently of the native event loop:

```text
target\release\fenestra-layout-inspector-verify.exe <artifact-path>
```

The versioned Windows operator performs the pure gates, runs the native binary
in the interactive desktop session through a scheduled task, invokes the
verifier, and prints the artifact SHA-256 and byte count:

```text
powershell -NoProfile -File apps\fenestra-layout-inspector\run-windows.ps1 `
  -Artifact C:\Users\sebas\fenestra-wu-0015-layout-inspector.txt
```

The artifact is not versioned until this command exits successfully and the
verifier reports `pass`.

## Nonclaims

A pure or single-machine native result does not establish broad platform,
renderer, accessibility, text, IME, packaging, or latency support.
