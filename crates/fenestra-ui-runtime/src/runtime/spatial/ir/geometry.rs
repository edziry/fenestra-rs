use fenestra_ui_ir::prototype::{
    SourceSpan, SpatialClipAddressV2, SpatialClipSymbolV2, SpatialFillRuleV2 as IrFillRule,
    SpatialPathVerbRecipeV2, SpatialShapeGeometryV2 as IrShapeGeometry, SpatialShapeSymbolV2,
};
use fenestra_ui_spatial::prototype::{
    SpatialClipFieldV2 as ClipField, SpatialClipKeyV2, SpatialClipV2, SpatialFillRuleV2,
    SpatialPathFieldV2 as PathField, SpatialPathKeyV2, SpatialPathV2,
    SpatialPathVerbFieldV2 as VerbField, SpatialPathVerbV2, SpatialPointV2,
    SpatialPolygonPointFieldV2 as PointField, SpatialShapeFieldV2 as ShapeField,
    SpatialShapeGeometryV2, SpatialShapeKeyV2, SpatialShapeV2,
};

use super::super::error::{RuntimeSpatialIrErrorKindV2, RuntimeSpatialIrErrorV2};
use super::super::view::RuntimeSpatialBuildViewV2;
use super::bindings;
use super::model::{ExpandedSpatialNode, LiveProgram};
use super::provenance::FieldSpans;

pub(super) struct GeometryTables {
    pub(super) polygon_points: Vec<SpatialPointV2>,
    pub(super) path_verbs: Vec<SpatialPathVerbV2>,
    pub(super) paths: Vec<SpatialPathV2>,
    pub(super) shapes: Vec<SpatialShapeV2>,
    pub(super) clips: Vec<SpatialClipV2>,
    pub(super) polygon_point_provenance: Vec<FieldSpans<PointField>>,
    pub(super) path_verb_provenance: Vec<FieldSpans<VerbField>>,
    pub(super) path_provenance: Vec<FieldSpans<PathField>>,
    pub(super) shape_provenance: Vec<FieldSpans<ShapeField>>,
    pub(super) clip_provenance: Vec<FieldSpans<ClipField>>,
    owner_keys: Vec<OwnerGeometryKeys>,
}

#[derive(Default)]
struct OwnerGeometryKeys {
    shapes: Vec<(SpatialShapeSymbolV2, SpatialShapeKeyV2)>,
    clips: Vec<(SpatialClipSymbolV2, SpatialClipKeyV2)>,
}

impl GeometryTables {
    pub(super) fn shape_key(
        &self,
        owner_key: u32,
        symbol: SpatialShapeSymbolV2,
    ) -> Option<SpatialShapeKeyV2> {
        self.owner_keys
            .get(usize::try_from(owner_key).ok()?)?
            .shapes
            .iter()
            .find_map(|(candidate, key)| (*candidate == symbol).then_some(*key))
    }

    pub(super) fn clip_key(
        &self,
        live: &LiveProgram<'_>,
        source: &ExpandedSpatialNode<'_>,
        address: SpatialClipAddressV2,
    ) -> Option<SpatialClipKeyV2> {
        let owner = live.resolve_node(source.context(), *address.owner().value())?;
        self.owner_keys
            .get(usize::try_from(owner.key()).ok()?)?
            .clips
            .iter()
            .find_map(|(candidate, key)| (*candidate == *address.clip().value()).then_some(*key))
    }
}

pub(super) fn materialize(
    live: &LiveProgram<'_>,
    view: RuntimeSpatialBuildViewV2<'_>,
) -> Result<GeometryTables, RuntimeSpatialIrErrorV2> {
    let mut tables = GeometryTables {
        polygon_points: Vec::new(),
        path_verbs: Vec::new(),
        paths: Vec::new(),
        shapes: Vec::new(),
        clips: Vec::new(),
        polygon_point_provenance: Vec::new(),
        path_verb_provenance: Vec::new(),
        path_provenance: Vec::new(),
        shape_provenance: Vec::new(),
        clip_provenance: Vec::new(),
        owner_keys: (0..=live.expanded().len())
            .map(|_| OwnerGeometryKeys::default())
            .collect(),
    };

    for expanded in live.expanded() {
        materialize_shapes(&mut tables, expanded, view)?;
    }
    for expanded in live.expanded() {
        materialize_clips(&mut tables, live, expanded)?;
    }
    Ok(tables)
}

