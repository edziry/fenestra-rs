# Authored text views

Work unit: WU-0019
Scope: fixed-box, read-only text in the public application and format-3 authoring APIs.

## Boundary

This increment extends the [typed application API](typed-application-api.md)
with owned text content, typography, measurements and paint. The
[text and input foundation](text-input-foundation.md) remains the separate
editing and native-event contract. Applications can connect a `TextBuffer` to
`Application::set_text`; a text element itself is not an editor or control.

The facade owns the public contract. An application supplies a `TextEngine`
and its fonts. The optional consumer crate `fenestra-ui-text` supplies one
provisional implementation using Parley and Swash under
[ADR 0001](../decisions/0001-provisional-text-adapter.md). The dependency points
from that adapter to `fenestra-ui`; the facade, generic runtime and frozen IR
do not depend on the adapter or expose candidate font, glyph or layout types.

## Authoring and application API

Both a standalone `.fen` file and the body of `ui!` accept this format-3 view:

```text
format 3;
view hello {
  column panel {
    width: 280;
    height: 120;
    padding: 12;
    text title {
      content: "Hello\nworld";
      width: 240;
      height: 80;
      font_size: 20;
      line_height: 28;
      color: rgba8(235, 241, 246, 255);
      background: rgba8(24, 32, 48, 255);
      input: accept;
    }
  }
}
```

`text` is a named leaf with required string-literal `content`. It accepts the
common width, height, background and input properties plus font size, absolute
line height and foreground color. It cannot contain children, padding or gap.
Typography is rejected on non-text elements. String escapes, Unicode content
and comment markers inside strings are preserved by both frontends; duplicate
or mistyped properties produce source diagnostics. The
[authoring tests](../../crates/fenestra-ui-authoring/tests/view_text.rs) compare
the emitted public constructors and diagnostics from both frontends.

The Rust equivalent uses `Element::text(name, content)`, `.style(Style)` and
`.text_style(TextStyle)`. Names follow the existing globally unique ASCII
identifier rule. Constructors own content; fonts are supplied at application
construction, independently of the authored view:

```rust
use fenestra_ui::{Application, Size, TextStyle, View};
use fenestra_ui_text::TextRenderer;

fn open(view: View, font_bytes: &[u8]) -> Result<Application, Box<dyn std::error::Error>> {
    let engine = TextRenderer::new([font_bytes])?;
    let mut app = Application::with_text_engine(view, Size::new(280, 120), engine)?;
    app.set_text("title", "Updated text")?;
    app.set_text_style("title", TextStyle::new().font_size(20).line_height(28))?;
    Ok(app)
}
```

`with_limits_and_text_engine` additionally accepts application `Limits`.
`Application::new` and `with_limits` still support views without text; a view
containing text requires an engine, including when a text box has zero area.
`text`, `text_style` and `text_metrics` return committed state by element name.
`set_text`, `set_text_style`, `set_style`, `set_size` and `resize` publish their
effects atomically. Unchanged values preserve the generation and skip shaping.

## Owned text contract and limits

The [text contract](../../crates/fenestra-ui/src/text.rs) consists of:

| Type | Responsibility |
| --- | --- |
| `TextStyle` | Integer font size and absolute line height, plus straight RGBA8 foreground color. Defaults: 16 pixels, 24 pixels and opaque white. Valid sizes: 1..=512 and 1..=2048 respectively. |
| `TextEngine` | Application-owned mutable adapter implementing `layout(TextRequest) -> Result<TextLayout, TextError>`. Its caches and candidate types stay private. |
| `TextRequest` | Borrowed UTF-8 content, copied style, nonempty fixed raster `Size` and inclusive `TextLimits`; construction validates style, byte count and raster area. |
| `TextLayout` | Owned `Raster` and measurements. Validation checks premultiplied channels, finite nonnegative measurements, consistent missing-glyph count, exact request dimensions and glyph budget. |
| `TextMetrics` | Maximum shaped line width excluding trailing whitespace, full laid-out height, line count, glyph count and missing-glyph count. Measurements include clipped lines and glyphs. |
| `TextLimits` | Aggregate application UTF-8 bytes and logical raster area, plus the glyph bound for each shaping request. |

`Limits::text_limits(TextLimits::new(bytes, pixels, glyphs))` sets these bounds;
`Limits::text()` reads them. Defaults are 262,144 UTF-8 bytes, 4,194,304 logical
text pixels and 32,768 glyphs per request. The application sums all text bytes
and `width * height` areas before invoking an engine, including offscreen
elements. Standalone requests also check their individual bytes and pixels.
Glyph counts include the complete shaped result, not just visible glyphs.
The application revalidates an engine's result before it can be published.

