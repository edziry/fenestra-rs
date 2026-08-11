//! Raw local-transform validation after Layout input preparation.

use super::islands::preflight::LayoutPreflightProof;
use super::make_resolve_error;
use super::topology::trusted_node_ordinal;
use crate::aggregate_input::SpatialInputV2;
use crate::error::SpatialErrorLocationV2;
use crate::limits::SpatialLimitsV2;
use crate::numeric_error::SpatialTransformErrorKindV2;
use crate::resolve_error::{SpatialResolveErrorKindV2, SpatialResolveErrorV2};
use crate::topology::SpatialPlacementV2;
use crate::vocabulary::{SpatialNodeFieldV2, SpatialTransformScalarFieldV2};

pub(super) struct LocalTransformProof<'a> {
    preflight: LayoutPreflightProof<'a>,
}

impl<'a> LocalTransformProof<'a> {
    pub(super) fn input(&self) -> SpatialInputV2<'a> {
        self.preflight.input()
    }

    pub(super) fn limits(&self) -> SpatialLimitsV2 {
        self.preflight.limits()
    }
}

pub(super) fn prepare_local_transforms(
    preflight: LayoutPreflightProof<'_>,
) -> Result<LocalTransformProof<'_>, SpatialResolveErrorV2> {
    let nodes = preflight.input().topology().nodes();

    for (index, node) in nodes.iter().copied().enumerate().skip(1) {
        let transform = match node.placement() {
            SpatialPlacementV2::Layout(layout) => layout.transform(),
            SpatialPlacementV2::Free(free) => free.transform(),
            SpatialPlacementV2::Root => {
                unreachable!("phase four rejected Root placement on nonroot nodes")
            }
        };
        let affine = transform.affine();
        let scalar_fields = [
            (affine.a(), SpatialNodeFieldV2::AffineA),
            (affine.b(), SpatialNodeFieldV2::AffineB),
            (affine.c(), SpatialNodeFieldV2::AffineC),
            (affine.d(), SpatialNodeFieldV2::AffineD),
            (affine.tx(), SpatialNodeFieldV2::AffineTx),
            (affine.ty(), SpatialNodeFieldV2::AffineTy),
            (transform.origin().x(), SpatialNodeFieldV2::TransformOriginX),
            (transform.origin().y(), SpatialNodeFieldV2::TransformOriginY),
        ];
        let node_index = trusted_node_ordinal(index);

        for (field, (scalar, node_field)) in SpatialTransformScalarFieldV2::ALL
            .into_iter()
            .zip(scalar_fields)
        {
            if !scalar.is_in_domain() {
                return Err(transform_error(
                    SpatialTransformErrorKindV2::ScalarOutOfDomain(field),
                    SpatialErrorLocationV2::NodeField {
                        index: node_index,
                        field: node_field,
                    },
                ));
            }
        }

        if affine.determinant_raw() == 0 {
            return Err(transform_error(
                SpatialTransformErrorKindV2::SingularTransform,
                SpatialErrorLocationV2::Node { index: node_index },
            ));
        }
    }

    Ok(LocalTransformProof { preflight })
}

fn transform_error(
    kind: SpatialTransformErrorKindV2,
    location: SpatialErrorLocationV2,
) -> SpatialResolveErrorV2 {
    make_resolve_error(SpatialResolveErrorKindV2::Transform(kind), location)
}

#[cfg(test)]
impl LocalTransformProof<'_> {
    pub(super) fn prepared_island_facts(&self) -> Vec<(u32, Vec<u32>)> {
        self.preflight.prepared_island_facts()
    }
}
