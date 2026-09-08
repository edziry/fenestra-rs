# WU-0023 verification: committed raster reuse

Status: bounded increment verified; product goal remains active
Branch: `perf/committed-raster-cache`
Baseline: `48f68b6`
Environment: Linux x86_64, Fedora 43, Wayland, Rust 1.97.1
Date: 2026-09-08 UTC

## Problem and intended behavior

[WU-0022](WU-0022-native-accessibility.md) measured stable memory in the native
preferences probe, but repeated reference rasterization consumed substantial
CPU while the accepted frame remained unchanged. Each call to
`Application::raster` traversed and sampled the complete paint frame. Native
presentation and instrumentation could request the same pixels more than once.

This increment retains one successful raster for the current committed
generation. Construction and mutations remain lazy with respect to final-frame
rasterization. A repeated read returns an owned raster with identical bytes
without repeating reference sampling. Successful publication invalidates the
cached frame; rejected mutations and true no-ops preserve it.

The generation is a conservative cache key. All accepted facade publications
advance the runtime generation, including changes to text, controls, focus and
state colors. Input bookkeeping that leaves rendered state unchanged does not
require new pixels. Semantic-only updates may invalidate unnecessarily; this
does not return stale pixels.

A failed render is not cached and never falls back to pixels from an older
generation. Retrying the same generation performs fresh rendering. The cache
does not retain runtime snapshots or a history of frames.

## Memory boundary

The cache retains at most one RGBA frame: four bytes per viewport pixel, bounded
by the application's existing pixel limit. The 640 x 520 preferences example
adds 1,331,200 retained pixel bytes, about 1.27 MiB. At the default maximum
viewport budget, the cache can retain 16 MiB. This is a pixel-storage bound,
not a whole-process heap ceiling.

`Raster` still owns its byte vector. Returning it preserves the public owned
snapshot contract and copies those bytes. Caller-held rasters and transient
renderer allocations remain separate costs. Rejected publication keeps the
last accepted cache usable; successful publication releases it before a later
render can populate the new one.

## Regressions and measured effect

[Private cache tests](../../crates/fenestra-ui/src/application/raster_cache/tests.rs)
first failed against the uncached path. Seven tests now verify render-call
counts at the actual application boundary, lazy construction/publication,
no-op reuse, focus attachments, latest-only storage, explicit invalidation and
failed-render retries without stale fallback. The counter exists only in tests.

Four [public regressions](../../crates/fenestra-ui/tests/control_atomicity/raster_cache.rs)
reuse the existing controlled text engine to check background, viewport, focus,
checkbox, state styles, text content and rejected text-style changes. Retained
old raster values preserve their pixels. The full atomicity target passes with
both default and native features.

The public-API [measurement example](../../examples/controls-app/examples/raster-cost.rs)
times five reads at generation zero, then an accepted checkbox change and
another read. Checksums, byte comparisons, retained-frame checks and JSON output
are outside the timed region. The exact same source was linked against the
pre-cache libraries for the baseline and rebuilt against this implementation.
The [protocol](artifacts/WU-0023/baseline-protocol.json) records source identity
and linking conditions. All seven samples in each before/after pair have
identical generations, dimensions, byte counts and checksums.

| Profile | Median of four unchanged reads before | Median after | Evidence |
| --- | --- | --- | --- |
| Debug | 987.718 ms | 0.109 ms | [Before](artifacts/WU-0023/before-debug.json), [after](artifacts/WU-0023/after-debug.json) |
| Release | 40.739 ms | 0.068 ms | [Before](artifacts/WU-0023/before-release.json), [after](artifacts/WU-0023/after-release.json) |

The first frame and the first read after an accepted visual change still render
fully: about 1.12/1.17 seconds in debug and 52.9/48.7 ms in release in this run.
Cached reads still allocate and copy the owned byte vector. Individual warm
samples vary, including an allocation outlier in each profile. These are short
observations amid other development work, without timing assertions, a latency
budget or a general performance SLA.

Reproduce from the repository root:

```sh
cargo run --manifest-path examples/controls-app/Cargo.toml --locked --example raster-cost
cargo run --manifest-path examples/controls-app/Cargo.toml --release --locked --example raster-cost
```

The initial checksum remains `9de7fa0cecfbc58e`; directly setting compact to
checked produces `34ac8e97cb4a004e`. This example uses the public setter without
the preferences event handler, so its changed state is distinct from the full
Apply workflow verified through AT-SPI.

## Native action and resource verification

The actual Linux AT-SPI probe passes all four stages again in
[debug](artifacts/WU-0023/atspi-debug.json) and
[release](artifacts/WU-0023/atspi-release.json). Every successfully presented
PPM is byte-identical to the corresponding WU-0022 release export, including
the focused checkbox, changed state and applied readout. Generations remain
0, 1, 5 and 7; role, bounds, state and focus checks pass. Both native processes
close normally through Finish and leave the original desktop accessibility
settings unchanged. The probe still uses its private activation-status fixture
and the real desktop accessibility bus.

Short process samples during these action runs are recorded separately:

| Profile | Helper duration | Observed post-presentation RSS | PSS | Evidence |
| --- | --- | --- | --- | --- |
| Debug | 8.001 s | 29.85 to 32.66 MiB | 26.24 to 29.05 MiB | [Samples](artifacts/WU-0023/memory-debug.json) |
| Release | 1.849 s | 17.95 to 20.00 MiB | 14.21 to 16.26 MiB | [Samples](artifacts/WU-0023/memory-release.json) |

The measured processes used no swap. These action runs have no deliberate idle
interval and do not establish a leak guarantee or a total memory budget. The
cache improves repeated rendering cost; it is not a claim of reduced RAM use.

## Executed checks

| Check | Result |
| --- | --- |
| Facade, text adapter and inspector, all targets/features, locked | 213 passed in 29 suites; zero failed or ignored |
| Same packages: strict Clippy and rustdoc with warnings/missing docs denied | Passed |
| Same packages: Windows MSVC all-target/all-feature cross-check | Passed |
| Root formatting and diff whitespace | Passed |
| Cache-specific regressions | Seven private tests and four public regressions passed |
| Typed/text/responsive/controls consumers, all targets/features | 3 / 8 / 6 / 6 passed |
| Isolated text screen, all targets/features | 37 passed |
| Controls consumer strict Clippy, debug/release builds and Windows MSVC check | Passed |

These affected application paths total 273 passing tests. Existing headless
results are unchanged: typed generation 3 with five nodes; text checksum
`23758408e9c7bfb6`; responsive `96872b7eb6ddf26b`; controls
`eca494dbc1c7b447`; isolated text-pad `69cc36a24a65d6e7`. The Parley and
cosmic-text candidate reports and Parley diagnostics remain byte-identical.

The previous increment's full workspace run passed 2,444 tests before these
eleven additions. This increment reruns the affected facade, adapter, inspector
and standalone application paths; it does not claim another full workspace
run. Dependencies and every lockfile remain unchanged.

## Remaining scope

The reference renderer still performs full work for a changed frame. This
increment does not implement damage tracking, a production GPU renderer,
zero-copy returned rasters or a process-wide memory budget. Accessibility tree
export also repeats linear geometry searches per node and needs separate
scaling measurements. Editable controls, scrolling and native Windows
qualification remain part of the active
[completion goal](../project-completion-goal.md).
