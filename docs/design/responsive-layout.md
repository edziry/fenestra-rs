# Responsive and intrinsic layout

Work unit: WU-0020
Scope: owned dimension policies, intrinsic text measurement and atomic reflow
in the public application and format-3 authoring APIs.

## Boundary and public contract

This increment extends [authored text views](authored-text-views.md). It
replaces that increment's fixed-box-only sizing and per-text revision behavior
with a facade-owned layout pass and one application revision. The generic
runtime and frozen IR continue to receive resolved integer dimensions. They
do not receive dimension policies, strings, fonts or candidate text types.

The [public style](../../crates/fenestra-ui/src/style.rs) accepts `Dimension`:

| Policy | Behavior |
| --- | --- |
| `Px(i32)` | A nonnegative pixel preference, clamped to the authored minimum and maximum. |
| `Auto` | Hugs intrinsic content within those bounds. It does not implicitly shrink to the viewport or available sibling space. |
| `Fill(u32)` | Shares remaining main-axis space by weight, or fills the available cross-axis space independently. Valid weights are 1 through 65,535. |

`Style::width(i32)` and `height(i32)` retain their fixed-pixel meaning.
`width_mode(Dimension)` and `height_mode(Dimension)` select a policy.
`min_width`, `max_width`, `min_height` and `max_height` accept nonnegative
`i32` bounds with minimum no greater than maximum. Defaults remain a fixed
64 by 64 element, minimum zero and maximum `i32::MAX` on each axis.
`Application::style` returns the authored policy; `bounds` returns accepted
resolved geometry. `set_size` selects fixed pixel policies for both axes.

For example, the root can follow the viewport while a label derives its
height from its available width:

```rust
use fenestra_ui::{Dimension, Element, Style, View};

let view = View::new(
    "responsive",
    Element::column("panel")
        .style(Style::new().width_mode(Dimension::Fill(1))
            .height_mode(Dimension::Fill(1)).padding(12))
        .child(Element::text("body", "A paragraph whose height follows wrapping.")
            .style(Style::new().width_mode(Dimension::Fill(1))
                .height_mode(Dimension::Auto))),
);
```

The view still requires an application-supplied text engine and explicit
fonts as described in WU-0019. A resize updates available space; callers do
not need to compute each descendant's text box in a resize handler.

## Authored syntax

Both standalone `.fen` files and `ui!` support the same format-3 properties:

```text
format 3;
view responsive {
  column panel {
    width: fill;
    height: fill;
    padding: 12;
    gap: 8;
    text body {
      content: "A paragraph whose height follows wrapping.";
      width: fill(2);
      height: auto;
      max_width: 640;
      min_height: 24;
    }
  }
}
```

`fill` means `fill(1)`. Numeric width and height syntax remains unchanged,
including its canonical emission as `.width(value)` and `.height(value)`.
The new policies emit the corresponding `Dimension` builders. Min/max
properties accept integer pixels, not dimension policies. The
[dimension authoring tests](../../crates/fenestra-ui-authoring/tests/view_dimensions.rs)
cover both frontends, fixed-output compatibility, ranges and diagnostics.

## Resolution order and allocation

The [owned solver](../../crates/fenestra-ui/src/layout.rs) resolves every width
before measuring wrapped text heights. Each axis lazily derives intrinsic
subtree sizes from children, then allocates final sizes from parents. A row's
main axis is horizontal; a column's main axis is vertical.

An intrinsic rectangle contributes zero before bounds. A container's main
axis sums child contributions and gaps, then adds both padding edges. Its
cross axis takes the maximum child contribution and adds both padding edges.
An `Auto` container asks a `Fill` child for its intrinsic contribution during
this calculation. The child receives its actual allocation afterward, so
intrinsic sizing never asks an unresolved parent for available space.

For main-axis allocation, subtract padding, gaps and the accepted extents of
fixed and `Auto` children. The
[fill allocator](../../crates/fenestra-ui/src/layout/fill.rs) then:

1. Reserves each fill child's minimum, even when minima exceed available space.
2. Distributes nonnegative remaining space in proportion to fill weights.
3. Freezes children that reach their maxima and redistributes their unused
   shares among the remaining children.
4. Assigns integer remainder pixels by largest fractional share, breaking
   equal fractions in authored child order.

For example, two unconstrained fill children with weights 1 and 2 divide
10 available pixels into 3 and 7 pixels. Three equal children divide 5
pixels into 2, 2 and 1. If all maxima are reached, remaining space is unused.
Fixed and `Auto` siblings retain their accepted extents during overflow;
there is no implicit shrinking step. Fill minima can also overflow a parent.

A cross-axis fill child independently takes the parent's inner extent,
clamped to its own bounds. Its weight does not compete with siblings there.
A fill root uses the corresponding viewport extent, subject to its bounds.

For `Auto` and `Fill` containers, effective minima include both padding
edges. A maximum smaller than `2 * padding` is rejected. Fixed dimensions
retain the existing core validation for padding that cannot fit their box.
Children can extend beyond container bounds; container backgrounds do not
create implicit clipping. The viewport still clips painting and hit testing.

Container aggregates use checked `u64` arithmetic. Authored bounds are applied
before narrowing an intrinsic aggregate to its final dimension. Thus a large
child sum can be capped by an explicit maximum, or the default `i32::MAX`,
without a premature signed-dimension error. Individual child preferences
remain subject to their own bounds. Arithmetic overflow still fails, and
the existing runtime layout validates final positions and far edges.

