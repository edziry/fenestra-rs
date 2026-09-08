# WU-0018 verification: text and input foundation

Status: bounded increment verified; product goal remains active
Branch: `feat/text-input-foundation`
Baseline: `bc43981`
Verified environment: Linux x86_64, Fedora 43, Wayland, Rust 1.97.1
Date: 2026-09-08 UTC

## Implemented result

The facade now exposes bounded owned `TextBuffer`, directed UTF-8 `Selection`,
typed editing errors and committed element `Bounds`. Native hosting delivers
owned keyboard, modifiers, focus and opt-in IME events, while preserving the
inspector's existing Space action. Candidate types remain private.

The [isolated text screen](../../probes/text-candidate-screen/README.md)
compares Parley 0.11.1 and cosmic-text 0.19.0 on twelve fixed cases and one
versioned DejaVu Sans font. Both agree on measured paragraph dimensions and
line counts at the recorded precision; source-range, whitespace, raster and
editing differences are documented. Neither is selected permanently.

The native text pad composes candidate text over a public facade panel. It
supports clicking to place a caret, logical grapheme editing, directed
selection, repeated text presses, provisional preedit, committed input and
focus cancellation. Its fixed text panes use the actual committed bounds.
Invalid edits and resizes preserve accepted state; text and caret paint remain
clipped to their pane and viewport.

## Regression evidence

- Text construction checks byte limits before copying. Selection endpoints,
  combining sequences, emoji ZWJ sequences, CRLF, regional indicators and
  grapheme changes after replacement are covered by 18 editing tests.
- Four bounds tests cover nested layout, successful updates, zero dimensions,
  numeric edges and atomic rejected overflow.
- Native mapping covers releases, repeats, synthetic events, AltGr-preserving
  text, modifiers, compatibility Space, focus loss and late unfocused preedit.
- The inspector clears hover on focus loss and does not duplicate insertion
  when it receives the expanded keyboard/IME event vocabulary.
- Ten candidate editor tests expose and correct LFCR offset underflow,
  partial RTL ligature selection/hits and soft-wrap caret choice, alongside
  hard-line, empty-text and grapheme-boundary cases. These corrections and
  remaining affinity limitations are recorded in the screen README.
- Native probe tests cover editing, preedit/commit/cancellation, composition
  rejection, stale pointers, cache behavior, clipping, tiny/skinny viewports
  and rejected resize state.

## Executed checks

| Check | Result |
| --- | --- |
| Root workspace tests, all targets/features, locked | 2,256 passed in 185 suites; zero failed or ignored |
| Root formatting | Passed |
| Root Clippy, all targets/features, warnings denied | Passed |
| Root rustdoc, all features, warnings and missing docs denied | Passed |
| Standalone typed-app tests | 3 passed |
| Standalone typed-app Clippy, all targets/features | Passed with warnings denied |
| Windows MSVC cross-check: facade and inspector, all targets/features | Passed |
| Windows MSVC cross-check: standalone typed-app, all targets/features | Passed |
| Isolated candidate libraries and editor geometry tests | 15 passed |
| Complete isolated screen, all targets/features, locked | 27 passed, including 12 text-pad tests |
| Isolated screen Clippy, all targets/features | Passed with warnings denied |
| Isolated screen rustdoc, all features | Passed with warnings denied |
| Windows MSVC cross-check: full isolated screen and native text-pad targets | Passed |
| Locked metadata and default facade dependency graph | Passed; no text/font/window candidates in the default graph |

These are local checks. CI now includes the isolated screen and text-pad
contracts on both configured operating systems; a remote CI run is not claimed.

## Native presentation and visual review

The current inspector completed a real Wayland smoke presentation and exited
successfully. The text pad also presented successfully in both debug and
optimized builds. Its smoke output reported `820x580`, `121` content bytes,
`1,902,400` RGBA bytes and one successful native presentation. A separately
executed headless edit sequence reported `126` content bytes and `3` selected
bytes; repeated runs returned identical output.

The [exported Wayland frame](../../probes/text-candidate-screen/evidence/text-pad-wayland.png)
preserves every RGB pixel of the native probe's explicit PPM export. Visual
inspection confirmed readable Latin, Greek, Arabic and Hebrew, correct pane
spacing and a visible caret. It is exported frame content after presentation,
not a screenshot of the desktop or evidence of manually exercised input.

Reproduce the native frame with:

```sh
cargo run --manifest-path probes/text-candidate-screen/Cargo.toml -p fenestra-text-screen-cosmic --features native --bin text-pad --release --locked -- --native-smoke
```

Replace `--native-smoke` with `--native` for the interactive probe. Use the
optimized build for interactive work: the debug reference raster is slow.
No startup or input-latency service level is established by these smoke runs.

## Bounded execution improvement

Text changes originally reran the full reference rasterizer for the unchanged
panel background. The text pad now retains that owned background lazily and
reuses it across text, selection and focus changes. It invalidates the cache
only after a successful changed-size layout. Final frames also stay cached
while input does not change their content.

The probe's typing/cache regression observes one background raster across an
initial frame and three typing frames, then two after a successful resize;
an invalid resize does not invalidate it. The same three warmed typing frames
took 35.956 seconds before caching and 137 milliseconds after caching in a
local debug run on the shared machine. This is a bounded diagnostic observation
under uncontrolled machine load, not a framework performance benchmark.
Initial background raster cost and fresh candidate font/layout contexts remain.

## Acceptance limits and next work

This increment exercises owned editing and native input foundations. It does
not qualify a normal text control or the complete text product gate. Required
next work includes text elements in the authored view/runtime, font discovery
and fallback, intrinsic measurement and style invalidation, scrolling, undo,
clipboard, visual bidi navigation/affinity, native accessibility and platform
IME lifecycle/candidate placement.

The fixture font has missing CJK/emoji glyphs by design. Full Unicode
conformance, malformed fonts, broad fallback, production text caches and
multiplatform raster/performance results are not established. Current Windows
execution and Windows IME interaction were not available on this Linux host;
cross-compilation is only build evidence. Linux X11 remains unsupported by the
present native shell. Text is CPU rendered in this probe; the separate Windows
GPU evidence is unchanged and does not qualify GPU text.

The [design](../design/text-input-foundation.md) and the
[completion goal](../project-completion-goal.md) retain these open gates.
