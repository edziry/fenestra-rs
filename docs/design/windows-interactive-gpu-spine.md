# WU-0014 Windows interactive GPU spine plan

Status: complete
Scope: real GPU presentation and native interaction evidence
Research baseline: `fenestra-research` commit
`176c42139776ed9f1ef879cd135bddadaf12a9da`
Implementation baseline: WU-0013 at commit
`b601183521e015fcd37aa3f239e4a7fb649723b6`
Required target: `x86_64-pc-windows-msvc:dx12-win32`
Developer control: `x86_64-unknown-linux-gnu:vulkan-wayland`

## Objective

WU-0014 must prove that one immutable runtime paint frame originating in the
registered format-2 `.fen` fixture can reach a real native window, a real
non-fallback GPU adapter, an accepted wgpu submission, visible presentation,
and GPU completion while native interaction changes the committed runtime.

The Windows evidence tuple is required. A Linux Vulkan run is a developer
control and cannot replace Windows evidence. Neither tuple establishes broad
platform support.

## Governing evidence

WU-0013 already proves candidate-neutral spatial resolution, reference pixels,
Vello scene construction, native presentation ordering, and exact Linux and
Windows pure gates. Its Vello lane stops at `target-unavailable` because it
does not create an adapter, execute GPU work, or present pixels.

The pinned [candidate screen](hybrid-spatial-candidate-screen.md) records Vello
0.9.0 and wgpu 29.0.3. The [presentation contract](hybrid-spatial-presentation-v2.md)
fixes the pre-accept and post-accept boundary. WU-0014 admits no new renderer
candidate and does not revisit the WU-0013 spatial contract.

The current native spine uses winit 0.30.13 and Softbuffer 0.4.8. It remains the
CPU control. WU-0014 must not make its driver, artifact, or dependency surface
conditional on a GPU candidate.

## User scenario

A developer performs this sequence in a release build from a fresh checkout:

1. start the WU-0014 executable with one artifact path;
2. observe the registered hybrid spatial scene in a native window;
3. move the pointer over the window and press the primary button;
4. observe a newly committed and presented color generation;
5. resize the window to a distinct nonzero extent;
6. minimize and restore the window;
7. observe a frame after restoration; and
8. close the window normally.

The executable writes one bounded artifact even when the sequence stops early.
A successful process exit is permitted only when the artifact is a closed
`pass` result.

## Package and dependency boundary

The disposable package is
`probes/exp-0014-windows-gpu`. It may depend on:

- `fenestra-ui-exp-0007-typed-authoring` for the exact generated format-2
  fixture;
- `fenestra-ui-ir`, `fenestra-ui-runtime`, and `fenestra-ui-spatial` for the
  validated unpublished contracts;
- Vello 0.9.0 and wgpu 29.0.3 with exact feature-minimal pins;
- winit 0.30.13 with the same target-scoped native features as WU-0009; and
- one exact minimal future executor used only during device initialization.

No WU-0014 package may enter the dependency graph of a framework crate or an
older probe. Vello, wgpu, winit, surface, adapter, device, queue, texture, and
native-window types remain private to the new probe.

All framework crates remain `forbid(unsafe_code)`. The new probe also forbids
unsafe code and uses wgpu's owned safe surface target.

## First GPU cut

The first accepted GPU cut intentionally uses the WU-0013 reference raster as
an immutable bridge:

```text
format-2 .fen
  -> validated spatial program
  -> UiRuntime and UiScheduler
  -> RuntimePaintFrameV2
  -> exact Fenestra reference RGBA8 raster
  -> one Vello image scene
  -> Vello compute render to an intermediate texture
  -> wgpu blit to the native surface
  -> present and wait for the accepted submission
```

This is a real Vello/wgpu GPU execution and native presentation result. It is
not evidence that geometry, paths, gradients, clips, or images are lowered
directly to candidate vector commands. That direct lowering remains a later
adapter iteration and cannot be inferred from this result.

The bridge is useful because its source pixels retain the WU-0013 independent
oracle. The GPU lane can therefore focus on device and surface behavior before
adding another semantic translation.

## Backend admission

The instance enables exactly one backend for the executable target:

- Windows: DX12;
- Linux developer control: Vulkan.