fn materialize_shapes(
    tables: &mut GeometryTables,
    expanded: &ExpandedSpatialNode<'_>,
    view: RuntimeSpatialBuildViewV2<'_>,
) -> Result<(), RuntimeSpatialIrErrorV2> {
    for shape in expanded.declaration().shapes() {
        let shape_key = key(tables.shapes.len());
        let geometry = match shape.geometry() {
            IrShapeGeometry::Rect {
                origin,
                width,
                height,
            } => SpatialShapeGeometryV2::Rect {
                origin: bindings::point(*origin, expanded.logical(), view)?,
                width: bindings::scalar(*width, expanded.logical(), view)?,
                height: bindings::scalar(*height, expanded.logical(), view)?,
            },
            IrShapeGeometry::Circle { center, radius } => SpatialShapeGeometryV2::Circle {
                center: bindings::point(*center, expanded.logical(), view)?,
                radius: bindings::scalar(*radius, expanded.logical(), view)?,
            },
            IrShapeGeometry::Polygon { points } => {
                let point_start = key(tables.polygon_points.len());
                let point_length = key(points.len());
                for point in points {
                    tables.polygon_points.push(bindings::point(
                        point.point(),
                        expanded.logical(),
                        view,
                    )?);
                    tables.polygon_point_provenance.push(FieldSpans::new(
                        point.span(),
                        vec![
                            (PointField::X, point.point().x().span()),
                            (PointField::Y, point.point().y().span()),
                        ],
                    ));
                }
                SpatialShapeGeometryV2::Polygon {
                    point_start,
                    point_length,
                }
            }
            IrShapeGeometry::Path { verbs } => {
                let path_key = key(tables.paths.len());
                let verb_start = key(tables.path_verbs.len());
                let verb_length = key(verbs.len());
                for verb in verbs {
                    tables
                        .path_verbs
                        .push(path_verb(verb, expanded.logical(), view)?);
                    tables.path_verb_provenance.push(verb_provenance(verb));
                }
                tables.paths.push(SpatialPathV2::new(
                    SpatialPathKeyV2::new(path_key),
                    verb_start,
                    verb_length,
                ));
                tables.path_provenance.push(FieldSpans::new(
                    shape.span(),
                    vec![
                        (PathField::Key, shape.span()),
                        (PathField::VerbStart, shape.span()),
                        (PathField::VerbLength, shape.span()),
                    ],
                ));
                SpatialShapeGeometryV2::Path {
                    path: SpatialPathKeyV2::new(path_key),
                }
            }
        };
        tables.shapes.push(SpatialShapeV2::new(
            SpatialShapeKeyV2::new(shape_key),
            fenestra_ui_spatial::prototype::SpatialNodeKeyV2::new(expanded.key()),
            geometry,
        ));
        tables
            .owner_keys
            .get_mut(
                usize::try_from(expanded.key()).expect("representation preflight guards node keys"),
            )
            .ok_or_else(|| invariant(shape.span()))?
            .shapes
            .push((*shape.symbol().value(), SpatialShapeKeyV2::new(shape_key)));
        tables.shape_provenance.push(shape_provenance(shape));
    }
    Ok(())
}

fn materialize_clips(
    tables: &mut GeometryTables,
    live: &LiveProgram<'_>,
    expanded: &ExpandedSpatialNode<'_>,
) -> Result<(), RuntimeSpatialIrErrorV2> {
    for clip in expanded.declaration().clips() {
        let clip_key = key(tables.clips.len());
        let shape = tables
            .shape_key(expanded.key(), *clip.shape().value())
            .ok_or_else(|| invariant(clip.shape().span()))?;
        let parent = clip
            .parent()
            .map(|address| {
                tables
                    .clip_key(live, expanded, address)
                    .ok_or_else(|| invariant(address.clip().span()))
            })
            .transpose()?;
        tables.clips.push(SpatialClipV2::new(
            SpatialClipKeyV2::new(clip_key),
            fenestra_ui_spatial::prototype::SpatialNodeKeyV2::new(expanded.key()),
            parent,
            shape,
            fill_rule(clip.fill_rule()),
        ));
        tables
            .owner_keys
            .get_mut(
                usize::try_from(expanded.key()).expect("representation preflight guards node keys"),
            )
            .ok_or_else(|| invariant(clip.span()))?
            .clips
            .push((*clip.symbol().value(), SpatialClipKeyV2::new(clip_key)));
        tables.clip_provenance.push(FieldSpans::new(
            clip.span(),
            vec![
                (ClipField::Key, clip.span()),
                (ClipField::Owner, clip.span()),
                (
                    ClipField::Parent,
                    clip.parent()
                        .map_or(clip.span(), |address| address.clip().span()),
                ),
                (ClipField::Shape, clip.shape().span()),
                (ClipField::FillRule, clip.span()),
            ],
        ));
    }
    Ok(())
}

