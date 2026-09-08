# WU-0022 verification: native accessibility

Status: bounded increment verified; product goal remains active
Branch: `feat/native-accessibility`
Baseline: `fce6b75`
Verified environment: Linux x86_64, Fedora 43, Wayland, Rust 1.97.1
Date: 2026-09-08 UTC

## Implemented result

The public application facade now produces an owned accessibility tree and
accepts semantic Focus and Activate requests without native dependencies.
The [design](../design/native-accessibility.md) defines stable application-local
IDs, roles, labels, physical bounds, logical focus and active window focus.
Buttons and checkboxes expose their explicit labels as leaf controls; ordinary
text outside controls remains a separate label. Composed visual descendants
do not duplicate the control's accessible content.

The native host translates these accepted snapshots through AccessKit to
AT-SPI on Linux and the UIA adapter on Windows. Application windows opt in
automatically. Existing custom `WindowContent` implementations retain the
default opt-out; an opted-in provider can later withdraw its content tree.
Window focus, activation, deactivation, resize and successful callback updates
follow the host lifecycle. Repeated identical trees do not republish updates.

Semantic actions reuse the existing control transition and atomic publication
path. Rejected or unavailable targets do nothing. A rendering failure preserves
the accepted tree, control state, pixels and pending physical activation, and
the same semantic request can be retried. Activate does not move logical focus.
The native host keeps its existing callback-error exit policy.

## Dependency admission and regression evidence

[ADR 0002](../decisions/0002-native-accessibility-adapter.md) and its
[admission evidence](../decisions/evidence/native-accessibility-admission-v1.json)
record exact candidate versions, features, declared MSRVs, licenses and the
dependency audit. The default facade remains independent of window, font and
accessibility backends. `native` adds optional target-scoped AccessKit adapters
alongside the existing winit and softbuffer path.

The pinned upstream AT-SPI translation incorrectly marked disabled buttons as
Enabled and Sensitive. The licensed
[local patch](../../third-party/accesskit-atspi-common/FENESTRA-PATCH.md) adds one
missing disabled-state guard. Regression tests first reproduced the disabled
state and transition failures, then passed with the guard. The control retains
its actual button role. This is a local correction, not an upstream release.

The root lockfile SHA-256 is
`f8b7c317837fac559abc955693d4d630f0130420047942ba4de99b30fb8463c2`.
All five standalone workspaces apply the same local correction and keep their
own updated lockfiles. No pre-existing package versions were replaced.
The typed-authoring lock inventory now records the actual root lock bytes.
The hybrid-spatial native-renderer artifact changes only its two dependency
closure hashes: the lock-only closure gains `futures-macro` through the
new adapter's `futures-util` features. The
[artifact note](../../probes/exp-0008-hybrid-spatial/tests/artifacts/README.md)
records this delta; baseline, observations and classifications are unchanged.

- [Owned-tree and action tests](../../crates/fenestra-ui/tests/accessibility.rs)
  cover identity, labels, subtree omission, focus, activation, cancellation and
  rejection of disabled, stale, non-control and offscreen requests.
- [Atomicity tests](../../crates/fenestra-ui/tests/control_atomicity/accessibility.rs)
  preserve old snapshots and pending input after rejected rasterization, then
  retry successfully. They also cover standalone text labels and updates.
- [Native adapter tests](../../crates/fenestra-ui/src/native/accessibility/tests.rs)
  cover role/state/action translation, bounds, window roots and request filters.
  [Host tests](../../crates/fenestra-ui/src/native/tests.rs) and
  [application window tests](../../crates/fenestra-ui/src/window/tests.rs) cover
  provider lifecycle, callback dispatch and closure.
- [AT-SPI disabled-state tests](../../third-party/accesskit-atspi-common/src/fenestra_disabled_tests.rs)
  query public translated states and state-change events. These dependency
  tests run separately from the root workspace tests.
- [Probe helper tests](../../examples/controls-app/tools/test_atspi_client.py)
  reject foreign-child traversal, check D-Bus argument types, compare labels
  and state, and distinguish retained logical focus from active focused state.

## Actual Linux AT-SPI and presented pixels

