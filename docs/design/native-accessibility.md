# Native accessibility

Work unit: WU-0022
Scope: owned semantic trees and actions, projected through the existing native
window adapter to Linux AT-SPI and Windows UI Automation.

## Owned semantics

This increment extends [keyboard and pointer controls](keyboard-controls.md)
without changing authored `.fen`, `ui!`, visible composition or state styles.
The [public types](../../crates/fenestra-ui/src/accessibility.rs) are
`AccessibilityId`, `AccessibilityRole`, `AccessibilityNode`,
`AccessibilityTree`, `AccessibilityAction` and `AccessibilityActionRequest`.
They contain no AccessKit types or native handles and are available to
headless applications.

`Application::accessibility_tree()` returns an immutable owned snapshot of
the accepted application state. It retains the generation, physical viewport,
logical focus and active-window flag. Mutations do not change old snapshots.
The tree begins with synthetic window identity zero, followed by included
authored elements in preorder. Element identities are stable within the
application; a control's accessibility identity has the same numeric value
as its `ControlId`. These are not globally unique or cross-application IDs.

The window root has an empty authored name and label, covers the viewport,
and contains the authored root. Other nodes retain their authored names and
ordered child identities. A control is a semantic leaf: its explicit label
represents its composed content, and its decorative and text descendants
are suppressed from this tree. Standalone text remains a label containing
the complete text, including content clipped by its raster box. Plain
rectangles and layout containers outside controls remain owned groups.

| Owned role | Native candidate projection | Label and state |
| --- | --- | --- |
| `Window` | `Window` | Native title replaces the empty owned label; clips children. |
| `Group` | `GenericContainer` | Preserves child order and authored identity. |
| `Label` | `Label` | Complete text uses the candidate value property. |
| `Button` | `Button` | Explicit semantic label and disabled state. |
| `Checkbox` | `CheckBox` | Explicit semantic label, disabled state and boolean toggled state. |

The candidate's platform filtering can flatten generic containers. Therefore,
the OS-visible hierarchy need not contain every owned group; relevant
descendant order and control identity remain meaningful. A neighboring
offscreen control stays in the owned tree with its role, label and bounds,
but it advertises no focus or activation action. Native filtering and platform
visibility states are separate from this owned eligibility decision.

The [snapshot exporter](../../crates/fenestra-ui/src/application/accessibility.rs)
uses accepted geometry. Bounds are unclipped rectangles in physical viewport
pixels, not parent-relative or logical-window coordinates. The native bridge
copies those coordinates without an extra transform; window position and
platform coordinate conversion belong to the candidate. `WindowOptions::size`
still specifies the initial logical window size, while resize callbacks,
raster dimensions and semantic bounds use physical pixels.

`tree.focus()` is retained logical focus, with zero meaning no focused control.
It may identify a control while `window_focused()` is false. This distinction
allows focus restoration without pretending that an inactive native window
has active keyboard focus. Native focus events also reach the candidate,
which applies host focus when exposing platform focus state.

## Eligibility and semantic actions

Current control eligibility requires an enabled control, nonzero width and
height, and intersection with the physical viewport. It uses the same
[control targets](../../crates/fenestra-ui/src/application/controls/api.rs)
as explicit logical focus. It does not test whether later siblings visually
occlude the control, and it does not automatically scroll offscreen controls
into view. The public `focusable()` flag also indicates whether the control
currently accepts activation.

`Application::dispatch_accessibility_action(request)` applies one of two
owned behaviors:

- `Focus` moves logical focus to an eligible control and reports a changed
  target through `Event::FocusChanged`.
- `Activate` invokes a button through `Event::Activated`, or toggles a
  checkbox through `Event::CheckedChanged`. It preserves logical focus and
  cancels pending pointer/key activation arms after successful publication.

These operations do not forge keyboard, pointer, IME or close-request events.
Semantic activation is independent of keyboard modifiers, composition and
active native-window focus. Cancelling an armed Space press prevents its
later physical release from toggling the checkbox a second time.

Zero, unknown, stale, non-control, disabled and fully offscreen targets are
ignored with no notifications. A previously valid snapshot is not authority
to activate a control after its current eligibility changes. The native host
first accepts only candidate `Focus` and `Click` actions for the root tree,
without action payload data. Unsupported actions are rejected before querying
application semantics. Supported requests are checked against a fresh owned
tree before calling the content callback; the Application dispatcher checks
eligibility again at its own boundary.

Candidate `Click` maps to owned `Activate`. This covers the platform's button
invoke and checkbox toggle requests without a separate fake key sequence.
After a successful semantic `Focus` callback, the host also asks the native
window to receive focus. Activation does not make that native-focus request.

## Publication and failure behavior

The dispatcher shares the existing transactional control path: it prepares
candidate control state, derived colors, text rasters and focus decoration,
then publishes accepted state before returning notifications. A failure in
layout or raster preparation preserves the old generation, geometry, raster,
semantic snapshot, focus, checked state and pending device arms. The caller
can retry the same semantic action after removing the failure condition.

The native host obtains semantics after accepted input, resize, presentation
and action callbacks. The `presented` callback runs only after successful
native presentation; any state it accepts is included in the following
semantic refresh. Candidate updates contain complete owned projections.
Their cache compares the exact optional owned tree and physical window size,
not only a generation number. It records a publication only when
`update_if_active` actually invokes the update closure.

