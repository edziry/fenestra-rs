# WU-0013 hybrid spatial evidence contract

Status: frozen for evidence RED/GREEN
Work unit: WU-0013
Parent plan: [hybrid spatial composition](hybrid-spatial-composition.md)
Reference: [hybrid spatial reference](hybrid-spatial-reference-v2.md)
Candidate screen: [hybrid spatial candidate screen](hybrid-spatial-candidate-screen.md)
Authoring runtime equivalence: [format-2 runtime equivalence](hybrid-spatial-authoring-runtime-equivalence-v2.md)
Presentation: [immutable spatial presentation](hybrid-spatial-presentation-v2.md)
Verification: [WU-0013 hybrid spatial verification](../verification/WU-0013-hybrid-spatial.md)
Format: hybrid spatial evidence version 2

## Purpose and experiment identity

This contract owns the bounded, host-neutral evidence for the implemented
hybrid spatial reference and for later disposable candidate comparisons. The
first result proves only the Fenestra reference against an independent literal
oracle. It neither evaluates nor selects a numeric, path, raster, renderer, or
image candidate.

New evidence lives in a new unpublished workspace probe:

```text
probes/exp-0008-hybrid-spatial
```

The historical `probes/exp-0008-layout-conformance` package and its
`layout-conformance-v1.txt` artifact remain the WU-0011 `layout-v1` result.
They are not renamed, extended, regenerated, or reinterpreted by WU-0013.
`EXP-0008` therefore contains separate `layout-v1` and `spatial-v2` evidence.

The first committed file is
`probes/exp-0008-hybrid-spatial/tests/artifacts/spatial-v2.txt`.

It contains no candidate row or candidate classification. The five later,
independently reviewed result files are:

```text
numeric-spatial-v2.txt
path-hit-v2.txt
cpu-reference-v2.txt
native-renderer-v2.txt
image-resource-v2.txt
```

The baseline file is not rewritten when a later result lands. A result file
does not exist until its complete registered lane has executable evidence. An
absent file means `not evaluated`, never `pass`, `adapt`, or `stop`.

The baseline alone cannot complete WU-0013 or EXP-0008. Implementing all five
registered candidate lanes and passing the Linux and Windows gates below is a
necessary, not sufficient, exit condition. All placement, authoring, runtime,
presentation, rollback, and branch-cleanliness criteria in the parent plan
still apply.

## Private ownership and package boundary

The probe has no product API. All evidence records, normalizers, literal
oracle types, comparison faults, candidate adapters, artifact limits, errors,
decoder, verifier, and encoder are private to the probe. They are private Rust
items or `pub(crate)` only where split modules require it. No evidence type is
re-exported from `src/lib.rs`, a core crate, the facade, or another probe.

The baseline has workspace path dependencies only on `fenestra-ui-ir`,
`fenestra-ui-layout`, `fenestra-ui-spatial`, and `fenestra-ui-runtime`. IR
constructors build the keyed runtime case and spatial constructors build direct
cases. Authoring, macros, testkit, native probes, and candidates are excluded.

The private roots are `SpatialEvidenceV2`, `SpatialEvidenceCaseV2`,
`SpatialEvidenceObservationV2`, `SpatialEvidenceArtifactV2`, its limits and
error types, and private `encode`, `decode`, and `verify` functions. Candidate
types occur only below `lanes/<lane>/`. The literal oracle cannot import a
candidate crate, `fenestra-ui-spatial`, runtime, authoring, macros, testkit, a
generated fixture, the manual authoring builder, or another probe. It may use
only `std` and probe-private literal types.

The reference runner constructs public Fenestra inputs directly. It does not
read the literal result, candidate output, or golden. Candidate runners receive
the same probe-private case value through explicit adapters and cannot produce
expected values. Every lane performs two fresh constructions and executions;
cloning a prior output, snapshot, or candidate context is not a reconstruction.

## Registered corpus

Corpus V2 has these cases in exact order:

```text
00 all-layout
01 all-free
02 free-to-layout
03 layout-to-free
04 mixed-siblings
05 transparent-wrapper
06 split-geometry
07 transformed-clip
08 polygon-path
09 rich-paint
10 anchor-forward
11 zero-extent
12 runtime-mutation
13 runtime-rollback
```

Cases 00 through 11 are direct candidate-neutral spatial inputs. Case 12 is
the nine successful observations from the exact init, resize, property, insert,
move, update, and remove script fixed by the authoring runtime-equivalence
contract. Case 13 is its singular-transform attempt and exact retained-state
rollback observation. The probe owns typed copies of the fixture values; it
does not import `exp-0007` test support or parse either authoring fixture.

The direct registry fixes these obligations:

- `all-layout` and `all-free` each contain at least three non-sentinel nodes
  with nested descendants and an observation after viewport resize;
- both mixed-direction cases contain the named outer mode, the opposite inner
  mode, and one child below the inner host;
- `mixed-siblings` has one Layout and one Free child under the same parent and
  proves that the Free child consumes no layout space;
- `transparent-wrapper` participates in layout and hosts a child while owning
  zero paint and hit items;
- `split-geometry` has distinct layout extent, overflowing paint, smaller
  circular hit, and independent semantic bounds;
- `transformed-clip` composes translate, quarter turn, and scale across three
  levels and applies a two-link clip chain;
- `polygon-path` includes concavity, an AABB miss, both fill rules, two
  subpaths, self-intersection, a zero-length segment, quadratic and cubic
  curves, fill, and round stroke;
- `rich-paint` includes solid, partial alpha, linear gradient, normalized
  premultiplied RGBA8 image, source-over overlap, and explicit clip;
- `anchor-forward` targets a later node and also covers parent and viewport
  anchors; its paired invalid control is a dependency cycle;
- `zero-extent` contains `0xN`, `Nx0`, and `0x0` geometry without treating any
  extent as suspension or absent presentation.

Exact scalar values, authored order, keys, tables, operations, and query points
are versioned in the private typed corpus constructors. A corpus value cannot
be updated merely to match output. Any incompatible case change advances the
corpus version and artifact filename. Before the first golden is accepted,
tests require every obligation above, every literal value, and every expected
record without consulting reference output.

The query set for every direct observation is, in order: every shape vertex,
every segment midpoint representable in Fixed16, every clip-bound corner,
every paint and hit AABB center, one point inside each AABB but outside its
nonrectangular shape, every authored boundary point, and the four viewport
outside points fixed by the runtime-equivalence contract. Duplicates remain.
Runtime cases additionally query every logical pixel center in row-major order
and then those four outside points.

## Normalized model and comparison

Normalization copies values into private owned Eq records. It retains signed
Fixed16 scalars as `i64`, determinants as `i128`, dimensions and keys in their
declared widths, colors and pixels as bytes, enum variants as closed tags, and
options as tagged values. It never compares floats, formatted Debug output,
pointer identity, allocation addresses, runtime IDs, candidate IDs, native
handles, or candidate diagnostics.

Each successful observation contains these sections in exact order:

```text
receipt, mapping, source, geometry, clips, paints, hits, semantics, queries,
raster
```

`receipt` stores the optional generation, viewport, mutation count, and
invalidation word. `mapping` uses the logical path grammar from the
runtime-equivalence contract and is empty for direct cases. `source` stores
polygon points, path verbs and descriptors, shapes, gradient stops, brushes,
images including exact bytes, clip primitives, and paint items in supplied
order.

`geometry`, `clips`, `paints`, `hits`, and `semantics` store every field fixed
by that same runtime-equivalence normalizer, in supplied table order. Clips
also store the same-index effective AABB. Queries store the raw scene point and
either `none` or key, owner, item ordinal, and raw local point. Raster stores
width, height, `u64` stride, and the complete RGBA8 byte slice.

The literal and reference models are compared recursively in the section order
above, then record order, then declaration field order. The first mismatch is
a private `section + record + field` location. All complete fields compare
before any artifact digest is computed. Two fresh literal builds, two fresh
reference builds, and the corresponding cross-build pairs must be exactly
equal.

A candidate comparison repeats this rule for every field its registered lane
claims. Unsupported fields produce the screen's explicit reduced or unsupported
candidate result; they are never omitted from the reference model. A structurally
valid candidate geometry difference is a comparison mismatch, not a resolver
validation error.

## Literal oracle and control matrix

The literal implementation independently performs checked Fixed16 arithmetic,
affine composition and inverse, layout-stack placement, stable dependency
evaluation, shape and clip coverage, reverse-order hits, brush and image
sampling, source-over, and 4x4 reference raster sampling. It does not call a
Fenestra validator, resolver, hit query, rasterizer, or candidate adapter.

The baseline cannot be encoded until all controls pass:

1. Mutate each tag, scalar, key, option marker, count, byte, and metadata field
   once and require the exact first private mismatch location.
2. In every ordered section, swap each adjacent pair, remove each row, and
   duplicate each row; all mutations must be detected.
3. Change one query from hit to miss and miss to hit, then mutate every hit
   result field and local coordinate.
4. Mutate raster width, height, stride, byte length, and one byte at the first,
   middle, and last positions.
5. Exercise every registered raw input limit at equality and one over, every
   structurally invalid output-table field fault, the dependency cycle, the
   singular runtime transform, and exact rollback.