The public-API [native probe](../../examples/controls-app/examples/a11y-probe.rs)
adds a Finish control to the preferences example. The
[verification helper](../../examples/controls-app/tools/README.md) discovers
only its own child process on the real desktop AT-SPI bus. It queries the
exported tree and sends actual GrabFocus and click requests, then waits for
matching Rust snapshots and successfully presented raster exports.

The host session had `IsEnabled=false` and `ScreenReaderEnabled=false` both
before and after the run. A private session-bus Status fixture activates the
child's adapter; the accessibility tree and actions use the original desktop
AT-SPI bus. This verifies an activated native bridge without changing desktop
preferences. It does not demonstrate screen-reader usability or activation
while accessibility remains disabled in the child's own session.

The probe ignores physical keyboard, pointer and IME input to isolate the
accessibility action sequence. Actual compositor focus and close events remain
enabled. Normal application input is unchanged. OS focused bits are compared
with accepted active-window state, and logical focus is checked separately.
The compositor can change window focus independently of accessibility actions.

The [debug report](artifacts/WU-0022/atspi-debug.json) records four successful
checkpoints, each with all eleven exported nodes and six controls. It verifies
roles, labels, bounds, enabled/sensitive bits, checked and focused state, stable
object identity, absence of composed control children, and updated Readout text.
The final Finish action closes the native process normally after presentation.

| Stage | Generation | Logical focus | Window focused | Apply count | RGBA checksum |
| --- | --- | --- | --- | --- | --- |
| Initial | 0 | None | true | 0 | `91dd60df59f8de21` |
| Focused | 1 | compact | true | 0 | `3dbd8c516a0c0d39` |
| Changed | 6 | compact | false | 0 | `2453bb7c79221284` |
| Applied | 11 | compact | true | 1 | `cdef0096d5590403` |

The Changed stage correctly has no active focus ring while the window is
inactive, despite retaining compact as the logical target. Apply becomes
enabled after the checkbox action, then disabled after application. Reset
remains enabled because the applied preferences differ from the defaults.

The [release report](artifacts/WU-0022/atspi-release.json) independently passes
the same four stages and closes normally. Its window remained focused, with
generations 0, 1, 5 and 7. Initial, Focused and Applied pixels are byte-identical
to debug. Changed has checksum `8f06055aecb4fbec` and an active focus ring,
matching its actual window state. Event-dependent generations and focus pixels
are observations, not fixed cross-run expectations.

Versioned PNGs are lossless RGB conversions of the successfully presented
640 x 520 PPM exports. Pixel equality was checked during conversion, and the
helper reconstructed opaque RGBA bytes to verify each application's checksum.
They show application pixels, without desktop decorations or compositor effects.
The raw report retains the source PPM hashes and ephemeral process/bus IDs as
observations from this run; source PPM filenames are not repository links.

| Frame | PNG bytes | SHA-256 |
| --- | --- | --- |
| [Initial](artifacts/WU-0022/initial.png) | 39,439 | `41ec54e67c42417ced95a538c37a729e8f20529356b1f80598b16e0bf7f0e2dc` |
| [Focused](artifacts/WU-0022/focused.png) | 39,497 | `9213e8cddff4ee8a80aa5e9630a272864da8b31f6a4231239daba1fc1d238650` |
| [Changed](artifacts/WU-0022/changed.png) | 41,296 | `27f5685f851cfb14b26c289a1725a57a9286b4142989e26d76ea7892bb970313` |
| [Applied](artifacts/WU-0022/applied.png) | 39,999 | `159e2b8a69850593237876480b96b2f5b7f4cb40798ec17f0ce40f94b0fbd75d` |
| [Release Changed](artifacts/WU-0022/release-changed.png) | 41,353 | `794800ca1c2662b9bdec82e4ea6244a35637f8ab192ccca50440fa9963e0a144` |

Visual review confirmed readable text, distinct checkbox and disabled-button
states, changing status text and focus decoration matching active window state.

## Bounded memory and CPU observation

