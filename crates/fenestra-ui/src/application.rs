use fenestra_ui_runtime::prototype::{CommittedRuntimeSnapshot, NodeId, UiRuntime};
use fenestra_ui_spatial::prototype::{
    ReferenceRasterLimitsV2, SpatialLimitsV2, SpatialPointV2, SpatialResolvedSnapshotV2,
    SpatialScalarV2, SpatialViewportV2,
};

use crate::control::ControlData;
use crate::model::ElementKind;
use crate::{Bounds, Error, Limits, Raster, Size, StateStyle, Style, TextEngine};

mod accessibility;
mod construction;
mod controls;
mod decoration;
mod interaction;
mod layout;
mod mutation;
mod publication;
mod text;

use text::TextState;

#[derive(Clone)]
struct NamedNode {
    name: String,
    id: NodeId,
    kind: ElementKind,
    style: Style,
    effective_style: Style,
    state_style: StateStyle,
    control: Option<ControlData>,
    control_owner: Option<usize>,
    resolved: Size,
    children: Vec<usize>,
    text: Option<TextState>,
}

/// A named, typed application backed by the deterministic Fenestra runtime.
pub struct Application {
    runtime: UiRuntime,
    nodes: Vec<NamedNode>,
    size: Size,
    limits: Limits,
    revision: i32,
    interaction: interaction::Interaction,
    spatial_limits: SpatialLimitsV2,
    text_engine: Option<Box<dyn TextEngine>>,
    text_frame: Option<SpatialResolvedSnapshotV2>,
}

impl Application {
    /// Returns the committed generation; no-op updates keep this value.
    #[must_use]
    pub fn generation(&self) -> u64 {
        self.runtime.committed().generation().get()
    }

    /// Returns the number of logical application elements.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Iterates element names in authored preorder.
    pub fn node_names(&self) -> impl Iterator<Item = &str> {
        self.nodes.iter().map(|node| node.name.as_str())
    }

    /// Returns the current viewport size in pixels.
    #[must_use]
    pub const fn size(&self) -> Size {
        self.size
    }

    /// Returns the committed style of one named element.
    pub fn style(&self, name: &str) -> Result<Style, Error> {
        Ok(self.nodes[self.node_index(name)?].style)
    }

    /// Returns geometry from the same committed snapshot as paint and hit testing.
    ///
    /// Bounds are not intersected with the viewport. Zero-size elements retain
    /// their layout origin, and children can extend beyond their containers.
    pub fn bounds(&self, name: &str) -> Result<Bounds, Error> {
        let node = &self.nodes[self.node_index(name)?];
        bounds_for(&self.runtime.committed(), node.id)
    }

    /// Returns the topmost input-enabled element at physical pixel coordinates.
    #[must_use]
    pub fn hit_test(&self, x: i32, y: i32) -> Option<&str> {
        let id = hit_node(&self.runtime.committed(), self.size, x, y)?;
        self.nodes
            .iter()
            .find(|node| node.id == id)
            .map(|node| node.name.as_str())
    }

    /// Renders the committed frame into owned, bounded premultiplied RGBA8 pixels.
    pub fn raster(&self) -> Result<Raster, Error> {
        let committed = self.runtime.committed();
        let spatial = committed
            .spatial()
            .ok_or_else(|| Error::InvalidProgram("missing spatial frame".into()))?;
        let snapshot = self
            .text_frame
            .as_ref()
            .unwrap_or_else(|| spatial.snapshot());
        let raster = snapshot
            .paint_frame()
            .rasterize_reference(ReferenceRasterLimitsV2::new(self.limits.max_pixels()))
            .map_err(|error| Error::Runtime(format!("{:?}", error.kind())))?;
        Raster::new(self.size, raster.bytes().to_vec())
    }

    fn node_index(&self, name: &str) -> Result<usize, Error> {
        self.nodes
            .iter()
            .position(|node| node.name == name)
            .ok_or_else(|| Error::UnknownNode { name: name.into() })
    }
}

fn hit_node(committed: &CommittedRuntimeSnapshot, size: Size, x: i32, y: i32) -> Option<NodeId> {
    if x < 0 || y < 0 || x as u32 >= size.width() || y as u32 >= size.height() {
        return None;
    }
    let spatial = committed.spatial()?;
    let point = SpatialPointV2::new(
        SpatialScalarV2::new(i64::from(x) * 65_536),
        SpatialScalarV2::new(i64::from(y) * 65_536),
    );
    let hit = spatial.snapshot().hit_test(point)?;
    spatial.logical_node(hit.owner())
}

fn bounds_for(committed: &CommittedRuntimeSnapshot, id: NodeId) -> Result<Bounds, Error> {
    let spatial = committed
        .spatial()
        .ok_or_else(|| Error::InvalidProgram("missing spatial frame".into()))?;
    let key = spatial
        .spatial_key(id)
        .ok_or_else(|| Error::InvalidProgram("missing element geometry".into()))?;
    let geometry = spatial
        .snapshot()
        .output()
        .geometry()
        .iter()
        .find(|geometry| geometry.key() == key)
        .ok_or_else(|| Error::InvalidProgram("missing element geometry".into()))?;
    // Public views use integer dimensions and identity transforms. World
    // translation includes the placement of every ancestor.
    let transform = geometry.world_from_local();
    Ok(Bounds {
        x: transform.tx().raw() / 65_536,
        y: transform.ty().raw() / 65_536,
        width: u32::try_from(geometry.base_width().raw() / 65_536)
            .map_err(|_| Error::CapacityOverflow)?,
        height: u32::try_from(geometry.base_height().raw() / 65_536)
            .map_err(|_| Error::CapacityOverflow)?,
    })
}

fn validate_size(size: Size, limits: Limits) -> Result<(), Error> {
    let pixels = size.pixel_count()?;
    if pixels > limits.max_pixels() {
        return Err(Error::LimitExceeded {
            resource: "pixels",
            limit: limits.max_pixels(),
            actual: pixels,
        });
    }
    Ok(())
}

fn viewport(size: Size) -> SpatialViewportV2 {
    SpatialViewportV2::new(size.width() as i32, size.height() as i32)
}