6. Exercise artifact model, grammar, record, line, and byte faults, plus two
   fresh byte-identical encodings and decode-encode identity.

Raw hit, paint, brush, image, clip, and shape faults must return the existing
typed input validation kind, location, and limit evidence. Malformed supplied
output must return the existing typed output-validation result. A raster pixel
limit fault must be exactly `LimitExceeded(Pixels)` at `Input`, with its real
observed and maximum values. A structurally valid wrong hit is a `queries`
mismatch; a wrong paint result is a `raster` or normalized-paint mismatch.
There is no invented `HitError`, `PaintError`, or renderer-shaped core error.

Only the later `native-renderer` result adds native presenter faults. They use
the private phase outcomes fixed by the presentation contract: pre-accept
rejection preserves the last successful digest, while post-accept present
failure reports renderer loss and preserves ordered retirement. They do not
roll back an already committed runtime generation and are not part of the
baseline `faults` count.

## Canonical artifact grammar

The artifact is printable ASCII, uses LF only, and has exactly one final LF.
It is a pipe-delimited sequence with no quoting or escaping. A token matches
`[a-z0-9][a-z0-9._:-]*`; a list is one or more comma-separated tokens;
unsigned decimal has no leading zero except `0`; signed decimal adds an
optional `-`; `hex16` and `hex64` are exactly 16 and 64 lowercase hex digits;
`-` is the sole absent value. Fields and records appear only in the order below:

```text
spatial-v2|artifact=2|contract=2|corpus=2|kind=baseline
packages|probe=0.2.0|ir=0.2.0|layout=0.2.0|spatial=0.2.0|runtime=0.2.0
profile|spatial=registered-v2|raster=registered-v2|candidate-count=0
limits|spatial=256,1024,256,512,1024,512,256,256,4096,4096,2048,64,32,64,64,128,192,256,1024,256,32,4096,4194304,32,64,64,4096,65536,192,256|raster-pixels=4194304|records=4096|line-bytes=1024|artifact-bytes=1048576
case|ordinal=<u>|name=<token>|observations=<u>
observation|case=<u>|step=<u>|generation=<u-or-dash>|viewport=<u>x<u>
section|case=<u>|step=<u>|name=<section>|records=<u>|bytes=<u>|digest=<hex16>
case-result|case=<u>|literal=match|reference=match|repeat=match
control|family=<metadata|records|fields|queries|raster|faults|codec>|registered=<u>|detected=<u>
result|literal=pass|reference=pass|candidate-count=0
end|spatial-v2
```

There is one `case` block per registered case. Each observation has exactly the
ten section rows in normalized order, including zero-record sections. The seven
control rows follow all case blocks in their displayed family order. Counts are
canonical typed-model facts, not caller-provided strings.

Section bytes use one private binary encoding: enum tags are their documented
zero-based order in one byte; booleans are `0` or `1`; integers are fixed-width
little-endian two's complement; options are a `0` or `1` byte followed by the
value when present; slices use a little-endian `u64` count; byte slices then
store their bytes; paths use a little-endian `u32` length and ASCII bytes;
struct fields and sections use the order above. The digest is FNV-1a-64 over
`spatial-evidence-v2`, one zero byte, the section token, one zero byte, and the
encoded section. Offset basis is `14695981039346656037`, prime is
`1099511628211`, multiplication wraps modulo 2^64, and output is big-endian
numeric spelling as 16 lowercase hex digits. This digest is a compact summary,
not a correctness or cryptographic input.

Later lane files replace `kind=baseline` with `kind=lane`, set a nonzero
candidate count, and add these rows in candidate ordinal order:

```text
candidate|ordinal=<u>|lane=<token>|name=<token>|versions=<list>|features=<list-or-dash>|target=<token>|closure-sha256=<hex64>|baseline-sha256=<hex64>
classification|candidate=<u>|outcome=<pass|adapt|stop>|reason=<token-or-dash>
```

Candidate rows follow `profile`; classification rows precede `result`.
`pass` requires reason `-`. Adapt reasons are exactly
`fixed16-conversion`, `edge-rounding`, `painter-reorder`,
`premultiplied-rgba8`, `orientation-normalization`, or `profile-rejection`.
Stop reasons are exactly `mismatch`, `unsupported`, `nondeterministic`,
`dependency-policy`, `target-unavailable`, `unsafe-boundary`,
`build-boundary`, or `resource-bound`. Adding a reason advances the artifact
version. A lane file is incomplete if any required tuple is missing; screened
alternatives outside a required lane do not silently become results.

`target` is not caller-selected text. Each pure candidate has exactly
`x86_64-unknown-linux-gnu` and `x86_64-pc-windows-msvc`. Vello instead has
`x86_64-unknown-linux-gnu:vulkan-wayland` and
`x86_64-pc-windows-msvc:dx12-win32`. Other or missing targets are invalid. A
proven unsupported tuple may be `stop`; absence still has no classification.

