# Spatial evidence metadata refresh

The canonical corpus and lane contract remain
[hybrid spatial evidence version 2](../../../../docs/design/hybrid-spatial-evidence-v2.md).
The original native and host measurements remain recorded in
[WU-0013](../../../../docs/verification/WU-0013-hybrid-spatial.md).

## WU-0022 lock-closure change

On 2026-09-08, native accessibility dependencies changed the root lockfile's
transitive dependency entries. The existing
[closure encoder](../../src/lanes/artifact/closure.rs) follows every dependency
entry from the candidate roots and hashes sorted `name@version` lines. This
is the all-target lock closure, including optional dependencies resolved for
other workspace consumers. It is not an active-feature or native execution
claim for an individual target. Both candidate target rows therefore use
the same closure digest.

The [native accessibility admission](../../../../docs/decisions/0002-native-accessibility-adapter.md)
adds accesskit_unix 0.23.0 with `async-io`. That dependency enables
futures-util's defaults, including `async-await-macro`. The already locked
futures-util 0.3.33 therefore gains a dependency on futures-macro 0.3.33.
Tracing the Vello 0.9.0 and wgpu 29.0.3 roots before and after this change
finds exactly one added package and no removed or upgraded package:
the comparison uses `Cargo.lock` at `fce6b75` and the root lock hash below.

```text
before packages: 123
after packages:  124
added: futures-macro@0.3.33
changed edge: futures-util@0.3.33 -> futures-macro@0.3.33
before closure: c0ed8c9b0434d2faf2ded977d486672d42b3751486fb538055cdbe2ff9dd9178
after closure:  c102101897f552977728090a73f71ec9cf9d50fb4c1a2a30f9b93e9f1471ab39
root lock:      f8b7c317837fac559abc955693d4d630f0130420047942ba4de99b30fb8463c2
```

Only the two `closure-sha256` fields in
[native-renderer-v2.txt](native-renderer-v2.txt) are refreshed. Every other
byte in that artifact is unchanged, including candidate versions, selected
features, target labels, classifications, limits, corpus records and section
digests. The lane still reports `stop|reason=target-unavailable`; this refresh
contains no new GPU execution result.

The refreshed artifact remains 25,288 bytes, 331 LF-terminated lines and a
maximum line length of 328 bytes. Its original SHA-256 was
`cd979393ba2686f44003145f89e7d3f979b9d0b1c0def9aabc17ea89dc56971f`;
its current SHA-256 is
`ff31ce4ff10b19b1b6922316c5aaec28824c2fe63e3f4b5f28938f0fa7dc84cb`.

The baseline [spatial-v2.txt](spatial-v2.txt) remains unchanged at SHA-256
`bc71d3f9167808984abf083613ea86a81eced60d8670d9b3133821dbb34d21a1`.
The numeric, path, CPU and image lane artifacts remain unchanged. No encoder,
oracle, fixture, candidate implementation or test expectation was changed.

The existing exact-fresh-encoding regression detected the stale metadata.
Its focused module reconstructs and verifies every lane, compares their
complete retained baseline bodies and checks semantic classifications:

```sh
cargo test --locked -p fenestra-ui-exp-0008-hybrid-spatial --all-features --lib tests::lane_artifacts
```

All four focused tests passed on the versioned development toolchain with
the root lockfile above.
