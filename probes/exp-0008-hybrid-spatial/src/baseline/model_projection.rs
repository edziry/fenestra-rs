use super::literal_types::{PointV2, SceneInputV2};
use super::model::{
    EvidenceFieldV2 as F, EvidenceRecordV2 as R, SpatialEvidenceObservationV2,
    observation_from_records_v2,
};
use super::model_records::source_records_v2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NormalizedAabbV2 {
    pub(crate) empty: bool,
    pub(crate) min_x: i64,
    pub(crate) min_y: i64,
    pub(crate) max_x: i64,
    pub(crate) max_y: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NormalizedGeometryV2 {
    pub(crate) key: u32,
    pub(crate) path: Option<String>,
    pub(crate) base: [i64; 4],
    pub(crate) affine: [i64; 6],
    pub(crate) determinant: i128,
    pub(crate) aabb: NormalizedAabbV2,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NormalizedClipV2 {
    pub(crate) key: u32,
    pub(crate) owner: u32,
    pub(crate) path: Option<String>,
    pub(crate) parent: Option<u32>,
    pub(crate) shape: u32,
    pub(crate) affine: [i64; 6],
    pub(crate) determinant: i128,
    pub(crate) primitive: NormalizedAabbV2,
    pub(crate) effective: NormalizedAabbV2,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NormalizedPaintReferenceV2 {
    Coverage { shape: u32, brush: u32 },
    Image { image: u32 },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NormalizedPaintV2 {
    pub(crate) key: u32,
    pub(crate) owner: u32,
    pub(crate) path: Option<String>,
    pub(crate) affine: [i64; 6],
    pub(crate) determinant: i128,
    pub(crate) aabb: NormalizedAabbV2,
    pub(crate) reference: NormalizedPaintReferenceV2,
    pub(crate) clip: Option<u32>,
    pub(crate) stack: u32,
    pub(crate) item: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NormalizedItemV2 {
    pub(crate) key: u32,
    pub(crate) owner: u32,
    pub(crate) path: Option<String>,
    pub(crate) affine: [i64; 6],
    pub(crate) determinant: i128,
    pub(crate) aabb: NormalizedAabbV2,
    pub(crate) shape: u32,
    pub(crate) clip: Option<u32>,
    pub(crate) stack: u32,
    pub(crate) item: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NormalizedHitV2 {
    pub(crate) key: u32,
    pub(crate) owner: u32,
    pub(crate) path: Option<String>,
    pub(crate) item: u32,
    pub(crate) local: PointV2,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NormalizedQueryV2 {
    pub(crate) scene: PointV2,
    pub(crate) result: Option<NormalizedHitV2>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NormalizedRasterV2 {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) stride: u64,
    pub(crate) bytes: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NormalizedProjectionV2 {
    pub(crate) mapping: Vec<(u32, Option<String>)>,
    pub(crate) geometry: Vec<NormalizedGeometryV2>,
    pub(crate) clips: Vec<NormalizedClipV2>,
    pub(crate) paints: Vec<NormalizedPaintV2>,
    pub(crate) hits: Vec<NormalizedItemV2>,
    pub(crate) semantics: Vec<NormalizedItemV2>,
    pub(crate) queries: Vec<NormalizedQueryV2>,
    pub(crate) raster: NormalizedRasterV2,
}

pub(crate) fn observation_from_projection_v2(
    case: u8,
    step: u8,
    scene: &SceneInputV2,
    projection: &NormalizedProjectionV2,
) -> SpatialEvidenceObservationV2 {
    observation_from_records_v2(
        case,
        step,
        scene.receipt.generation,
        scene.viewport,
        [
            receipt_records(scene),
            mapping_records(&projection.mapping),
            source_records_v2(scene),
            projection.geometry.iter().map(geometry_record).collect(),
            projection.clips.iter().map(clip_record).collect(),
            projection.paints.iter().map(paint_record).collect(),
            projection.hits.iter().map(item_record).collect(),
            projection.semantics.iter().map(item_record).collect(),
            projection
                .queries
                .iter()
                .enumerate()
                .map(|(ordinal, query)| query_record(ordinal, query))
                .collect(),
            vec![raster_record(&projection.raster)],
        ],
    )
}

fn receipt_records(scene: &SceneInputV2) -> Vec<R> {
    vec![R::new(vec![
        F::optional_u64("generation", scene.receipt.generation),
        F::u32("viewport-width", scene.viewport.0),
        F::u32("viewport-height", scene.viewport.1),
        F::u64("mutation-count", scene.receipt.mutation_count),
        F::u64("invalidation", scene.receipt.invalidation),
    ])]
}

fn mapping_records(mapping: &[(u32, Option<String>)]) -> Vec<R> {
    mapping
        .iter()
        .map(|(key, path)| R::new(vec![F::u32("key", *key), optional_string("path", path)]))
        .collect()
}

fn geometry_record(value: &NormalizedGeometryV2) -> R {
    let mut fields = vec![
        F::u32("key", value.key),
        optional_string("path", &value.path),
    ];
    fields.extend(base_fields(value.base));
    fields.extend(affine_fields(value.affine));
    fields.push(F::i128("determinant", value.determinant));
    fields.extend(aabb_fields("world", value.aabb));
    R::new(fields)
}

fn clip_record(value: &NormalizedClipV2) -> R {
    let mut fields = vec![
        F::u32("key", value.key),
        F::u32("owner", value.owner),
        optional_string("path", &value.path),
        F::optional_u32("parent", value.parent),
        F::u32("shape", value.shape),
    ];
    fields.extend(affine_fields(value.affine));
    fields.push(F::i128("determinant", value.determinant));
    fields.extend(aabb_fields("primitive", value.primitive));
    fields.extend(aabb_fields("effective", value.effective));
    R::new(fields)
}

fn paint_record(value: &NormalizedPaintV2) -> R {
    let mut fields = vec![
        F::u32("key", value.key),
        F::u32("owner", value.owner),
        optional_string("path", &value.path),
    ];
    fields.extend(affine_fields(value.affine));
    fields.push(F::i128("determinant", value.determinant));
    fields.extend(aabb_fields("world", value.aabb));
    let (tag, shape, brush, image) = match value.reference {
        NormalizedPaintReferenceV2::Coverage { shape, brush } => {
            (0, Some(shape), Some(brush), None)
        }
        NormalizedPaintReferenceV2::Image { image } => (1, None, None, Some(image)),
    };
    fields.extend([
        F::tag("reference-tag", tag),
        F::optional_u32("shape", shape),
        F::optional_u32("brush", brush),
        F::optional_u32("image", image),
        F::optional_u32("clip", value.clip),
        F::u32("stack", value.stack),
        F::u32("item", value.item),
    ]);
    R::new(fields)
}

fn item_record(value: &NormalizedItemV2) -> R {
    let mut fields = vec![
        F::u32("key", value.key),
        F::u32("owner", value.owner),
        optional_string("path", &value.path),
    ];
    fields.extend(affine_fields(value.affine));
    fields.push(F::i128("determinant", value.determinant));
    fields.extend(aabb_fields("world", value.aabb));
    fields.extend([
        F::u32("shape", value.shape),
        F::optional_u32("clip", value.clip),
        F::u32("stack", value.stack),
        F::u32("item", value.item),
    ]);
    R::new(fields)
}

fn query_record(ordinal: usize, value: &NormalizedQueryV2) -> R {
    let mut fields = vec![
        F::u64(
            "query-ordinal",
            u64::try_from(ordinal).expect("query ordinal should fit"),
        ),
        F::i64("scene-x", value.scene.x),
        F::i64("scene-y", value.scene.y),
    ];
    match &value.result {
        Some(hit) => fields.extend([
            F::tag("result-tag", 1),
            F::optional_u32("key", Some(hit.key)),
            F::optional_u32("owner", Some(hit.owner)),
            optional_string("path", &hit.path),
            F::optional_u32("item-ordinal", Some(hit.item)),
            optional_i64("local-x", Some(hit.local.x)),
            optional_i64("local-y", Some(hit.local.y)),
        ]),
        None => fields.extend([
            F::tag("result-tag", 0),
            F::optional_u32("key", None),
            F::optional_u32("owner", None),
            optional_string("path", &None),
            F::optional_u32("item-ordinal", None),
            optional_i64("local-x", None),
            optional_i64("local-y", None),
        ]),
    }
    R::new(fields)
}

fn raster_record(value: &NormalizedRasterV2) -> R {
    R::new(vec![
        F::u32("width", value.width),
        F::u32("height", value.height),
        F::u64("stride", value.stride),
        F::bytes("bytes", &value.bytes),
    ])
}

fn base_fields(value: [i64; 4]) -> Vec<F> {
    vec![
        F::i64("base-x", value[0]),
        F::i64("base-y", value[1]),
        F::i64("base-width", value[2]),
        F::i64("base-height", value[3]),
    ]
}

fn affine_fields(value: [i64; 6]) -> Vec<F> {
    [
        "affine-a",
        "affine-b",
        "affine-c",
        "affine-d",
        "affine-tx",
        "affine-ty",
    ]
    .into_iter()
    .zip(value)
    .map(|(name, value)| F::i64(name, value))
    .collect()
}

fn aabb_fields(prefix: &'static str, value: NormalizedAabbV2) -> Vec<F> {
    let names = match prefix {
        "world" => [
            "world-aabb-empty",
            "world-aabb-min-x",
            "world-aabb-min-y",
            "world-aabb-max-x",
            "world-aabb-max-y",
        ],
        "primitive" => [
            "primitive-aabb-empty",
            "primitive-aabb-min-x",
            "primitive-aabb-min-y",
            "primitive-aabb-max-x",
            "primitive-aabb-max-y",
        ],
        "effective" => [
            "effective-aabb-empty",
            "effective-aabb-min-x",
            "effective-aabb-min-y",
            "effective-aabb-max-x",
            "effective-aabb-max-y",
        ],
        _ => unreachable!("closed AABB prefix"),
    };
    vec![
        F::bool(names[0], value.empty),
        F::i64(names[1], value.min_x),
        F::i64(names[2], value.min_y),
        F::i64(names[3], value.max_x),
        F::i64(names[4], value.max_y),
    ]
}

fn optional_string(name: &'static str, value: &Option<String>) -> F {
    let mut encoded = vec![u8::from(value.is_some())];
    if let Some(value) = value {
        encoded.extend_from_slice(&(value.len() as u32).to_le_bytes());
        encoded.extend_from_slice(value.as_bytes());
    }
    F::raw(name, encoded)
}

fn optional_i64(name: &'static str, value: Option<i64>) -> F {
    let mut encoded = vec![u8::from(value.is_some())];
    if let Some(value) = value {
        encoded.extend_from_slice(&value.to_le_bytes());
    }
    F::raw(name, encoded)
}