An unchanged tree does not generate another candidate update. Idle handling
only checks closure conditions; it does not continuously fetch snapshots,
rebuild trees or redraw. Deactivation invalidates the publication cache, and
an initial-tree request forces a fresh complete projection.

Snapshot and action callback errors retain their original error in
`NativeError::Application` and terminate the native loop. A snapshot failure
before action dispatch does not invoke the action callback. This does not
promise rollback of arbitrary custom `WindowContent` code or of a user event
handler's already accepted mutations. Application transaction atomicity is
the boundary described above, not a transaction around every native callback.

## Window lifecycle and compatibility

The [WindowContent contract](../../crates/fenestra-ui/src/native.rs) adds
default methods while preserving existing implementations:

| Callback | Default and contract |
| --- | --- |
| `accessibility()` | Returns `Ok(None)`. The first call follows initial resize; `None` opts out for that window's lifetime. |
| `accessibility_action(request)` | Returns `Ok(())`; implementations opting in handle supported semantic requests on the native event-loop thread. |
| `should_close()` | Returns `false`; `true` requests closure at the next event-loop idle point. |

`Application::run` uses the
[Application window wrapper](../../crates/fenestra-ui/src/window.rs), which
opts in with `Some(app.accessibility_tree())` and forwards committed semantic
events to the same application handler used for input. Custom window content
may opt in by returning an owned tree. After opting in, returning `None`
withdraws all semantic children and focus through a complete empty window
root; it does not leave the previous controls visible to accessibility clients.
That opted-in adapter can later publish `Some(tree)` again.

The [native shell](../../crates/fenestra-ui/src/native/shell.rs) creates a
hidden window, performs initial resize, obtains the initial semantic choice,
creates the candidate adapter if opted in, and only then shows the window.
The adapter receives initial physical bounds and native focus. Subsequent
native window events reach the candidate before application handling.

Activation is asynchronous through an event-loop proxy. The candidate may
briefly expose a placeholder until `InitialTreeRequested` reaches the host.
Platform action callbacks enqueue requests rather than mutating Application
on provider threads. Events for another native window are ignored. The first
active update is complete, and `AccessibilityDeactivated` causes a later
activation to republish the tree.

`should_close()` does not synthesize a device or close-request event. Content
needing a final visible frame can wait for `presented()` before returning
true. Existing one-frame smoke mode still exits after successful presentation.
A minimized zero-area native window suspends resize/frame callbacks until
restored. These are lifecycle controls, not accessibility actions.

## Dependency and platform boundary

The existing `native` feature enables the bridge on supported Linux and
Windows targets. The default headless graph does not activate AccessKit or
native services. The candidate remains private; replacing it requires new
projection and lifecycle code without changing authored controls or owned
snapshot consumers.

[ADR 0002](../decisions/0002-native-accessibility-adapter.md) records exact
AccessKit versions, target features, licenses, advisory checks and native
service requirements. Linux uses real AT-SPI over D-Bus and waits for the
accessibility status service to enable registration. Windows uses a UI
Automation provider associated with the native window. A plain in-memory
snapshot is not a substitute for either provider.

The [local AT-SPI patch](../../third-party/accesskit-atspi-common/FENESTRA-PATCH.md)
corrects upstream 0.20.0 exposing disabled buttons as enabled and sensitive.
It preserves the actual button role and includes source provenance, retained
licenses and regressions over public state and transition events. It is a
local correction, not an assertion that the upstream release already fixes
the problem.

## Evidence and remaining scope

The [owned-tree tests](../../crates/fenestra-ui/tests/accessibility.rs) cover
identity, control leaf semantics, focus, activation and ignored targets.
The [text and failure tests](../../crates/fenestra-ui/tests/control_atomicity/accessibility.rs)
cover standalone labels, immutable snapshots and retry after failed
state-color rasterization. The
[native projection tests](../../crates/fenestra-ui/src/native/accessibility/tests.rs)
check roles, labels, bounds, actions and semantic withdrawal; the
[host action tests](../../crates/fenestra-ui/src/native/action_tests.rs)
check callback routing, current eligibility, errors and application closure.

The [WU-0022 native evidence](../verification/WU-0022-native-accessibility.md)
records a successful real Linux consumer run with four checkpoints: initial,
focused, changed and applied. The consumer discovered the application's
provider on the desktop accessibility bus, queried control roles, labels,
bounds and states, and invoked focus and activation. It compared those
observations and the changing standalone readout with accepted Rust snapshots
and frames exported after successful native presentation. Disabled-button
enabled/sensitive flags were checked before and after activation changed
availability. A semantic Finish action closed the probe normally.

Only the probe process's activation-status service was a private fixture;
the provider, queries and actions used the real desktop AT-SPI bus. Global
`IsEnabled` and `ScreenReaderEnabled` remained false before and after the run.
This demonstrates the activated native bridge, not automatic activation while
desktop accessibility is disabled or a screen-reader user experience.

Portable builds and headless tests do not establish real Windows UIA behavior
or screen-reader usability. Further roles, editable text, selection, caret
geometry, richer navigation, live regions and broader desktop/scale coverage
remain outside this increment. Owned node and label limits do not impose a
hard bound on provider allocations, IPC traffic or platform event queues.
