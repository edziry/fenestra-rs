# WU-0011 layout reference contract

Parent plan: [layout-conformance.md](layout-conformance.md)
Format: layout contract and corpus version 1

## Logical model

All boundary values are logical integer units. Candidate implementations may
use another internal representation, but no floating-point or candidate-owned
value crosses the boundary.

`LayoutNodeKeyV1` is a dense `u32` index local to one computation. Input nodes
are stored in authored preorder. Node zero is the only root. Every later node
has a parent key smaller than itself. A key is neither a runtime `NodeId` nor a
persistent identity and must never be serialized outside WU-0011 evidence.

Preorder validation maintains an active ancestor stack. Before each later node,
completed sibling subtrees are popped until its declared parent is the stack
top; a parent that was already popped is an authored-preorder failure. The root
has depth one, and each accepted node is then pushed.

The closed axis vocabulary is `Row` and `Column`. Every node carries an axis for
its children, a width and height constraint, padding, and a gap. Leaves retain
an axis but it has no effect.

Each dimension contains nonnegative `minimum`, `preferred`, and `maximum`
integers with `minimum <= maximum`. Its resolved border-box extent is
`preferred.clamp(minimum, maximum)`. Padding sides and gap are nonnegative.
Padding belongs inside the resolved border box. There is no border or margin.
Horizontal padding must fit the resolved width and vertical padding must fit the
resolved height. Validation compares each side sum in widened `i64` arithmetic,
so an oversized sum remains a padding-fit error rather than integer overflow.
Version 1 does not derive intrinsic sizes from children: left and top padding
establish the child origin, while right and bottom reserve the far-side content
inset. Fixed children may overflow the remaining content area and are neither
resized nor clipped by layout.

The content extent is `width - left - right` by
`height - top - bottom`, using checked subtraction after the fit validation.
Layout does not clip to that content extent. Runtime clips later against the
ancestor border box and the viewport, never against the content box.

The viewport is a present nonnegative logical width and height. `0xN`, `Nx0`,
and `0x0` are valid degenerate viewports. They are not lifecycle, suspension,
backgrounding, or surface absence.
It is available-space metadata: fixed Stack version 1 does not resize bounds
from it, and runtime uses it for the later clip.

## Stack algorithm

The root origin is `(0,0)`. Its bounds use its resolved width and height.

For each parent in preorder:

1. The content origin is parent origin plus left and top padding.
2. Children retain authored order.
3. In a column, each child begins at the content x and current y. The cursor
   advances by the child's resolved height.
4. In a row, each child begins at the current x and content y. The cursor
   advances by the child's resolved width.
5. One gap is added only between adjacent children, never before the first or
   after the last.
6. A child is not stretched, shrunk, wrapped, aligned, or resized by overflow.
7. Descendant origins are absolute logical coordinates.

Visibility is not a layout input. A hidden child occupies the same stack space
as a visible child; runtime projection later filters its derived outputs.
Right and bottom padding do not move fixed children in this version because no
intrinsic sizing, stretch, or far-edge alignment exists.

All origin, edge, padding, and cursor operations use checked arithmetic.
Negative geometry is rejected before computation. Overflow is typed and does
not publish a partial output.

Layout output contains one key and one absolute border-box rectangle per input
node in the same order. Width and height are nonnegative. Output has no clip,
visibility, z order, hit target, color, runtime generation, or surface epoch.

Viewport and ancestor clipping remain a runtime projection operation. Clips
are half-open rectangles. An empty intersection keeps the maximum input origin
and uses zero width or height. Scene and hit records use the resulting clip;
they do not reinterpret layout bounds.

## Rounding and scale

The reference stack engine performs integer arithmetic and therefore has no
rounding. A candidate with fractional internals must round cumulative absolute
near and far edges to the nearest integer, with exact halves away from zero,
then compute the extent as `far - near`. It must reject non-finite values,
negative extents, out-of-domain coordinates, missing records, and duplicate or
reordered keys.

Physical scale is not a layout input. Equal logical viewport and node
constraints must produce equal logical output when an exterior projection
context is 1.0, 1.25, or 2.0. Physical edge conversion is an adapter
responsibility and does not authorize scale-dependent layout output.

## Closed validation priority

Validation stops at the first applicable failure:

1. node-count capacity;
2. empty input;
3. root key then root parent;
4. each later dense key, missing parent, forward parent, then authored preorder
   shape;