These pixel limits count one logical bitmap per text element, not the total
heap. The application retains pixels in `Arc<TextLayout>` and copies them
into an owned spatial image. Staged commits, retained frames and private
engine work buffers can retain additional allocations. Neither these defaults
nor the font bounds establish a general allocation or execution-time ceiling.

Width and height are fixed nonnegative integer pixel dimensions inherited
from `Style`; they do not derive from measurements. Shaping receives the new
dimensions before runtime publication. A zero width or height retains content
and typography but skips shaping and image attachment, returns zero metrics,
and contributes zero raster area. Its UTF-8 bytes still count. Growing the box
prepares its current content and typography before publication.

Text raster output is clipped to the element's bitmap dimensions and then to
the application viewport. Full metrics may exceed that bitmap. Existing
containers allow children to extend beyond their dimensions; their backgrounds
are not implicit ancestor clips. `bounds` continues to report unclipped
committed element geometry, and input remains a hit over the element box,
independent of glyph coverage or transparent pixels. There is no intrinsic
measurement callback or content-based parent sizing in this increment.

## Coherent preparation and publication

[Application text preparation](../../crates/fenestra-ui/src/application/text.rs)
uses the same runtime layout and spatial raster as the other elements:

1. Stage owned node state and validate the applicable aggregate limits. Shape
   changed content, typography or box dimensions into an owned `TextLayout`.
   Check the output against its request before starting publication.
2. Write ordinary style changes to the runtime transaction. Content and
   typography changes also update the private `TEXT_REVISION` scalar property
   declared in the [facade schema](../../crates/fenestra-ui/src/lower/construction.rs).
   Its paint invalidation makes a text-only update advance the existing runtime
   generation. Strings, fonts and mutable images are not added to frozen IR.
3. `UiRuntime::commit_with` builds and validates the complete candidate snapshot,
   then invokes a fallible preparation closure before publishing it. The
   application attaches text images against that exact candidate's owners and
   accepted geometry. An attachment failure rejects the entire publication.
4. After successful runtime publication, replace the application's staged
   content, styles, metrics and augmented paint snapshot together. Already
   returned rasters and retained spatial snapshots own their data and remain
   unchanged by subsequent updates.

The [runtime seam](../../crates/fenestra-ui-runtime/src/runtime/transaction/commit.rs)
returns a receipt and caller-owned prepared value. Runtime validation and stale
transaction checks precede preparation; effective transactions also check
retention limits and generation exhaustion before the callback. A preparation
error or unwinding panic preserves the published runtime snapshot; a no-op
callback runs once against the existing generation.
The ordinary `commit` retains its behavior through an infallible unit callback.
This rollback guarantee does not restore engine caches or other caller-owned
side effects. An engine may mutate its caches on failure; application-visible
text, metrics, geometry, pixels and generation remain at the previous commit.

Changes that only translate a text element reuse its shaped layout. Background,
input or viewport changes also avoid reshaping unchanged text. Each effective
application commit nevertheless copies all text images into a new augmented
snapshot and re-resolves spatial geometry, including these changes. The
logical area bound limits the copied image size; general copy and resolution
costs have not been measured by the raster timing below.

## Snapshot-owned image paints

The generic [image attachment seam](../../crates/fenestra-ui-spatial/src/input_validation/prepared/snapshot/attachments.rs)
adds owned image paints without depending on text.
`SpatialImagePaintAttachmentV2::new(owner, image, destination, clip)` describes
an addition; `SpatialResolvedSnapshotV2::with_image_paints(additions, limits)`
returns a new owned snapshot. Supplied image keys are reassigned densely. Each attachment
paints its full premultiplied source at opacity 255 in owner-local coordinates;
an explicit clip must belong to that owner or an ancestor.

Existing paints precede additions for the same owner; same-owner additions
preserve supplied order, and other owners retain their accepted order. Facade
text therefore paints after its background and before later overlapping
children or siblings. Transparent texels use the existing source-over kernel.
The application attaches at local origin with destination dimensions exactly
matching the text raster. Its bounded bitmap supplies the element-local clip.

The seam validates complete resource counts, image dimensions, aggregate
pixels, strides and premultiplied bytes using existing spatial preparation.
It re-resolves through the reference layout and requires exact equality of
the accepted viewport, geometry, clips, hits, semantics and effective clip
bounds. A divergent injected layout returns `GeometryChanged`; malformed
input returns `Resolve`. Success and failure both leave the original snapshot
intact. Paint and hit geometry cannot silently diverge during attachment.

