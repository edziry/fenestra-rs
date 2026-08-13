# WU-0014 Windows interactive GPU spine verification

Status: Windows registered execution pending
Linux pure result: pass
Linux Vulkan developer result: pending
Windows cross-compile result: pass
Windows native pure result: pending
Windows DX12 interactive result: pending
Branch: `feat/windows-interactive-gpu-spine`
Research baseline: `fenestra-research` commit
`176c42139776ed9f1ef879cd135bddadaf12a9da`

## Scope

The versioned [WU-0014 plan](../design/windows-interactive-gpu-spine.md)
defines the required scenario, candidate boundary, protocol, artifact, exit
criteria, and nonclaims.

No result is recorded until its commands, source commit, environment, artifact,
and limitations are independently checkable. A local or Linux GPU run cannot
substitute for the required Windows DX12 interactive result.

## Local implementation evidence

The probe implementation at commit `c60686f` provides:

- a target-scoped winit 0.30.13 window and exact DX12 or Vulkan wgpu instance;
- non-fallback adapter admission before Vello renderer creation;
- the registered format-2 `.fen` scene through runtime scheduling, the exact
  reference raster, one Vello image scene, wgpu submission, surface present,
  and bounded GPU completion;
- pointer mutation, native resize, suspend, restore, redraw, and close handling;
- a typed bounded artifact writer and independent verifier; and
- release-only artifact admission so a debug executable cannot claim release
  evidence.

These Linux commands passed on Rust 1.97.1:

```text
cargo build --release -p fenestra-ui-exp-0014-windows-gpu --locked
cargo test -p fenestra-ui-exp-0014-windows-gpu --all-targets --locked
cargo clippy -p fenestra-ui-exp-0014-windows-gpu --all-targets --locked -- -D warnings
RUSTDOCFLAGS='-D warnings -D missing-docs' cargo doc -p fenestra-ui-exp-0014-windows-gpu --no-deps --locked
```

All 27 retained probe tests passed. The release runner was 12 MiB on the local
Linux host. The host exposed a Wayland session and the release event loop
remained live until the unattended manual control was cancelled. No Linux
artifact was written and no Linux GPU pass is claimed.

The MSVC Rust standard library was then installed for the pinned toolchain and
this source-only cross-target check passed:

```text
cargo check -p fenestra-ui-exp-0014-windows-gpu --all-targets --target x86_64-pc-windows-msvc --locked
```

This establishes Windows type checking only. It does not link with the MSVC
toolchain, open a Win32 window, select DX12, execute GPU work, or present a
frame.

## Observed friction and corrections

Three corrections were made through failing regression tests before the
registered run:

- debug builds are rejected before forming release-labeled evidence;
- adapter identity fields are bounded without splitting UTF-8; and
- a burst of resize-drag events stages the latest extent so the recorded resize
  and completed presentation cannot diverge.

The standalone verifier is also executed by an integration test. It accepts a
complete pass artifact, rejects invalid bytes, and prints only bounded summary
facts.

## Registered execution

The exact build, interaction, and independent verification commands are in the
[Windows operator protocol](WU-0014-windows-operator.md). The goal remains open
until that protocol produces `evidence/windows-dx12-v1.txt` from a physical
Windows DX12 adapter and the native Windows gates pass at the recorded source
commit.