5. depth capacity, with root depth equal to one;
6. children-per-node capacity;
7. negative viewport width then height;
8. constraints in node order, width before height; within each dimension check
   negative minimum, preferred, and maximum, then inverted minimum/maximum;
9. negative padding in node order, left, right, top, bottom;
10. horizontal then vertical padding exceeding the resolved box in node order;
11. negative gap in node order;
12. checked arithmetic in authored traversal order: root far x then y; each
    parent's content x then y; each child's far x then y; then the non-final
    main-axis gap advance. The main-axis far edge becomes the next cursor;
13. global output phases: record count; every key in order; every nonnegative
    x, y, width, and height in record/field order; then every far x and y edge.

Equality at every declared capacity is accepted. The first value above it is
rejected without allocating the next retained record. Caller-owned input is
outside retained-memory accounting; allocator exhaustion is not a typed result.
The node ceiling also bounds the required output cardinality; output has no
separate core record limit.

Diagnostics use trusted input or output ordinals rather than malformed supplied
keys. Their closed kind and location are public; `Display` and `Debug` expose no
input scalar, candidate diagnostic, pointer, or allocation detail.
Far-edge arithmetic names the root or child whose edge failed. Content-origin
and gap-advance arithmetic name the parent that owns the padding or gap.

The registered WU-0011 profile is:

```text
nodes = 32
depth = 8
children-per-node = 16
candidate-input-scalar = 4096
candidate-output-edge = 524288
artifact-records = 512
artifact-line-bytes = 512
artifact-bytes = 65536
```

The core integer reference accepts the full nonnegative `i32` domain subject to
checked arithmetic. The candidate adapter admits each viewport, constraint,
padding, and gap scalar only through 4096. With the registered tree bounds this
keeps every legitimate cumulative edge below 524288, well inside exact integer
representation by `f32`; the adapter validates the output edge ceiling too.

Candidate admission is a separate closed probe result, not a core layout
validation error. The adapter validates every candidate scalar against its
ceiling in viewport then node/field order after core input validation and before
constructing a candidate tree. A scalar at 4096 is admitted and 4097 is a typed
`CoordinateLimit` refusal while remaining valid for the integer reference. Its
output-domain validation first rejects non-finite raw `x`, `y`, `width`, and
`height` values in record/field order, then rejects negative raw values in the
same order. It next accumulates unrounded absolute edges in record order, with
the horizontal near/far edge before the vertical near/far edge. Every raw
absolute near and far edge must remain finite, nonnegative, and at or below the
524288 ceiling before rounding. Only then does the adapter apply the Fenestra
cumulative-edge rule exactly once. Thus a raw edge above the ceiling cannot be
admitted merely because rounding would move it back to the ceiling. These
failures are classified as `NonFiniteOutput`, `NegativeOutput`, and
`OutputEdgeLimit`, respectively, before mapping to the neutral
`UnrepresentableOutput` engine result. Taffy's default pixel rounding is
disabled throughout this path.

## Corpus

The version 1 corpus contains these valid families:

1. one fixed root;
2. two-child column;
3. two-child row;
4. nested row inside column;
5. asymmetric padding on both axes;
6. nonzero gap with three children;
7. preferred width and height below minimum;
8. preferred width and height above maximum;
9. mixed min, preferred, max and padding;
10. child main-axis overflow retained beyond its parent;
11. child cross-axis overflow retained beyond its parent's content area;
12. padding exactly equal to each resolved extent;
13. zero-width child among nonzero siblings;
14. zero-height child among nonzero siblings and combined with nonzero gap;
15. `0xN`, `Nx0`, and `0x0` viewports with unchanged bounds;
16. exact node, depth, child, and candidate scalar ceilings;
17. the registered WU-0008 vertical runtime fixture;
18. the keyed runtime sequence after insert, move, update, remove, and resize.

Invalid families cover empty input, second root, non-dense and reordered keys,
forward or missing parent, node/depth/child one-over, negative viewport,
negative constraint fields, inverted min/max, negative padding and gap, padding
that exceeds its resolved box, child far-edge and non-final gap-advance
overflow, candidate scalar and output-edge one-over, and malformed engine
output.

Output-edge exact and one-over cases use a synthetic candidate-output validator
fixture because admitted version 1 inputs cannot legitimately reach the 524288
ceiling. Artifact record, line, and byte exact/one-over cases likewise exercise
the encoder and verifier rather than Taffy layout.

