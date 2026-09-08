# Keyboard and pointer controls

Work unit: WU-0021
Scope: compositional buttons and checkboxes, owned input dispatch, conditional
paint, visible focus and atomic public control snapshots.

## Composition and semantic labels

This increment builds on [responsive layout](responsive-layout.md) and
[authored text views](authored-text-views.md). The public
[element model](../../crates/fenestra-ui/src/model.rs) adds
`Element::button(name, label)` and `Element::checkbox(name, label)`.
Buttons arrange their children as columns; checkboxes arrange them as rows.
They use the existing pixel, intrinsic and fill sizing policies.

Each control requires an explicit semantic label that is nonempty after
trimming. The label is metadata, not hidden visible text. Authors compose
their own text, indicators and backgrounds as ordinary descendants. Changing
`set_control_label` does not change a visible child's content, and `set_text`
does not change its owner's label. There is no automatic checkbox mark.

Controls cannot contain other controls. A descendant belongs to its nearest
control owner for state and input; it cannot enable independent input.
Control input is derived, so setting `Style::input(true)` on the owner also
fails instead of introducing a second activation path. Ordinary rectangles
outside controls retain the existing input behavior.

`disabled(bool)` is valid only on a button or checkbox. `checked(bool)` is
valid only on a checkbox, including an explicit `false`. The corresponding
`Application::set_disabled` and `set_checked` methods enforce the same roles.
Disabling preserves layout and semantic identity while excluding the control
from focus traversal and semantic activation.

## State styles and authored syntax

[StateStyle](../../crates/fenestra-ui/src/state_style.rs) contains optional
color overrides. Its default supplies none. Descendants observe their
owner's state, but colors are selected independently for each styled element;
an owner's text color does not implicitly propagate to its children.

| Property | Valid context |
| --- | --- |
| `hover_background`, `pressed_background`, `disabled_background` | A control or any descendant of a control. |
| `checked_background` | A checkbox or its descendant. |
| `checked_color` | A text descendant of a checkbox. |
| `disabled_color` | A text descendant of a button or checkbox. |
| `focus_color` | The button or checkbox itself. |

Background selection starts with the authored background and applies
checked, hovered, pressed, then disabled overrides. Text selection starts
with the authored text color and applies checked then disabled overrides.
An absent higher-priority override keeps the last selected color. Focus is
a separate decoration, not another background state. Conditional styles
change paint colors; they do not change typography or dimension policies.

`Element::state_style` and `Application::set_state_style` accept these typed
builders. `Application::style`, `text_style` and `state_style` return authored
values rather than their state-derived colors.

Both format-3 `.fen` and `ui!` frontends accept the same control syntax:

```text
format 3;
view preferences {
  checkbox notifications {
    label: "Show notifications";
    checked: true;
    disabled: false;
    width: fill;
    height: auto;
    checked_background: rgba8(40,80,120,255);
    focus_color: rgba8(255,255,255,255);
    text caption {
      content: "Show notifications";
      width: fill;
      height: auto;
      disabled_color: rgba8(110,110,110,255);
    }
  }
}
```

Labels and content are strings, checked and disabled are booleans, and state
colors use the existing `rgba8` byte syntax. Frontend validation rejects
nested controls, independent descendant input and misplaced state properties.
The typed facade also checks these constraints at construction and mutation.
The [authoring tests](../../crates/fenestra-ui-authoring/tests/view_controls.rs)
cover both forms and canonical emission; the
[macro diagnostics](../../crates/fenestra-ui-macros/tests/ui/view_control_diagnostics.rs)
exercise invalid authored contexts.

## Input and activation

The [owned input vocabulary](../../crates/fenestra-ui/src/input.rs) includes
logical key presses and releases, repeat and synthetic flags, modifier
snapshots, primary pointer presses and releases, pointer departure, window
focus and IME notifications. `Application::dispatch_input` uses the same
control reducer as the native host and returns owned `Event` notifications.

| Input | Accepted control behavior |
| --- | --- |
| Tab press | Focus the next eligible control in authored order, wrapping within the view. |
| Shift+Tab press | Traverse the same order in reverse. |
| Button Enter press | Activate once for a fresh accepted press. Release clears the pressed state. |
| Button or checkbox Space press | Arm the focused control and show it pressed. |
| Matching Space release | Activate the button or toggle the checkbox once. |
| Primary pointer press | Focus and arm the enabled control under the pointer. |
| Matching pointer release over the armed control | Activate or toggle once. |
| Escape press | Cancel pending keyboard and pointer activation. |

Checkbox Enter has no semantic action. Traversal and explicit `focus(Some)`
require an enabled, nonempty control whose bounds intersect the viewport.
`focus(None)` clears logical focus. Disabled controls are skipped. The
traversal wraps within this application; there is no cross-window focus chain.

Repeated presses, including duplicate downs without a release, do not repeat
semantic activation. Synthetic key events do not activate controls.
Control, Alt and Super modifiers inhibit keyboard activation and traversal
and cancel pending key activation. Shift selects reverse Tab traversal.
The compatibility `SpacePressed` event is not a second activation source.

Pointer movement outside an armed control removes its pressed appearance.
Releasing elsewhere discards the arm. Moving back before release can restore
the pressed appearance; leaving the window cancels the pointer arm entirely.
Changing focus or disabling an armed control also cancels pending activation.
This contract does not require native pointer capture.

