use std::sync::Arc;

use fenestra_ui_runtime::prototype::CommittedRuntimeSnapshot;
use fenestra_ui_spatial::prototype::{
    SpatialImageDestinationRectV2, SpatialImageKeyV2, SpatialImagePaintAttachmentV2,
    SpatialImageV2, SpatialLimitKindV2, SpatialLimitsV2, SpatialResolvedSnapshotV2,
    SpatialScalarV2,
};

use super::NamedNode;
use crate::{
    Error, Limits, Size, TextEngine, TextError, TextLayout, TextLimits, TextRequest, TextStyle,
};

#[derive(Clone)]
pub(super) struct TextState {
    pub(super) content: Arc<str>,
    pub(super) style: TextStyle,
    pub(super) layout: Option<Arc<TextLayout>>,
    pub(super) revision: i32,
}

impl TextState {
    pub(super) fn new(content: &str, style: TextStyle) -> Self {
        Self {
            content: Arc::from(content),
            style,
            layout: None,
            revision: 0,
        }
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
    if node.style.width == 0 || node.style.height == 0 {
        text.layout = None;
        return Ok(());
    }
    let size = Size::new(node.style.width as u32, node.style.height as u32);
    let request = TextRequest::new(&text.content, text.style, size, limits)?;
    let layout = engine.layout(request)?;
    layout.validate_request(request)?;
    text.layout = Some(Arc::new(layout));
    Ok(())
}

pub(super) fn prepare_frame(
    committed: &CommittedRuntimeSnapshot,
    nodes: &[NamedNode],
    spatial_limits: SpatialLimitsV2,
    limits: Limits,
) -> Result<Option<SpatialResolvedSnapshotV2>, Error> {
    let spatial = committed
        .spatial()
        .ok_or_else(|| Error::InvalidProgram("missing spatial frame".into()))?;
    let mut additions = Vec::new();
    for node in nodes {
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
        let owner = spatial
            .spatial_key(node.id)
            .ok_or_else(|| Error::InvalidProgram("missing text owner".into()))?;
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
            limits.text().max_pixels()
        }
        SpatialLimitKindV2::PaintItemsPerNode => 2,
        _ => spatial_limits.limit(kind),
    });
    spatial
        .snapshot()
        .with_image_paints(additions.into_boxed_slice(), SpatialLimitsV2::new(extended))
        .map(Some)
        .map_err(|error| Error::Runtime(error.to_string()))
}