Each admitted valid direct case is solved by the owned reference engine and
Taffy, then compared against normalized expected tables written independently
for the corpus. Core-valid cases outside the candidate profile run only through
the integer reference and prove the typed candidate refusal separately. Each
output is compared field by field in input order. Fault controls independently
mutate record order, key, x, y, width, and height and must identify the first
unequal field. A second run must produce identical owned output and artifact
bytes. The registered runtime script continues to compare against the existing
clean-rebuild oracle.

## Runtime mapping

The existing headless fixture maps to the stack contract without extending the
IR:

- every container axis is `Column`;
- padding and gap are zero;
- the root uses minimum zero and preferred/maximum equal to each materialized
  dimension;
- every child height uses minimum zero and preferred/maximum equal to its
  materialized height;
- every child width uses minimum zero, preferred equal to its materialized
  width, and maximum equal to the already resolved parent width, preserving the
  existing clamp even when the authored child width is larger;
- keys are assigned from the current authored preorder on every rebuild.

The registered 32-node, depth-8, child-16 limits belong only to the direct
corpus, candidate profile, and evidence. The default runtime reference derives
per-pass tree limits from its already preflighted projected node count, using
that count for node, depth, and child ceilings, so WU-0011 introduces no new
runtime capacity rejection. Patch-parity tests cover valid headless trees above
each registered probe limit.

Row, padding, gap, and general min/max inputs are exercised only by the direct
layout corpus. They are not authorable or mutable through the provisional IR in
WU-0011. Adding those authored semantics requires a later IR work unit and a
new version decision; scalar sentinels are forbidden.

At every effective runtime transaction:

- the receipt generation equals the projection generation;
- layout invalidation is present for width, height, structure, keyed order, and
  viewport changes, and absent for color-only changes;
- output keys are mapped back only to current draft `NodeId` values;
- geometry bounds equal the engine result;
- each visible nonempty hit and scene rectangle equals its geometry clip;
- failed engine input or output retains the complete previous allocation and
  generation.

Patch compatibility preserves the existing runtime failure order. Initial
configuration validates the specification and then its surface first; each
build or rebuild then orders surface validation, fixed projection capacity, the
global negative width/height scan, layout, and derived-record capacity. The
negative scan retains its current operation attribution. Layout arithmetic maps
to
`HeadlessProjectionErrorKind::ArithmeticExhausted`; any other impossible input,
malformed engine output, or candidate refusal maps to `InvariantViolation` and
rolls back. The engine trait is `Send + Sync + Unpin + UnwindSafe +
RefUnwindSafe + 'static`, and compile-time tests prove that injecting it
preserves `UiRuntime`'s `Send`, `Sync`, `Unpin`, `UnwindSafe`, and
`RefUnwindSafe` traits.

## Artifact contract

The canonical artifact is printable ASCII plus LF with exactly one final LF.
It records format versions, package and candidate versions, registered limits,
case names, input nodes, oracle output, reference output, candidate output,
classification, runtime generations, invalidation words, geometry, hit paths,
scene paths, and dependency feature facts. It records no host path, clock,
username, runtime IDs, candidate IDs, pointer values, `Debug` output, or source
payload.

Bounds are enforced in order: records, line bytes, artifact bytes. Exact limits
are accepted and one-over failures are typed. Hashes and environmental timing,
RSS, and target size summarize versioned bytes but are not correctness inputs.

The version 1 artifact is one pipe-delimited file with exactly 412 records. Its
direct section contains 23 case headers, 120 input-node rows, 120 output rows,
and 23 case-result rows. Every output row carries independently obtained
oracle, integer-reference, and candidate records together; separate rows per
lane would exceed the registered record ceiling. The runtime section contains
one header, 7 generation rows, 38 geometry rows, 24 hit rows, 38 scene rows,
and one result row. Its stable steps are initial, color, insert, move, update,
remove, and resize, and each payload row carries the oracle, reference, and
candidate lanes explicitly with normalized semantic paths rather than runtime
identities.

Artifact validation is two-phase. It first validates the complete typed model
and record count before rendering any row. It then validates every rendered
line before the accumulated byte count. This makes simultaneous crossings
observe the required Records, LineBytes, ArtifactBytes priority and prevents a
late 513th record from losing to an earlier long line.

## Exit

WU-0011 passes only when all supported cases and the runtime script match the
independent oracle, every negative control is detected, all failures are
atomic, the artifacts are deterministic, and Linux plus Windows pure gates pass.
The result remains a bounded stack-subset result and does not select final
layout semantics or a final candidate.
