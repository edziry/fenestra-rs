# WU-0015 first usable Fenestra application

Status: implementation in progress
Branch: `feat/first-usable-application`

## Objective

Build one small application that dogfoods the framework end to end. The first
vertical is a spatial layout inspector: authored structure and style come from
the format-2 `.fen` boundary, the runtime owns committed state, and a native
shell will present the resulting paint frame.

The first implementation reuses the frozen WU-0013 format-2 fixture as its
source while the application boundary is being exercised. This keeps the
authored grammar and spatial coverage stable while application-specific
friction is measured. The fixture already covers nested components, direct
property bindings, a keyed region, input policy, layout/free nesting, images,
clips, transforms, and viewport changes.

## User task

1. Build the application from a clean checkout.
2. Start it and observe the authored spatial scene.
3. Move the pointer over the scene and press to select the hit node.
4. Add a keyed tile and confirm the committed key order remains deterministic.
5. Resize the viewport and observe a new committed frame.
6. Close the application normally.

The deterministic application core currently exercises steps 2 through 5. The
Win32 presentation shell and its operator protocol are the next implementation
slice.

## Application boundary

`apps/fenestra-layout-inspector` owns only application state and user actions.
It validates the generated `.fen` program through the existing IR boundary,
constructs the spatial runtime, maps exact hit results to logical nodes, and
commits selection, keyed insertion, and resize as runtime transactions.

The application does not add renderer, GPU, window, or candidate types to the
framework crates. Native presentation remains an adapter around an immutable
runtime paint frame.

## Deterministic observations

Every observed frame records:

- committed generation and viewport;
- live logical node count and keyed tile order;
- image, paint, hit, and semantic output counts;
- reference-raster byte count; and
- hover and selection state.

The observation is an application diagnostic, not a product performance claim.

## Limits and nonclaims

The first cut keeps explicit runtime and reference-raster bounds. It does not
claim broad Windows support, input latency, accessibility, text, IME,
packaging, multi-window behavior, or permanent selection of WU-0014's GPU
candidate dependencies.

## TDD order

1. Assert the authored source boundary and initial deterministic observation.
2. Assert pointer hit-to-selection and keyed insertion behavior.
3. Assert resize publication and rollback on rejected transactions.
4. Add the native shell only after the application core remains independently
   testable.
