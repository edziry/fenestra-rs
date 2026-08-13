use fenestra_ui_exp_0007_typed_authoring::generated_hybrid_spatial_v2;
use fenestra_ui_ir::prototype::{
    PropertyId, PropertyValue, SpatialValidationLimitsV2, StyleValidationLimits, ValidationLimits,
    validate_construction, validate_schema, validate_spatial, validate_style,
};
use fenestra_ui_runtime::prototype::{RuntimeCapacity, UiRuntime};
use fenestra_ui_spatial::prototype::{
    REGISTERED_REFERENCE_RASTER_LIMITS_V2, REGISTERED_SPATIAL_LIMITS_V2, SpatialPaintFrameV2,
    SpatialViewportV2,
};
use vello::kurbo::Affine;
use vello::peniko::{ImageAlphaType, ImageBrush, ImageData, ImageFormat};

const IR_LIMITS: ValidationLimits = ValidationLimits::new(1, 8, 7, 1, 6, 19, 2, 4, 8);
const STYLE_LIMITS: StyleValidationLimits = StyleValidationLimits::new(3);
const SPATIAL_LIMITS: SpatialValidationLimitsV2 =
    SpatialValidationLimitsV2::new([7, 5, 3, 3, 4, 4, 3, 1, 5, 3, 3, 1, 16]);
const CAPACITY: RuntimeCapacity = RuntimeCapacity::new(4, 4, 12, 2, 96, 3);
const VIEWPORT: SpatialViewportV2 = SpatialViewportV2::new(192, 128);
const TONE_PROPERTY: PropertyId = PropertyId::new(4);
const MUTATED_TONE: PropertyValue = PropertyValue::Rgba8([80, 40, 24, 255]);
const FNV_OFFSET_BASIS: u64 = 14_695_981_039_346_656_037;

/// Closed failures while preparing the registered runtime scenes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RegisteredSceneErrorKindV1 {
    /// The generated registered program did not validate.
    Validation,
    /// The validated program could not initialize the runtime.
    Runtime,
    /// The registered property mutation could not commit.
    Mutation,
    /// The spatial frame could not produce its bounded reference raster.
    Raster,
    /// A required registered spatial publication was absent.
    Invariant,
}

/// Candidate-neutral facts from one registered runtime scene.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RegisteredSceneObservationV1 {
    generation: u64,
    width: u32,
    height: u32,
    rgba_bytes: usize,
    raster_digest: u64,
    scene_commands: usize,
    scene_fingerprint: u64,
    used_vello_scene: bool,
    executed_gpu: bool,
}

impl RegisteredSceneObservationV1 {
    /// Returns the committed runtime generation.
    #[must_use]
    pub const fn generation(self) -> u64 {
        self.generation
    }

    /// Returns the scene width in pixels.
    #[must_use]
    pub const fn width(self) -> u32 {
        self.width
    }

    /// Returns the scene height in pixels.
    #[must_use]
    pub const fn height(self) -> u32 {
        self.height
    }

    /// Returns the packed RGBA8 byte count used to build the scene.
    #[must_use]
    pub const fn rgba_bytes(self) -> usize {
        self.rgba_bytes
    }

    /// Returns the deterministic source-raster digest.
    #[must_use]
    pub const fn raster_digest(self) -> u64 {
        self.raster_digest
    }

    /// Returns the number of Vello draw commands in the scene.
    #[must_use]
    pub const fn scene_commands(self) -> usize {
        self.scene_commands
    }

    /// Returns the deterministic encoded-scene fingerprint.
    #[must_use]
    pub const fn scene_fingerprint(self) -> u64 {
        self.scene_fingerprint
    }

    /// Reports whether this observation was formed through a Vello scene.
    #[must_use]
    pub const fn used_vello_scene(self) -> bool {
        self.used_vello_scene
    }

    /// Reports whether this inspection-only bridge executed GPU work.
    #[must_use]
    pub const fn executed_gpu(self) -> bool {
        self.executed_gpu
    }
}

/// Builds the initial and registered mutation scenes from the canonical `.fen` fixture.
#[must_use = "registered scene preparation failures must be handled"]
pub fn inspect_registered_scene_pair_v1()
-> Result<[RegisteredSceneObservationV1; 2], RegisteredSceneErrorKindV1> {
    let mut runtime = build_registered_runtime_v1(VIEWPORT)?;
    let initial = observe_committed_scene(&runtime)?;
    let before = runtime.committed();
    let mut transaction = runtime.begin_transaction();
    transaction
        .set_property(before.root(), TONE_PROPERTY, MUTATED_TONE)
        .map_err(|_| RegisteredSceneErrorKindV1::Mutation)?;
    runtime
        .commit(transaction)
        .map_err(|_| RegisteredSceneErrorKindV1::Mutation)?;
    let mutated = observe_committed_scene(&runtime)?;
    Ok([initial, mutated])
}

