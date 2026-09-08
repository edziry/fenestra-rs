# Text editing geometry and native IME context

Work unit: WU-0024
Status: implemented; integrated editable controls remain open

## Why this precedes an authored field

The [editing foundation](text-input-foundation.md) preserves bounded UTF-8 text
and extended grapheme selections. The [text view contract](authored-text-views.md)
provides shaped pixels and aggregate measurements. An editable field also needs
source-indexed carets, hit testing, selection highlights and a clipped text
viewport. These must come from the same shaping pipeline as its visible text.

This increment establishes those owned, validated adapter operations and an
outbound native IME context. The next integrated control will reuse them with
`TextBuffer`, focus traversal, atomic application publication and authored
syntax. Geometry alone does not constitute an editable native control.

## Source positions and queries

Positions use original UTF-8 byte offsets plus upstream/downstream affinity.
Anchor and focus retain selection direction. Both offsets must be extended
grapheme boundaries under the existing `TextBuffer` policy. Affinity resolves
the two visual positions possible at a bidi or soft-wrap boundary; it does not
permit an offset inside a grapheme.

A geometry request combines the existing bounded measurement request, a
selection and one query: current geometry, point hit, visual horizontal
movement, logical start/end of a visually wrapped line or vertical movement
with a preferred x.
Results contain the selected source positions, caret rectangles, bounded
highlight fragments, full metrics and a scroll extent including trailing
whitespace and empty final lines. Query processing allocates no raster.
The scroll extent follows layout advances rather than glyph ink bounds.

The facade validates requests before candidate work and validates returned
offsets, metrics, rectangles and cardinality before use. Coordinates remain
fractional text-space positions until painting. They must be finite and fit
the supported signed pixel domain. Highlight counts have their own bound:
glyph count alone cannot bound a selection containing only newlines.

Engines that implement only the previous text API report geometry unavailable.
They must not silently fabricate geometry or estimate a position by measuring
independent string prefixes.

## Parley compatibility policy

The admitted adapter uses the pinned layout's cursor and selection operations.
Candidate hit and navigation positions are filtered to the facade's extended
grapheme boundaries; a candidate byte inside a combining sequence is not an
editable position. Progress and fallback searches are bounded.

The [licensed local Parley source](../../third-party/parley/FENESTRA-PATCH.md)
contains focused corrections for RTL line-breaking groups and caret edges
around hard breaks. Public wrap-style overrides could not reliably preserve
combining groups because their RTL component metadata is reversed. The local
delta keeps the existing shaping and word-breaking policies and the exact
dependency version. Historical isolated candidate evidence retains registry
Parley and is not rewritten to conceal the original behavior.

CRLF is represented as one line break for shaping, with checked mapping back
to original source byte offsets. Stored content remains unchanged. The same
projection is used for measurement, rasterization and geometry, so a caret
cannot disagree with pixels due to different line-break handling.

Parley divides some ligature advances among component positions. This adapter
retains that candidate caret policy and does not inspect GDEF caret tables.
These positions are not claimed to be font-designed internal caret values.
They remain distinct from arbitrary per-character prefix-width estimates.
Empty content must have a valid caret and line height without treating a
candidate's phantom space as actual scrollable text width.

## Viewport rasterization

An explicit viewport request adds wrap width and nonnegative whole-pixel text
offsets to the existing text raster request. The adapter translates actual
glyph positions before clipping to the requested raster. Metrics and geometry
remain in full text coordinates; changing the viewport does not change source
positions or erase clipped content.

The default method delegates to an old engine only for its previous contract:
zero offset and wrap width equal to the raster width. Other combinations
report viewport rendering unavailable. A custom old engine must never silently
ignore a requested scroll offset or no-wrap mode.

## Native IME context

Custom native content may return an owned disabled context or an active context
with stable window-local editor identity and accepted physical caret bounds.
The default absent context preserves the existing static window option. No
candidate or window-library types cross this seam.

The host gates explicit contexts on focus and drawable size, caches applied
state and enables IME before setting the cursor area. Switching editor identity
resets the active context even when the caret rectangles match. Successful
input, resize, presentation and accessibility callbacks refresh the requested context;
failed callbacks keep the existing native error policy.

This gives a later field a way to position candidate UI. It does not qualify
Windows TSF, IBus/Fcitx workflows, reconversion or surrounding-text operations.
The pinned Wayland event path does not expose deletion of surrounding text.

## Product boundary

An integrated field still needs atomic value/selection/scroll publication,
composition ownership, visible selection and caret decoration, pointer dragging,
bindings and authored diagnostics. Its native text accessibility additionally
needs text-run identities and checked byte/character conversions. Clipboard,
undo, multi-line controls and platform-wide IME qualification remain explicit
parts of the [completion goal](../project-completion-goal.md).

The integration must preserve a separate incoming composition guard when
provisional content is rejected. Otherwise an editing key could mutate accepted
text while the platform is still composing. A rejected user edit must also
remain nonfatal to the native window. Accepted field decoration should use one
bounded composited bitmap, reusing the current per-node attachment budget
instead of consuming an attachment for every highlight fragment.
