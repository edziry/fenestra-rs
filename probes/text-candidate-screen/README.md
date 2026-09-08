# Replaceable text candidate screen v1

This isolated workspace screens Parley 0.11.1 and cosmic-text 0.19.0 on the
same versioned font and corpus. It admits neither into the public facade and
makes no permanent library selection. The separate candidate packages share
only owned probe reports and correctness assertions. The enclosing workspace
manifest and lockfile do not depend on these candidates.

Current stable versions were checked through the crates.io API and pinned docs
on 2026-09-08 UTC. The captured environment was Linux x86_64 with rustc 1.97.1.
Search-engine snippets still showed Parley 0.10.0; the live registry and docs
reported 0.11.1. The isolated `Cargo.lock` fixes transitive resolutions.

## Fixed inputs and checks

- Font: `assets/fonts/dejavu-sans-2.37/DejaVuSans.ttf` from the upstream 2.37
  release, with the original license and SHA-256 provenance alongside it.
- Corpus: the twelve original fixtures under `corpus/`, including Latin,
  proportional widths, word wrapping, combining marks, ligatures, mixed
  Latin/Hebrew/Arabic bidi, Greek, hard line breaks, and missing CJK/emoji.
- Font size 20 px, absolute line height 28 px, scale 1, width 360 px except the
  130 px wrapping fixture and 80 px overlong-token fixture. Raster height is
  240 px. Layout height is unbounded; only raster writes are clipped. Host fonts
  are never loaded.
- Unicode extended-grapheme reference: unicode-segmentation 1.13.3. The shared
  tests check combining-cluster ranges, UTF-8 boundaries, RTL visual reordering,
  finite geometry, hard-line offsets, proportional widths, narrower-width
  wrapping, deterministic repeat output, clipped output, and expected missing
  glyphs. They do not equate glyph count with grapheme count.
- TDD: corpus imports and both candidate adapter imports were observed failing
  before implementation. The combining-range assertion then exposed a real
  Parley projection defect; the adapter was fixed without weakening the test.

## Measured result

The two candidates agree on paragraph width and height to the recorded 0.001 px
precision for all twelve fixtures. They agree on wrapped-line count. Full line
ranges, glyph IDs, source byte ranges, positions, advances, direction, and ink
coverage counts are in `evidence/parley-v1.txt` and `evidence/cosmic-v1.txt`.

| Fixture | Width, both (px) | Height, both (px) | Lines | Glyphs, Parley / cosmic |
| --- | ---: | ---: | ---: | ---: |
| Latin | 132.266 | 28 | 1 | 13 / 13 |
| Eight narrow letters | 44.453 | 28 | 1 | 8 / 8 |
| Eight wide letters | 158.203 | 28 | 1 | 8 / 8 |
| Wrapped sentence | 116.973 | 140 | 5 | 48 / 44 |
| Overlong token | 76.787 | 112 | 4 | 26 / 26 |
| Combining marks | 49.951 | 28 | 1 | 5 / 5 |
| Ligatures | 175.381 | 28 | 1 | 15 / 15 |
| Mixed bidi | 249.668 | 28 | 1 | 26 / 26 |
| Arabic | 109.111 | 28 | 1 | 11 / 11 |
| Greek | 166.816 | 28 | 1 | 14 / 14 |
| Unsupported CJK/emoji | 78.369 | 28 | 1 | 10 / 10 |
| Two hard lines | 116.055 | 56 | 2 | 21 / 21 |

Both candidates report six missing glyphs for the unsupported fixture and zero
for the other eleven. Missing glyphs are evidence of this font's coverage limit;
this screen does not establish working emoji, CJK, or broad fallback support.

The adapter differences matter for a future product contract:

1. Matching emergency wrapping requires Parley `OverflowWrap::BreakWord` and
   cosmic `Wrap::WordOrGlyph`; Parley defaults to `OverflowWrap::Normal`.
   The overlong-token test failed before the explicit Parley setting.
   Parley retains trailing-space glyphs and includes those spaces in line source
   ranges and `LineMetrics::advance`. cosmic-text omits the four wrap-ending
   spaces from its visible glyph runs. Both paragraph measurements exclude
   those trailing advances. Hard-line source ranges likewise include the
   newline in Parley and omit it from cosmic's visible glyph envelope.
2. Parley exposes ligature component ranges separately. A composed glyph for
   the first combining fixture initially had range `0..1`, with a zero-glyph
   continuation at `1..3`. `parley/src/lib.rs::cluster_source` extends the range
   through the documented ligature continuations, yielding the same `0..3`
   shaping span that cosmic-text exposes directly in `LayoutGlyph.start/end`.
   Source ranges on glyphs still cannot replace grapheme-aware editor state.
