# Linux AT-SPI native probe

This helper launches the public-API `a11y-probe` example, locates only its own
child PID on the desktop accessibility bus, and exercises real AT-SPI focus and
click requests. It compares the operating system tree with Rust snapshots and
PPM frames exported after successful native presentation.

The helper requires a running Linux Wayland session, its existing AT-SPI bus,
`dbus-run-session`, Python 3, `dbus-python`, PyGObject and GLib. It does not use
`pyatspi`, synthesize keyboard or pointer events, or change desktop preferences.

From the repository root:

```sh
cargo build --manifest-path examples/controls-app/Cargo.toml \
  --example a11y-probe --features native --locked
OUT="$(mktemp -d /tmp/fenestra-atspi.XXXXXX)"
python3 examples/controls-app/tools/atspi_probe.py --output "$OUT" -- \
  examples/controls-app/target/debug/examples/a11y-probe "$OUT"
python3 -m unittest discover -s examples/controls-app/tools -p 'test_*.py'
```

Use a new output directory for each run. `report.json` records the result,
activation conditions and four checkpoints: initial, focused, changed and
applied. Each checkpoint includes the complete queried tree, Rust control
snapshots and a PPM frame. `latest-observation.json` remains available when a
semantic assertion fails; `native.log` captures only the launched child and its
private fixture. `--inspect-only` records a presented tree and closes through
the Finish button without asserting semantic parity or exercising preferences.

The main probe checks labels, button/checkbox roles, enabled and sensitive bits,
checked and focused state, window-relative bounds, stable control object paths,
and omission of composed control children. It then focuses Compact, toggles it,
activates Apply, and activates Finish to close the example normally. Each action
must be followed by a matching committed Rust snapshot and presented pixels.
The standalone Readout label must also match its current Rust text after edits.
The PPM checksum reconstructs opaque alpha; this is valid for this example's
opaque full-window background. PPM files are raster exports, not screenshots.

## Activation fixture and actual operating-system path

AccessKit's Unix adapter waits for `org.a11y.Status.IsEnabled` before creating
its accessibility connection. `AT_SPI_BUS_ADDRESS` selects the address only
after this gate; it does not bypass activation. See the pinned upstream
[activation code](https://github.com/AccessKit/accesskit/blob/accesskit_winit-v0.34.0/adapters/unix/src/context.rs)
and [bus connection code](https://github.com/AccessKit/accesskit/blob/accesskit_winit-v0.34.0/adapters/unix/src/atspi/bus.rs).

The helper first reads the real session's status and AT-SPI address. It starts a
private session bus for the example with a minimal `org.a11y.Status` fixture
whose `IsEnabled` is true. Only this activation status is a fixture. The child
connects to the original desktop AT-SPI bus, and AccessKit exports its real tree
there. All queries, focus and click requests use that real bus. The helper
compares the original session's status before and after the run. It never sets
global accessibility or screen-reader preferences. This qualifies an activated
native bridge; it does not demonstrate automatic activation while the original
session has accessibility disabled, or a screen-reader user experience.

## Protocol and failure handling

The probe discovers application roots through `Accessible.GetChildren` and
matches their bus connection's Unix PID to the child it created. It selects
controls by `AccessibleId`, not translated labels. It queries numeric roles with
`GetRole`, records those values and gives the example's supported roles readable
names from the [AT-SPI role enum](https://gnome.pages.gitlab.gnome.org/at-spi2-core/libatspi/enum.Role.html).
It reads fresh D-Bus values
without a client-side accessibility cache, traverses at most 128 nodes and
applies bounded polling and per-method timeouts. The interface contracts are
defined by the upstream [Accessible XML](https://github.com/GNOME/at-spi2-core/blob/main/xml/Accessible.xml).

`Component.GetExtents` uses window coordinates because global screen positions
are not reliable on Wayland. It invokes `Component.GrabFocus` and the advertised
`Action` named `click`. Method success only acknowledges the request; the helper
waits for actual state and frame agreement. See the upstream
[Component XML](https://github.com/GNOME/at-spi2-core/blob/main/xml/Component.xml),
[Action XML](https://github.com/GNOME/at-spi2-core/blob/main/xml/Action.xml), and
[state definitions](https://gnome.pages.gitlab.gnome.org/at-spi2-core/libatspi/enum.StateType.html).

On failure, the helper terminates only the process group it created: the private
bus wrapper, fixture and example. Unrelated applications and the desktop AT-SPI
daemon remain running. A successful run exits through the example's authored
Finish button and requires a successful native process exit.
