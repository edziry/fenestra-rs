#![forbid(unsafe_code)]

//! First usable application slice for the Fenestra workspace.
//!
//! The application owns interaction state while the authored structure and
//! spatial behavior remain in the format-2 `.fen` fixture. Native presentation
//! is intentionally a later shell around this deterministic application core.

use fenestra_ui_ir::prototype::{
    PropertyId, PropertyValue, SpatialValidationLimitsV2, StructuralRegionId,
    StyleValidationLimits, ValidationLimits, validate_construction, validate_schema,
    validate_spatial, validate_style,
};
use fenestra_ui_runtime::prototype::{
    CommittedRuntimeSnapshot, FragmentId, NodeId, RuntimeCapacity, UiRuntime,
};
use fenestra_ui_spatial::prototype::{
    REGISTERED_REFERENCE_RASTER_LIMITS_V2, REGISTERED_SPATIAL_LIMITS_V2, SpatialPointV2,
    SpatialScalarV2, SpatialViewportV2,
};

#[cfg(any(target_os = "linux", target_os = "windows"))]
/// Native window and CPU presentation shell for the application core.
pub mod native;

/// Bounded ASCII evidence contract for the native application sequence.
pub mod evidence;

const IR_LIMITS: ValidationLimits = ValidationLimits::new(1, 8, 7, 1, 6, 19, 2, 4, 8);
const STYLE_LIMITS: StyleValidationLimits = StyleValidationLimits::new(3);
const SPATIAL_LIMITS: SpatialValidationLimitsV2 =
    SpatialValidationLimitsV2::new([7, 5, 3, 3, 4, 4, 3, 1, 5, 3, 3, 1, 16]);
const CAPACITY: RuntimeCapacity = RuntimeCapacity::new(8, 8, 32, 2, 256, 8);
const TONE_PROPERTY: PropertyId = PropertyId::new(4);
const SELECTED_TONE: PropertyValue = PropertyValue::Rgba8([255, 192, 32, 255]);
const TILES_REGION: StructuralRegionId = StructuralRegionId::new(0);
const FIXED_ONE: i64 = 65_536;

/// Exact authored source consumed by this application.
pub const AUTHORED_FEN_V2: &[u8] =
    include_bytes!("../../../probes/exp-0007-typed-authoring/fixtures/hybrid-spatial-v2.fen");

/// Canonical Rust generated from [`AUTHORED_FEN_V2`].
pub const GENERATED_AUTHORING_RUST_V2: &str =
    include_str!(concat!(env!("OUT_DIR"), "/layout_inspector_fen_v2.rs"));

/// Default logical viewport used by the deterministic application core.
pub const DEFAULT_VIEWPORT: SpatialViewportV2 = SpatialViewportV2::new(192, 128);

/// Failures exposed by the application core.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InspectorErrorKind {
    /// The generated authored programs failed framework validation.
    Validation,
    /// The runtime could not publish the initial or updated spatial state.
    Runtime,
    /// A requested interaction could not be committed.
    Transaction,
    /// The authored keyed region was not available in the committed tree.
    MissingKeyedRegion,
    /// The bounded reference raster could not be formed.
    Rasterization,
}

/// User actions understood by the first application slice.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InspectorAction {
    /// Move the pointer in logical viewport pixels.
    PointerMove {
        /// Horizontal logical pixel coordinate.
        x: i32,
        /// Vertical logical pixel coordinate.
        y: i32,
    },
    /// Select the currently hovered node and update its authored tone.
    PointerPress,
    /// Insert one keyed tile into the authored `tiles` region.
    InsertTile {
        /// Stable key for the new tile.
        key: u64,
    },
    /// Resize the logical spatial viewport.
    Resize {
        /// Logical viewport width.
        width: i32,
        /// Logical viewport height.
        height: i32,
    },
}

/// Deterministic observation of one application frame.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InspectorFrame {
    generation: u64,
    viewport: SpatialViewportV2,
    node_count: usize,
    keyed_keys: Box<[u64]>,
    image_count: usize,
    paint_count: usize,
    hit_count: usize,
    semantic_count: usize,
    raster_bytes: usize,
    has_hover: bool,
    has_selection: bool,
}

/// Bounded RGBA8 reference pixels for native presentation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InspectorRaster {
    viewport: SpatialViewportV2,
    bytes: Box<[u8]>,
}

impl InspectorRaster {
    /// Returns the logical viewport represented by these pixels.
    #[must_use]
    pub const fn viewport(&self) -> SpatialViewportV2 {
        self.viewport
    }

