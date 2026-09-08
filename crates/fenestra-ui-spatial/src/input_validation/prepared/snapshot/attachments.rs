//! Owned image additions bound to retained spatial geometry and painter order.

use std::fmt;
use std::sync::Arc;

use fenestra_ui_layout::prototype::ReferenceStackEngineV1;

use super::SpatialResolvedSnapshotV2;
use crate::aggregate_input::SpatialInputV2;
use crate::content_key::SpatialImageKeyV2;
use crate::direct_counts::preflight_spatial_direct_counts_v2;
use crate::geometry_key::SpatialClipKeyV2;
use crate::image::{SpatialImageDestinationRectV2, SpatialImageSourceRectV2, SpatialImageV2};
use crate::input_validation::paint_p4_mapping::map_image_p4_error;
use crate::input_validation::resolve_spatial_v2;
use crate::limits::{SpatialLimitKindV2, SpatialLimitsV2};
use crate::model::SpatialNodeKeyV2;
use crate::owned_input::SpatialOwnedInputV2;
use crate::paint::{SpatialPaintContentV2, SpatialPaintV2};
use crate::paint_kernel::prepare_image_p4;
use crate::resolve_error::SpatialResolveErrorV2;

/// Owned full-image paint appended after an owner's existing paint items.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpatialImagePaintAttachmentV2 {
    owner: SpatialNodeKeyV2,
    image: SpatialImageV2,
    destination: SpatialImageDestinationRectV2,
    clip: Option<SpatialClipKeyV2>,
}

impl SpatialImagePaintAttachmentV2 {
    /// Takes ownership of one raw image and its owner-local destination.
    ///
    /// The supplied image key is discarded and assigned densely when attached.
    /// The full source image is painted at opacity 255; its premultiplied alpha
    /// remains effective. A clip must belong to the owner or one of its ancestors.
    #[must_use]
    pub const fn new(
        owner: SpatialNodeKeyV2,
        image: SpatialImageV2,
        destination: SpatialImageDestinationRectV2,
        clip: Option<SpatialClipKeyV2>,
    ) -> Self {
        Self {
            owner,
            image,
            destination,
            clip,
        }
    }
}

/// Failure to attach images without changing accepted non-paint geometry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialImageAttachmentErrorV2 {
    /// Existing spatial validation rejected the complete augmented input.
    Resolve(SpatialResolveErrorV2),
    /// Reference resolution differs from the snapshot's accepted geometry.
    GeometryChanged,
}

impl From<SpatialResolveErrorV2> for SpatialImageAttachmentErrorV2 {
    fn from(error: SpatialResolveErrorV2) -> Self {
        Self::Resolve(error)
    }
}

impl fmt::Display for SpatialImageAttachmentErrorV2 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Resolve(error) => error.fmt(formatter),
            Self::GeometryChanged => {
                formatter.write_str("image attachment changed accepted geometry")
            }
        }
    }
}

impl std::error::Error for SpatialImageAttachmentErrorV2 {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Resolve(error) => Some(error),
            Self::GeometryChanged => None,
        }
    }
}

impl SpatialResolvedSnapshotV2 {
    /// Returns an owned snapshot with images appended in owner painter order.
    ///
    /// Existing paints precede additions for their owner, and additions for the
    /// same owner retain their supplied order. Other owners keep their accepted
    /// order. The original snapshot remains intact on success or rejection.
    /// All resources and items are validated against the supplied total limits.
    /// Reference layout must reproduce the accepted viewport, geometry, clips,
    /// hits and semantics exactly; a divergent injected layout is rejected.
    #[must_use = "image attachment errors must be handled before publication"]
    pub fn with_image_paints(
        &self,
        additions: Box<[SpatialImagePaintAttachmentV2]>,
        limits: SpatialLimitsV2,
    ) -> Result<Self, SpatialImageAttachmentErrorV2> {
        let input = self.prepared.source.as_input();
        preflight(input, additions.len(), limits)?;
        let mut additions = additions.into_vec();
        for (index, addition) in additions.iter_mut().enumerate() {
            let key = u32::try_from(input.resources().images().len() + index)
                .expect("direct-count preflight bounds every image key");
            addition.image.set_key(SpatialImageKeyV2::new(key));
        }
        validate_images(input, &additions, limits)?;
        let source = augmented_source(input, additions);
        let augmented =
            resolve_spatial_v2(&ReferenceStackEngineV1::new(), Arc::new(source), limits)?;
        if self.viewport() != augmented.viewport()
            || self.geometry != augmented.geometry
            || self.clips != augmented.clips
            || self.hits != augmented.hits
            || self.semantics != augmented.semantics
            || self.effective_clip_aabbs() != augmented.effective_clip_aabbs()
        {
            return Err(SpatialImageAttachmentErrorV2::GeometryChanged);
        }
        Ok(augmented)
    }
}