Window focus loss clears held input, hover and pending activations. Logical
focus is retained, so `focused_control()` can still return its name while the
window is inactive. `ControlState::focused()` and focus decoration are active
window states. IME preedit cancels pending key activation and suppresses new
semantic activation while composition is active. IME data remains owned input;
these controls do not insert or edit text.

`Activated` identifies a button command. `CheckedChanged` identifies the
checkbox and its already accepted new value. `FocusChanged` reports logical
focus changes caused by dispatch. These notifications are returned only after
successful publication. Explicit setters update snapshots without dispatching
callbacks. Legacy `Click` remains a pointer-press notification; applications
should use semantic events to execute control actions. The
[control tests](../../crates/fenestra-ui/tests/control_atomicity.rs) demonstrate
that a rejected Space release returns an error, keeps the old arm and can be
retried with the same release after the rendering failure is removed.

## Focus paint and snapshot boundary

An enabled focused control in the active window receives an interior focus
ring for keyboard, pointer and explicit focus alike. The default combines a
black outer one-pixel border and white inner one-pixel border. If either
dimension is less than three pixels, it uses a single white border. An
explicit `focus_color` selects a two-pixel border instead. Zero-area controls
produce no decoration.

The [decoration builder](../../crates/fenestra-ui/src/application/decoration.rs)
uses at most eight owned one-pixel premultiplied images. Their integer
destination strips do not overlap, so transparent colors are composited once
at corners and in small controls. Image storage does not grow with the
control's area. The existing integer sampling fast path accepts full 1x1
images scaled to these rectangles; fractional geometry retains full sampling.

Focus is appended after the control's entire subtree, including text images,
so an opaque child cannot hide it. The implementation attaches it to the last
descendant's paint order and converts the control bounds into that owner's
local coordinates. Later siblings retain their authored painter order.
The image attachment resolver verifies unchanged geometry, clips, hits and
semantics. The [focus regressions](../../crates/fenestra-ui/tests/control_atomicity/semantics.rs)
cover opaque rectangle and text children, translated descendants and later
overlapping siblings.

`control_snapshot(name)` returns an owned `ControlSnapshot`; `control_snapshots`
returns all controls in authored order. Each record contains an
application-local stable `ControlId`, name, label, role, committed bounds and
disabled, checked, focused, hovered and pressed state. A button has no checked
value. Old records remain unchanged after later accepted mutations. IDs are
stable within the static application tree and are not interchangeable between
applications.

These records provide an owned semantic boundary. They are not native
accessibility nodes. This increment does not install AccessKit, expose an
operating-system accessibility tree, or qualify assistive-technology support.

## Candidate geometry, measurements and publication

State, geometry and paint are prepared as one candidate. Layout uses authored
typography, independently of conditional text color. It retains one natural
measurement, one wrapped measurement keyed by width, and one accepted raster.
Conditional text-color changes discard only the raster; natural and wrapped
measurements remain reusable. Background and focus-only changes reuse all
text work. Content or authored `TextStyle` changes still invalidate all three.

When intrinsic resolution has obtained natural text metrics, it also obtains
metrics at the final nonzero raster width. Complete wrapped and raster-layout
metrics must agree exactly, including for a conditional color. A candidate
engine that makes a paint color change geometry is rejected with
`TextError::InconsistentMeasurement`. Fixed text without intrinsic measurement
dependencies still supports layout-only engines.

Publication proceeds in this order:

1. Validate applicable authoring and aggregate byte limits before copying
   changed content or labels and before invoking a text engine.
2. Resolve candidate sizes from authored typography, validate measurements
   and bound aggregate text raster area.
3. If geometry changes with a retained pointer position, obtain candidate
   core geometry using runtime preview. Recompute hover from its actual hit
   test, so resize and layout changes need no synthetic pointer movement.
4. Derive final control colors, prepare changed text rasters and validate
   measurement agreement.
5. Commit resolved properties and prepare owned text and focus images against
   that exact candidate. Publish nodes, interaction, caches and paint together.

The [runtime preview](../../crates/fenestra-ui-runtime/src/runtime/transaction.rs)
uses ordinary transaction validation and projection without publishing state
or reserving identities. The facade does not expose its provisional
generation as an accepted application update. A failure at any later stage
preserves accepted state, geometry, frame and generation. Candidate cache
changes are discarded; private engines may retain their own internal cache
effects. Event notifications are withheld on failure.

The existing root revision mechanism publishes effective control and label
changes even when pixels happen to agree. Identical setters remain no-ops.
Pointer coordinates and held-input bookkeeping can change without advancing
the generation when no observable control state changes.

## Budgets, consumer and remaining scope

Semantic labels and visible text share the aggregate UTF-8 byte budget in
`TextLimits`. Label metadata does not allocate a text raster. Logical text
raster area retains its own aggregate limit; focus contributes at most eight
image pixels independently of its destination area. Staged snapshots, image
copies and private engine buffers remain additional allocations, so these
bounds are not a hard heap ceiling.

The [controls consumer](../../examples/controls-app/README.md) composes a
preferences form through equivalent `.fen` and `ui!` sources, supplies an
explicit font, dispatches headless input and uses the same handlers with the
optional native window host. It exercises checked, disabled, focused and
pressed states together with responsive reflow. The provisional text adapter
and font qualification in [ADR 0001](../decisions/0001-provisional-text-adapter.md)
remain unchanged.

Native accessibility adapters and qualification remain separate work.
Editable text, caret and selection geometry, IME candidate-window placement,
tri-state checkboxes, radio groups, menus and richer focus navigation are not
provided by this increment. Native feature compilation and a runnable
example do not establish platform-wide input or accessibility conformance.
