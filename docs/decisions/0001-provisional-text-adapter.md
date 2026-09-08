# ADR 0001: Provisional owned-font text adapter

Status: provisional, scoped to authored text views and owned editing geometry.
Record version: 2. Checked: 2026-09-08 UTC.

## Decision and boundary

Use Parley 0.11.1 with Swash 0.2.6 as one replaceable implementation behind a
Fenestra-owned text adapter contract. Keep candidate types, font databases,
layout objects and raster caches private to that adapter. Generic runtime,
authoring IR and public text styles retain owned Fenestra types. The adapter
owns font registration, measurement, shaping and glyph coverage; the generic
runtime continues to own identity, publication, scheduling and invalidation.
Cargo features describe supported capabilities rather than backend brands.

This is a provisional dependency admission for a bounded application path. It
does not select a permanent text backend, admit arbitrary downloaded fonts, or
establish full Unicode, editing, platform or accessibility conformance. A later
adapter may replace this one without changing authored text content or styles.

The [WU-0018 comparison](../../probes/text-candidate-screen/README.md) measured
both Parley and cosmic-text against the same twelve fixtures. Both agreed on
paragraph width, height and wrapped-line count. Parley requires an explicit
glyph raster bridge, while cosmic-text provides convenient editor helpers.
Read-only authored views need the former capabilities. Existing cosmic-text
editing experiments remain useful independent evidence.

## Dependency admission inventory

Exact direct versions belong in `workspace.dependencies`, with defaults
disabled. The lockfile retains exact transitive resolutions. This inventory
records dependency licenses; it does not choose the Fenestra project license.

| Package | Version / enabled features | Purpose | Declared MSRV | License |
| --- | --- | --- | --- | --- |
| Parley | 0.11.1 / `std`, `complex-scripts` | Layout, shaping integration and line breaking | 1.88 | Apache-2.0 OR MIT |
| Fontique | 0.11.1 / `std` through Parley | Explicit owned-font metadata and matching | 1.88 | Apache-2.0 OR MIT |
| HarfRust | 0.12.0 / `std` through Parley | OpenType shaping | 1.85 | MIT |
| Skrifa | 0.44.0 / `std` through Parley | Font metrics and metadata | 1.85 | MIT OR Apache-2.0 |
| read-fonts | 0.41.0 / through Fontique and Skrifa | Font parsing | 1.85 | MIT OR Apache-2.0 |
| Swash | 0.2.6 / `std`, `scale`, `render` | Outline glyph rasterization | Not declared | Apache-2.0 OR MIT |
| Zeno | 0.3.3 / through Swash; `std`, `eval` | Bounded outline mask scan conversion | Not declared | Apache-2.0 OR MIT |
| Skrifa / read-fonts | 0.37.0 / 0.35.0 through Swash | Raster font parsing | 1.82 / 1.82 | MIT OR Apache-2.0 |

The MSRVs above are published package metadata, not executed compiler lanes.
The screen runs on the pinned development compiler. This record does not
declare a product-wide MSRV. Swash's second Skrifa/read-fonts chain is a known
duplication cost. The complete 61-package screening closure is captured in
[the audit evidence](../../probes/text-candidate-screen/evidence/text-dependency-admission-v1.json).

