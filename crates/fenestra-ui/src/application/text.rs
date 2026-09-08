use std::sync::Arc;

use fenestra_ui_runtime::prototype::CommittedRuntimeSnapshot;
use fenestra_ui_spatial::prototype::{
    SpatialImageDestinationRectV2, SpatialImageKeyV2, SpatialImagePaintAttachmentV2,
    SpatialImageV2, SpatialLimitKindV2, SpatialLimitsV2, SpatialResolvedSnapshotV2,
    SpatialScalarV2,
};

use super::{NamedNode, controls, decoration, interaction::Interaction};
use crate::{
    Error, Limits, TextEngine, TextError, TextLayout, TextLimits, TextMetrics, TextRequest,
    TextStyle,
};

#[derive(Clone)]
pub(super) struct TextState {
    pub(super) content: Arc<str>,
    pub(super) style: TextStyle,
    pub(super) effective_style: TextStyle,
    pub(super) layout: Option<Arc<TextLayout>>,
    pub(super) natural: Option<TextMetrics>,
    pub(super) wrapped: Option<(u32, TextMetrics)>,
}

impl TextState {
    pub(super) fn new(content: &str, style: TextStyle) -> Self {
        Self {
            content: Arc::from(content),
            style,
            effective_style: style,
            layout: None,
            natural: None,
            wrapped: None,
        }
    }

    pub(super) fn invalidate(&mut self) {
        self.layout = None;
        self.natural = None;
        self.wrapped = None;
    }
}

pub(super) fn prepare_node(
    node: &mut NamedNode,
    engine: &mut Option<Box<dyn TextEngine>>,
    limits: TextLimits,
) -> Result<(), Error> {
    let Some(text) = &mut node.text else {
        return Ok(());
    };
    let engine = engine.as_mut().ok_or(TextError::EngineUnavailable)?;
    let size = node.resolved;
    if size.width() == 0 || size.height() == 0 {
        text.layout = None;
        return Ok(());
    }
    if let Some(layout) = &text.layout
        && layout.raster().size() == size
    {
        return check_measurement(text, layout);
    }
    let request = TextRequest::new(&text.content, text.effective_style, size, limits)?;
    let layout = engine.layout(request)?;
    layout.validate_request(request)?;
    check_measurement(text, &layout)?;
    text.layout = Some(Arc::new(layout));
    Ok(())
}

fn check_measurement(text: &TextState, layout: &TextLayout) -> Result<(), Error> {
    if let Some((width, metrics)) = text.wrapped
        && width == layout.raster().size().width()
        && metrics != layout.metrics()
    {
        return Err(TextError::InconsistentMeasurement.into());
    }
    Ok(())
}

pub(super) fn prepare_frame(
    committed: &CommittedRuntimeSnapshot,
    nodes: &[NamedNode],
    interaction: &Interaction,
    spatial_limits: SpatialLimitsV2,
    limits: Limits,
) -> Result<Option<SpatialResolvedSnapshotV2>, Error> {
    let spatial = committed
        .spatial()
        .ok_or_else(|| Error::InvalidProgram("missing spatial frame".into()))?;
    let mut additions = Vec::new();
    for node in nodes {
        let owner = spatial
            .spatial_key(node.id)
            .ok_or_else(|| Error::InvalidProgram("missing image owner".into()))?;
        let Some(layout) = node.text.as_ref().and_then(|text| text.layout.as_ref()) else {
            continue;
        };
        let raster = layout.raster();
        let size = raster.size();
        let stride = size.width().checked_mul(4).ok_or(Error::CapacityOverflow)?;
        let image = SpatialImageV2::new(
            SpatialImageKeyV2::new(0),
            size.width(),
            size.height(),
            stride,
            raster.bytes().to_vec().into_boxed_slice(),
        );
        let scalar = |value: u32| SpatialScalarV2::new(i64::from(value) * 65_536);
        additions.push(SpatialImagePaintAttachmentV2::new(
            owner,
            image,
            SpatialImageDestinationRectV2::new(
                scalar(0),
                scalar(0),
                scalar(size.width()),
                scalar(size.height()),
            ),
            None,
        ));
    }
    for (index, node) in nodes.iter().enumerate() {
        if node.control.is_none() || !controls::state(nodes, index, interaction).focused() {
            continue;
        }
        // Place the ring after the control's complete subtree, before later
        // siblings. A child background must not hide its owner's focus indicator.
        let mut last = index;
        while let Some(&child) = nodes[last].children.last() {
            last = child;
        }
        let owner = spatial
            .spatial_key(nodes[last].id)
            .ok_or_else(|| Error::InvalidProgram("missing focus painter".into()))?;
        let bounds = super::bounds_for(committed, node.id)?;
        let anchor = super::bounds_for(committed, nodes[last].id)?;
        additions.extend(decoration::focus_paints(
            owner,
            node.resolved,
            node.state_style.focus_color,
            (bounds.x() - anchor.x(), bounds.y() - anchor.y()),
        ));
    }
    if additions.is_empty() {
        return Ok(None);
    }
    let paint_count = nodes
        .len()
        .checked_add(additions.len())
        .ok_or(Error::CapacityOverflow)?;
    let extended = SpatialLimitKindV2::ALL.map(|kind| match kind {
        SpatialLimitKindV2::Images => additions.len(),
        SpatialLimitKindV2::PaintItems => paint_count,
        SpatialLimitKindV2::ImageEdge | SpatialLimitKindV2::ImagePixelsTotal => {
            limits.text().max_pixels().saturating_add(8)
        }
        SpatialLimitKindV2::PaintItemsPerNode => 10,
        _ => spatial_limits.limit(kind),
    });
    spatial
        .snapshot()
        .with_image_paints(additions.into_boxed_slice(), SpatialLimitsV2::new(extended))
        .map(Some)
        .map_err(|error| Error::Runtime(error.to_string()))
}
