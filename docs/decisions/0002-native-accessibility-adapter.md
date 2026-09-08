# ADR 0002: Provisional native accessibility adapter

Status: provisional, scoped to the existing native window capability.
Record version: 1. Checked: 2026-09-08 UTC.

## Decision and owned boundary

Use AccessKit 0.25.0 and accesskit_winit 0.34.0 behind the private native
adapter in [fenestra-ui](../../crates/fenestra-ui/src/native/accessibility.rs).
The public snapshot, stable identity, roles, labels, bounds and action requests
remain Fenestra types. The application owns control state, validation and
publication. AccessKit owns operating-system provider translation and events;
it does not become the generic runtime, authoring IR or source of application
truth. No candidate node, handle or pointer enters the public snapshot.

Enable this adapter through the existing `native` capability on Linux and
Windows. Default headless consumers do not activate the candidate. There is
no backend-brand feature and no second parallel provider implementation.
The exact target-specific dependencies are recorded in the
[facade manifest](../../crates/fenestra-ui/Cargo.toml); the product lockfile
retains exact transitive versions.

This admits a replaceable native bridge for windows, groups, read-only labels,
buttons and checkboxes. It does not establish screen-reader usability,
complete accessibility conformance, editable text, rich document navigation,
live regions, virtualized lists or macOS/mobile platform support. Native
query evidence and portable compilation have different scopes.

## Exact dependency and feature inventory

| Package | Version / selected features | Declared MSRV | License metadata |
| --- | --- | --- | --- |
| accesskit | 0.25.0 / defaults disabled | 1.85 | MIT OR Apache-2.0 |
| accesskit_winit | 0.34.0 / Linux: `rwh_06`, `accesskit_unix`, `async-io`; Windows: `rwh_06` | 1.85 | Apache-2.0 |
| accesskit_unix | 0.23.0 / `async-io` | 1.85 | MIT OR Apache-2.0 |
| accesskit_atspi_common | 0.20.0 / local disabled-state patch | 1.85 | MIT OR Apache-2.0 |
| accesskit_consumer | 0.39.0 | 1.85 | MIT OR Apache-2.0 |
| accesskit_windows | 0.35.0 | 1.85 | MIT OR Apache-2.0 |
| atspi / atspi-common / atspi-proxies | 0.29.0 / 0.13.0 / 0.13.0 | 1.77.2 | Apache-2.0 OR MIT |
| zbus / zvariant | 5.19.0 / 5.15.0 | 1.87 | MIT |
| windows / windows-core | 0.62.2 / 0.62.2 | 1.82 | MIT OR Apache-2.0 |

