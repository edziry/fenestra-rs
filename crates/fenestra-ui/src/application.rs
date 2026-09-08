use fenestra_ui_runtime::prototype::{NodeId, UiRuntime};
use fenestra_ui_spatial::prototype::{
    ReferenceRasterLimitsV2, SpatialPointV2, SpatialScalarV2, SpatialViewportV2,
};

use crate::model::ElementKind;
use crate::{Color, Error, Limits, Raster, Size, Style, View, lower};

struct NamedNode {
    name: String,
    id: NodeId,
    kind: ElementKind,
    style: Style,
}

/// A named, typed application backed by the deterministic Fenestra runtime.
pub struct Application {
    runtime: UiRuntime,
    nodes: Vec<NamedNode>,
    size: Size,
    limits: Limits,
}

impl Application {
    /// Constructs a view with default resource limits.
    pub fn new(view: View, size: Size) -> Result<Self, Error> {
        Self::with_limits(view, size, Limits::default())
    }

    /// Constructs a view within explicit resource limits.
    pub fn with_limits(view: View, size: Size, limits: Limits) -> Result<Self, Error> {
        validate_size(size, limits)?;
        let lowered = lower::lower(&view, limits)?;
        let runtime = UiRuntime::new_spatial_ir(
            lowered.program,
            viewport(size),
            lowered.spatial_limits,
            lowered.capacity,
        )
        .map_err(|error| Error::Runtime(format!("{:?}", error.kind())))?;
        let committed = runtime.committed();
        let mut logical = vec![committed.root()];
        let mut by_template = vec![None; lowered.names.len()];
        while let Some(node) = logical.pop() {
            let template = committed
                .template(node)
                .ok_or_else(|| Error::InvalidProgram("missing node template".into()))?;
            let slot = by_template
                .get_mut(template.get() as usize)
                .ok_or_else(|| Error::InvalidProgram("unknown node template".into()))?;
            *slot = Some(node);
            logical.extend(committed.children(node).unwrap_or(&[]).iter().copied());
        }
        let mut elements = vec![&view.root];
        let mut nodes = Vec::with_capacity(lowered.names.len());
        while let Some(element) = elements.pop() {
            let index = nodes.len();
            let id = by_template[index]
                .ok_or_else(|| Error::InvalidProgram("missing logical element".into()))?;
            nodes.push(NamedNode {
                name: element.name.clone(),
                id,
                kind: element.kind,
                style: element.style,
            });
            elements.extend(element.children.iter().rev());
        }
        Ok(Self {
            runtime,
            nodes,
            size,
            limits,
        })
    }

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

    /// Applies all typed style properties in one atomic runtime transaction.
    pub fn set_style(&mut self, name: &str, style: Style) -> Result<(), Error> {
        let index = self.node_index(name)?;
        let node = &self.nodes[index];
        style.validate(name, node.kind)?;
        let mut transaction = self.runtime.begin_transaction();
        for (property, value) in style.values() {
            transaction
                .set_property(node.id, property, value)
                .map_err(|error| Error::Runtime(format!("{:?}", error.kind())))?;
        }
        self.runtime
            .commit(transaction)
            .map_err(|error| Error::Runtime(format!("{:?}", error.kind())))?;
        self.nodes[index].style = style;
        Ok(())
    }

    /// Updates one element's background without changing its other properties.
    pub fn set_background(&mut self, name: &str, color: Color) -> Result<(), Error> {
        self.set_style(name, self.style(name)?.background(color))
    }

    /// Updates width and height together, including layout, paint and hit geometry.
    pub fn set_size(&mut self, name: &str, width: i32, height: i32) -> Result<(), Error> {
        self.set_style(name, self.style(name)?.width(width).height(height))
    }

    /// Resizes the viewport after validating its pixel budget.
    pub fn resize(&mut self, size: Size) -> Result<(), Error> {
        validate_size(size, self.limits)?;
        let mut transaction = self.runtime.begin_transaction();
        transaction
            .resize_spatial(viewport(size))
            .map_err(|error| Error::Runtime(format!("{:?}", error.kind())))?;
        self.runtime
            .commit(transaction)
            .map_err(|error| Error::Runtime(format!("{:?}", error.kind())))?;
        self.size = size;
        Ok(())
    }

    /// Returns the topmost input-enabled element at physical pixel coordinates.
    #[must_use]
    pub fn hit_test(&self, x: i32, y: i32) -> Option<&str> {
        if x < 0 || y < 0 || x as u32 >= self.size.width() || y as u32 >= self.size.height() {
            return None;
        }
        let committed = self.runtime.committed();
        let spatial = committed.spatial()?;
        let point = SpatialPointV2::new(
            SpatialScalarV2::new(i64::from(x) * 65_536),
            SpatialScalarV2::new(i64::from(y) * 65_536),
        );
        let hit = spatial.snapshot().hit_test(point)?;
        let id = spatial.logical_node(hit.owner())?;
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
        let raster = spatial
            .snapshot()
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
