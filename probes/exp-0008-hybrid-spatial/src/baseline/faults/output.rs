use fenestra_ui_layout::prototype::ReferenceStackEngineV1;
use fenestra_ui_spatial::prototype::{
    SpatialBrushContentV2, SpatialBrushKeyV2, SpatialBrushV2, SpatialClipKeyV2, SpatialClipV2,
    SpatialFillRuleV2, SpatialGeometryOutputRecordV2, SpatialHitOutputRecordV2, SpatialHitV2,
    SpatialInputPolicyV2, SpatialNodeKeyV2, SpatialOutputAabbV2, SpatialOutputErrorKindV2,
    SpatialOutputV2, SpatialPaintContentV2, SpatialPaintOutputRecordV2,
    SpatialPaintOutputReferenceV2, SpatialPaintV2, SpatialRgba8V2, SpatialScalarV2,
    SpatialSemanticGeometryV2, SpatialSemanticOutputRecordV2, SpatialShapeKeyV2, SpatialViewportV2,
    prepare_spatial_v2, resolve_spatial_v2, validate_spatial_output_v2,
};

use super::input::{Parts, fill, free, identity, root, valid_shape};

struct Tables {
    geometry: Vec<SpatialGeometryOutputRecordV2>,
    clips: Vec<fenestra_ui_spatial::prototype::SpatialClipOutputRecordV2>,
    paints: Vec<SpatialPaintOutputRecordV2>,
    hits: Vec<SpatialHitOutputRecordV2>,
    semantics: Vec<SpatialSemanticOutputRecordV2>,
}

impl Tables {
    fn view(&self) -> SpatialOutputV2<'_> {
        SpatialOutputV2::new(
            &self.geometry,
            &self.clips,
            &self.paints,
            &self.hits,
            &self.semantics,
        )
    }
}

pub(super) fn output_faults() -> [SpatialOutputErrorKindV2; 10] {
    for expected in SpatialOutputErrorKindV2::ALL {
        let (prepared, mut tables) = fresh_case();
        mutate(&mut tables, expected);
        let actual = validate_spatial_output_v2(prepared, tables.view())
            .err()
            .expect("malformed output table must be rejected");
        assert_eq!(
            actual.kind(),
            fenestra_ui_spatial::prototype::SpatialResolveErrorKindV2::Output(expected)
        );
    }
    SpatialOutputErrorKindV2::ALL
}

fn fresh_case() -> (fenestra_ui_spatial::prototype::PreparedSpatialV2, Tables) {
    let source = rich_owned();
    let limits = fenestra_ui_spatial::prototype::REGISTERED_SPATIAL_LIMITS_V2;
    let prepared = prepare_spatial_v2(&ReferenceStackEngineV1::new(), source.clone(), limits)
        .expect("fault fixture prepares");
    let reference = resolve_spatial_v2(&ReferenceStackEngineV1::new(), source, limits)
        .expect("fault fixture resolves");
    let output = reference.output();
    (
        prepared,
        Tables {
            geometry: output.geometry().to_vec(),
            clips: output.clips().to_vec(),
            paints: output.paints().to_vec(),
            hits: output.hits().to_vec(),
            semantics: output.semantics().to_vec(),
        },
    )
}

fn rich_owned() -> std::sync::Arc<fenestra_ui_spatial::prototype::SpatialOwnedInputV2> {
    let mut parts = Parts {
        nodes: vec![
            root(),
            free(
                1,
                0,
                fenestra_ui_spatial::prototype::SpatialAnchorTargetV2::Viewport,
                identity(),
            ),
        ],
        ..Parts::default()
    };
    parts.shapes.push(valid_shape());
    parts.clips.push(SpatialClipV2::new(
        SpatialClipKeyV2::new(0),
        SpatialNodeKeyV2::new(1),
        None,
        SpatialShapeKeyV2::new(0),
        SpatialFillRuleV2::NonZero,
    ));
    parts.brushes.push(SpatialBrushV2::new(
        SpatialBrushKeyV2::new(0),
        SpatialBrushContentV2::Solid {
            color: SpatialRgba8V2::new(32, 64, 96, 255),
        },
    ));
    parts.paints.push(SpatialPaintV2::new(
        SpatialNodeKeyV2::new(1),
        0,
        SpatialPaintContentV2::CoveragePaint {
            coverage: fill(0),
            brush: SpatialBrushKeyV2::new(0),
            opacity: 255,
            clip: Some(SpatialClipKeyV2::new(0)),
        },
    ));
    parts.hits.push(SpatialHitV2::new(
        SpatialNodeKeyV2::new(1),
        0,
        fill(0),
        Some(SpatialClipKeyV2::new(0)),
        SpatialInputPolicyV2::Accept,
    ));
    parts.semantics.push(SpatialSemanticGeometryV2::new(
        SpatialNodeKeyV2::new(1),
        0,
        SpatialShapeKeyV2::new(0),
        SpatialFillRuleV2::NonZero,
        Some(SpatialClipKeyV2::new(0)),
    ));
    parts.owned(SpatialViewportV2::new(8, 8))
}

