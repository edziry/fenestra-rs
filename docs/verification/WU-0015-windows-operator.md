# WU-0015 Windows native operator

Status: verified

## Scope

This operator validates the release layout inspector in the existing Windows
interactive desktop session. It drives the application through the native
window boundary and retains one bounded ASCII artifact. It does not claim GPU
backend support, broad Windows coverage, accessibility, text, IME, packaging,
latency, or multi-window behavior.

## Command

Run from the repository root on Windows with an artifact path that does not
already exist:

```text
powershell -NoProfile -File apps\fenestra-layout-inspector\run-windows.ps1 `
  -Artifact C:\Users\sebas\fenestra-wu-0015-layout-inspector.txt
```

The script refuses a dirty worktree, runs format, package tests, clippy,
documentation, and release binary gates on toolchain
`1.97.1-x86_64-pc-windows-msvc`, then registers one limited interactive
scheduled task. The task starts the release native binary with
`--artifact=<path>`.

## Native sequence

The interactive task waits for the native window and performs this exact
sequence:

1. Move the real cursor to client coordinate `(4,3)` and send a left click.
2. Send one physical `Space` key press to insert keyed tile `30`.
3. Resize the window to `704x460`.
4. Send a real `Alt+F4` close input.

The application writes the artifact only after the close event and verifies
the bytes before returning success. The separate release verifier then checks
the header, printable ASCII and LF encoding, record bounds, milestone order,
runtime generations, hit and selection flags, keyed order, viewport changes,
and raster byte counts.

## Retention

Preserve the exact artifact bytes, verifier output, source commit, Windows
version, Rust toolchain, artifact byte count, and SHA-256 together. Do not
claim a pass from a screenshot, process exit alone, or an artifact that the
standalone verifier did not accept.

## Recorded result

The registered session used Windows 10 build `26200`, Rust
`1.97.1-x86_64-pc-windows-msvc`, and source commit
`95a5f35290e6b8574fe2aadcf5cfc333d38821b1`. The verifier reported:

```text
pass|records=10|bytes=608|generation=6
```

The retained artifact SHA-256 is
`641d34244cc3e16cc9ad618d999b3ab093e87c714c2b0ea903a58ab9fd677e29`.