## Provisional owned-font adapter

The [optional adapter](../../crates/fenestra-ui-text/src/renderer.rs) exposes
`fenestra-ui-text::TextRenderer::new`, which copies an explicit set of 1..=32
single-face outline TTF/OTF fonts. Each font is bounded to 8 MiB and the set to
32 MiB before copying or parsing. Construction uses temporary registration
state; invalid fonts, collections, unsupported color/bitmap formats or
missing outline tables reject the constructor. The first font is preferred and
remaining fonts are fallback candidates in caller order, including when their
upstream family names coincide. There is no host font discovery or implicit
download. Uncovered glyphs produce `TextError::MissingGlyphs`.

The adapter wraps and aligns text within the requested width, records full
metrics before clipping, and returns owned premultiplied RGBA8 pixels. Its
private raster bridge uses Swash outlines and an alpha mask; it checks mask
dimensions before allocating and rendering that mask. Outline point and
coordinate checks provide additional admission bounds. These checks do not
turn font parsing, outline scaling or candidate caches into a hard heap cap.
The current input scope and dependency rationale remain governed by ADR 0001.

## Exact reference-raster optimization

The [pixel-constant predicate](../../crates/fenestra-ui-spatial/src/input_validation/prepared/snapshot/raster/pixel_constant.rs)
selects an exact fast path for the whole accepted frame only when:

- Every paint uses identity linear transformation plus integer translation,
  with integer accepted output bounds.
- Coverage paints are integer rectangles with fill coverage and solid brushes.
  Image destinations have integer origins and exactly match source extents
  at 1:1 scale; cropped source rectangles are allowed.
- Every clip is an integer rectangle with identity plus integer translation,
  integer accepted primitive bounds and integer effective clip bounds.

All sixteen registered subpixel samples then have identical coverage, image
texels and ordered source-over results within each output pixel. Evaluating
the existing sample kernel once at the pixel center equals averaging sixteen
identical byte colors. Text bitmap antialiasing and premultiplied alpha remain
intact. Fractional coordinates, scale, rotation, skew, gradients, strokes,
paths, polygons, circles or any failed predicate retain the original complete
sixteen-sample algorithm for the frame. Raster budget checks are unchanged.

The [comparison tests](../../crates/fenestra-ui-spatial/src/input_validation/tests/prepared_spatial_contract/raster_pixel_constant.rs)
explicitly compare bytes with the original sampling path, including negative
coordinates, ancestor clip chains, alpha, cropped images, overlap and domain
edges. [Fallback tests](../../crates/fenestra-ui-spatial/src/input_validation/tests/prepared_spatial_contract/raster_pixel_constant_fallback.rs)
cover excluded geometry and accepted fractional projections.
`full_frame_comparison_records_focused_raster_timing` compares a 320x180 frame
with three solid rectangles and one 160x40 image. One local release observation
was 80.15 ms for full sampling and 4.93 ms for the constant-sample path, with
identical bytes. This is an observational fixture, not an SLA, general
benchmark, shaping measurement or native presentation claim. Reproduce it with:

```sh
cargo test -p fenestra-ui-spatial --lib --release full_frame_comparison_records_focused_raster_timing -- --nocapture
```

## Focused evidence and remaining scope

[Facade tests](../../crates/fenestra-ui/tests/text_views.rs) cover transactional
content, typography and geometry, no-op generations, output rejection,
aggregate budgets, zero-area recovery, painter order and viewport clipping.
[Runtime preparation tests](../../crates/fenestra-ui-runtime/src/runtime/tests/prepublication.rs)
cover rejection, unwind, no-op, stale bases, retention and successful generation.
[Image attachment tests](../../crates/fenestra-ui-spatial/tests/image_attachment.rs)
cover owner order, transparency, ancestor clipping, offscreen images, bounds
and rejection of changed geometry.
[Adapter tests](../../crates/fenestra-ui-text/tests/renderer.rs) cover font
admission, ordered fallback, missing coverage, full metrics, glyph and mask
budgets, and premultiplied output. The
[consumer example](../../examples/text-app/src/lib.rs) connects authored views,
an explicit font and the existing `TextBuffer` through the public facade.

This increment does not qualify intrinsic sizing, rich text, focusable text
controls, caret or selection geometry, visual bidi navigation, accessibility,
IME candidate placement or platform composition. The existing native input
bridge remains separate. Broader font/script conformance, adversarial font
hardening and worst-case resource measurements remain outside this bounded
read-only text path.