    /// Returns premultiplied RGBA8 pixels in row-major order.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl InspectorFrame {
    /// Returns the committed runtime generation.
    #[must_use]
    pub const fn generation(&self) -> u64 {
        self.generation
    }

    /// Returns the logical viewport used by this frame.
    #[must_use]
    pub const fn viewport(&self) -> SpatialViewportV2 {
        self.viewport
    }

    /// Returns the number of live logical nodes.
    #[must_use]
    pub const fn node_count(&self) -> usize {
        self.node_count
    }

    /// Returns keyed tile keys in committed order.
    #[must_use]
    pub fn keyed_keys(&self) -> &[u64] {
        &self.keyed_keys
    }

    /// Returns the number of authored image resources.
    #[must_use]
    pub const fn image_count(&self) -> usize {
        self.image_count
    }

    /// Returns the number of resolved paint items.
    #[must_use]
    pub const fn paint_count(&self) -> usize {
        self.paint_count
    }

    /// Returns the number of resolved hit items.
    #[must_use]
    pub const fn hit_count(&self) -> usize {
        self.hit_count
    }

    /// Returns the number of resolved semantic items.
    #[must_use]
    pub const fn semantic_count(&self) -> usize {
        self.semantic_count
    }

    /// Returns the size of the reference raster in bytes.
    #[must_use]
    pub const fn raster_bytes(&self) -> usize {
        self.raster_bytes
    }

    /// Reports whether a pointer hit is currently hovered.
    #[must_use]
    pub const fn has_hover(&self) -> bool {
        self.has_hover
    }

    /// Reports whether a node is selected.
    #[must_use]
    pub const fn has_selection(&self) -> bool {
        self.has_selection
    }
}

/// Single-owner deterministic application state.
pub struct LayoutInspector {
    runtime: UiRuntime,
    hovered: Option<NodeId>,
    selected: Option<NodeId>,
}

impl LayoutInspector {
    /// Builds the application from the generated format-2 `.fen` program.
    pub fn new() -> Result<Self, InspectorErrorKind> {
        let programs = generated_programs();
        let schema =
            validate_schema(programs.0, IR_LIMITS).map_err(|_| InspectorErrorKind::Validation)?;
        let construction = validate_construction(&schema, programs.1, IR_LIMITS)
            .map_err(|_| InspectorErrorKind::Validation)?;
        let style = validate_style(&construction, programs.2, STYLE_LIMITS)
            .map_err(|_| InspectorErrorKind::Validation)?;
        let spatial = validate_spatial(&style, programs.3, SPATIAL_LIMITS)
            .map_err(|_| InspectorErrorKind::Validation)?;
        let runtime = UiRuntime::new_spatial_ir(
            spatial,
            DEFAULT_VIEWPORT,
            REGISTERED_SPATIAL_LIMITS_V2,
            CAPACITY,
        )
        .map_err(|_| InspectorErrorKind::Runtime)?;
        Ok(Self {
            runtime,
            hovered: None,
            selected: None,
        })
    }

    /// Applies one user action without exposing runtime internals.
    pub fn dispatch(&mut self, action: InspectorAction) -> Result<(), InspectorErrorKind> {
        match action {
            InspectorAction::PointerMove { x, y } => {
                self.pointer_move(x, y);
                Ok(())
            }
            InspectorAction::PointerPress => self.pointer_press(),
            InspectorAction::InsertTile { key } => self.insert_tile(key),
            InspectorAction::Resize { width, height } => self.resize(width, height),
        }
    }

    /// Observes the latest committed frame and its interaction state.
    pub fn observe(&self) -> Result<InspectorFrame, InspectorErrorKind> {
        let committed = self.runtime.committed();
        let spatial = committed.spatial().ok_or(InspectorErrorKind::Runtime)?;
        let paint = spatial.snapshot().paint_frame();
        let raster = paint
            .rasterize_reference(REGISTERED_REFERENCE_RASTER_LIMITS_V2)
            .map_err(|_| InspectorErrorKind::Rasterization)?;
        let keyed_keys = keyed_keys(&committed).ok_or(InspectorErrorKind::MissingKeyedRegion)?;
        Ok(InspectorFrame {
            generation: committed.generation().get(),
            viewport: paint.viewport(),
            node_count: committed.node_count(),
            keyed_keys,
            image_count: paint.images().len(),
            paint_count: paint.resolved_paints().len(),
            hit_count: spatial.snapshot().output().hits().len(),
            semantic_count: spatial.snapshot().output().semantics().len(),
            raster_bytes: raster.bytes().len(),
            has_hover: self.hovered.is_some(),
            has_selection: self.selected.is_some(),
        })
    }

