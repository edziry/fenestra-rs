# Application work units

Continuation of the [bootstrap work units](bootstrap-work-units.md).
The [completion goal](project-completion-goal.md) remains the product gate.

## WU-0021: Keyboard controls

Branch: `feat/keyboard-controls`
Design: [keyboard controls](design/keyboard-controls.md)
Verification: [keyboard control verification](verification/WU-0021-keyboard-controls.md)

- Define authored buttons and checkboxes, explicit semantic labels, checked
  and disabled state, composition rules and state-dependent colors.
- Share owned input and deterministic keyboard/pointer transitions between
  headless callers and the native host.
- Publish control state, layout, text colors, focus decoration, hits and
  semantic snapshots atomically; retry rejected changes without losing arms.
- Exercise the public API in a standalone preferences application and extend
  the existing Linux/Windows CI consumer checks.

Exit: bounded controls can be authored, focused, activated, styled and inspected
without importing prototype APIs. Native accessibility, editable controls,
clipboard, scrolling, general component bindings and platform qualification
remain open.

## WU-0022: Native accessibility

Branch: `feat/native-accessibility`
Design: [native accessibility](design/native-accessibility.md)
Admission: [provisional native adapter](decisions/0002-native-accessibility-adapter.md)
Verification: [native accessibility verification](verification/WU-0022-native-accessibility.md)

- Export immutable owned window/group/label/button/checkbox trees with stable
  application-local identities, physical bounds and retained logical focus.
- Route current eligible focus and activation requests through transactional
  control publication, without forged input or partial state on failure.
- Integrate a private AccessKit bridge under the existing native capability,
  preserving default headless dependency isolation and legacy window opt-out.
- Correct the upstream AT-SPI disabled-button mapping with a minimal licensed
  vendored patch and public state/transition regressions.
- Verify four live Linux AT-SPI stages against committed snapshots and native
  presented frames, including focus, checkbox activation, Apply and updated
  standalone readout text.

Exit: the activated Linux bridge exposes real controls and accepts semantic
actions over the desktop accessibility bus. The probe's private activation
status fixture leaves global accessibility and screen-reader settings
unchanged. Windows is cross-compiled; real UIA interaction and screen-reader
usability remain open, along with editable controls, clipboard, scrolling,
general component bindings and broader platform qualification.

## WU-0023: Committed raster reuse

Branch: `perf/committed-raster-cache`
Verification: [committed raster reuse](verification/WU-0023-committed-raster-cache.md)

- Measure the preferences application's per-process memory and unchanged-frame
  rendering cost before attributing desktop resource use to the native window.
- Retain one successful owned raster for the accepted generation, with lazy
  rendering, bounded pixel storage and no retained runtime snapshot history.
- Preserve no-op and rejected-update behavior, invalidate after successful
  publication, and retry failed rendering without returning stale pixels.
- Exercise public mutation and snapshot contracts and repeat native AT-SPI
  actions against the cached rendering path.

Exit: repeated reads of one accepted frame avoid reference sampling, while
changed frames and failed updates preserve their existing correctness
contracts. Whole-process memory budgets, incremental rendering and larger
accessibility-tree performance remain separate gates.
