# Parley editing and line-breaking patches

This directory retains Parley 0.11.1 from crates.io for targeted corrections
required by Fenestra's owned text geometry contract. It is not an upstream
release containing those fixes.

- Upstream repository: <https://github.com/linebender/parley>.
- Source commit: `eea3503dd6cf17130cbb07348e0ff2c918300e94`.
- Upstream source directory: `parley`.
- Registry archive: <https://static.crates.io/crates/parley/parley-0.11.1.crate>.
- Archive SHA-256: `22d2ff88bd3f7d68d1d9b09c7e6209f9a8e8c05088295140a2bcf2e9b17038c5`.

The normalized registry manifest, original manifest, README, source, VCS
metadata and both license files were copied byte-for-byte from that archive.
The generated package lockfile and registry cache marker are omitted. Product
workspace lockfiles control resolution. Copyright and license notices remain
intact; these dependency terms do not select the Fenestra project license.

The root workspace's exact-version path dependency supplies this implementation
to `fenestra-ui-text` and its standalone consumers. The isolated historical
text-candidate screen keeps its original registry implementation and lockfile.
The vendor is excluded from workspace membership; upstream code organization
and typography are retained rather than rewritten to local source conventions.

## Regression boundary

The [adapter geometry tests](../../crates/fenestra-ui-text/tests/geometry.rs)
cover source-byte carets, highlights and navigation through RTL hard breaks,
and preserve combining graphemes when emergency wrapping has very little room.
Both behaviors fail with the unmodified registry source. These are layout
corrections over the existing font inputs and shaper; no font engine upgrade,
system-font discovery or native text service is introduced.

## Local source delta

- `src/layout/line_break.rs` groups ligature components before considering
  their width. Logical RTL traversal encounters continuation components before
  the group's start, so it consumes that complete group and uses the start's
  metadata. Lookahead is relative to the current run and bounded by its end.
  Break opportunities remain before the whole group, preserving ordinary word
  breaks without separating an Arabic combining mark from its base.
- `src/editing/cursor.rs` chooses caret edges from logical neighbors and
  affinity. A hard break attaches to the following line, including an empty
  final line. Visual neighbors alone could incorrectly move an RTL line's
  first caret to its second line; the correction also repairs the existing
  selection highlight and Home/End operations through their shared line lookup.
- `src/bidi.rs` expects the existing deprecation of ICU's ordinal conversion
  only at `mask`. The pinned ICU 2.3 API deprecates that conversion without a
  replacement. Retaining the conversion preserves the existing bidi bitmask
  representation; an unfulfilled expectation after an upgrade requires review.
  This annotation changes no text behavior.

All other upstream files remain byte-identical to the archive. No upstream
test suite is claimed: the registry package omits its external test resources.
Fenestra's explicit-font public adapter tests exercise the changed paths,
including 21 geometry and three exact viewport tests.

Retain these regressions when upgrading and remove a patch only when the
replacement package passes them. Distribution remains a release gate: a
published package must not silently substitute the unpatched registry version
for this local implementation.

See the [editing geometry design](../../docs/design/text-editing-geometry.md)
and [provisional text adapter decision](../../docs/decisions/0001-provisional-text-adapter.md).