    /// Produces the latest committed frame as bounded premultiplied RGBA8.
    pub fn reference_raster(&self) -> Result<InspectorRaster, InspectorErrorKind> {
        let committed = self.runtime.committed();
        let spatial = committed.spatial().ok_or(InspectorErrorKind::Runtime)?;
        let paint = spatial.snapshot().paint_frame();
        let raster = paint
            .rasterize_reference(REGISTERED_REFERENCE_RASTER_LIMITS_V2)
            .map_err(|_| InspectorErrorKind::Rasterization)?;
        Ok(InspectorRaster {
            viewport: paint.viewport(),
            bytes: raster.bytes().into(),
        })
    }

    /// Returns the currently hovered logical node, if any.
    #[must_use]
    pub const fn hovered(&self) -> Option<NodeId> {
        self.hovered
    }

    /// Returns the currently selected logical node, if any.
    #[must_use]
    pub const fn selected(&self) -> Option<NodeId> {
        self.selected
    }

    fn pointer_move(&mut self, x: i32, y: i32) {
        let committed = self.runtime.committed();
        self.hovered = committed
            .spatial()
            .and_then(|spatial| spatial.snapshot().hit_test(pixel_point(x, y)))
            .and_then(|hit| committed.spatial()?.logical_node(hit.owner()));
    }

    fn pointer_press(&mut self) -> Result<(), InspectorErrorKind> {
        let Some(node) = self.hovered else {
            return Ok(());
        };
        let mut transaction = self.runtime.begin_transaction();
        transaction
            .set_property(node, TONE_PROPERTY, SELECTED_TONE)
            .map_err(|_| InspectorErrorKind::Transaction)?;
        self.runtime
            .commit(transaction)
            .map_err(|_| InspectorErrorKind::Transaction)?;
        self.selected = Some(node);
        Ok(())
    }

    fn insert_tile(&mut self, key: u64) -> Result<(), InspectorErrorKind> {
        let committed = self.runtime.committed();
        let fragment =
            find_tiles_fragment(&committed).ok_or(InspectorErrorKind::MissingKeyedRegion)?;
        let index = committed
            .keyed_members(fragment)
            .ok_or(InspectorErrorKind::MissingKeyedRegion)?
            .len();
        let mut transaction = self.runtime.begin_transaction();
        transaction
            .insert_keyed(fragment, key, index)
            .map_err(|_| InspectorErrorKind::Transaction)?;
        self.runtime
            .commit(transaction)
            .map_err(|_| InspectorErrorKind::Transaction)?;
        Ok(())
    }

    fn resize(&mut self, width: i32, height: i32) -> Result<(), InspectorErrorKind> {
        let mut transaction = self.runtime.begin_transaction();
        transaction
            .resize_spatial(SpatialViewportV2::new(width, height))
            .map_err(|_| InspectorErrorKind::Transaction)?;
        self.runtime
            .commit(transaction)
            .map_err(|_| InspectorErrorKind::Transaction)?;
        Ok(())
    }
}

fn generated_programs() -> (
    fenestra_ui_ir::prototype::SchemaManifest,
    fenestra_ui_ir::prototype::ConstructionProgram,
    fenestra_ui_ir::prototype::StyleProgram,
    fenestra_ui_ir::prototype::SpatialProgramV2,
) {
    include!(concat!(env!("OUT_DIR"), "/layout_inspector_fen_v2.rs"))
}

fn keyed_keys(committed: &CommittedRuntimeSnapshot) -> Option<Box<[u64]>> {
    let fragment = find_tiles_fragment(committed)?;
    Some(
        committed
            .keyed_members(fragment)?
            .map(|(key, _)| key)
            .collect(),
    )
}

fn find_tiles_fragment(committed: &CommittedRuntimeSnapshot) -> Option<FragmentId> {
    let mut pending = vec![committed.root()];
    while let Some(node) = pending.pop() {
        if let Some(fragment) = committed.fragment(node, TILES_REGION) {
            return Some(fragment);
        }
        pending.extend(committed.children(node)?.iter().copied());
    }
    None
}

fn pixel_point(x: i32, y: i32) -> SpatialPointV2 {
    SpatialPointV2::new(
        SpatialScalarV2::new(i64::from(x) * FIXED_ONE),
        SpatialScalarV2::new(i64::from(y) * FIXED_ONE),
    )
}
