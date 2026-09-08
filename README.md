# Fenestra

Fenestra is a native UI framework written in Rust for Windows and Linux. It uses HTML-like components and typed CSS-inspired styling, but does not embed a browser, WebView, or JavaScript runtime.

The framework is being built around its own UI tree, layout, styling, rendering, and native window integration. It is intended for normal application interfaces as well as transparent windows, overlays, notifications, advanced 2D graphics, and safely exportable framework-owned surfaces.

The idea is to keep the useful parts of writing web interfaces without actually shipping a web browser with the application.

Native capture, audio, encoding, and transport are application or ecosystem concerns rather than Fenestra core responsibilities. The Cargo package family uses `fenestra-ui` and `fenestra-ui-*`; its pre-alpha bootstrap is active under the [initial implementation plan](docs/initial-implementation-plan.md).

Workspace packages follow the ratified [pre-1.0 versioning policy](docs/versioning-policy.md): versions use `MAJOR.MINOR.PATCH`, and an intentional pre-1.0 compatibility break advances `MINOR` without requiring a compatibility shim.

## Current status

This is an unpublished `0.2.0` prototype. The `fenestra-ui` facade now exposes
named views, typed styles, application updates, hit testing and optional native
windows. Format 3 compiles nested `.fen` and `ui!` views into that public API.
The facade also provides authored text views, bounded Unicode editing and owned
keyboard, focus and IME events. An explicit-font adapter and a standalone text
application exercise multilingual shaping, rendering and atomic text updates.
Rows and columns support intrinsic sizing, weighted fill, minimum/maximum
bounds and text wrapping under resize. Qualified IME, accessible controls,
general layout alignment and release packaging remain unfinished.
See the [completion goal](docs/project-completion-goal.md) for
acceptance gates and the [work units](docs/bootstrap-work-units.md) for evidence.

## Run the examples

