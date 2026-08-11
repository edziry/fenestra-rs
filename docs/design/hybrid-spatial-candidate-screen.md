# WU-0013 hybrid spatial candidate screen

Status: screened; no product selection
Work unit: WU-0013
Snapshot date: 2026-08-10
Research baseline: `fenestra-research` commit
`176c42139776ed9f1ef879cd135bddadaf12a9da`

## Purpose

This screen records replaceable implementation candidates for the
[hybrid spatial composition plan](hybrid-spatial-composition.md). It does not
make a product dependency, renderer, numeric, path, image, or indexing
selection.

Every candidate remains probe-only behind Fenestra-owned spatial, paint, hit,
image, and presenter contracts. Versions and proposed feature profiles are
exact when a probe admits them. Active dependency, unsafe, build, native, and
license facts are recorded per target; lock-only edges remain labeled
separately. Candidate geometry, tessellation, pixels, handles, identities,
surfaces, and errors never become oracle data or public vocabulary.

Primary evidence is the versioned crate documentation, published manifest and
source archive, and official project repository for each exact release. Unsafe
counts below are lexical inventories of published Rust source, including
feature-gated modules. They are not safety audits or complete transitive counts.

## Fenestra-owned boundary

The following responsibilities are not delegated:

- pass-local keys, stable authored order, limits, diagnostics, and artifacts;
- closed affine scalar domain, composition, inverse, and rounding;
- path events, fill rule, stroke description, shapes, and clip chains;
- independent visual, hit, and semantic geometry;
- exact reverse-painter-order reference hit testing;
- logical image descriptors, normalized pixel format, dimensions, stride,
  color and alpha declaration, byte limits, and resource lifetime;
- immutable geometry and presentation snapshots;
- conversion adapters and typed unsupported or reduced results.

A geometry or renderer library may compute a candidate result. It cannot define
the serialized contract, validation order, semantic identity, or correctness
oracle.

## Geometry and numeric candidates

### Euclid 0.22.14