fn mutate(tables: &mut Tables, kind: SpatialOutputErrorKindV2) {
    match kind {
        SpatialOutputErrorKindV2::RecordCountMismatch => {
            let _ = tables.geometry.pop();
        }
        SpatialOutputErrorKindV2::KeyMismatch => mutate_geometry(tables, |row| row.key = 9),
        SpatialOutputErrorKindV2::ScalarOutOfDomain => {
            mutate_geometry(tables, |row| row.x = SpatialScalarV2::MAX_RAW + 1);
        }
        SpatialOutputErrorKindV2::NegativeBaseExtent(
            fenestra_ui_spatial::prototype::SpatialExtentV2::Width,
        ) => mutate_geometry(tables, |row| row.width = -SpatialScalarV2::SCALE),
        SpatialOutputErrorKindV2::NegativeBaseExtent(
            fenestra_ui_spatial::prototype::SpatialExtentV2::Height,
        ) => mutate_geometry(tables, |row| row.height = -SpatialScalarV2::SCALE),
        SpatialOutputErrorKindV2::InvalidWorldDeterminant => {
            mutate_geometry(tables, |row| row.determinant += 1);
        }
        SpatialOutputErrorKindV2::InvalidAabb => {
            mutate_geometry(tables, |row| row.aabb[0] += 1);
        }
        SpatialOutputErrorKindV2::InvalidClipChain => {
            let row = tables.clips[0];
            tables.clips[0] = fenestra_ui_spatial::prototype::SpatialClipOutputRecordV2::new(
                row.key(),
                row.world_from_local(),
                row.world_determinant(),
                row.primitive_world_aabb(),
                row.owner(),
                Some(SpatialClipKeyV2::new(0)),
                row.shape(),
            );
        }
        SpatialOutputErrorKindV2::InvalidProjectionOrder => mutate_paint(tables, true),
        SpatialOutputErrorKindV2::InvalidReference => mutate_paint(tables, false),
    }
}

struct GeometryValues {
    key: u32,
    x: i64,
    y: i64,
    width: i64,
    height: i64,
    determinant: i128,
    aabb: [i64; 4],
}

fn mutate_geometry(tables: &mut Tables, change: impl FnOnce(&mut GeometryValues)) {
    let row = tables.geometry[0];
    let bounds = row.world_aabb();
    let mut values = GeometryValues {
        key: row.key().get(),
        x: row.base_x().raw(),
        y: row.base_y().raw(),
        width: row.base_width().raw(),
        height: row.base_height().raw(),
        determinant: row.world_determinant(),
        aabb: [
            bounds.min_x().raw(),
            bounds.min_y().raw(),
            bounds.max_x().raw(),
            bounds.max_y().raw(),
        ],
    };
    change(&mut values);
    tables.geometry[0] = SpatialGeometryOutputRecordV2::new(
        SpatialNodeKeyV2::new(values.key),
        SpatialScalarV2::new(values.x),
        SpatialScalarV2::new(values.y),
        SpatialScalarV2::new(values.width),
        SpatialScalarV2::new(values.height),
        row.world_from_local(),
        values.determinant,
        SpatialOutputAabbV2::new(
            bounds.is_empty(),
            SpatialScalarV2::new(values.aabb[0]),
            SpatialScalarV2::new(values.aabb[1]),
            SpatialScalarV2::new(values.aabb[2]),
            SpatialScalarV2::new(values.aabb[3]),
        ),
    );
}

fn mutate_paint(tables: &mut Tables, projection: bool) {
    let row = tables.paints[0];
    let reference = if projection {
        row.reference()
    } else {
        SpatialPaintOutputReferenceV2::Coverage {
            shape: SpatialShapeKeyV2::new(0),
            brush: SpatialBrushKeyV2::new(9),
        }
    };
    tables.paints[0] = SpatialPaintOutputRecordV2::new(
        row.key(),
        row.world_from_local(),
        row.world_determinant(),
        row.world_aabb(),
        row.owner(),
        reference,
        row.clip(),
        if projection {
            u32::MAX
        } else {
            row.stack_ordinal()
        },
        row.item_ordinal(),
    );
}
