# Authoring examples

Start with the [standalone typed application](typed-app/README.md) for the
public API and format-3 named component syntax. Continue with the
[text application](text-app/README.md) for multilingual rendering and basic
editing, or the [responsive application](responsive-app/README.md) for
intrinsic text sizing, weighted fill and layout under window resize.
The [preferences application](controls-app/README.md) adds keyboard navigation,
checkboxes, buttons and state styling with ordinary Rust event handlers.
The format-2 example below remains a reference for the earlier experimental
IR authoring contract.

`hello-panel.fen` describes a dark panel containing two blue cards. Its matching
`hello-panel.ui` contains the same program inside `ui! { ... }`. Both use the
existing experimental format-2 grammar and compile into the same schema,
construction, style, and spatial programs.

Run these commands from the workspace root:

```sh
cargo run -p fenestra-ui-authoring --bin fenestra-check --locked -- examples/hello-panel.fen
cargo run -p fenestra-layout-inspector --example hello-panel --locked
cargo test -p fenestra-ui-authoring --test examples --locked
```

The checker validates the source and reports compiler diagnostics. The test
also checks the Rust token frontend, all four lowered programs, and canonical
Rust generation. The executable selects the first card, inserts a third and
resizes the viewport. It reports generation `3`, nodes `4`, keys
`[10, 20, 30]`, and viewport `224x160`, without opening a window.

To interact with the same content on Windows or a Linux Wayland desktop:

```sh
cargo run -p fenestra-layout-inspector --example hello-panel --locked -- --native
```

Click a blue card to select it, press Space to add cards, and resize or close
the window. Use `--native-smoke` for one presentation followed by automatic
exit. The current inspector is bounded; it is not an unlimited canvas.

## Reading the example

The file has four sections:

1. `schema` declares typed properties and the projections invalidated by each
   change. Numeric IDs connect authored properties to runtime updates.
2. `construction` creates one root and a keyed region with two instances of
   the `tile` template, identified by keys `10` and `20`.
3. `style` assigns the initial blue tone to the tile template.
4. `spatial` gives each template a layout recipe, rectangle geometry, and paint.
   Each card also declares its own hit region and semantic geometry.

With a 192 by 128 viewport, the panel begins at `(8, 8)` and measures 160 by
88. The cards begin at `(20, 20)` and `(84, 20)` and measure 56 by 64. Changing
`tile.tone` in `style` changes both initial card colors. Changing the `span_x`
default changes both card widths and their placement in the row.

`dimension(minimum, preferred, maximum)` bounds a layout dimension. Shape
coordinates are local to their spatial node. `fixed(65536)` means one unit in
the signed 16.16 representation; `fixed(0)` is the local origin. A `property`
binding reads the matching typed property from the logical node.

Property names are identifiers, so grammar words such as `width`, `height`,
and `input` cannot be reused as names in format 2. This example uses `span_x`,
`span_y`, and `policy` instead. Keep both authoring files equivalent when
editing them; the parity test detects divergence.

## Runtime integration

The `.fen` file is compiler input. A host build script can call
`compile_fen_v2` and `canonical_rust_v2`, then include the generated Rust in
the application. The `.ui` file is an expression that can be included where
`fenestra_ui_macros::ui` is in scope. Both return the raw program quadruple.
The [layout inspector build script](../apps/fenestra-layout-inspector/build.rs)
compiles this example and its registered conformance scene, checking `.fen`
and `ui!` parity for each. The [application example](../apps/fenestra-layout-inspector/examples/hello-panel.rs)
includes the generated program and passes the resulting inspector to the
same native presenter used by the default scene.

The example uses tone property ID `4` and keyed region ID `0`, matching the
inspector's selection and insertion conventions. Its cards accept hit tests;
application code still owns pointer dispatch and property mutations. A
semantic shape describes geometry and does not provide a native accessibility
control or label.

The format is an experimental typed intermediate authoring language. It has
no text widgets, CSS cascade, HTML element vocabulary, runtime parser, or
stable public API. See the [format-2 grammar](../docs/design/hybrid-spatial-authoring-v2.md)
and [source contract](../docs/design/hybrid-spatial-authoring-source-v2.md) for
the complete supported syntax and limits.
