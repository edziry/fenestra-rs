# AccessKit AT-SPI disabled-state patch

This directory vendors `accesskit_atspi_common` 0.20.0 from crates.io, with
one local behavior patch and regression tests. It is not an upstream release
containing the fix.

- Upstream repository: <https://github.com/AccessKit/accesskit>.
- Source commit: `42e53b0d829e7a0b34dc8803bb06012db2e80cc6`.
- Upstream source directory: `adapters/atspi-common`.
- Registry archive: <https://static.crates.io/crates/accesskit_atspi_common/accesskit_atspi_common-0.20.0.crate>.
- Archive SHA-256: `c9d47ad644916f6cb7e432a5ca0dbd7cc78281a9772881057bb72d4aadb93257`.

The normalized registry `Cargo.toml`, `Cargo.toml.orig`, `README.md`,
`.cargo_vcs_info.json` and source files were copied from that archive. The
package's generated lockfile, registry cache marker and upstream changelog
are omitted. The product workspace lockfiles control resolution.
`LICENSE-APACHE`, `LICENSE-MIT` and `LICENSE.chromium` are copied verbatim
from the source commit's repository root. Source copyright notices remain
intact. These terms concern the dependency; they do not select a license for
the Fenestra project.

## Local delta

In `src/node.rs`, the branch which inserts AT-SPI `Enabled` and `Sensitive`
now requires `!state.is_disabled()`. Upstream inserted those states for a
disabled `Button` because that role does not support the read-only property.
The fix retains the actual button role and keeps existing enabled and
read-only mappings. It does not broaden the supported control set or alter
the separate upstream read-only behavior.

`src/lib.rs` includes the test-only `src/fenestra_disabled_tests.rs`. Its
tests query the public `PlatformNode::state()` result and observe state-change
events while disabling and reenabling a button. They also preserve enabled
button, default-button and checked-checkbox roles and action support.

Run from the Fenestra workspace:

```sh
cargo test --locked -p accesskit_atspi_common --lib
```

The registry source reproduces failing disabled-state and transition tests;
the one-condition patch makes them pass. An actual AT-SPI consumer test is a
separate native integration check, not simulated by these translation tests.

Review this small delta when upgrading AccessKit. Remove the override only
after the replacement package passes these regressions and the native query.
Do not replace the role or suppress disabled-state tests to avoid the defect.
The broader admission and dependency audit belong to
[ADR 0002](../../docs/decisions/0002-native-accessibility-adapter.md).