fn path_verb(
    verb: &SpatialPathVerbRecipeV2,
    owner: crate::logical_tree::NodeId,
    view: RuntimeSpatialBuildViewV2<'_>,
) -> Result<SpatialPathVerbV2, RuntimeSpatialIrErrorV2> {
    Ok(match verb {
        SpatialPathVerbRecipeV2::MoveTo { to, .. } => SpatialPathVerbV2::MoveTo {
            to: bindings::point(*to, owner, view)?,
        },
        SpatialPathVerbRecipeV2::LineTo { to, .. } => SpatialPathVerbV2::LineTo {
            to: bindings::point(*to, owner, view)?,
        },
        SpatialPathVerbRecipeV2::QuadraticTo { control, to, .. } => {
            SpatialPathVerbV2::QuadraticTo {
                control: bindings::point(*control, owner, view)?,
                to: bindings::point(*to, owner, view)?,
            }
        }
        SpatialPathVerbRecipeV2::CubicTo {
            control1,
            control2,
            to,
            ..
        } => SpatialPathVerbV2::CubicTo {
            control1: bindings::point(*control1, owner, view)?,
            control2: bindings::point(*control2, owner, view)?,
            to: bindings::point(*to, owner, view)?,
        },
        SpatialPathVerbRecipeV2::Close { .. } => SpatialPathVerbV2::Close,
    })
}

fn verb_provenance(verb: &SpatialPathVerbRecipeV2) -> FieldSpans<VerbField> {
    let mut fields = vec![(VerbField::Kind, verb.span())];
    match verb {
        SpatialPathVerbRecipeV2::MoveTo { to, .. } | SpatialPathVerbRecipeV2::LineTo { to, .. } => {
            fields.extend([
                (VerbField::ToX, to.x().span()),
                (VerbField::ToY, to.y().span()),
            ]);
        }
        SpatialPathVerbRecipeV2::QuadraticTo { control, to, .. } => {
            fields.extend([
                (VerbField::ControlX, control.x().span()),
                (VerbField::ControlY, control.y().span()),
                (VerbField::ToX, to.x().span()),
                (VerbField::ToY, to.y().span()),
            ]);
        }
        SpatialPathVerbRecipeV2::CubicTo {
            control1,
            control2,
            to,
            ..
        } => {
            fields.extend([
                (VerbField::Control1X, control1.x().span()),
                (VerbField::Control1Y, control1.y().span()),
                (VerbField::Control2X, control2.x().span()),
                (VerbField::Control2Y, control2.y().span()),
                (VerbField::ToX, to.x().span()),
                (VerbField::ToY, to.y().span()),
            ]);
        }
        SpatialPathVerbRecipeV2::Close { .. } => {}
    }
    FieldSpans::new(verb.span(), fields)
}

fn shape_provenance(
    shape: &fenestra_ui_ir::prototype::SpatialShapeDeclarationV2,
) -> FieldSpans<ShapeField> {
    let record = shape.span();
    let mut fields = vec![
        (ShapeField::Key, record),
        (ShapeField::Owner, record),
        (ShapeField::Kind, record),
    ];
    match shape.geometry() {
        IrShapeGeometry::Rect {
            origin,
            width,
            height,
        } => fields.extend([
            (ShapeField::RectX, origin.x().span()),
            (ShapeField::RectY, origin.y().span()),
            (ShapeField::RectWidth, width.span()),
            (ShapeField::RectHeight, height.span()),
        ]),
        IrShapeGeometry::Circle { center, radius } => fields.extend([
            (ShapeField::CircleCenterX, center.x().span()),
            (ShapeField::CircleCenterY, center.y().span()),
            (ShapeField::CircleRadius, radius.span()),
        ]),
        IrShapeGeometry::Polygon { .. } => fields.extend([
            (ShapeField::PolygonPointStart, record),
            (ShapeField::PolygonPointLength, record),
        ]),
        IrShapeGeometry::Path { .. } => fields.push((ShapeField::Path, record)),
    }
    FieldSpans::new(record, fields)
}

pub(super) fn fill_rule(value: IrFillRule) -> SpatialFillRuleV2 {
    match value {
        IrFillRule::NonZero => SpatialFillRuleV2::NonZero,
        IrFillRule::EvenOdd => SpatialFillRuleV2::EvenOdd,
    }
}

fn key(value: usize) -> u32 {
    u32::try_from(value).expect("representation preflight guards raw keys and ranges")
}

fn invariant(span: SourceSpan) -> RuntimeSpatialIrErrorV2 {
    RuntimeSpatialIrErrorV2::new(RuntimeSpatialIrErrorKindV2::InvariantViolation, span)
}
