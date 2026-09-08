# Typed application example

This standalone Rust consumer builds a native UI view through the public
`fenestra-ui` API. Its only application dependency is `fenestra-ui`. A host build
script compiles [panel.fen](src/panel.fen) to a `View`, and
[main.rs](src/main.rs) includes that expression from Cargo's `OUT_DIR`.
The separate [panel.ui](src/panel.ui) uses the real `fenestra_ui::ui!` macro;
the example's tests compare the resulting views for equality.

The package has its own workspace and committed lockfile. Run these commands
from the repository root:

```sh
cargo run --manifest-path examples/typed-app/Cargo.toml --locked
cargo test --manifest-path examples/typed-app/Cargo.toml --locked
```

The default run opens no window. It highlights `primary`, expands its width
from 64 to 112, and resizes the viewport from 320x192 to 360x220. It reports:

```text
generation=3 nodes=5 viewport=360x220 rgba_bytes=316800 hit(100,16)=Some("primary")
```

The tests check literal pixel colors and named hit targets before and after
those updates. They also exercise both authoring entry points in a consumer
outside the framework workspace. Generated Rust currently expects the canonical crate name `fenestra_ui`. If
the Cargo dependency is renamed, add `extern crate <alias> as fenestra_ui;`
at the consuming crate root.

No application code assigns numeric schema,
property, template, or spatial identities.

## Native window

The `native` feature opts into the window dependencies. On Windows or a Linux
Wayland desktop, run:

```sh
cargo run --manifest-path examples/typed-app/Cargo.toml --locked --features native -- --native
```

Click either card to toggle its highlight. The event handler receives a named
`Event::Click` target and uses `Application::set_background`. Two ordinary Rust
booleans in the closure retain each card's selection state. Resize or close
the window normally. `--native-smoke` exits after one successful presentation
without generating synthetic input:

```sh
cargo run --manifest-path examples/typed-app/Cargo.toml --locked --features native -- --native-smoke
```

Passing either native option without its feature produces an explanatory
error. The default dependency set does not include a window system. To inspect
application dependencies without host build scripts and procedural macros:

```sh
cargo tree --manifest-path examples/typed-app/Cargo.toml --locked --edges normal,no-proc-macro
```

`fenestra-ui-authoring` is a build dependency and does not appear in this
runtime dependency tree. During development, `CARGO_TARGET_DIR=target` can
reuse the repository's build directory when these commands run from its root.

## View syntax

Format 3 nests named `row`, `column`, and `rect` elements directly:

```text
format 3;
view example {
  row root {
    width: 200;
    height: 96;
    padding: 16;
    gap: 12;
    rect action {
      background: rgba8(48, 128, 192, 255);
      input: accept;
    }
  }
}
```

Every element defaults to 64x64 viewport pixels, zero padding and gap, a
transparent background, and `input: ignore`. Width, height, padding, and gap
are nonnegative integers. Colors use four RGBA8 channels from 0 through 255.
Names are unique ASCII identifiers beginning with a letter or underscore.
Rectangles cannot contain children or use padding and gap.

Dimensions are fixed. Containers preserve child order and do not grow to fit
their contents; resizing the viewport does not stretch the view. Children can
extend outside their containers, and the viewport clips rendered output and
input. The public facade remains experimental and currently exposes colored
rectangles and stacks; this example does not provide text widgets or native
accessibility controls.

See the [typed application design](../../docs/design/typed-application-api.md)
for the supported API and its boundaries.