The [distribution tests](../../crates/fenestra-ui/src/layout/tests/distribution.rs)
cover weights, maxima, minima, deterministic remainders and padding. The
[measurement ordering tests](../../crates/fenestra-ui/src/layout/tests/measurement.rs)
cover final-width measurement, intrinsic fill contributions, zero width and
bounded aggregate sizes.

## Measurement without a raster

The [owned measurement request](../../crates/fenestra-ui/src/text/measurement.rs)
is `TextMeasureRequest::new(text, style, width, limits)`, where width is
`Option<u32>`. `None` measures max-content with hard line breaks preserved
and soft wrapping disabled. `Some(width)` measures wrapping at that width.
`Some(0)` is a valid request and is distinct from `None`; an indivisible glyph
can still have positive measured width when available width is zero.

`TextEngine::measure` returns `TextMetrics`. Its default returns
`TextError::MeasurementUnavailable`, preserving engines that only implement
`layout` for boxes without intrinsic text dependencies. An intrinsic policy
that requires unsupported measurement fails explicitly and atomically.

Requests validate typography, UTF-8 byte count and the signed coordinate
range of an optional width. Results are revalidated for finite nonnegative
dimensions, consistent missing-glyph counts and the glyph limit.
`TextMetrics::ceil_size` rounds upward using `f64` and checks against
`i32::MAX` before integer conversion; it allows zero dimensions. The raster
pixel limit does not apply to measurement. Shaping can allocate private work
buffers, but the measurement path does not allocate a raster.

The [Parley adapter](../../crates/fenestra-ui-text/src/renderer.rs) shares
shaping, line breaking and glyph preflight between measurement and rendering.
It obtains max-content metrics from actual `break_all_lines(None)` layout,
rather than a separate content-width shortcut. Missing coverage remains a
typed failure. The [adapter tests](../../crates/fenestra-ui-text/tests/measurement.rs)
check width-driven wrapping, mixed-direction text, limits and exact equality
with complete raster-layout metrics at the same width.

Measurement retains WU-0019 line-box behavior: empty text has one line of
height, trailing spaces do not contribute measured width, and a final
newline adds an empty line. The facade separately treats a resolved zero
width as zero intrinsic height before applying height bounds, without asking
for wrapped measurement. A final zero-area box skips raster preparation,
retains its content and typography, and reports zero application text metrics.
These facade rules do not change direct `measure(Some(0))` behavior.

## Retained state, caches and publication

Each [named application node](../../crates/fenestra-ui/src/application.rs)
retains authored `Style`, resolved `Size` and direct child indices. Text state
retains at most one natural measurement, one wrapped measurement keyed by
width, and one laid-out raster. Content or `TextStyle` changes invalidate all
three. Dimension and viewport changes rerun resolution, reusing measurements
whose dependencies agree and reusing a raster when its dimensions agree.
Translation-only changes can reuse text work.

When a wrapped measurement and raster layout describe the same width,
their complete metrics must match exactly. A discrepancy returns
`TextError::InconsistentMeasurement`, including when a cached raster is reused.
No consistency comparison is required against an absent zero-area raster.

[Application mutations](../../crates/fenestra-ui/src/application/mutation.rs)
prepare a candidate node vector before publication:

1. Validate applicable input limits. Aggregate UTF-8 bytes are bounded before
   copying changed text or invoking measurement.
2. Resolve every candidate size, validating engine measurements as they arrive.
3. Bound aggregate resolved text raster area before preparing any text raster.
4. Prepare changed rasters and validate their dimensions, glyph counts and
   measurement consistency.
5. Submit resolved properties and viewport changes to the runtime transaction.
   `commit_with` prepares owned image attachments against the same candidate
   geometry before publishing it.
6. Replace accepted nodes, caches, viewport and augmented paint together.

Construction applies the same input, measurement and aggregate pixel checks
before raster work. Failure preserves the previously accepted content,
policies, dimensions, metrics, hit geometry, pixels and generation. Candidate
caches are discarded; a private engine may retain its own cache side effects.
The [failure tests](../../crates/fenestra-ui/tests/responsive_measurement.rs)
exercise invalid results, layout-only engines, inconsistent pixels/metrics
and retries after rejected candidate measurements.

Every effective application update advances the root's private
`VIEW_REVISION` property at slot 6. This publishes a policy change even if its
current resolved pixels happen to be identical, such as changing `Px(64)` to
`Fill(1)` in a 64-pixel viewport. Identical authored inputs remain no-ops.
The [facade schema](../../crates/fenestra-ui/src/lower/construction.rs) keeps
seven property slots per node, and runtime operation capacity is `7 * N + 1`
for `N` nodes. A resize can therefore publish changes across the whole view
instead of relying on a small fixed transaction capacity.

## Limits and remaining scope

Text pixel limits count logical bitmap area, including offscreen elements.
They are not a hard heap ceiling: staged states, spatial image copies,
retained frames and private shaping/raster buffers can allocate additional
memory. The provisional font admission and platform scope in
[ADR 0001](../decisions/0001-provisional-text-adapter.md) are unchanged.

This increment does not claim CSS flexbox conformance. General alignment,
flex shrinking, container wrapping and grid layout remain open. Text editing,
selection, caret geometry, IME, accessibility semantics and native presentation
qualification retain their separate gates from the text and input plans.