The adapter request is compatible with the created surface, uses high
performance preference, and does not force a fallback adapter. Admission then
requires:

- the observed backend equals the target backend;
- the device type is neither CPU nor virtual GPU;
- the surface exposes `Rgba8Unorm` or `Bgra8Unorm`;
- the device satisfies Vello's default required limits; and
- the selected present and alpha modes are members of the surface capability
  set.

Adapter absence, wrong backend, fallback-shaped device type, unsupported
surface format, device request failure, and renderer creation failure are
closed typed results. Environment variables cannot widen the backend policy.

## Presentation protocol

For one nonzero surface and one offered frame, phases are ordered:

1. validate the viewport and physical extent;
2. form the bounded reference raster and Vello scene;
3. resize or configure private GPU targets if needed;
4. acquire the surface texture;
5. call `Window::pre_present_notify`;
6. accept the scheduler offer exactly once;
7. execute Vello compute work and the surface blit;
8. submit and present;
9. wait for the accepted submission with a bounded timeout; and
10. complete the matching scheduler submission.

Failures before phase 6 reject the offer. Failures after phase 6 report
renderer loss and never claim completion. A timeout or occlusion rejects the
current offer and may be retried. Surface loss recreates only the candidate
surface and private targets; it does not mutate the committed runtime.

Either zero physical axis and native occlusion represent absent presentation,
not a zero logical viewport. Restoration requests a fresh frame even when the
runtime generation is unchanged.

## Interaction protocol

Native events are distinguished from startup observations. The pass sequence
requires these milestones:

```text
adapter
initial-present
pointer-move
pointer-press
mutation-present
resize
resize-present
suspend
restore
restore-present
close
```

The primary press updates the root `tone` property through one runtime
transaction. The resulting generation must be greater than the generation in
`initial-present`. Resize changes the logical viewport through the existing
spatial transaction. Suspend and restore cannot fabricate a runtime
generation.

Unexpected duplicates are retained as bounded observations but cannot advance
the milestone state twice. A close before the required prefix produces `stop`,
never `pass`.

## Evidence artifact

The artifact is printable ASCII, LF terminated, and limited to 256 records,
512 bytes per line, and 64 KiB total. It contains:

- schema and probe versions;
- target tuple, Rust target, package version, and build profile;
- OS family and bounded OS version observation;
- adapter backend, device type, vendor, device, and hexadecimal UTF-8 encoding
  of name, driver, and driver information;
- surface format, present mode, and alpha mode;
- ordered milestone and typed failure records;
- runtime generation, scheduler frame, submission, physical extent, logical
  viewport, and reference-raster digest where applicable; and
- one terminal `pass`, `adapt`, or `stop` result with a closed reason.

It contains no host name, user name, home path, process ID, native handle,
window title supplied by a user, pointer coordinates, pixels, or environment
variable contents.

The runtime artifact is expected to differ by environment. A separate verifier
checks grammar, limits, target/backend coherence, milestone order, generation
rules, and the terminal result. It does not compare an environment artifact to
one universal golden.

## TDD sequence

1. Add package, dependency, public-surface, source-boundary, and CLI contracts.
2. Add pure failing tests for backend admission and closed outcomes.
3. Add pure failing tests for the milestone reducer and early-close behavior.
4. Add failing scene-bridge tests against the exact reference raster.
5. Add failing artifact grammar, bounds, redaction, and verifier tests.
6. Implement the candidate-neutral probe model and Vello scene bridge.
7. Implement the native GPU resources and presentation protocol behind target
   modules.
8. Implement the winit application and release CLI.
9. Run the Linux pure gates and a local Vulkan developer execution.
10. Record build or operator friction through regression tests and minimal
    corrections.
11. Run the required Windows DX12 sequence and version its verified artifact.

## Exit and nonclaims

WU-0014 passes only when the complete release scenario produces a verified
Windows DX12 non-fallback `pass` artifact and the Linux and Windows pure gates
pass from fresh checkouts.

It does not select Vello, wgpu, winit, DX12, Vulkan, surface formats, present
modes, or raster bridging as permanent architecture. It does not claim direct
GPU vector lowering, performance, frame pacing, input latency, device-loss
recovery, text, IME, accessibility, packaging, multi-window, transparent
windows, Linux product support, or Windows product support.