accesskit_winit accepts winit `^0.30.5`, which includes the existing exact
0.30.13 pin. Its default features additionally enable X11 and Wayland;
disabling defaults preserves the selected Wayland-only Linux host. Linux
selects one asynchronous backend and raw-window-handle 0.6. Windows omits
the Unix backend entirely. Platform-specific `cargo tree` output, rather
than the union of features in all-platform metadata, determines each closure.
[Pinned manifest](https://github.com/AccessKit/accesskit/blob/42e53b0d829e7a0b34dc8803bb06012db2e80cc6/adapters/winit/Cargo.toml).

The Linux accessibility chain declares a maximum MSRV of 1.87 through zbus
and related packages; direct AccessKit's 1.85 declaration is insufficient
to describe that chain. These declarations are metadata, not executed MSRV
lanes or a product-wide MSRV claim. Validation uses the versioned development
toolchain. The [admission evidence](evidence/native-accessibility-admission-v1.json)
records the actual product lock hash, package inventory, target features and
security scan separately from any earlier isolated screen.

AccessKit's admitted packages were published on 2026-08-29 and were unyanked
at this check. Their shared source commit is
`42e53b0d829e7a0b34dc8803bb06012db2e80cc6`. Release recency and the active
upstream repository are maintenance observations, not future guarantees.
[Versioned release](https://github.com/AccessKit/accesskit/releases/tag/accesskit_winit-v0.34.0).

## Native lifecycle and action constraints

Create the winit window hidden, create the adapter, then show the window.
The candidate rejects construction after a window is already visible.
The selected event-loop proxy constructor can expose a temporary platform
placeholder until `InitialTreeRequested` is handled. Its first accepted
`update_if_active` result must contain the full tree. Feed every native window
event to `process_event` before application handling, including focus and
bounds changes. Seed initial bounds/focus because the Unix constructor does
not infer them from the existing window.
[Pinned adapter API](https://github.com/AccessKit/accesskit/blob/42e53b0d829e7a0b34dc8803bb06012db2e80cc6/adapters/winit/src/lib.rs).

Handle `InitialTreeRequested`, `ActionRequested` and
`AccessibilityDeactivated` through the event loop. Provider callbacks can
originate on other threads; they may enqueue owned requests but must not
mutate the Application directly. Validate the current tree identity,
supported action and current disabled state before invoking the owned action
path. Rejected application transactions retain the accepted semantic tree
and visual state. Publishing an updated tree follows accepted application
state rather than anticipating a requested action.

Map checkbox state to `Role::CheckBox` plus `Toggled::True` or `False`.
AT-SPI `Action.DoAction(0)` and UIA Invoke/Toggle request `Action::Click`;
AT-SPI `Component.GrabFocus` and UIA focus request `Action::Focus`.
Only supported focusable controls advertise these actions. Disabled nodes
retain their role and disabled flag, and the application still rejects stale
or forged actions even if an external client requests them directly.

The bridge sends full owned projections with stable IDs. Standalone labels
use the candidate label role's value property, while named controls use its
label property. Decorative descendants do not duplicate a control's semantic
label. A window with no opted-in snapshot does not activate a tree; removing
an already published snapshot replaces it with an empty window root.
Geometry and focus come from the committed snapshot, with window movement,
scale and focus handled at the native boundary.

## Local AT-SPI correctness patch

Unmodified accesskit_atspi_common 0.20.0 exposes a disabled Button as both
`Enabled` and `Sensitive`. Its state conversion only excludes those flags
inside a read-only branch, while accesskit_consumer excludes Button from
roles supporting read-only. Removing the button's actions does not fix
the incorrect state flags. GNOME defines sensitivity in terms of user
interaction; read-only values are a separate state.
[AT-SPI state contract](https://gnome.pages.gitlab.gnome.org/at-spi2-core/libatspi/enum.StateType.html).

Use the minimal, versioned
[vendored package](../../third-party/accesskit-atspi-common/FENESTRA-PATCH.md).
Its one-condition change requires `!state.is_disabled()` before adding
`Enabled | Sensitive`. Existing enabled and read-only mappings remain intact.
The override keeps the actual button role and is explicitly a local patch,
not a claim that the upstream registry release contains the fix.

Public `PlatformNode::state()` regressions reproduced the original failure
and missing disabled/enabled transition events before the fix. The corrected
package passes those regressions while preserving enabled button and checkbox
roles, action support and checked state. Its seven original unit tests are
retained. The separate live AT-SPI check must exercise the provider process,
not merely this in-process translation layer. Remove the override only after
an upgraded candidate passes both checks.

## Native services and qualification boundary

On Linux, AccessKit implements AT-SPI over D-Bus using the Rust zbus stack.
It connects to the session accessibility status service and waits for
`org.a11y.Status.IsEnabled` before registering on the accessibility bus.
`AT_SPI_BUS_ADDRESS` selects that bus after activation; it does not force
activation. The admitted API has no public force-enable switch.
[Status handling](https://github.com/AccessKit/accesskit/blob/42e53b0d829e7a0b34dc8803bb06012db2e80cc6/adapters/unix/src/context.rs),
[bus selection](https://github.com/AccessKit/accesskit/blob/42e53b0d829e7a0b34dc8803bb06012db2e80cc6/adapters/unix/src/atspi/bus.rs).

A test process may use a private session bus whose status service reports
enabled, while directing `AT_SPI_BUS_ADDRESS` to the real desktop accessibility
bus. This supplies only the activation condition: AccessKit must still
register the real application tree and execute actions across the real
AT-SPI bus. The fixture must not change desktop accessibility preferences,
impersonate a provider tree or substitute fake action results. Initial and
final desktop status, process cleanup and consumer-observed states belong
in the native evidence.

On Windows, accesskit_winit owns a native window subclass through
accesskit_windows, providing UI Automation and raising its event queue after
updates. This is the provider required for a custom-drawn control tree;
rendering controls or publishing an in-memory snapshot alone does not supply
UIA. Cross-compilation and headless CI do not validate UIA queries, Narrator,
Inspect or real Windows focus/event behavior.
[UIA provider requirements](https://learn.microsoft.com/en-us/windows/win32/winauto/uiauto-providersoverview).

## Security, unsafe and distribution review

The versioned evidence records a full actual-product-lock `cargo-audit`
0.22.2 run with `--deny warnings`, zero ignored advisories, zero known
vulnerabilities and zero warnings, against RustSec commit
`8a1eb4f933fb5821add5b4e98601ebd90b8b3538` dated 2026-09-07.
The local patch is reviewed as source because registry advisory matching
does not establish correctness of vendored modifications.

The closure includes fixed security history: event-listener 5.4.2 repairs
the StackSlot Send/Sync issue in RUSTSEC-2026-0221, and quick-xml 0.41.0 is
past the RUSTSEC-2026-0194/0195 fixes for duplicate-attribute work and namespace
memory growth. Other historical matched package advisories and resolved
versions are retained in the evidence. No advisory is suppressed to admit
the bridge. No published AccessKit repository advisory was listed at the
check date; absence is not a security audit or a guarantee against unknown
flaws.
[StackSlot advisory](https://rustsec.org/advisories/RUSTSEC-2026-0221.html),
[XML CPU advisory](https://rustsec.org/advisories/RUSTSEC-2026-0194.html),
[XML memory advisory](https://rustsec.org/advisories/RUSTSEC-2026-0195.html),
[upstream advisories](https://github.com/AccessKit/accesskit/security/advisories).

Source inspection found no unsafe block in the AccessKit schema, consumer,
AT-SPI common or Unix adapter source. Windows COM/UIA interoperation includes
unsafe native calls, and the transitive zbus, zvariant and windows bindings
contain unsafe internals. The winit adapter also contains unsafe code for
unselected mobile platforms. This is a surface inventory, not a complete
unsafe audit; the Fenestra bridge uses the safe candidate APIs.

Native IPC, executor threads, provider caches, platform event queues and
client calls remain external resource surfaces. Owned node/label budgets
bound the application's submitted tree, not every candidate allocation,
native request rate, D-Bus queue or OS provider lifetime. No hard heap,
latency or hostile-client robustness claim follows from this admission.

Package license metadata is recorded without choosing the Fenestra project
license. The vendored source retains its MIT/Apache terms and the upstream
Chromium-derived BSD notice verbatim. Distributions must preserve applicable
copyright, license and notice obligations for the complete dependency graph;
the manifest license string alone is not the entire notice inventory.

## Replacement cost and remaining gates

Replacing the candidate requires a new private snapshot projection, native
window lifecycle adapter, platform event delivery and owned-action mapping.
It also requires rerunning translation, failure-atomicity and native consumer
tests. Stable Fenestra IDs and authored controls remain independent of
AccessKit node IDs or wire representations.

Before broadening admission, execute real Windows UIA query/action tests,
screen-reader interaction tests, additional Linux desktop and scale/focus
cases, shutdown/reactivation stress, and resource/adversarial-client review.
Review the local patch on every upgrade and refresh the full product-lock
security and license inventory before distribution. Editable text and further
roles require their own contracts and platform evidence.