[Euclid 0.22.14](https://crates.io/crates/euclid/0.22.14) is a low-cost probe for
typed local, island, scene, and device spaces. It supports `no_std`, declares
Rust 1.63 and MIT OR Apache-2.0, and depends normally on `num-traits`. Its 44
published `unsafe impl` occurrences are under optional `bytemuck`; the proposed
profile leaves that feature disabled. It has no native or build-script path.

Euclid types remain adapter-private. Fenestra space markers and scalar policy
remain authoritative so the dependency is replaceable.

### Kurbo 0.13.1

[Kurbo 0.13.1](https://docs.rs/kurbo/0.13.1/kurbo/) is the primary affine,
curve, bounds, winding, and containment comparison. It declares Rust 1.85,
MIT OR Apache-2.0, `std` or `libm` plus `alloc`, direct dependencies on
`arrayvec`, `polycool`, and `smallvec`, no direct unsafe code, and no native or
build script.

Kurbo uses `f64` and tolerance-driven algorithms. Its floats, paths, tolerance,
and bounds cannot define the Fenestra scalar or wire policy or exact oracle.

### Fixed 1.30.0

[Fixed 1.30.0](https://crates.io/crates/fixed/1.30.0) is a comparison for a
candidate-neutral checked scalar reference, not a selected scalar. It is the
latest stable line; the global latest line is a 2.0 prerelease. Version 1.30.0
declares Rust 1.85, MIT OR Apache-2.0, supports `no_std`, depends on `typenum`,
and contains a build script and unsafe source.

The probe compares exact endpoints, rounding, multiplication, composition,
inverse, and serialization. Admitting the crate requires a measured advantage
over the small checked implementation rather than API convenience.

## Path and hit candidates

### Lyon Tessellation 1.0.20

[Lyon Tessellation 1.0.20](https://docs.rs/lyon_tessellation/1.0.20/lyon_tessellation/)
is the primary fill and stroke tessellation candidate. It declares MIT OR
Apache-2.0 and `no_std`, has no declared MSRV, and uses
`lyon_path 1.0.19`, `num-traits`, and `float_next_after`. The published source
contains one unsafe block and no native or build script.

Lyon tolerance, `f32` coordinates, vertex order, and triangle output are
adapter results, not serialized semantics or oracle records.

### Rstar 0.13.0

[Rstar 0.13.0](https://crates.io/crates/rstar/0.13.0) is a broadphase-only
candidate. It declares Rust 1.85, MIT OR Apache-2.0, `no_std`, empty defaults,
and no unsafe code or build script. Its active profile uses `heapless`,
`num-traits`, and `smallvec`.

Every returned candidate is reordered by Fenestra painter order and subjected
to exact transform, clip, shape, visibility, and input-policy tests. An R-tree
result never decides the topmost target.

## CPU raster candidates

### Tiny-Skia 0.12.0

[Tiny-Skia 0.12.0](https://github.com/linebender/tiny-skia) is the primary CPU
probe. The proposed profile is exactly
`default-features = false, features = ["std"]`, excluding its default PNG and
SIMD features. It is BSD-3-Clause and has no declared MSRV or build/native
path. Its minimal closure still includes `arrayref`, `arrayvec`, `bytemuck`,
`cfg-if`, `log`, and `tiny-skia-path`, and still contains unsafe source.

Pixel bytes are exact evidence only if the registered SIMD-disabled scalar
profile agrees on Linux and Windows. Otherwise the artifact records a versioned
classification and tolerance; structural scene and hit records remain exact.

### Raqote 0.8.5

[Raqote 0.8.5](https://crates.io/crates/raqote/0.8.5) is a CPU comparison with
`default-features = false`, excluding default text and PNG. It is
BSD-3-Clause, has no declared MSRV or native/build path, and retains `euclid`,
`lyon_geom`, `sw-composite`, and `typed-arena`. Its published source contains
19 unsafe blocks. Its older release cadence and documented quality and speed
tradeoffs make it a comparison rather than the primary integration lane.

## Native rich-renderer candidates

### Vello 0.9.0

[Vello 0.9.0](https://github.com/linebender/vello) is the primary rich-renderer
probe, not a selection. It declares Rust 1.88, MIT OR Apache-2.0, is officially
alpha, and supports paths, images, gradients, and text through compute shaders.

Vello 0.9.0 uses `wgpu ^29.0.3`, not wgpu 30. A Vello probe pins 29.0.3 and
explicit native backends without `wgpu` defaults or WebGPU. Vello still brings
PNG, and its wgpu feature activates `vello_shaders`, whose build script
generates shader tables. The wgpu path also has a build script and a substantial
backend, native, and unsafe closure. GPU pixels cannot be one universal golden
across devices and drivers.

### Wgpu 30.0.0

[Wgpu 30.0.0](https://github.com/gfx-rs/wgpu) is a current GPU substrate, not a
2D renderer. It declares Rust 1.87 and MIT OR Apache-2.0 and supports native
Vulkan, Metal, D3D12, and GLES paths through a large `wgpu-core`, `wgpu-hal`,
and Naga closure. It is considered only inside a named renderer adapter.

Co-linking standalone wgpu 30 with Vello's wgpu 29 would duplicate major
versions and distort dependency evidence. WU-0013 does not do that.

### FemtoVG 0.26.0 and Skia-safe 0.99.0

[FemtoVG 0.26.0](https://github.com/femtovg/femtovg) is a separate canvas
comparison. It declares Rust 1.88 and MIT OR Apache-2.0, supports OpenGL and an
optional wgpu 30 path, and has default image and text features. Its stateful
canvas model, missing dash and path-clip coverage, and larger unsafe surface
raise replacement cost.

[Skia-safe 0.99.0](https://github.com/rust-skia/rust-skia) is a high-cost
control alternative. It declares Rust 1.85 and MIT for the bindings, depends on
the matching `skia-bindings`, and crosses C++ FFI. The bindings declare
`links = "skia"` and use build tooling that downloads cached binaries or
compile Skia with LLVM and C++ dependencies. It is neither pure Rust nor
build-neutral.

FemtoVG, Skia, and Vello remain separate probes. One successful lane does not
select or establish support for the others.

## Image and brush candidates

[Peniko 0.6.1](https://crates.io/crates/peniko/0.6.1) is a probe for brushes,
colors, and image resource handles. It declares Rust 1.85, MIT OR Apache-2.0,
`no_std`, no build/native path, and uses Kurbo. Its handles and enums remain
adapter-private.

[PNG 0.18.1](https://github.com/image-rs/image-png) is the first bounded decoder
baseline. It declares Rust 1.73, MIT OR Apache-2.0, is pure Rust, forbids unsafe
code, and has no build script. It is preferred over a broad format dependency
for the first resource probe.

[Image 0.25.10](https://github.com/image-rs/image) is admitted only with
`default-features = false` and named formats such as `png` and optionally
`jpeg`. It declares Rust 1.88 and MIT OR Apache-2.0. Defaults would add Rayon
and the `default-formats` set; `avif-native` would add native DAV1D and is
excluded.

[Resvg and usvg 0.48.1](https://github.com/linebender/resvg) remain a later SVG
probe. Both declare Rust 1.85 and MIT OR Apache-2.0 and forbid direct unsafe
code. Defaults enable text, system fonts, memory mapping, and raster formats,
which introduce host-dependent inputs and a larger closure. SVG is not mixed
into the first PNG resource result.

## Required probe lanes

1. `numeric-spatial`: the Fenestra checked scalar reference against Euclid,
   Kurbo, and Fixed; composition, inverse, singularity, range, transform origin,
   and typed coordinate spaces.
2. `path-hit`: Fenestra path and linear hit oracle against Kurbo and Lyon;
   convex, concave, holes, fill rules, self-intersection, degeneracy, curves,
   strokes, clips, and hits inside an AABB but outside the shape.
3. `cpu-reference`: current rectangle raster plus Tiny-Skia and Raqote on the
   same shapes, alpha, transform, clip, gradient, and image corpus.
4. `native-renderer`: immutable Fenestra frame through Vello and exact native
   backend features; missing capability, resize, resource bounds, adapter
   failure, and no candidate identity leaks.
5. `image-resource`: versioned bytes through PNG and a feature-minimal Image
   comparison; malformed input, dimensions, stride, byte bombs, alpha, gamma,
   profile, and orientation.

Each lane uses two fresh reconstructions, an independent literal oracle,
per-field typed faults, a bounded canonical artifact, and `pass`, `adapt`, or
`stop`. Candidate output never generates oracle expectations.

## Disposition and exclusions

A `pass` admits only the registered bounded corpus. `Adapt` requires one
documented adapter rule that makes every supported case pass. `Stop` leaves the
Fenestra reference implementation authoritative. No screen result selects a
public API, final dependency, renderer, path engine, rasterizer, index, image
stack, or platform support tier.

Three-dimensional scene authoring, browser canvas, DOM, WebView, JavaScript,
runtime SVG document behavior, arbitrary shaders, platform lifecycle, density,
safe areas, surface epochs, and mobile support remain outside this screen.