## Limits and failure priority

Artifact limits are inclusive and exact:

```text
records = 4096
line-bytes = 1024
artifact-bytes = 1048576
```

A record is one line including its LF. Line bytes exclude LF; artifact bytes
include every LF, including the one final LF.

The private encoder first validates the complete typed model, versions,
registered case order, section order, references, counts, comparisons,
controls, and allowed baseline or lane outcome. It then applies this closed
priority:

1. invalid typed model;
2. record count;
3. invalid record grammar in record order;
4. line bytes in record order;
5. accumulated artifact bytes.

Equality passes. Every exact and one-over limit has a synthetic encoder test.
The encoder renders nothing until model validation and record-count preflight
succeed, validates a complete line before adding it, and returns no partial
artifact. Decoder verification repeats grammar, canonical decimal and digest
spelling, versions, order, counts, references, limits, and semantic replay.

## Candidate lane pins and result rules

Each later result records the exact candidate tuples from the candidate screen.
Dependencies remain private to this probe and enter only with their executable
lane; placeholder dependencies are forbidden. In particular, `image-resource`
must use exactly `png = 0.18.1` and
`image = 0.25.10, default-features = false, features = ["png"]` when that lane
is implemented. Other Image defaults, formats, Rayon, `avif-native`, native
DAV1D, and host-discovered resources are excluded.

The required tuple registry is Euclid 0.22.14, Kurbo 0.13.1, and Fixed 1.30.0
for `numeric-spatial`; Kurbo 0.13.1 and Lyon Tessellation 1.0.20 for `path-hit`;
Tiny-Skia 0.12.0 and Raqote 0.8.5 for `cpu-reference`; Vello 0.9.0 with wgpu
29.0.3 for `native-renderer`; and PNG 0.18.1 plus Image 0.25.10 for
`image-resource`. The owned Fenestra implementation is a control, not a
candidate tuple.

`numeric-spatial`, `path-hit`, `cpu-reference`, `native-renderer`, and
`image-resource` each compare two fresh candidate runs with the same literal
and reference model. Every result records exact versions, features, target,
licenses, declared Rust versions, active and lock-only closure, unsafe source
inventory, build scripts, native or FFI edges, and replacement constraints.
Candidate output and errors never enter the baseline artifact or public API.

`pass` requires every registered case and field. `adapt` requires one named,
closed adapter rule that makes the complete registered lane match. `stop`
retains the Fenestra reference and records a closed reason. A partial corpus,
tolerance invented after observation, or unavailable target cannot be `pass`.
GPU pixels are not a universal golden; the native lane records structural and
protocol equality plus its predeclared target-qualified pixel classification.

## Host-neutral evidence and platform gates

Canonical artifacts contain no path, username, hostname, clock, duration,
process or thread ID, pointer, runtime identity, native handle, GPU device or
driver string, environment variable, panic text, Debug output, source payload,
or arbitrary candidate message. Closed target and backend labels are allowed
only in lane tuples. Timing, RSS, target size, OS build, compiler details, and
interactive native observations belong in a later verification record, never
in canonical bytes.

The baseline and every applicable pure lane must pass target-native Linux and
Windows gates from clean versioned source with `Cargo.lock` unchanged:

```text
cargo fmt --all -- --check
cargo test --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --workspace --all-features --no-deps --locked
cargo metadata --format-version 1 --locked
git diff --check
```

Both systems independently rebuild the typed model twice, compare complete
literal and reference values, run every mutation and fault control, decode and
re-encode the committed artifact, and measure identical bytes, LF count,
maximum line bytes, final LF, and SHA-256. The reported SHA-256 is an external
measurement of versioned bytes, not generated by the oracle.

Candidate profiles also run their exact lane command and dependency-tree audit
on every supported required target. A target unavailable to a lane is reported
as unverified and blocks that lane's `pass`. Native interactive execution is
reported separately from pure build and protocol tests; Windows compilation
does not claim interactive Win32 or GPU parity.

## RED/GREEN and nonclaims

The baseline uses separate focused commits for its exclusive artifact RED and
reference-plus-literal GREEN. Each candidate lane later uses its own RED,
adapter GREEN, result artifact, and verification update. No golden is copied
from stdout or refreshed until literal/reference equality, controls, codec
round trip, limits, privacy, and two fresh constructions pass.

This evidence does not select a product dependency, public renderer, stable
API, ABI, MSRV, support matrix, product capacity, GPU pixel golden, mobile
platform, lifecycle model, or physical-pixel policy. WU-0012 remains deferred.
