# Application checkpoint handoff: 2026-09-08

Product goal: active and incomplete
Execution: stopped at the owner's requested checkpoint; resume on request
Source branch: `feat/text-editing-geometry`
Integration target: `main`, using squash merge
Audited remote baseline: `a32025e14ceb9fdc88c385732437cf2b8d4ac344`
Verified code and inventory revision: `4359dcba8c9041f03dea815b5566f8f5cf7234af`
Workspace: unpublished `0.2.0`, Rust `1.97.1`

This handoff accompanies the checkpoint integration. Its enclosing pull request
records the final squash revision. The source branch retains the granular
implementation and verification history. A merged checkpoint does not mark the
product goal complete or publish a release.

## Goal and completed scope

Complete Fenestra as the documented native Rust UI framework for Windows and
Linux, without a browser: improve typed authoring, execution, diagnostics and
examples; close the inspector authoring loop; implement normal application
interfaces; qualify the promised platform and graphics behavior. The
[completion goal](../project-completion-goal.md) retains the mandatory gates.
No goal token budget was requested. The goal remains active in Codex because
mandatory product behavior is still missing.

The checkpoint integrates the previously local WU-0016 through WU-0024 work:

| Increment | Implemented result | Evidence |
| --- | --- | --- |
| WU-0016 | Reproducible inspector/authoring loop, diagnostics and native idle fixes | [Verification](../verification/WU-0016-inspector-authoring-loop.md) |
| WU-0017 | Public named-view API, typed styles, format-3 `.fen`/`ui!` and consumer build integration | [Verification](../verification/WU-0017-typed-application-api.md) |
| WU-0018 | Bounded Unicode text buffer, owned keyboard/focus/IME events and isolated editor | [Verification](../verification/WU-0018-text-input-foundation.md) |
| WU-0019 | Authored text, explicit fonts, shaped pixels and atomic text updates | [Verification](../verification/WU-0019-authored-text-views.md) |
| WU-0020 | Intrinsic/weighted dimensions, min/max bounds and responsive text measurement | [Verification](../verification/WU-0020-responsive-layout.md) |
| WU-0021 | Buttons, checkboxes, focus, activation and explicit state styles | [Verification](../verification/WU-0021-keyboard-controls.md) |
| WU-0022 | Native semantic bridge and actual Linux AT-SPI queries/actions | [Verification](../verification/WU-0022-native-accessibility.md) |
| WU-0023 | One lazily cached raster per accepted generation and measured native resource use | [Verification](../verification/WU-0023-committed-raster-cache.md) |
| WU-0024 | Source-indexed editing geometry, clipped text viewports and native IME caret-context hook | [Verification](../verification/WU-0024-text-editing-geometry.md) |

The preferences probe used about 30 MiB RSS in debug and 20 MiB in release
during the recorded action runs. No RAM leak was reproduced in the bounded
observations. Repeated full raster work was reproduced and corrected: the
release median for an unchanged read fell from 40.739 ms to 0.068 ms in that
measurement. Changed frames still render fully. The retained cache has its
own pixel-storage bound; these observations do not establish a process-wide
memory or latency guarantee.

## Final checkpoint verification

The following checks were executed locally on Fedora 43, Linux x86_64,
Wayland, with the pinned Rust toolchain after the text dependency corrections:

| Check | Result |
| --- | --- |
| Full workspace, all targets/features, locked | 2,504 passed in 206 suites; zero failed or ignored |
| Typed/text/responsive/controls standalone consumers, all targets/features | 3 / 8 / 6 / 6 passed |
| Isolated text-candidate workspace, all targets/features | 37 passed |
| Full workspace formatting, strict Clippy, strict rustdoc including missing docs | Passed |
| Full workspace Windows MSVC cross-check, all targets/features | Passed |
| Text consumer formatting, strict Clippy and Windows MSVC cross-check | Passed |
| Locked metadata, normal dependency tree and diff whitespace | Passed |
| Geometry gallery, headless/native, debug/release | Four successful runs; identical exported pixels |

The root and consumer paths total 2,564 passing tests. The broader historical
records additionally contain native AT-SPI actions and vendor/helper tests;
they are not counted as newly executed tests in this table.
The aggregate change retains one upstream trailing space in the licensed
DejaVu `LICENSE` file. Its original bytes are preserved; owned source and
checkpoint changes pass whitespace checks.

