# Text and input foundation

Work unit: WU-0018
Status: implemented and locally verified; full product gates remain open

## Problem and scope

The [typed application API](typed-application-api.md) can construct and update
native panels, but it cannot yet express a text control. Applications also need
owned keyboard events and an editing model that does not split a Unicode
grapheme. Text shaping, focus, editing and native composition are separate
responsibilities; one successful shaping fixture does not qualify them all.

This increment introduces a bounded public `TextBuffer`, an owned native input
bridge, committed element bounds and an isolated comparison of Parley and
cosmic-text. A native text-pad probe exercises these seams together. Text
candidate dependencies remain outside the production workspace and facade.
Neither candidate is selected permanently by this work unit.

## Owned editing contract

`TextBuffer` owns UTF-8 text, an inclusive byte budget and a directed selection.
Selection endpoints must be extended grapheme boundaries. Invalid endpoints
or replacements exceeding the byte budget leave the previous state intact.
Deletion and logical previous/next movement operate on whole graphemes,
including combining sequences, emoji sequences and CRLF. Selection direction
is retained; selecting all, collapsing a selection and extending it are
explicit operations. If replacement joins neighboring graphemes, the new
caret advances to the next valid boundary.

```rust
use fenestra_ui::{EditingError, TextBuffer};

fn edit() -> Result<(), EditingError> {
    let mut text = TextBuffer::new("Cafe\u{301}", 128)?;
    text.backspace();
    assert_eq!(text.text(), "Caf");
    text.select_all();
    text.replace_selection("Hello")?;
    Ok(())
}
```

Logical movement is not visual bidi movement. This model does not supply
shaping, line layout, undo history, clipboard, password storage, accessibility,
or a native text control. Those responsibilities require separate contracts
and qualified adapters. Text and selection are not automatically logged.

## Native event contract

The optional native host converts its private winit input into owned Fenestra
key, modifier, key-state, text, focus and IME values. Applications choose their
shortcut policy. Key repeats are observable. Compatibility `SpacePressed`
notifications remain available for the existing inspector and must not be
treated as a second text insertion.

IME is opt-in. Preedit is provisional application state; only commit events
change the editor buffer. Focus loss cancels provisional state and modifier
bookkeeping. The event bridge is a foundation, not evidence that Windows TSF,
IBus, Fcitx, composition candidate placement or reconversion are qualified.

## Geometry and rendering boundary

`Application::bounds` reports an element's geometry from the same committed
spatial snapshot used by paint and hit testing. Bounds are not clipped to the
viewport; consumers must clip output. Invalid updates cannot publish new
bounds. This avoids reconstructing layout independently in an adapter.

The disposable text screen owns measurements, line and glyph positions,
source ranges and white glyph coverage pixels. The native `Raster` remains
premultiplied RGBA8. Candidate font or layout objects do
not cross that seam. Both candidates receive the same versioned font and
corpus. Missing-glyph coverage is reported explicitly, and measurements are
compared rather than assumed identical.

The native probe composes fixed, nonoverlapping text surfaces over a facade
panel using committed bounds. It is not a general text paint implementation
inside the spatial runtime. A later text-view design must preserve painter
order, clipping, transactional content/style updates, font lifetimes, resource
bounds and caret geometry before `.fen` or `ui!` text elements are promoted.

## Acceptance evidence

- Editing rejection is atomic and logical movement stays on grapheme boundaries.
- Native mapping tests cover repeat, release, synthetic events, modifiers,
  compatibility input, focus loss and composition bookkeeping.
- Element bounds follow nested layout and successful updates.
- The candidate screen uses one font/corpus and records correctness gaps.
- The native probe renders readable text and exercises owned editing events.
- Existing inspector, authoring and spatial fixtures remain valid.

The full [completion goal](../project-completion-goal.md) remains active.
