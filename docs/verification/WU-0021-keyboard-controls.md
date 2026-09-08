# WU-0021 verification: keyboard controls

Status: bounded increment verified; product goal remains active
Branch: `feat/keyboard-controls`
Baseline: `1c6423d`
Verified environment: Linux x86_64, Fedora 43, Wayland, Rust 1.97.1
Date: 2026-09-08 UTC

## Implemented result

The public facade and both format-3 frontends now support compositional
buttons and checkboxes, explicit semantic labels, checked and disabled state,
and optional state colors. The [design](../design/keyboard-controls.md)
defines the accepted syntax, composition, input and publication contracts.
Visible labels and indicators are ordinary authored descendants.

The owned input vocabulary is available without native dependencies.
Headless dispatch and the native host share one deterministic reducer for
Tab traversal, Enter and Space activation, pointer release, cancellation,
window focus and composition suppression. Legacy press notifications remain
available; semantic control actions use `Activated` and `CheckedChanged`.
Owned snapshots expose stable application-local IDs, labels, roles, bounds
and accepted control state. They do not install a native accessibility adapter.

Layout, interaction, descendant colors, text rasters and focus decoration
publish atomically. Conditional colors preserve authored text measurements.
Runtime preview supplies validated candidate hit geometry without publishing,
so resize and style changes retarget hover under a stationary pointer.
Focus decoration follows the complete control subtree and remains visible
over opaque children, while preserving later sibling painter order.

The [preferences consumer](../../examples/controls-app/README.md) builds three
checkboxes and Apply/Reset buttons through matching `.fen` and real `ui!`
sources. It uses ordinary Rust state, semantic events, responsive dimensions
and the existing explicit-font adapter through public runtime dependencies.
Its preferences model has no persistence or external service side effects.

## Regression evidence

- [Authoring tests](../../crates/fenestra-ui-authoring/tests/view_controls.rs)
  cover both frontends, canonical emission, explicit labels, legal state
  contexts and invalid nested input. [Macro diagnostics](../../crates/fenestra-ui-macros/tests/ui/view_control_diagnostics.rs)
  preserve source spans for eight invalid control cases. Existing format-1/2
  fixtures and earlier format-3 canonical output remain unchanged.
- [Reducer tests](../../crates/fenestra-ui/src/application/interaction/tests.rs)
  and [cancellation cases](../../crates/fenestra-ui/src/application/interaction/tests/cancellation.rs)
  cover traversal, disabled targets, repeats, synthetic input, modifiers,
  matching release, focus loss, composition and pointer departure. Escape
  initially left a pointer arm live; its failing regression now passes.
- [Public control tests](../../crates/fenestra-ui/tests/controls.rs) exercise
  keyboard and pointer semantics without native features. [Geometry tests](../../crates/fenestra-ui/tests/control_geometry.rs)
  reproduced stale hover and pressed paint after a sibling resize or viewport
  clipping. Both failed before candidate preview was integrated and now pass
  with one accepted generation, current hits and preserved logical focus.
- [Atomicity tests](../../crates/fenestra-ui/tests/control_atomicity.rs)
  reject checked/disabled color rendering and retry the same operation while
  preserving accepted snapshots, text, pixels, focus and pending activation.
  They require measurement reuse and fresh raster attempts on color retries.
- [Semantic and focus tests](../../crates/fenestra-ui/tests/control_atomicity/semantics.rs)
  verify shared label/text budgets before engine work, owned snapshot identity,
  invalid setter rollback and exact focus pixels. An opaque child initially
  hid its owner's focus ring; regression coverage now includes rectangle and
  text children, nested offsets and later overlapping siblings.
- [Preview tests](../../crates/fenestra-ui-runtime/src/runtime/tests/preview.rs)
  and [spatial preview tests](../../crates/fenestra-ui-runtime/src/runtime/tests/spatial/preview.rs)
  cover candidate geometry, next generation, no-op identity, stale or poisoned
  transactions, retained-generation limits, identity reuse and rejected
  projections. Existing commit/commit_with panic and corruption checks pass.
  Exact public-method inventories explicitly include the new preview method.
- [Decoration tests](../../crates/fenestra-ui/src/application/decoration.rs)
  verify bounded allocation, small controls, transparency and interior rings.
  [Single-texel raster tests](../../crates/fenestra-ui-spatial/src/input_validation/tests/prepared_spatial_contract/raster_pixel_constant/solid_images.rs)
  compare the optimized integer case with all sixteen reference samples;
  fractional destinations and transforms retain the full sampling path.
- [Native bridge tests](../../crates/fenestra-ui/src/native/tests.rs) and
  [application window tests](../../crates/fenestra-ui/src/window/tests.rs)
  cover release, departure, negative-coordinate flooring and owned input
  delivery. The existing inspector and isolated editor retain their explicit
  press-oriented application behavior.

## Executed checks

