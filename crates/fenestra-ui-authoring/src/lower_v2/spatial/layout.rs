use fenestra_ui_ir::prototype::{
    SpatialAnchorTargetRecipeV2, SpatialContainerRecipeV2, SpatialDimensionRecipeV2,
    SpatialFreePlacementRecipeV2, SpatialImageDeclarationV2, SpatialLayoutPlacementRecipeV2,
    SpatialPaddingRecipeV2, SpatialPlacementRecipeV2, SpatialPointRecipeV2,
    SpatialTransformRecipeV2, SpatialViewportContainerV2, ValueType,
};

use crate::diagnostic_v2::{AuthoringDiagnosticKindV2, AuthoringDiagnosticV2};
use crate::parsed_v2::{
    ParsedDimensionV2, ParsedImageV2, ParsedNodeV2, ParsedPointV2, ParsedTransformV2,
};
use crate::resolved::logical_span;

use super::SpatialLowerer;

impl SpatialLowerer<'_> {
    pub(super) fn lower_viewport(
        &self,
    ) -> Result<SpatialViewportContainerV2, AuthoringDiagnosticV2> {
        let viewport = &self.parsed.spatial.viewport;
        Ok(SpatialViewportContainerV2::new(
            viewport.axis,
            self.literal_field(&viewport.left)?,
            self.literal_field(&viewport.right)?,
            self.literal_field(&viewport.top)?,
            self.literal_field(&viewport.bottom)?,
            self.literal_field(&viewport.gap)?,
            logical_span(viewport.anchor),
        ))
    }

    pub(super) fn lower_images(
        &self,
    ) -> Result<Vec<SpatialImageDeclarationV2>, AuthoringDiagnosticV2> {
        self.parsed
            .spatial
            .images
            .iter()
            .enumerate()
            .map(|(index, image)| self.lower_image(index, image))
            .collect()
    }

    fn lower_image(
        &self,
        index: usize,
        image: &ParsedImageV2,
    ) -> Result<SpatialImageDeclarationV2, AuthoringDiagnosticV2> {
        if self.image_symbols.is_duplicate(index) {
            return Err(self.error(
                AuthoringDiagnosticKindV2::DuplicateSpatialImageName,
                image.symbol.anchor,
            ));
        }
        let symbol = self.image_symbol_field(&image.symbol, index)?;
        let width = self.literal_field(&image.width)?;
        let height = self.literal_field(&image.height)?;
        let stride = self.literal_field(&image.stride)?;
        let bytes = image
            .bytes
            .iter()
            .map(|byte| self.literal(byte, image.anchor))
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice();
        Ok(SpatialImageDeclarationV2::new(
            symbol,
            width,
            height,
            stride,
            bytes,
            logical_span(image.anchor),
        ))
    }

    pub(super) fn lower_container(
        &self,
        node: &ParsedNodeV2,
        component: u32,
    ) -> Result<SpatialContainerRecipeV2, AuthoringDiagnosticV2> {
        let container = &node.container;
        Ok(SpatialContainerRecipeV2::new(
            container.axis,
            SpatialPaddingRecipeV2::new(
                self.binding_field(&container.padding.left, component, ValueType::ScalarI32)?,
                self.binding_field(&container.padding.right, component, ValueType::ScalarI32)?,
                self.binding_field(&container.padding.top, component, ValueType::ScalarI32)?,
                self.binding_field(&container.padding.bottom, component, ValueType::ScalarI32)?,
            ),
            self.binding_field(&container.gap, component, ValueType::ScalarI32)?,
        ))
    }

    pub(super) fn lower_placement(
        &self,
        node: &ParsedNodeV2,
        component: u32,
    ) -> Result<SpatialPlacementRecipeV2, AuthoringDiagnosticV2> {
        let placement = match &node.placement {
            crate::parsed_v2::ParsedPlacementV2::Layout { width, height, .. } => {
                SpatialPlacementRecipeV2::Layout(SpatialLayoutPlacementRecipeV2::new(
                    self.lower_dimension(width, component)?,
                    self.lower_dimension(height, component)?,
                    self.lower_transform(&node.transform, component)?,
                ))
            }
            crate::parsed_v2::ParsedPlacementV2::Free {
                width,
                height,
                self_anchor,
                target,
                target_anchor,
                offset,
                ..
            } => {
                let target = match target {
                    crate::parsed_v2::ParsedAnchorTargetV2::Viewport => {
                        SpatialAnchorTargetRecipeV2::Viewport
                    }
                    crate::parsed_v2::ParsedAnchorTargetV2::Parent => {
                        SpatialAnchorTargetRecipeV2::Parent
                    }
                    crate::parsed_v2::ParsedAnchorTargetV2::Node(field) => {
                        SpatialAnchorTargetRecipeV2::Node(self.resolve_node(field)?)
                    }
                };
                SpatialPlacementRecipeV2::Free(SpatialFreePlacementRecipeV2::new(
                    self.binding_field(width, component, ValueType::ScalarI32)?,
                    self.binding_field(height, component, ValueType::ScalarI32)?,
                    [self_anchor.horizontal, self_anchor.vertical],
                    target,
                    [target_anchor.horizontal, target_anchor.vertical],
                    self.lower_point(offset, component)?,
                    self.lower_transform(&node.transform, component)?,
                ))
            }
        };
        Ok(placement)
    }

    pub(super) fn lower_point(
        &self,
        point: &ParsedPointV2,
        component: u32,
    ) -> Result<SpatialPointRecipeV2, AuthoringDiagnosticV2> {
        Ok(SpatialPointRecipeV2::new(
            self.binding_field(&point.x, component, ValueType::ScalarI32)?,
            self.binding_field(&point.y, component, ValueType::ScalarI32)?,
        ))
    }

    fn lower_dimension(
        &self,
        dimension: &ParsedDimensionV2,
        component: u32,
    ) -> Result<SpatialDimensionRecipeV2, AuthoringDiagnosticV2> {
        Ok(SpatialDimensionRecipeV2::new(
            self.binding_field(&dimension.minimum, component, ValueType::ScalarI32)?,
            self.binding_field(&dimension.preferred, component, ValueType::ScalarI32)?,
            self.binding_field(&dimension.maximum, component, ValueType::ScalarI32)?,
        ))
    }

    fn lower_transform(
        &self,
        transform: &ParsedTransformV2,
        component: u32,
    ) -> Result<SpatialTransformRecipeV2, AuthoringDiagnosticV2> {
        if let Some(physical) = transform.invalid_turn {
            return Err(self.error_at(
                AuthoringDiagnosticKindV2::InvalidLiteral,
                transform.anchor,
                physical,
            ));
        }
        let [a, b, c, d, tx, ty] = transform
            .coefficients
            .as_ref()
            .expect("valid parsed transforms must retain six coefficients");
        Ok(SpatialTransformRecipeV2::new(
            self.binding_field(a, component, ValueType::ScalarI32)?,
            self.binding_field(b, component, ValueType::ScalarI32)?,
            self.binding_field(c, component, ValueType::ScalarI32)?,
            self.binding_field(d, component, ValueType::ScalarI32)?,
            self.binding_field(tx, component, ValueType::ScalarI32)?,
            self.binding_field(ty, component, ValueType::ScalarI32)?,
            self.lower_point(&transform.origin, component)?,
        ))
    }
}