3. Both adapters produce owned white-alpha RGBA pixels with the same pinned
   Swash 0.2.6 outline rasterizer. Parley requires the explicit glyph-run/font
   bridge; cosmic-text supplies `Buffer::draw` and `SwashCache`. Both raster
   paths yield nonzero ink on every fixture. Ink counts differ slightly because
   the cosmic path quantizes fractional X through its cache bins and Parley
   uses the full fractional X. This is raster feasibility, not pixel identity.
4. Parley with `complex-scripts` disabled emits the captured ICU diagnostic for
   the unsupported CJK fixture. Enabling dictionary-based complex-script
   segmentation and providing matching fonts need a separate comparison.

The optional editor integration adds ten focused regressions in
`cosmic/tests/editor.rs`. They exposed LFCR paragraph-offset underflow, incorrect
partial selection/hit geometry within an RTL lam-alef ligature, and ambiguous
caret placement at soft-wrap boundaries. The private adapter corrects those
cases and uses a documented downstream bias at soft wraps. The owned editor
model still has no visual affinity; newline-only selections have no painted
highlight. These fixes do not establish complete Unicode or platform editing
conformance and should remain replaceable with the candidate.

cosmic-text is convenient for the optional experimental `text-pad` binary
because it already supplies raster, cursor, hit, and selection helpers. This
is a reversible probe choice. The measurements do not establish that either
candidate is faster, more correct overall, or the final Fenestra text backend.

## Replacement boundary and pinned APIs

Both libraries expose the same probe entry point:

```rust
pub fn render(
    text: &str, width: u32, height: u32, font_size: f32, line_height: f32,
) -> fenestra_text_screen_common::Report;
```

`Report`, `Line`, and `Glyph` own only pixel dimensions, floats, IDs, source
ranges, flags, and bytes. Their definitions live in `common/src/lib.rs`; they
are disposable screening contracts. Each call builds a fresh font/layout/raster
context. A product adapter would retain private caches and introduce explicit
limits, font identities, fallback policy, richer styles, and invalidation.
No candidate type should cross into the generic runtime or public UI facade.

The Parley bridge uses `FontContext`, a Fontique `Collection` with
`system_fonts: false`, `register_fonts`, `LayoutContext::ranged_builder`,
`StyleProperty`, `Layout::break_all_lines`, `Layout::align`,
`Line::items`, `GlyphRun::positioned_glyphs`, and logical cluster continuations.
Rasterization uses `Swash::FontRef`, `ScaleContext`, and `Render` privately.

The cosmic-text bridge uses an explicit `fontdb::Database`,
`FontSystem::new_with_locale_and_db`, `Buffer::set_size`, `Wrap::WordOrGlyph`,
`Buffer::set_text` with `Shaping::Advanced`, `shape_until_scroll`, `layout_runs`,
and `Buffer::draw`. Its line-relative source ranges are converted to absolute
UTF-8 offsets with each `BufferLine`'s actual line ending length. The optional
editor helper uses candidate cursor/selection APIs behind owned geometry.

## Dependency screen record

| Dependency | Pin / features | Purpose | Declared MSRV / license |
| --- | --- | --- | --- |
| Parley | 0.11.1; defaults off, `std` | Private rich layout candidate | 1.88; Apache-2.0 OR MIT |
| cosmic-text | 0.19.0; defaults off, `std`, `swash` | Private layout/raster candidate | 1.89; MIT OR Apache-2.0 |
| Swash | 0.2.6; defaults off, `std`, `scale`, `render` | Fixed common raster implementation | Not declared in package manifest; Apache-2.0 OR MIT |
| unicode-segmentation | 1.13.3; defaults | Grapheme reference for the corpus | 1.85.0; MIT OR Apache-2.0 |

The common raster version is deliberately fixed across both paths; it is not a
claim that Swash 0.2.6 is the latest release. Parley resolves Fontique 0.11.1,
HarfRust 0.12.0, and Skrifa 0.44.0. cosmic-text resolves fontdb 0.23.0,
HarfRust 0.5.2, and Skrifa 0.40.0. Swash also resolves Skrifa 0.37.0.
The direct package MSRVs are metadata, not separately executed MSRV lanes.