From the repository root, install [rustup](https://rustup.rs/) if needed. Cargo
uses the exact Rust toolchain pinned in [rust-toolchain.toml](rust-toolchain.toml)
(`1.97.1`), including rustfmt and Clippy. Package downloads require network
access on the first build.

Check a small named view, then run the standalone public API consumer:

```sh
cargo run -p fenestra-ui-authoring --bin fenestra-check --locked -- examples/typed-app/src/panel.fen
cargo run --manifest-path examples/typed-app/Cargo.toml --locked
```

This consumer compiles `.fen` at build time, compares it with the real `ui!`
macro in its tests, and depends only on `fenestra-ui` at runtime. It changes
one card's color and width, then resizes the viewport. To interact with it:

```sh
cargo run --manifest-path examples/typed-app/Cargo.toml --locked --features native -- --native
```

Click either card to toggle its highlight. Its handler keeps ordinary Rust
state and updates elements by name. The [typed example guide](examples/typed-app/README.md)
explains the syntax, build helper, API and that example's fixed-size layout.
Native window dependencies are optional; headless applications need no desktop.

The [responsive application](examples/responsive-app/README.md) combines a
fixed sidebar with flexible content, text that sizes to its lines, and weighted
cards. Window resizing automatically recalculates their geometry and wrapping:

```sh
cargo run -p fenestra-ui-authoring --bin fenestra-check --locked -- examples/responsive-app/src/workspace.fen
cargo run --manifest-path examples/responsive-app/Cargo.toml --locked
cargo run --manifest-path examples/responsive-app/Cargo.toml --release --locked --features native -- --native
```

Use `width: auto`, `width: fill`, or `width: fill(2)` in either frontend, with
optional `min_width` and `max_width`; height has the same policies. The
[layout design](docs/design/responsive-layout.md) defines sizing, measurement
and overflow behavior. The example needs no application resize calculations.

The earlier format-2 example remains available as a conformance reference:

```sh
cargo run -p fenestra-ui-authoring --bin fenestra-check --locked -- examples/hello-panel.fen
cargo run -p fenestra-layout-inspector --example hello-panel --locked
```

The example starts with two blue cards. Its headless run selects the first
card, inserts a third, resizes, and reports generation `3`, `4` logical nodes,
keys `[10, 20, 30]`, and a `224x160` viewport. The
[example guide](examples/README.md) explains the matching `.fen` and `ui!`
sources and their current syntax restrictions.

Open the same compiled content in a native window:

```sh
cargo run -p fenestra-layout-inspector --example hello-panel --locked -- --native
```

Click a card to select it, press Space to add a card, resize the window, and
close it normally. For one native presentation followed by automatic exit,
replace `--native` with `--native-smoke`.

Native execution requires an interactive Windows desktop or Linux Wayland
session with its client libraries. The current inspector enables Wayland on
Linux; an X11-only session is not supported by this application. Native
presentation here uses a CPU raster and softbuffer. The separate
[Windows GPU experiment](docs/design/windows-interactive-gpu-spine.md) has
its own evidence and does not change the inspector's renderer.

Run the more extensive spatial inspector fixture with:

```sh
cargo run -p fenestra-layout-inspector --bin fenestra-layout-inspector --locked
cargo run -p fenestra-layout-inspector --bin fenestra-layout-inspector-native --locked
```

## Try text and keyboard editing

The [authored text application](examples/text-app/README.md) compiles matching
format-3 `.fen` and `ui!` text elements, supplies a versioned font explicitly,
and updates content and typography through the public facade:

```sh
cargo run -p fenestra-ui-authoring --bin fenestra-check --locked -- examples/text-app/src/panel.fen
cargo run --manifest-path examples/text-app/Cargo.toml --locked
cargo run --manifest-path examples/text-app/Cargo.toml --release --locked --features native -- --native
```

The window demonstrates basic append and Backspace editing. Replace `--native`
with `--native-smoke` for one presentation and automatic exit. Text is wrapped
and clipped to fixed boxes; full measurements include clipped lines. The
replaceable `fenestra-ui-text` adapter uses explicit ordered fonts and reports
unsupported glyphs. The default `fenestra-ui` dependency graph has no font
backend. See the [text view design](docs/design/authored-text-views.md) and
[adapter decision](docs/decisions/0001-provisional-text-adapter.md) for scope.

The [text candidate screen](probes/text-candidate-screen/README.md) compares
Parley and cosmic-text against the same bundled font and multilingual corpus.
Its native text pad uses Fenestra's public editing and window APIs with a
disposable cosmic-text adapter:

```sh
cargo run --manifest-path probes/text-candidate-screen/Cargo.toml -p fenestra-text-screen-cosmic --features native --bin text-pad --release --locked -- --native
```

Click the editor to place its caret, type, select text with Shift and the arrow
keys, or use Ctrl+A, Backspace and Delete. Omitting `--native` runs the bounded
headless exercise. `--native-smoke` presents one frame and exits. The probe
is CPU rendered and retains its separate experimental editor geometry. See
the [text and input design](docs/design/text-input-foundation.md)
for the current editing and composition boundaries.

## Check authored syntax

`fenestra-check` reports errors as `file:line:byte-column`, followed by the
diagnostic and exact byte range. It detects format 2 or 3 and applies each
format's bounds. Format 3 supports ordinary comments, named elements and text
content in cooked or raw Rust string literals, fixed/auto/fill dimensions and
numeric minimum/maximum bounds;
format 2 retains its original typed IR and diagnostic contracts. The checker
does not execute a source file.

```sh
cargo run -p fenestra-ui-authoring --bin fenestra-check --locked -- --help
cargo run -p fenestra-ui-authoring --bin fenestra-check --locked -- --emit-rust examples/hello-panel.fen
```

The second command writes only canonical Rust to stdout. Applications compile
this output at build time; the inspector's build also checks that `.fen` and
`ui!` produce the same four typed programs.

## Verify the workspace

```sh
cargo fmt --all -- --check
cargo test --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo doc --workspace --all-features --no-deps --locked
```

CI runs tests on Ubuntu and Windows. Headless tests are reproducible without a
desktop; native presentation and manual usability require separate evidence.
