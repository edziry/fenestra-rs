# WU-0013 hybrid spatial composition plan

Status: planning
Work unit: WU-0013
Branch: `feat/hybrid-spatial-composition`
Depends on: [WU-0011 layout conformance](layout-conformance.md)
Candidate screen: [hybrid spatial candidate screen](hybrid-spatial-candidate-screen.md)
Target incompatible package line: `0.2.0`
Research baseline: `fenestra-research` commit
`176c42139776ed9f1ef879cd135bddadaf12a9da`

## Objective

Fenestra must support layout-managed and freely positioned two-dimensional
content as optional peer capabilities. Neither mode is the root model for every
interface. An entire interface or any subtree may use either mode, and the two
modes may contain one another recursively.

The work unit must prove all four compositions:

1. layout content containing layout content;
2. free content containing free content;
3. a freely placed object hosting a layout island;
4. a layout participant hosting freely placed content.

The logical ownership tree remains authoritative for identity, state, keyed
structure, and component lifetime. Spatial parentage and layout participation
are derived responsibilities that may coincide with logical ownership but are
not defined by it. The immutable presentation scene is an output, not a source
of logical or layout truth.

This plan executes the hybrid spatial subset of the original
[EXP-0008 advanced 2D graphics corpus](https://github.com/edziry/fenestra-research/blob/176c42139776ed9f1ef879cd135bddadaf12a9da/init/experiments/EXP-0008-rich-2d-graphics.md).
The existing `exp-0008-layout-conformance` probe remains the versioned layout
prerequisite produced by WU-0011. WU-0013 does not reinterpret that completed
artifact as a full EXP-0008 result.

## Current boundary

The current runtime already publishes one atomic generation containing
computed style, geometry, semantics, hit records, and scene records. It retains
authored order, reverse-order hit testing, typed invalidation, rollback, and an
injectable candidate-neutral layout engine.

The current rectangle projection is intentionally insufficient for WU-0013:

- every logical node is mapped into one layout traversal;
- geometry stores one axis-aligned bounds rectangle and its intersected clip;
- hit testing reuses that clipped rectangle;
- paint reuses that clipped rectangle and one color;
- the native probe obtains those records through a testkit observation;
- the IR value vocabulary has no spatial, transform, shape, clip, or resource
  types.

This coupling is the root gap. WU-0013 replaces it with explicit parallel
contracts rather than adding an absolute-position escape to the stack engine.

## Ownership model

The target flow is:

```text
logical tree plus typed values
  -> spatial input and pass-local keys
  -> layout islands plus free-placement dependency graph
  -> immutable geometry snapshot
       -> paint projection
       -> hit projection
       -> semantic geometry projection
  -> immutable presentation frame
  -> disposable native presenter adapter
```

The owned responsibilities are:

- logical runtime: stable identity, state, property storage, keyed structure,
  transaction, invalidation, generation, and publication;
- layout engine: solve only one supplied island in its local logical extent;
- spatial boundary: placement, island orchestration, anchors, transforms,
  clips, ordering, validation, and immutable geometry output;
- paint projection: visual primitives and brushes in local geometry;
- hit projection: independently authored input shapes and policies;
- semantic projection: independently authored semantic bounds and metadata;
- presenter: consume one immutable paint frame without reading logical state,
  layout input, testkit data, or candidate-specific authoring records;
- renderer adapter: convert Fenestra-owned primitives and resources into one
  candidate API without exporting candidate types upstream.

Paint, hit, and semantics may share spatial node keys, transforms, and clips.
They never derive correctness from one another or from rasterized pixels.

## Placement and layout islands

One spatial edge has one closed placement source:

- `Root`: the root local coordinate space;
- `Layout`: the child participates in its containing layout island;
- `Free`: the child receives a size and a free-placement specification.

Maximal connected `Layout` edges form derived layout islands. A `Free` edge
breaks an island; a later `Layout` edge starts another island below that free
object. Island identifiers are pass-local derived data, not authored persistent
identity.

Every node may both receive placement from its incoming edge and host a mixed
set of outgoing `Layout` and `Free` edges. Layout siblings consume layout space;
free siblings do not. A node's optional layout slot, local extent, visual
geometry, hit shape, and semantic bounds remain separate values.

The first reference resolver uses explicit sizes. Intrinsic and auto sizing may
enter only after a measurement port and cycle rules are versioned. Paint bounds,
renderer output, or resource readiness must never feed layout implicitly.

## Free placement and computed dependencies

Free placement combines:

- an explicit local size;
- one self anchor;
- a target anchor on the parent, viewport, or another spatial node;
- a typed offset supplied by resolved properties;
- a local transform and transform origin.

The spatial boundary owns a closed dependency graph, not a general expression
language. Runtime property computation occurs before spatial resolution. Island
host dependencies and free anchor references form one bounded directed graph.
A stable ordinal Kahn traversal accepts acyclic forward references and rejects
cycles before invoking any layout engine or publishing partial output.

Anchors read unpainted geometry and never visual pixels. Transforms do not feed
back into layout. The exact coordinate and transform arithmetic, rounding,
singularity, overflow, and inverse-hit rules must be frozen in the companion
reference contract before the first behavior GREEN.

## Independent geometry

The spatial model must support independently:

- an optional layout slot reserved in a parent island;
- local visual shapes and their un-clipped paint bounds;
- local hit shapes and input policy;
- local semantic bounds;
- a local transform composed into a scene-logical world transform;
- a separate ordered clip chain;
- stable painter and topmost-hit ordering.

An organizational wrapper may reserve layout space while producing no paint,
hit, or semantic item. A painted object may accept no input. A hit shape may be
larger, smaller, or topologically different from its paint shape. Visual
overflow is retained unless an explicit clip removes it.

The required bounded shape vocabulary covers at least rectangle, circle,
polygon, and path. Translate, rotate, and scale are required transform
operations. Path verbs, fill semantics, transform precision, and hit-test
flattening or analytic rules must be candidate-neutral and independently
tested before a renderer is admitted.

## Package and dependency direction

WU-0013 may add an unpublished `fenestra-ui-spatial` package only after its API
RED exists. The intended normal dependency direction is:

```text
fenestra-ui-layout
        ^
fenestra-ui-spatial
        ^
fenestra-ui-runtime <- fenestra-ui-testkit
        ^
native and EXP-0008 probes
```

`fenestra-ui-ir` remains independent from runtime, layout, spatial, renderer,
and testkit types. Authoring depends on IR. Runtime maps validated IR and its
private logical identities into pass-local spatial keys. The native presenter
may depend on Fenestra-owned presentation types but not on testkit, layout
engines, the authoring compiler, or a disposable renderer candidate.

Candidate geometry, path, raster, GPU, image, or spatial-index dependencies
remain probe-only until their exact version, features, active and lock-only
closure, license, MSRV, unsafe and build surface, native requirements, and
replacement cost are recorded. A candidate adapter cannot become the contract.

## Versioning

The full work intentionally changes closed IR value and format vocabularies,
observable geometry, hit, paint, and presentation behavior, and internal exact
dependencies. Under the ratified pre-1.0 policy, every workspace package moves
from `0.1.1` to `0.2.0` in lockstep. No compatibility shim or dual V1/V2 runtime
path is required.

Persisted authoring, IR, trace, and artifact formats advance their own explicit
versions. Unsupported old formats fail with closed diagnostics. Historical
WU-0010 and WU-0011 evidence remains versioned; any lock hash it intentionally
records is refreshed without changing its semantic or runtime claims.

## TDD sequence

1. Planning gate: freeze the reference contract, candidate screen, ownership,
   validation priority, limits, corpus, artifacts, version decision, and
   WU-0012 audit.
2. Workspace RED/GREEN: add the version `0.2.0` package boundary, exact internal
   dependency, private prototype export, and package/version guards.
3. Spatial API RED/GREEN: values, keys, nodes, edges, shapes, transforms,
   clips, paint, hit, semantic geometry, limits, errors, privacy, and auto
   traits; no resolver behavior yet.
4. Validation RED/GREEN: global record limits, dense keys, spatial topology,
   island derivation, membership, anchor references, dependency cycles, scalar
   domains, path grammar, transform validity, ordering, and typed locations.
5. Reference resolver RED/GREEN: all-layout, all-free, free-to-layout,
   layout-to-free, forward anchors, nested transforms, explicit clips, visual
   overflow, and independent paint, hit, and semantic geometry.
6. Runtime RED/GREEN: pass-local identity mapping, exact invalidation, one
   immutable generation, engine calls per island, no-op behavior, mutation,
   resize, failure mapping, and exact rollback.
7. Authoring RED/GREEN: new IR and authoring formats, manual raw programs,
   `.fen`, and `ui!` lower independently to byte-identical typed semantics and
   the same runtime behavior.
8. Presentation RED/GREEN: a Fenestra-owned immutable frame, fake presenter,
   CPU reference lane, transformed hit testing independent of pixels, and one
   disposable rich-renderer adapter if its screen passes.
9. Evidence RED/GREEN: independent literal oracle, clean reconstruction,
   mutation and fault controls, bounded canonical artifact, native reference
   result, dependency facts, and Linux plus Windows pure verification.

Every behavior slice first records a failing test whose failure is exclusive to
the missing behavior. RED and GREEN remain separate focused Conventional
Commits. No golden is blessed before the typed model, independent oracle, and
mutation controls pass.

## Canonical acceptance scenarios

The corpus contains at least:

- all-layout: nested row and column stacks under resize;
- all-free: overlapping freely placed objects with explicit order;
- free-to-layout: a freely placed panel containing a responsive layout island;
- layout-to-free: a layout participant containing a freely placed overlay;
- mixed siblings: layout and free children under the same spatial parent;
- transparent wrapper: layout participation with no paint or hit item;
- split geometry: layout slot, overflowing paint, smaller circular hit, and
  distinct semantic bounds;
- nested translate, rotate, and scale with transformed clip and inverse hit;
- polygon and path paint and hit cases, including misses inside their AABB;
- anchor forward reference, parent and viewport anchors, and a rejected cycle;
- keyed insert, move, update, and remove plus logical resize;
- injected validation, layout, spatial, paint, hit, and presentation failures
  preserving the prior generation and allocation.

Manual input, `.fen`, and `ui!` lanes produce the same normalized projections.
Reference full reconstruction is authoritative. Candidate lanes are compared
field by field and never generate expected values for the oracle.

## WU-0012 exclusion

WU-0012 remains deferred. WU-0013 consumes one present logical viewport and
owns no density, physical pixel, safe-area, orientation, keyboard inset,
surface epoch, platform handle, lifecycle, background, memory-pressure,
Android, iOS, or multi-scene vocabulary.

Zero extent remains present degenerate geometry, never suspension or surface
absence. A renderer adapter performs logical-to-physical conversion outside the
spatial contract.

Stop WU-0013 and reevaluate WU-0012 only if correctness requires distinguishing
absent presentation from zero geometry, using surface or lifecycle state in
spatial resolution, purging a cache to obtain correct output, or tying geometry
identity to a platform surface epoch. No such trigger is present at entry.

## Exit and nonclaims

WU-0013 exits only when the three primary modes and both nesting directions are
implemented through manual, `.fen`, and `ui!` authoring; geometry, paint, hit,
and semantic bounds remain independent; transforms, clips, ordering, anchors,
paths, transactions, and presentation pass their independent oracles; evidence
is bounded and reproducible on Linux and Windows; and the branch is clean and
ready for an authorized squash merge.

The result does not by itself select a final layout or renderer engine, promise
incremental performance, support arbitrary shaders or filters, establish a
public facade, publish crates, define a stable API or MSRV, or claim mobile,
interactive Win32, accessibility-platform, GPU, or product support.