Downloaded source was inspected. No handwritten unsafe occurs in these probe
adapters. Parley's own layout source contains no unsafe occurrence, but its
Fontique dependency supports memory-mapped files and native platform backends.
cosmic-text's `FontSystem::get_font` contains an unsafe call to
`fontdb::Database::make_shared_face_data`; the probe supplies owned byte data,
not a mapped external font file. Swash contains unchecked parsing and other
unsafe internals. This is a surface inventory, not a complete safety audit.

The candidates are maintained in the Linebender and System76/COSMIC upstream
repositories respectively. No native font service is needed by these explicit
font configurations: Parley's `system` and cosmic's `fontconfig` defaults are
disabled. Distribution must include the fixture license. Font loading, malformed
font robustness, security advisory history, resource budgets, broader script
coverage, full Unicode conformance, cross-platform raster results, accessibility,
IME placement, rich text, and production cache behavior remain unestablished.
A permanent admission requires those records and the ADR required by
`docs/initial-implementation-plan.md`.

## Reproduce and validate

Run from the repository root:

```sh
cargo fmt --manifest-path probes/text-candidate-screen/Cargo.toml --all -- --check
cargo test --locked --manifest-path probes/text-candidate-screen/Cargo.toml --workspace --all-targets --all-features
cargo clippy --locked --manifest-path probes/text-candidate-screen/Cargo.toml --workspace --all-targets --all-features -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --locked --manifest-path probes/text-candidate-screen/Cargo.toml --workspace --all-features --no-deps
cargo run --locked --quiet --manifest-path probes/text-candidate-screen/Cargo.toml -p fenestra-text-screen-parley
cargo run --locked --quiet --manifest-path probes/text-candidate-screen/Cargo.toml -p fenestra-text-screen-cosmic --bin fenestra-text-screen-cosmic
```

### Native text pad

```sh
cargo run --manifest-path probes/text-candidate-screen/Cargo.toml -p fenestra-text-screen-cosmic --features native --bin text-pad --release --locked -- --native
```

The first optimized build takes longer; the resulting binary avoids the much
slower debug reference raster path. The window uses Fenestra's public named
panels, committed bounds, owned text buffer and native events. Text remains
inside this disposable adapter rather than the public authoring grammar.

Click to focus and place the caret. Shift with Left/Right extends a logical
grapheme selection; Ctrl+A selects all. Home/End moves through the whole
document. Backspace, Delete, Enter and repeated text presses edit the buffer.
Native preedit is shown separately until commit, and focus loss cancels it.
There is no scrolling, clipboard, undo or visual bidi arrow navigation yet.
At ambiguous soft wraps, the caret chooses the following row. Font coverage
is limited to the bundled fixture.

Omit `--native` for a deterministic headless edit sequence. Replace it with
`--native-smoke` to present once and exit. Add `--ppm PATH` only when explicitly
exporting the final raster, which contains the current text. Normal stdout
contains counts and a raster checksum, not the buffer contents.

The [Wayland raster](evidence/text-pad-wayland.png) was exported after one
successful native presentation. It shows the frame content, not a screenshot
of the desktop window. The [work-unit verification](../../docs/verification/WU-0018-text-input-foundation.md)
records the native and cross-compilation limits.

The [text and input design](../../docs/design/text-input-foundation.md) describes
the owned boundaries. The fixture evidence above is headless;
it is not a native-window, IME, platform acceptance, or performance result.

## Primary references

- [Parley 0.11.1 documentation](https://docs.rs/parley/0.11.1/parley/)
- [Parley 0.11.1 package metadata](https://docs.rs/crate/parley/0.11.1/source/Cargo.toml)
- [Parley cluster source](https://docs.rs/crate/parley/0.11.1/source/src/layout/cluster.rs)
- [Parley glyph placement](https://docs.rs/crate/parley/0.11.1/source/src/layout/line.rs)
- [cosmic-text 0.19.0 documentation](https://docs.rs/cosmic-text/0.19.0/cosmic_text/)
- [cosmic-text 0.19.0 package metadata](https://docs.rs/crate/cosmic-text/0.19.0/source/Cargo.toml)
- [cosmic-text buffer source](https://docs.rs/crate/cosmic-text/0.19.0/source/src/buffer.rs)
- [cosmic-text glyph source](https://docs.rs/crate/cosmic-text/0.19.0/source/src/layout.rs)
- [cosmic-text Swash bridge](https://docs.rs/crate/cosmic-text/0.19.0/source/src/swash.rs)
- [DejaVu 2.37 release](https://github.com/dejavu-fonts/dejavu-fonts/releases/tag/version_2_37)
