# WU-0013 authoring format-2 reference fixture

Status: frozen for authoring RED/GREEN
Work unit: WU-0013
Grammar: [hybrid spatial authoring format 2](hybrid-spatial-authoring-v2.md)
Sources and limits:
[format-2 authoring sources](hybrid-spatial-authoring-source-v2.md)
Fixture:
[hybrid_spatial_v2.fen](../../crates/fenestra-ui-authoring/tests/fixtures/hybrid_spatial_v2.fen)

## Purpose and authority

The linked `.fen` file is the exact versioned reference source. It ends with
one LF, contains only ASCII grammar characters, and is copied token-for-token
into the `ui!` reference lane. This document records its measured compiler
resources and the semantic branches that it must continue to cover.

`GeneratedRustBytes` remains deferred until the canonical format-2 emitter and
its mutation controls exist. The other 27 reference limits are frozen here for
the parser, resolver, source-map, and IR-lowering cuts.

## Exact source measurements and limits

The abstract-token count follows the shared lexical contract: identifiers,
unsigned decimal tokens, and each punctuation character count once after
whitespace is discarded. Delimiter depth counts braces, brackets, and
parentheses together.

| Authoring category | Observed | Inclusive reference limit |
| --- | ---: | ---: |
| FenSourceBytes | 7,714 | 8,192 |
| Tokens | 1,610 | 2,048 |
| IdentifierBytes | 15 | 15 |
| NestingDepth | 8 | 12 |
| Components | 1 | 1 |
| Properties | 8 | 8 |
| Templates | 7 | 7 |
| Regions | 1 | 1 |
| ChildSlots | 6 | 6 |
| InitialProperties | 19 | 19 |
| InitialKeys | 2 | 2 |
| StyleAssignments | 3 | 3 |
| Images | 1 | 1 |
| ImageBytes | 16 | 16 |
| SpatialNodes | 7 | 7 |
| SpatialFields | 264 | 264 |
| Shapes | 5 | 5 |
| Paths | 1 | 1 |
| PathVerbs | 5 | 5 |
| PolygonPoints | 3 | 3 |
| Brushes | 3 | 3 |
| GradientStops | 3 | 3 |
| Clips | 3 | 3 |
| PaintItems | 4 | 4 |
| HitItems | 4 | 4 |
| SemanticItems | 3 | 3 |
| SourceAnchors | 380 | 512 |
| GeneratedRustBytes | deferred | deferred |

The longest identifier is `linear_gradient`, at 15 bytes. Deliberate
headroom is limited to 478 source bytes, 438 tokens, four delimiter levels,
and 132 source anchors. All other frozen limits are exact authored counts.

The first 27 values of `REFERENCE_AUTHORING_LIMITS_V2`, in
`AuthoringLimitKindV2::ALL` order, are:

```text
[
  8192, 2048, 15, 12,
  1, 8, 7, 1, 6, 19, 2, 3,
  1, 16, 7, 264, 5, 1, 5, 3, 3, 3, 3, 4, 4, 3,
  512,
]
```

The final value is added only after measuring canonical format-2 Rust,
including its required final LF.

## Private construction-validation profile

Format 2 uses these private graph-derived construction bounds:

```text
REGISTERED_TEMPLATE_DEPTH_V2 = 4
REGISTERED_INITIAL_INSTANCES_V2 = 8
```

Depth is root-inclusive. The longest paths are
`root -> stack -> free_host -> free_child` and
`root -> stack -> free_host -> island`. The eight initially expanded logical
instances are root, stack, free_host, free_child, two keyed island members,
anchor_ref, and overlay.

## Spatial field accounting

The exact 264 `SpatialFieldV2<T>` values decompose as follows:

| Owner or table | Fields |
| --- | ---: |
| Viewport container | 5 |
| Image declaration | 4 |
| `scene` base and layout recipes | 21 |
| `scene` shapes | 31 |
| `scene` brushes | 13 |
| `scene` clips | 6 |
| `scene` paint items | 21 |
| `scene` hit items | 9 |
| `scene` semantic items | 4 |
| `stack_node` base and layout recipes | 22 |
| `floating` base and free recipe with node target | 21 |
| `floating_child` base and free recipe with parent target | 20 |
| `tile_node` base, layout, and content recipes | 45 |
| `guide` base and layout recipes | 22 |
| `viewport_layer` base and free recipe with viewport target | 20 |
| Total | 264 |

A top-level node has symbol and template fields. Each nested node adds its
derived parent field. Every node has five container fields and eight canonical
affine-transform fields. Layout adds six dimension fields. Free placement adds
width, height, and two offset fields, plus one field only for an explicit node
target. Content then follows its exact IR constructor fields, including two
fields for every qualified clip address.

## Source-anchor accounting

The exact 380 logical anchors decompose as:

```text
format-1-compatible records     51
spatial records                 65
spatial fields                 264
                               ---
total                          380
```

The 51 compatible records are document 1, schema 1, component 1, properties
8, construction 1, templates 7, initial properties 19, child slots 6, region
1, initial keys 2, style 1, and style assignments 3.

The 65 spatial records are spatial, viewport container, resources, and image
records 4; node, container, placement, and transform records 28; and shapes
5, path verbs 5, polygon points 3, brushes 3, gradient stops 3, clips 3,
paint items 4, hit items 4, and semantic items 3. Each of the 264 spatial
fields has its own `SpatialField` anchor.

## Hybrid composition coverage

| Required composition | Authored relationship |
| --- | --- |
| Layout contains layout | `scene -> stack_node`, `scene -> guide` |
| Free contains free | `floating -> floating_child` |
| Free contains layout | `floating -> tile_node` |
| Layout contains free | `stack_node -> floating`, `guide -> viewport_layer` |

`floating` also mixes a free child with keyed layout children under one
container. The `tiles` region expands `tile_node` twice while preserving the
symbolic subtree and owner-local item ordinals.

## Recipe coverage

| Recipe area | Covered branches |
| --- | --- |
| Container | row, column; literal and property padding and gap |
| Placement | layout; free; viewport, parent, and forward node targets |
| Anchors | start, center, end in horizontal and vertical positions |
| Transform | identity, translate, scale, quarter-turn, affine |
| Shape | rectangle, circle, polygon, path |
| Path | move, line, quadratic, cubic, close |
| Brush | solid and linear gradient; literal and property colors |
| Clip | none, same-owner earlier parent, ancestor parent and terminal clip |
| Coverage | nonzero and even-odd fill; round stroke |
| Paint | coverage paint and normalized image paint |
| Hit | literal accept, literal ignore, and property input policy |
| Semantic | clipped and unclipped independent geometry |
| Binding | literal and property I32, Fixed16, color, and input policy |
| Resource | one valid premultiplied RGBA8 image |

The image is 2 by 2 pixels with stride 8 and 16 bytes. Every encoded color
channel is at most its alpha channel. The gradient has valid endpoint stops
and a distinct start and end.

The forward `floating -> guide` anchor is context-valid because both templates
have an empty structural-region signature. `tile_node` has signature
`[tiles]`; its `floating` spatial parent and `scene` clip owner have empty
signatures, which are valid prefixes. Node declarations remain valid symbolic
preorder, and every same-owner clip parent is declared earlier.
