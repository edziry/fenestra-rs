# WU-0017 typed application API verification

Status: application API and format-3 increment verified locally
Branch: `feat/typed-application-api`
Code checkpoint: `a356b7f`
Design: [typed application API](../design/typed-application-api.md)

## Result

Applications can construct named row, column and rectangle views with typed
styles through `fenestra-ui`, mutate them by name, hit-test their committed
geometry, render their pixels and host them in the shared native shell.
The new format-3 `.fen` and `ui!` frontends emit those public constructors.

The [standalone consumer](../../examples/typed-app/README.md) has its own
workspace and lockfile. Its sole target dependency is `fenestra-ui`; the
compiler is a build dependency and is absent from the normal target graph.
The default facade graph contains only IR, runtime, layout and spatial crates.
Optional native hosting activates private winit and softbuffer dependencies.

## Verification executed

The complete workspace passed 2,221 tests across 183 suites with no ignored
tests. The external consumer passed three additional tests. These commands
passed after integration:

```sh
cargo fmt --all -- --check
cargo test --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
RUSTDOCFLAGS='-D warnings -D missing-docs' cargo doc --workspace --all-features --no-deps --locked
cargo metadata --locked --no-deps --format-version 1
cargo tree -p fenestra-ui --edges normal,no-proc-macro --no-default-features --locked
cargo check -p fenestra-ui -p fenestra-layout-inspector --all-targets --all-features --target x86_64-pc-windows-msvc --locked
cargo fmt --manifest-path examples/typed-app/Cargo.toml -- --check
cargo test --manifest-path examples/typed-app/Cargo.toml --locked
cargo clippy --manifest-path examples/typed-app/Cargo.toml --all-targets --all-features --locked -- -D warnings
cargo check --manifest-path examples/typed-app/Cargo.toml --all-targets --all-features --target x86_64-pc-windows-msvc --locked
cargo run --manifest-path examples/typed-app/Cargo.toml --locked
cargo run -p fenestra-ui-authoring --bin fenestra-check --locked -- examples/typed-app/src/panel.fen
```

The external consumer used the repository target directory for local build
reuse. Its separate locked manifest also passed tests offline. CI now includes
the consumer's formatting, linting, tests and headless execution; the updated
workflow has not been run remotely by this increment.

## Behavioral evidence

- Six public application tests cover literal pixels, authored name order,
  exact hit targets and gaps, size changes that move sibling layout, style
  mutation isolation, no-op generations, atomic failed changes, viewport
  limits and clipping of overflowing hits to the visible viewport.
- Seven lowering tests include 301 nodes, 300 siblings and a 40-level tree,
  duplicate/invalid names, invalid styles, rectangle restrictions and exact
  node/depth limits. Three internal tests check typed lowering and bindings.
- Ten native shell tests and the application event-adapter test cover
  no perpetual redraw, pointer and keyboard dispatch, zero-size suspension,
  restored presentation, content retention and original callback errors.
- Thirteen authoring tests and five build-helper tests cover public output
  parity, names, properties, values, source spans, output limits, source/token/
  depth/element limits, ordinary and nested comments, CRLF, malformed UTF-8,
  Cargo paths and generated files. Three new macro compile-fail fixtures
  verify source-local diagnostics.
- Twelve checker tests retain format-2 diagnostic behavior and add format-3
  dispatch, comments, public generated Rust and failure locations.
- The external consumer compiles both sources with the real macro and Cargo
  build helper, compares the resulting `View` values and verifies runtime
  pixels and named hits before and after updates.

The format-3 checker reports:

```text
ok|format=3|source-bytes=560|generated-bytes=1052
```

The headless public consumer reports:

```text
generation=3 nodes=5 viewport=360x220 rgba_bytes=316800 hit(100,16)=Some("primary")
```

The new checker tests failed against the format-2-only command before its
dispatch was added. A viewport overflow regression returned a named hit at
`x == width` before the facade clipped input coordinates; it now returns no
hit. The standalone behavior test failed at generation 0 before the actual
style, width and viewport changes were implemented.

## Native and platform evidence

On the available Fedora 43 Wayland desktop, Rust 1.97.1, both commands returned
exit code 0 after creating a real window, presenting one CPU raster and
exiting:

```sh
cargo run -p fenestra-layout-inspector --bin fenestra-layout-inspector-native --locked -- --smoke
cargo run --manifest-path examples/typed-app/Cargo.toml --locked --features native -- --native-smoke
```

The public example's smoke returned a 320x192 raster, five nodes and generation
0; it does not synthesize pointer or keyboard events. The Windows MSVC
cross-checks prove compilation of the facade, inspector and external consumer,
including native code. No fresh Windows interactive run is claimed.

Historical WU-0014 GPU and WU-0015 inspector evidence remains unchanged. The
native dependency tests now explicitly permit the optional facade CPU shell
while keeping window dependencies out of the underlying core and GPU
dependencies out of all core/facade manifests. Format-1/2 fixtures, semantic
goldens, public prototype exports and diagnostic expectations still pass.

## Remaining limitations

This API is unpublished and experimental. Elements use fixed viewport pixel
dimensions, without automatic DPI scaling, flexible sizing, text measurement
or layout-driven container growth. Rust component functions can compose
elements, but authored imports, component declarations, property expressions,
keyed reconstruction and general event syntax remain later work.

Generated format-3 Rust expects the canonical `fenestra_ui` crate name. The
crate-root `extern crate <alias> as fenestra_ui;` workaround was separately
compiled successfully for a renamed dependency. Automatic dependency-name
hygiene is not implemented yet.

The independent capacities are checked correctness/resource defaults; the
301-node and depth-40 tests do not establish latency or scale budgets. Native
smoke does not prove manual usability, full input coverage, Linux GPU support,
IME, accessibility, transparency or packaging. These remain requirements of
the active [completion goal](../project-completion-goal.md).