| Check | Result |
| --- | --- |
| Root workspace, all targets/features, locked | 2,427 passed in 202 suites; zero failed or ignored |
| Root formatting | Passed |
| Root Clippy, all targets/features, warnings denied | Passed |
| Root rustdoc, all features, warnings and missing docs denied | Passed |
| Standalone typed-app, all targets/features | 3 passed |
| Standalone text-app, all targets/features | 8 passed |
| Standalone responsive-app, all targets/features | 6 passed |
| Standalone controls-app, all targets/features | 6 passed; stage export also compiled |
| Isolated text screen, all targets/features | 37 passed |
| All four consumers and isolated screen formatting/Clippy | Passed; warnings denied |
| Windows MSVC cross-check: facade, text adapter and inspector, all targets/features | Passed |
| Windows MSVC cross-check: all consumers and isolated screen, all targets/features | Passed |
| Locked metadata and default facade/adapter dependency trees | Passed |
| Format-3 checker on preferences | Passed; 5,765 source bytes and 11,149 generated bytes |

These runs total 2,487 passing tests. Existing headless consumers retain
their results: typed-app generation 3 with five nodes; text-app checksum
`23758408e9c7bfb6`; responsive-app checksum `96872b7eb6ddf26b`; isolated
text-pad checksum `69cc36a24a65d6e7`. The new control exercise finishes at
generation 25 with 22 nodes and checksum `eca494dbc1c7b447`.

The root lockfile remains unchanged, with SHA-256
`251b7fa38f06dc44ec50f1a73544aa719eabafad58f68186a02015ccb2a1a614`.
The default facade has no font or window backend. The new consumer keeps its
own workspace and lockfile. [CI](../../.github/workflows/ci.yml) now includes
its format, lint, all-feature test and headless commands alongside the other
consumers in the existing Linux/Windows configuration. Local checks passed;
this record does not claim a remote CI run.

## Native presentation and visual review

The current preferences application completed real Wayland smoke runs in
debug and release builds. Both exited after successful presentation with
generation 0, 22 nodes, 1,331,200 RGBA bytes and checksum `9de7fa0cecfbc58e`.
Their explicit PPM exports were byte-identical and matched the initial
headless stage. The current inspector separately completed native smoke.

The [state export utility](../../examples/controls-app/examples/export-states.rs)
renders accepted states through the same public input dispatch and handlers:

| Stage | Viewport | Focus | Current choices: notifications/compact/autosave | Apply disabled | RGBA checksum |
| --- | --- | --- | --- | --- | --- |
| Initial, also native presented | 640 x 520 | None | true / false / true | true | `9de7fa0cecfbc58e` |
| Changed | 640 x 520 | compact | true / true / true | false | `7652bf0643139ab3` |
| Reset after one application | 420 x 560 | apply | true / false / true | false | `eca494dbc1c7b447` |

Visual inspection confirmed readable labels, distinct checked and disabled
states, visible focus and wrapped instructions at the narrower viewport.
Apply remains enabled after Reset because the last applied settings differ
from the restored defaults; Reset itself becomes disabled.

Versioned PNGs are lossless RGB conversions of the explicit PPM exports:

| Frame | PNG bytes | SHA-256 |
| --- | --- | --- |
| [Wayland frame](../../examples/controls-app/evidence/wayland-frame.png) | 37,075 | `19110430166d9ac337d98cdcd1da04653e77a06b1706abbf0e3af90969d90542` |
| [Changed headless frame](../../examples/controls-app/evidence/changed-frame.png) | 38,891 | `e480fec662e6aead44ba5163c09a4a5b3a154be66894dea64cfcdab565eca6eb` |
| [Reset headless frame](../../examples/controls-app/evidence/reset-frame.png) | 38,054 | `d7b4ffbaab77364f5111c75cc7a198f7807cdaead660a031a171ff8a06e999ad` |

Reproduce from the repository root:

```sh
cargo run --manifest-path examples/controls-app/Cargo.toml --release --locked --features native -- --native-smoke --ppm controls-native.ppm
cargo run --manifest-path examples/controls-app/Cargo.toml --locked --example export-states -- controls-states
```

The native export contains application pixels after presentation, without
desktop decorations or compositor effects. Changed and reset frames are
headless exports. Smoke does not inject native input or manually resize the
compositor window; tests and the exercise drive the shared application path.

## Remaining scope

Native accessibility and assistive-technology verification, editable controls,
clipboard, caret and selection geometry, scrolling and qualified native IME
remain open. Broader component bindings, layout alignment, platform lifecycle
and performance budgets also remain separate product gates.

Reference raster limits bound logical text area and focus image texels, not
total heap use or the cost of retained snapshots and image copies. No new
performance or accessibility conformance claim follows from the example.
Windows cross-compilation is build evidence; current Windows control
presentation, X11 and platform-wide input behavior are not qualified here.
The [completion goal](../project-completion-goal.md) remains active.