/// Builds the canonical format-2 `.fen` fixture at one caller-selected viewport.
#[must_use = "registered runtime preparation failures must be handled"]
pub fn build_registered_runtime_v1(
    viewport: SpatialViewportV2,
) -> Result<UiRuntime, RegisteredSceneErrorKindV1> {
    let programs = generated_hybrid_spatial_v2();
    let schema = validate_schema(programs.0, IR_LIMITS)
        .map_err(|_| RegisteredSceneErrorKindV1::Validation)?;
    let construction = validate_construction(&schema, programs.1, IR_LIMITS)
        .map_err(|_| RegisteredSceneErrorKindV1::Validation)?;
    let style = validate_style(&construction, programs.2, STYLE_LIMITS)
        .map_err(|_| RegisteredSceneErrorKindV1::Validation)?;
    let spatial = validate_spatial(&style, programs.3, SPATIAL_LIMITS)
        .map_err(|_| RegisteredSceneErrorKindV1::Validation)?;
    UiRuntime::new_spatial_ir(spatial, viewport, REGISTERED_SPATIAL_LIMITS_V2, CAPACITY)
        .map_err(|_| RegisteredSceneErrorKindV1::Runtime)
}

fn observe_committed_scene(
    runtime: &UiRuntime,
) -> Result<RegisteredSceneObservationV1, RegisteredSceneErrorKindV1> {
    let committed = runtime.committed();
    let spatial = committed
        .spatial()
        .ok_or(RegisteredSceneErrorKindV1::Invariant)?;
    scene_observation(
        committed.generation().get(),
        spatial.snapshot().paint_frame(),
    )
}

fn scene_observation(
    generation: u64,
    frame: SpatialPaintFrameV2<'_>,
) -> Result<RegisteredSceneObservationV1, RegisteredSceneErrorKindV1> {
    let raster = frame
        .rasterize_reference(REGISTERED_REFERENCE_RASTER_LIMITS_V2)
        .map_err(|_| RegisteredSceneErrorKindV1::Raster)?;
    let raster_digest = fold_bytes(FNV_OFFSET_BASIS, raster.bytes());
    let image = ImageBrush::new(ImageData {
        data: raster.bytes().to_vec().into(),
        format: ImageFormat::Rgba8,
        alpha_type: ImageAlphaType::AlphaPremultiplied,
        width: raster.width(),
        height: raster.height(),
    });
    let mut scene = vello::Scene::new();
    scene.draw_image(&image, Affine::IDENTITY);
    let encoding = scene.encoding();
    let scene_commands = encoding.draw_tags.len();
    let mut scene_fingerprint = raster_digest;
    for tag in &encoding.path_tags {
        scene_fingerprint = fold_u32(scene_fingerprint, u32::from(tag.0));
    }
    for value in &encoding.path_data {
        scene_fingerprint = fold_u32(scene_fingerprint, *value);
    }
    for tag in &encoding.draw_tags {
        scene_fingerprint = fold_u32(scene_fingerprint, tag.0);
    }
    for value in &encoding.draw_data {
        scene_fingerprint = fold_u32(scene_fingerprint, *value);
    }
    for transform in &encoding.transforms {
        for value in transform.matrix.into_iter().chain(transform.translation) {
            scene_fingerprint = fold_u32(scene_fingerprint, value.to_bits());
        }
    }
    scene_fingerprint = fold_u32(scene_fingerprint, encoding.n_paths);
    Ok(RegisteredSceneObservationV1 {
        generation,
        width: raster.width(),
        height: raster.height(),
        rgba_bytes: raster.bytes().len(),
        raster_digest,
        scene_commands,
        scene_fingerprint,
        used_vello_scene: true,
        executed_gpu: false,
    })
}

fn fold_bytes(mut hash: u64, bytes: &[u8]) -> u64 {
    for byte in bytes {
        hash = (hash ^ u64::from(*byte)).wrapping_mul(1_099_511_628_211);
    }
    hash
}

fn fold_u32(hash: u64, value: u32) -> u64 {
    fold_bytes(hash, &value.to_le_bytes())
}