The gallery's RGBA checksum is `957477744035acf4`. Existing outputs remain
unchanged: text `23758408e9c7bfb6`, responsive `96872b7eb6ddf26b`, controls
`eca494dbc1c7b447`, isolated text-pad `69cc36a24a65d6e7`, and typed generation 3
with five nodes. The [gallery evidence](../verification/artifacts/WU-0024/gallery.json)
includes the real Wayland presentation exports. Cross-compilation and headless
Windows checks do not qualify actual Windows UIA or IME behavior.

## Dependency and evidence constraints

Keep both licensed local corrections and their public regressions:

- [AccessKit AT-SPI](../../third-party/accesskit-atspi-common/FENESTRA-PATCH.md)
  preserves disabled button semantics.
- [Parley 0.11.1](../../third-party/parley/FENESTRA-PATCH.md) preserves RTL
  combining groups during wrapping and corrects caret/selection line lookup.
  The source ledger identifies every changed upstream file.

No dependency version or edge changed in WU-0024. Four product lockfiles only
change Parley's source from registry to the retained path. The isolated
candidate-screen lockfile and original candidate implementation are unchanged.
The current typed-authoring root-lock inventory was refreshed; frozen source,
generated Rust, semantic/runtime fixtures and historical audit snapshots were
preserved. Do not silently substitute unpatched registry packages when preparing
distribution artifacts. Font discovery remains explicit and system fonts are
not loaded by the production text adapter.

## Next product increment: an authored editable field

No ordinary authored editable field exists yet. The gallery is static, and the
main text consumer still demonstrates append/backspace editing for one window.
The next coherent vertical slice should integrate a single-line field through
the public element API, equivalent `.fen`/`ui!` syntax, application publication,
native hosting and an actual consumer. Useful existing entry points are:

- [Text buffer](../../crates/fenestra-ui/src/editing.rs),
  [owned geometry](../../crates/fenestra-ui/src/text/geometry_request.rs) and
  [viewport contract](../../crates/fenestra-ui/src/text/viewport.rs).
- [Control dispatch](../../crates/fenestra-ui/src/application/controls/dispatch.rs),
  [publication](../../crates/fenestra-ui/src/application/publication.rs),
  [text preparation](../../crates/fenestra-ui/src/application/text.rs) and
  [layout](../../crates/fenestra-ui/src/application/layout.rs).
- [Native IME context](../../crates/fenestra-ui/src/native/ime.rs) and
  [accessibility adapter](../../crates/fenestra-ui/src/native/accessibility.rs).

Preserve these integration invariants:

1. Keep one canonical `TextBuffer`. Publish accepted value, directed selection,
   preedit display, scroll, geometry, raster and semantics atomically. Rejected
   preparation preserves the generation and earlier retained frames.
2. Keep incoming composition ownership separate from accepted preedit content.
   A rejected oversized preedit must still block ordinary editing keys until
   composition ends. Rejected user edits must not close the native window.
3. Use unwrapped shaping for a single-line field and define one line-separator
   policy across construction, setters, keyboard, IME and accessibility. Tab
   traverses focus; Space inserts from owned text input only; Enter can submit.
4. Derive selection, hit testing and scrolling from accepted shaped geometry.
   Composite decoration into one bounded bitmap per field: the existing
   per-node paint attachment budget is ten, not one slot per highlight.
5. Project only accepted physical caret bounds through a stable editor identity.
   Cancel composition on focus transfer and programmatic replacement. Raw IME
   events have no editor identity, so the field must own the composition guard.
6. A TextInput role and value alone do not implement native text accessibility.
   Add checked text-run identities and byte/character conversions, reject stale
   selection actions, and explicitly handle the backend's `u8` character-length
   limit. Verify real AT-SPI text actions; do not invent bidi character geometry.

Clipboard, undo, multiline controls, reusable components/imports/bindings,
inspector property editing, general scrolling/alignment, graphics/window
capabilities, X11, actual Windows UIA/IME, broader font qualification, measured
scaling budgets, MSRV, project license and distribution decisions remain in the
product gates. Do not mark the goal complete after the next field increment.

## Resume and verify

Read the goal and work-unit evidence, and follow the applicable repository
instructions. Start a new focused branch from the synchronized `main`; preserve
the checkpoint branch.
Use TDD and granular Conventional Commits. There are no unfinished source edits
to recover from this checkpoint.

```sh
cargo test --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo run --manifest-path examples/text-app/Cargo.toml --locked --example geometry-gallery
cargo run --manifest-path examples/controls-app/Cargo.toml --release --locked --features native -- --native
```

Use release builds for interactive evaluation. Linux native execution requires
a Wayland desktop; the current shell does not support an X11-only session.
Future pushes, merges and releases need the owner's authorization for their
new scope. Authorization for integrating this checkpoint is already recorded
by its pull request and does not publish a product release.
