# Keyboard control preferences

This standalone consumer builds three checkboxes and two buttons using only
the public `fenestra-ui` application API and `fenestra-ui-text` adapter at
runtime. Its [preferences.fen](src/preferences.fen) and
[preferences.ui](src/preferences.ui) produce the same view through the host
build helper and real `ui!` macro. It has its own workspace and lockfile.

Run from the repository root:

```sh
cargo run --manifest-path examples/controls-app/Cargo.toml --locked -- --headless
cargo test --manifest-path examples/controls-app/Cargo.toml --locked
```

No arguments also select the headless exercise. It navigates with Tab,
changes a checkbox with Space, sends a repeat, applies with Enter, cancels
another Space press through focus loss, resets the defaults, and resizes from
640x520 to 420x560. Checkpoints report committed preferences, focus, Apply
availability, and the number of applications. The final line reports a frame
checksum. Independent runs are compared by the integration tests.

## Application behavior

Show notifications and Save changes automatically begin checked; Use compact
spacing begins unchecked. Apply is enabled when current preferences differ
from the last applied preferences. Reset is enabled when current preferences
differ from the defaults. Reset changes the current checkboxes; Apply accepts
those choices. These preferences and the application count live in ordinary
Rust state for this process. The example does not persist settings or perform
notification and automatic-save services.

The framework commits checkbox state before `CheckedChanged` reaches the
handler. [DemoState](src/state.rs) reads public snapshots, handles
`CheckedChanged` and `Activated`, and updates the dependent controls and
readout. Pointer press alone does not activate buttons. The handler does not
use the legacy `Click` event to run commands. Native input uses the framework's
automatic dispatch; the headless helper uses public `dispatch_input` followed
by the same handler.

Every control has an explicit semantic label and separate visible children.
The indicator is an authored text `X` with a transparent base color and white
`checked_color`; its parent supplies `checked_background`. Owners provide
hover, pressed, disabled, and focus colors. Visible labels specify their own
disabled text color. The control state applies to descendants without
inheriting colors or creating implicit child nodes.

```text
checkbox compact {
  label: "Use compact spacing";
  checked: false;
  width: fill;
  height: auto;
  text caption {
    content: "Use compact spacing";
    width: fill;
    height: auto;
    disabled_color: rgba8(109,123,140,255);
  }
}
```

The layout fills the viewport and wraps text through `fill` and `auto` sizing.
The application does not compute dimensions in resize callbacks. It supplies
the versioned [DejaVu Sans font](../../assets/fonts/dejavu-sans-2.37/README.md)
explicitly, without system font discovery. Public semantic snapshots expose
stable application-local IDs, labels, roles, bounds, and accepted control
state. With `native`, the framework projects these snapshots through its
OS accessibility adapter. Composed control children are represented by their
owner's explicit label; standalone text, including the changing readout,
remains available as separate labels.

## Native window and frame export

On Linux Wayland or Windows, opt into the native feature:

```sh
cargo run --manifest-path examples/controls-app/Cargo.toml --locked --features native -- --native
cargo run --manifest-path examples/controls-app/Cargo.toml --locked --features native -- --native-smoke
cargo test --manifest-path examples/controls-app/Cargo.toml --locked --all-targets --all-features
```

Use Tab to move among enabled controls, Space to toggle a checkbox, and Enter
or Space to activate a button. Pointer activation requires a matching release
over the control. Smoke mode exits after a successful presentation without
injecting input.

The [native accessibility contract](../../docs/design/native-accessibility.md)
routes focus and activation requests to these same committed controls without
simulating keyboard or pointer input. Disabled and offscreen controls advertise
no actions, and stale requests are checked against current eligibility. A
[versioned AT-SPI correction](../../third-party/accesskit-atspi-common/FENESTRA-PATCH.md)
keeps disabled buttons from being reported as enabled or sensitive.

[WU-0022](../../docs/verification/WU-0022-native-accessibility.md) records an
actual Linux AT-SPI consumer run covering initial, focused, changed and applied
states. The consumer checked control labels, roles, bounds and state, the
standalone readout, and corresponding presented frames. Focus and activation
requests crossed the real desktop accessibility bus. Only the probe's
activation-status service used a private fixture; desktop `IsEnabled` and
`ScreenReaderEnabled` remained false before and after. This is evidence for
the activated native bridge, not a screen-reader usability test. Windows has
cross-compilation evidence; real UIA queries and interaction remain open.

Export the final frame, or several inspectable control states:

```sh
cargo run --manifest-path examples/controls-app/Cargo.toml --locked -- --headless --ppm /tmp/controls-headless.ppm
cargo run --manifest-path examples/controls-app/Cargo.toml --locked --features native -- --native-smoke --ppm /tmp/controls-presented.ppm
cargo run --manifest-path examples/controls-app/Cargo.toml --locked --example export-states -- /tmp/controls-states
```

The utility exports the initial disabled actions, a focused changed checkbox
with enabled actions, and reset defaults with a focused Apply button at the
narrower viewport. PPM contains the RGB channels of the public premultiplied
RGBA8 raster; transparent pixels appear over black. Native smoke export occurs
after successful presentation. These are application frame exports, without
window decorations or compositor effects, rather than desktop screenshots.

Inspect the direct runtime dependency boundary:

```sh
cargo tree --manifest-path examples/controls-app/Cargo.toml --locked --edges normal,no-proc-macro --depth 1
```