A separate activated debug run measured only the owned native probe process
every five seconds for one minute after its first presented frame and AT-SPI
registration. The [measurements](artifacts/WU-0022/memory-debug.json) record
RSS, proportional set size (PSS), anonymous memory, swap and interval CPU from
Linux process counters. CPU percentages refer to one core; the first sample
has no prior interval and is not an idle measurement.

The process used 27.4 to 31.4 MiB RSS, ending at 29.0 MiB, and 24.3 to 27.7 MiB
PSS, ending at 25.3 MiB. Its swap use stayed zero. CPU approached one full core
during the first redraws, then settled to zero in every interval from 20 to
60 seconds. Presentation count stopped at seven with generation zero. There
was no self-sustaining idle redraw loop in this sample. Finish closed the
native process normally and the original accessibility settings stayed intact.

The [release action run](artifacts/WU-0022/memory-release.json) completed in
2.137 seconds, with six process samples at roughly 250 ms intervals. Observed
RSS was 16.6 to 20.0 MiB, PSS 12.8 to 16.2 MiB, and swap zero. This short action
run has no deliberate idle interval and is not a memory regression threshold.

Concurrent compiler processes were measured separately: individual rustc and
rustdoc processes reached about 445 and 673 MiB RSS respectively. These
observations do not attribute total desktop memory or existing host swap to
the window. They also do not prove the absence of a long-running leak.

The remaining cost is concrete: `Application::raster` recomputes the committed
reference raster on each call, and this instrumentation requests it for both
presentation and export. Reusing unchanged committed pixels is a separate
performance improvement; this increment does not claim to implement it.

## Executed checks

| Check | Result |
| --- | --- |
| Root workspace, all targets/features, locked | 2,444 passed in 203 suites; zero failed or ignored |
| Root formatting and Clippy, warnings denied | Passed |
| Root rustdoc, all features, warnings and missing docs denied | Passed |
| Standalone typed/text/responsive/controls consumers, all targets/features | 3 / 8 / 6 / 6 passed |
| Isolated text screen, all targets/features | 37 passed |
| All standalone workspaces formatting, strict Clippy and headless exercises | Passed |
| Vendored AT-SPI package, separate locked library tests | 10 passed |
| Vendored package strict Clippy under the product lock | Passed |
| Portable AT-SPI helper unit tests | 11 passed, including without Python site packages |
| Windows MSVC facade/text adapter/inspector, all targets/features | Passed |
| Windows MSVC all standalone workspaces, all targets/features | Passed |
| Native probe strict Clippy and Windows MSVC cross-check after input isolation | Passed |
| Locked metadata and default facade/adapter dependency trees | Passed |
| Product dependency audit, warnings denied | Zero reported vulnerabilities or warnings |

These checks total 2,525 passing tests. Existing headless consumers retain
their results: typed-app generation 3 with five nodes; text-app checksum
`23758408e9c7bfb6`; responsive-app checksum `96872b7eb6ddf26b`; controls-app
checksum `eca494dbc1c7b447`; isolated text-pad checksum `69cc36a24a65d6e7`.
The text candidate evidence remains byte-identical.

[CI](../../.github/workflows/ci.yml) runs the portable helper tests on Linux and
Windows and explicitly tests/lints the patched AT-SPI dependency under the root
lock on Linux. The real desktop bus probe remains a separate native check.
Local checks passed; this record does not claim a remote CI run or native
Windows UIA qualification.

The ordinary controls application also completed release Wayland native smoke
through `Application::run`: generation 0, 22 nodes, 640 x 520 pixels and checksum
`9de7fa0cecfbc58e`. The existing inspector completed its separate `--smoke`
path with the default accessibility opt-out. Both processes exited normally.

## Remaining scope

Windows UIA interaction, screen-reader workflows, accessibility event
announcement behavior, scrolling and offscreen navigation, editable controls,
caret/selection geometry and native IME remain open. Current generic containers
may be flattened by the platform adapter. Root clipping does not establish
that every offscreen sibling is absent from the AT-SPI tree; unavailable
controls expose no focus/click actions and application requests reject them.

Performance and total heap budgets require separate measurements. Cross-builds
do not qualify native Windows presentation, and no new X11 support is claimed.
The [product completion goal](../project-completion-goal.md) remains active.