Parley and Fontique are maintained together by Linebender. Their 0.11.1 release
was published on 2026-08-16 and updates the font parsing and shaping dependencies.
Swash is maintained in `dfrg/swash`; the admitted 0.2.6 package was published on
2025-10-01. All three pins were unyanked at this check. These are observed
release facts, not future maintenance guarantees. Review releases and advisory
changes before upgrades and before broadening the supported input scope.
[Parley release](https://github.com/linebender/parley/releases/tag/v0.11.1),
[Swash package metadata](https://crates.io/api/v1/crates/swash/0.2.6).

## Security history and candidate consequence

`cargo-audit` 0.22.2 checked the isolated screen lockfile against RustSec commit
`8a1eb4f933fb5821add5b4e98601ebd90b8b3538`, updated 2026-09-07. The complete
249-package screen returned zero known vulnerabilities and one maintenance
warning. A separate audit of its transitive dependency closure rooted at
`fenestra-text-screen-parley` returned zero known vulnerabilities and zero
warnings; `--deny warnings` exited successfully. The snapshot records package
lists, lockfile hashes and the exact results. The subset was produced by
following each selected package's lockfile dependency entries and preserving
those package blocks verbatim in a temporary lockfile; no versions were
re-resolved and no advisory was ignored.

The [product audit](evidence/text-product-admission-v1.json) additionally checks
the actual root lockfile after adding the owned text adapter: 342 packages,
zero known vulnerabilities, zero warnings, and a successful `--deny warnings`
exit. Its Parley feature set enables `complex-scripts`, which was disabled in
the WU-0018 comparison. This permits dictionary-based segmentation through the
already resolved ICU dependencies; it does not supply missing fonts or establish
script conformance. Both audit snapshots retain their separate lockfile hashes.

The warning is [RUSTSEC-2026-0192](https://rustsec.org/advisories/RUSTSEC-2026-0192.html):
`ttf-parser` 0.25.1 is unmaintained with no patched version, reached through
cosmic-text 0.19.0 and fontdb 0.23.0. The referenced upstream issue is a request
to report a security issue privately; the public material does not establish
its technical details. This warning is absent from the Parley closure. It is a
concrete reason to keep the cosmic-text implementation in its isolated probe
instead of promoting it for authored views at this admission.

The retained cosmic-text screen pins 0.19.0 with defaults disabled and `std`,
`swash` enabled (declared Rust 1.89, MIT OR Apache-2.0). Its fontdb 0.23.0
dependency declares Rust 1.60 and MIT. cosmic-text's `std` feature nevertheless
enables fontdb's `memmap` and `fs` features, and `FontSystem::get_font` calls
unsafe `make_shared_face_data`; its owned `Source::Binary` branch clones the
retained allocation. File and system font loading remain excluded. The
shared Swash pin has the same raster and unsafe inventory in either candidate.

The current upstream advisory pages for
[Parley/Fontique](https://github.com/linebender/parley/security/advisories),
[Swash](https://github.com/dfrg/swash/security/advisories),
[cosmic-text](https://github.com/pop-os/cosmic-text/security/advisories) and
[fontdb](https://github.com/RazrFalcon/fontdb/security/advisories) displayed no
published advisories on the check date. The local RustSec snapshot contains no
crate advisories for those packages. Absence of published advisories is not a
security audit or evidence that malformed inputs are harmless.

## Owned-font and resource constraints

The implementation must satisfy these constraints before using this admission:

1. Begin with `CollectionOptions { system_fonts: false, shared: false }` and
   register immutable owned bytes with `Blob` and `register_fonts`. Never call
   filesystem scanning or accept a `SourceKind::Path`. Specify the default
   family from a successfully registered font. Fallback is restricted to the
   explicitly registered application font set; uncovered text requires a typed
   failure rather than silent host-font use.
2. Set finite product limits before candidate calls: font bytes per face,
   total font bytes, font count, UTF-8 text bytes, font size, line height,
   layout width and raster pixels. Their actual values belong to the owned
   adapter contract and tests. Upstream APIs provide no universal safe font
   byte limit; a cap is an application budget, not a parser guarantee.
3. Restrict the first admission to single-face TTF/OTF data and reject TTC
   headers before entering the candidate. Require one registered face at
   index zero and a usable Swash font reference. Empty registration is failure,
   not successful registration of a font with zero faces. Validate in temporary
   state so a failed registration cannot alter the active font set or caches.
4. Fontique's memory scanner uses `read_fonts::FileRef` and the collection's
   available offsets slice; it does not preallocate from an unchecked raw TTC
   count. This differs from fontdb's `TinyVec::with_capacity(n)` followed by a
   loop over `ttf_parser::fonts_in_collection`. This source inspection is not
   all-target malformed-TTC validation; TTC remains outside the first contract.
5. Preserve full content measurements independently of clipped raster output.
   Bound input before shaping and check output sizes before allocation.
   Clipping a raster is not a shaping-work limit. The cosmic viewport test
   likewise demonstrates that a single paragraph may be fully shaped behind a
   short visible viewport. Neither screen establishes worst-case timing or
   allocation bounds for arbitrary font programs.

The product implementation in [fenestra-ui-text](../../crates/fenestra-ui-text/src/lib.rs)
implements these conditions with 32 fonts, 8 MiB per font and 32 MiB total,
checked before copying bytes. It validates sfnt directory ranges and the
OpenType `unitsPerEm` range of 16 through 16,384 before candidate registration.
Private family aliases preserve caller order when source fonts have equal family
names. Missing coverage returns `TextError::MissingGlyphs` before rasterization.

Complete glyph counts and measurements are checked before glyph raster work.
Swash then decomposes each monochrome outline; Zeno inspects that exact outline
and its subpixel offset before mask allocation. The mask must fit both the
request's pixel budget and 4,194,304 pixels. Before scan conversion, the outline
must contain at most 65,536 finite points with coordinate magnitude at most
1,048,576 pixels. These checks avoid trusting a font's declared bounding box.
The outline, mask buffer, scan-conversion scratch and scaler cache are private
and reused. Font parsing and outline decomposition still perform candidate
allocations before the output checks, so these bounds are not a hard heap or
time guarantee for hostile fonts.

The new [Parley registration tests](../../probes/text-candidate-screen/parley/tests/font_registration.rs)
exercise empty and invalid bytes, 65 short prefixes of the versioned font,
two bounded malformed TTC headers, owned source retention, single-face metadata,
preservation after failed registration, explicit missing glyph source ranges,
and measurement independent of raster clipping. The
[cosmic registration tests](../../probes/text-candidate-screen/cosmic/tests/font_registration.rs)
retain the corresponding candidate facts and its viewport limitation. These
are focused regressions, not fuzzing or adversarial-font hardening. They add
no font asset; the sole valid fixture remains the licensed DejaVu Sans 2.37
font with its versioned provenance.

The [product adapter tests](../../crates/fenestra-ui-text/tests/renderer.rs) add
ordered fallback using an in-memory reduced-coverage derivative of that same
fixture, complete clipped measurements, premultiplied color and transparency,
repeated contexts, malformed offsets and unsupported raster formats. A zero-em
font was observed passing registration before the validation fix; a separately
expanded glyph bypassed its mask budget before mask preflight was implemented.
Both cases now return typed failures. No derivative font file is distributed.

## Unsafe, native and distribution surface

Source inspection found no unsafe code occurrence in Parley's layout source.
Fontique allows unsafe code for platform services and compiles memory mapping
under `std`. Its `font.rs`, `source_cache.rs` and `scan.rs` contain file mapping
paths. Owned `SourceKind::Memory` registration and loading avoid those paths.
Swash contains unchecked reads, unchecked Unicode conversions and other unsafe
internals; its outline parser and scaler remain part of the trusted dependency
surface. This is an inventory, not a complete unsafe audit. No handwritten
unsafe is needed in the Fenestra adapter.

Parley's `system` and `accesskit` features remain disabled.
There is no Fontconfig, DirectWrite or CoreText service requirement for this
configuration. `fontique/std` still enables memmap2 (0.9.11 in the lockfiles),
which declares Rust 1.65 and MIT OR Apache-2.0, and its normal platform support
dependencies.
Do not describe the dependency graph as having no native capability. Native
font discovery, accessibility and system-font distribution require separate
capability decisions. Enabling `complex-scripts` does not enlarge font coverage.

Shipping applications must preserve the relevant dependency license and notice
obligations and the provenance and redistribution terms of every bundled font.
The repository's fixture license does not grant rights to unrelated fonts.
No font or native text service is downloaded implicitly by this admission.

## Replacement cost and remaining gates

Replacement requires one private registration/layout/raster implementation,
cache invalidation changes and rerunning the owned corpus and application
tests. Retain fonts as owned bytes and styles, measurements, coverage facts and
errors as Fenestra types to avoid migrating user documents or generic runtime
state. Glyph identifiers, cache keys and candidate source ranges are internal
facts with no serialization compatibility promise. Parley's trailing-space and
ligature-range differences are recorded in the WU-0018 screen and must remain
behind this projection boundary.

Further admission needs fuzzing or equivalent malformed-font evidence,
worst-case resource measurements, an executed MSRV lane, broader script/font
fixtures with provenance, broader multiple-font/script fallback evidence, native platform and
scale evidence, and review of the full distributable dependency graph.
The original read-only evidence does not admit editing behavior. WU-0024 adds
an independently validated, bounded
[editing geometry and viewport contract](../design/text-editing-geometry.md)
over the same explicit font inputs and pinned dependencies. Its source-byte
mapping, extended grapheme queries, selection fragments and scrolled raster
tests extend this provisional adapter boundary. The adapter retains Parley's
component interpolation for ligature carets and does not inspect GDEF tables.
Its [licensed local Parley delta](../../third-party/parley/FENESTRA-PATCH.md)
corrects RTL line-break group traversal and hard-break caret geometry. The
version and transitive resolutions remain fixed. Product lockfiles switch
Parley's source to this retained path; the historical screen keeps its registry
source. The earlier audit snapshots remain evidence of their recorded lockfiles
and are not presented as fresh audits of this local source delta.
This extension does not admit arbitrary fonts, integrated editable controls,
rich text, text accessibility or qualified platform IME composition.
Refresh the advisory check against the actual product lockfile before release.

## Reproduce the focused tests

Run from the repository root:

```sh
cargo test --locked --manifest-path probes/text-candidate-screen/Cargo.toml -p fenestra-text-screen-parley --all-targets
cargo test --locked --manifest-path probes/text-candidate-screen/Cargo.toml -p fenestra-text-screen-cosmic --test font_registration
cargo clippy --locked --manifest-path probes/text-candidate-screen/Cargo.toml -p fenestra-text-screen-parley --all-targets -- -D warnings
cargo audit --file probes/text-candidate-screen/Cargo.lock --json
```

The last command intentionally checks both candidates and reports the retained
cosmic dependency warning. It must not be described as a warning-free Parley
product audit. The subset selection and package list are in the versioned audit
snapshot; a product lockfile audit remains required for product distribution.

## Pinned implementation references

- [Parley package metadata](https://docs.rs/crate/parley/0.11.1/source/Cargo.toml)
- [Fontique feature manifest](https://github.com/linebender/parley/blob/v0.11.1/fontique/Cargo.toml)
- [Fontique memory registration](https://github.com/linebender/parley/blob/v0.11.1/fontique/src/collection/mod.rs)
- [Fontique memory scanner](https://github.com/linebender/parley/blob/v0.11.1/fontique/src/scan.rs)
- [Swash 0.2.6 manifest](https://github.com/dfrg/swash/blob/v0.2.6/Cargo.toml)
- [cosmic-text 0.19.0 manifest](https://docs.rs/crate/cosmic-text/0.19.0/source/Cargo.toml.orig)
- [fontdb 0.23.0 manifest](https://docs.rs/crate/fontdb/0.23.0/source/Cargo.toml)
- [OpenType font header](https://learn.microsoft.com/en-us/typography/opentype/spec/head)
