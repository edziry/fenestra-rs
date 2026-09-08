# WU-0016 inspector and authoring loop

Status: executable authoring and native example slice implemented
Branch: `feat/inspector-authoring-loop`

## Objective

Continue the first usable application by making the layout inspector consume a
configurable compiled-program boundary while dogfooding both typed authoring
frontends. The slice keeps the existing native artifact stable and exposes
only bounded diagnostics needed to inspect the application state.

## Boundary

`LayoutInspector::new` remains the default application entry point. It uses the
format-2 `.fen` fixture compiled by `build.rs`. The build script also expands
the equivalent `ui!` fixture and rejects the build if the schema, construction,
style, or spatial programs differ.

`LayoutInspector::from_programs` accepts another raw program quadruple after
the same application validation boundary. This is an application seam for
future configurable content; it does not parse authoring syntax at runtime and
does not make the experimental IR public or stable.

The default inspector exposes bounded diagnostics for:

- `.fen`, `ui!`, and generated Rust byte counts;
- build-time frontend equivalence;
- the latest deterministic frame;
- the selected logical node; and
- the selected RGBA8 tone property.

Programs supplied without source metadata report that metadata as unknown
instead of attributing the default fixture to external content.

## Acceptance

The pure application tests must prove:

1. the default `.fen` source remains deterministic;
2. a configurable program quadruple can initialize the same application core;
3. the default build records `.fen`/`ui!` equivalence;
4. selection diagnostics expose the committed tone; and
5. the existing keyed mutation, resize, and native evidence contracts remain
   unchanged;
6. the small `examples/hello-panel.fen` and `.ui` sources compile equivalently
   and initialize the same application and native presenter;
7. the panel renders expected pixels, selects only the clicked card, inserts
   another keyed card and resizes without losing selection;
8. repeated native Space presses use distinct keys, idle redraw does not
   schedule itself, and zero-size windows skip presentation; and
9. `fenestra-check` reports physical source locations and typed causes under
   the existing format-2 compiler limits.

The command-line smoke output includes the authoring byte counts, equivalence
flag, and selected tone as bounded diagnostic facts.

## Executable development loop

The build compiles both the conformance scene and the small panel example.
`run_native_with_inspector` and `run_native_smoke_with_inspector` preserve the
supplied compiled content, so the example does not introduce another runtime
or presenter. Frame observations live in a separate module without changing
their public accessors or the registered native artifact.

The host-only `fenestra-check` executable reads at most the registered source
bound plus one byte, validates and generates canonical Rust, and optionally
writes that Rust to stdout. It reports one-based lines and byte columns with
zero-based byte ranges, including malformed UTF-8 and CRLF input. It displays
the public typed diagnostic kind without changing the compiler's deliberately
redacted `Display` or macro contracts.

The native shell schedules redraw only for supported interactions and valid
resize events. Key allocation begins at 30 and skips occupied keys in steps
of 10; the first registered insertion and evidence protocol remain unchanged.

## Nonclaims

This slice does not define final `.fen` or `ui!` syntax, runtime parsing,
reload, a public authoring API, a node-property editor, a renderer choice, or
broad platform support. EXP-0007 remains an open feasibility experiment.
