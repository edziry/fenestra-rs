# Responsive workspace example

This standalone consumer uses the public `fenestra-ui` application API and
`fenestra-ui-text` adapter to build a responsive workspace. Its
[workspace.fen](src/workspace.fen) and [workspace.ui](src/workspace.ui) describe
the same view through the host build helper and real `ui!` macro. The package
has its own workspace and lockfile.

Run from the repository root:

```sh
cargo run --manifest-path examples/responsive-app/Cargo.toml --locked -- --headless
cargo test --manifest-path examples/responsive-app/Cargo.toml --locked
```

No arguments also select the headless exercise. It begins at 820x520, narrows
the viewport to 520x520, updates the paragraph and card highlight, changes text
typography, and expands to 960x520. Each checkpoint reports the resulting
sidebar, main-column and paragraph dimensions, wrapped line count, and card
position. The final line reports the frame checksum. Bounds come from the
committed application, and the application computes all layout dimensions.

## Responsive composition

The root fills the viewport with 24 pixels of padding. Its header chooses an
automatic height, and a row fills the remaining space. That row contains a
170-pixel sidebar and a main column that fills its remaining width. The
paragraph fills the main width and chooses its height from wrapped text.
The cards follow it in source order, so rewrapping updates their positions,
paint and hit targets together.

```text
column main {
  width: fill;
  height: fill;
  text paragraph {
    content: "This paragraph wraps to the available width.";
    width: fill;
    height: auto;
  }
}
```

The two cards request weights 1 and 2. Their explicit minimum and maximum
widths bound distribution; when both reach their maxima, unused space remains
at the end of the row. The sidebar also demonstrates fixed pixel sizing with
minimum and maximum bounds. Layout below those minima overflows and the
viewport clips output. A tiny viewport does not force negative dimensions.

There are no breakpoint branches or resize callbacks that calculate child
widths or heights. The example supplies the versioned
[DejaVu Sans font](../../assets/fonts/dejavu-sans-2.37/README.md) explicitly and
does not discover system fonts. It does not implement scrolling, text editing,
a focus system, keyboard activation, accessibility controls, or a reusable
button component. The cards are ordinary named containers with click handlers.

## Native window and frame export

On Linux Wayland or Windows, opt into the native feature:

```sh
cargo run --manifest-path examples/responsive-app/Cargo.toml --locked --features native -- --native
cargo run --manifest-path examples/responsive-app/Cargo.toml --locked --features native -- --native-smoke
cargo test --manifest-path examples/responsive-app/Cargo.toml --locked --features native
```

Resize the window normally. Click Highlight to toggle its background and
Change notes to replace the paragraph with a shorter message or restore it.
The callback changes application state and authored properties; it never
computes layout. Smoke mode exits after one successful presentation without
injecting input.

An explicit `--ppm PATH` exports the final frame:

```sh
cargo run --manifest-path examples/responsive-app/Cargo.toml --locked -- --headless --ppm /tmp/responsive-headless.ppm
cargo run --manifest-path examples/responsive-app/Cargo.toml --locked --features native -- --native-smoke --ppm /tmp/responsive-presented.ppm
```

The binary PPM contains the RGB channels of the final premultiplied RGBA8
raster; transparency appears over black. Native smoke export happens after
successful presentation. This is an application frame export, not a desktop
screenshot, and includes no window decorations or compositor effects.

Export all three headless checkpoints to compare wrapping and card placement:

```sh
cargo run --manifest-path examples/responsive-app/Cargo.toml --locked --example export-stages -- /tmp/responsive-stages
```

This writes `initial.ppm` at 820x520, `narrow.ppm` at 520x520, and
`updated.ppm` at 960x520. The first two retain the original paragraph; the
last uses the same content, typography, and highlight updates as the headless
exercise. Each export reports its bounds, line count, and checksum.

Runtime dependencies are `fenestra-ui` and `fenestra-ui-text`.
`fenestra-ui-authoring` is a host build dependency; no generated target code
invokes it. Inspect the application dependency boundary with:

```sh
cargo tree --manifest-path examples/responsive-app/Cargo.toml --locked --edges normal,no-proc-macro --depth 1
```
