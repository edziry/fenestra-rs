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