fn preflight(
    input: SpatialInputV2<'_>,
    additions: usize,
    limits: SpatialLimitsV2,
) -> Result<(), SpatialResolveErrorV2> {
    let geometry = input.geometry();
    let resources = input.resources();
    let items = input.items();
    preflight_spatial_direct_counts_v2(
        [
            input.topology().nodes().len() as u128,
            geometry.shapes().len() as u128,
            resources.brushes().len() as u128,
            geometry.clips().len() as u128,
            items.paint_items().len() as u128 + additions as u128,
            items.hit_items().len() as u128,
            items.semantic_items().len() as u128,
            geometry.paths().len() as u128,
            geometry.path_verbs().len() as u128,
            geometry.polygon_points().len() as u128,
            resources.gradient_stops().len() as u128,
            resources.images().len() as u128 + additions as u128,
        ],
        limits,
    )
}

fn validate_images(
    input: SpatialInputV2<'_>,
    additions: &[SpatialImagePaintAttachmentV2],
    limits: SpatialLimitsV2,
) -> Result<(), SpatialResolveErrorV2> {
    let mut pixels = 0;
    for image in input
        .resources()
        .images()
        .iter()
        .chain(additions.iter().map(|item| &item.image))
    {
        prepare_image_p4(
            image,
            &mut pixels,
            limits.limit(SpatialLimitKindV2::ImageEdge),
            limits.limit(SpatialLimitKindV2::ImagePixelsTotal),
        )
        .map_err(map_image_p4_error)?;
    }
    Ok(())
}

fn augmented_source(
    input: SpatialInputV2<'_>,
    additions: Vec<SpatialImagePaintAttachmentV2>,
) -> SpatialOwnedInputV2 {
    let mut images = input.resources().images().to_vec();
    let mut paints = input.items().paint_items().to_vec();
    for addition in additions {
        let source =
            SpatialImageSourceRectV2::new(0, 0, addition.image.width(), addition.image.height());
        paints.push(SpatialPaintV2::new(
            addition.owner,
            0,
            SpatialPaintContentV2::ImagePaint {
                image: addition.image.key(),
                source,
                destination: addition.destination,
                opacity: 255,
                clip: addition.clip,
            },
        ));
        images.push(addition.image);
    }
    // Stable sorting preserves existing owner-local order before new additions.
    paints.sort_by_key(|paint| paint.owner());
    let mut previous_owner = None;
    let mut ordinal = 0;
    for paint in &mut paints {
        if previous_owner == Some(paint.owner()) {
            ordinal += 1;
        } else {
            previous_owner = Some(paint.owner());
            ordinal = 0;
        }
        *paint = SpatialPaintV2::new(paint.owner(), ordinal, paint.content());
    }
    let geometry = input.geometry();
    let resources = input.resources();
    let items = input.items();
    SpatialOwnedInputV2::new(
        input.topology().viewport(),
        input.topology().nodes().to_vec().into_boxed_slice(),
        geometry.polygon_points().to_vec().into_boxed_slice(),
        geometry.path_verbs().to_vec().into_boxed_slice(),
        geometry.paths().to_vec().into_boxed_slice(),
        geometry.shapes().to_vec().into_boxed_slice(),
        geometry.clips().to_vec().into_boxed_slice(),
        resources.gradient_stops().to_vec().into_boxed_slice(),
        resources.brushes().to_vec().into_boxed_slice(),
        images.into_boxed_slice(),
        paints.into_boxed_slice(),
        items.hit_items().to_vec().into_boxed_slice(),
        items.semantic_items().to_vec().into_boxed_slice(),
    )
}
