use fenestra_ui_ir::prototype::{
    SpatialBrushContentV2, SpatialBrushDeclarationV2, SpatialBrushSymbolV2, SpatialFieldV2,
    SpatialGradientStopV2, SpatialPathVerbRecipeV2, SpatialPolygonPointV2,
    SpatialShapeDeclarationV2, SpatialShapeGeometryV2, SpatialShapeSymbolV2, ValueType,
};

use crate::diagnostic_v2::{AuthoringDiagnosticKindV2, AuthoringDiagnosticV2};
use crate::limits_v2::AuthoringLimitKindV2;
use crate::parsed_v2::{
    ParsedBrushContentV2, ParsedBrushV2, ParsedNameFieldV2, ParsedPathVerbKindV2, ParsedPathVerbV2,
    ParsedShapeGeometryV2, ParsedShapeV2,
};
use crate::resolved::logical_span;

use super::SpatialLowerer;

impl SpatialLowerer<'_> {
    pub(super) fn lower_shapes(
        &self,
        owner: usize,
        component: u32,
    ) -> Result<Vec<SpatialShapeDeclarationV2>, AuthoringDiagnosticV2> {
        self.nodes[owner]
            .node
            .shapes
            .iter()
            .enumerate()
            .map(|(index, shape)| self.lower_shape(owner, component, index, shape))
            .collect()
    }

    fn lower_shape(
        &self,
        owner: usize,
        component: u32,
        index: usize,
        shape: &ParsedShapeV2,
    ) -> Result<SpatialShapeDeclarationV2, AuthoringDiagnosticV2> {
        if self.owner_symbols[owner].shapes.is_duplicate(index) {
            return Err(self.error(
                AuthoringDiagnosticKindV2::DuplicateSpatialShapeName,
                shape.symbol.anchor,
            ));
        }
        let symbol = self.shape_symbol_field(&shape.symbol, index)?;
        let geometry = match &shape.geometry {
            ParsedShapeGeometryV2::Rect {
                origin,
                width,
                height,
            } => SpatialShapeGeometryV2::Rect {
                origin: self.lower_point(origin, component)?,
                width: self.binding_field(width, component, ValueType::ScalarI32)?,
                height: self.binding_field(height, component, ValueType::ScalarI32)?,
            },
            ParsedShapeGeometryV2::Circle { center, radius } => SpatialShapeGeometryV2::Circle {
                center: self.lower_point(center, component)?,
                radius: self.binding_field(radius, component, ValueType::ScalarI32)?,
            },
            ParsedShapeGeometryV2::Polygon(points) => SpatialShapeGeometryV2::Polygon {
                points: points
                    .iter()
                    .map(|point| {
                        Ok(SpatialPolygonPointV2::new(
                            self.lower_point(&point.point, component)?,
                            logical_span(point.anchor),
                        ))
                    })
                    .collect::<Result<Vec<_>, AuthoringDiagnosticV2>>()?,
            },
            ParsedShapeGeometryV2::Path(verbs) => SpatialShapeGeometryV2::Path {
                verbs: verbs
                    .iter()
                    .map(|verb| self.lower_path_verb(verb, component))
                    .collect::<Result<Vec<_>, _>>()?,
            },
        };
        Ok(SpatialShapeDeclarationV2::new(
            symbol,
            geometry,
            logical_span(shape.anchor),
        ))
    }

    fn lower_path_verb(
        &self,
        verb: &ParsedPathVerbV2,
        component: u32,
    ) -> Result<SpatialPathVerbRecipeV2, AuthoringDiagnosticV2> {
        let span = logical_span(verb.anchor);
        Ok(match &verb.kind {
            ParsedPathVerbKindV2::MoveTo(to) => SpatialPathVerbRecipeV2::MoveTo {
                to: self.lower_point(to, component)?,
                span,
            },
            ParsedPathVerbKindV2::LineTo(to) => SpatialPathVerbRecipeV2::LineTo {
                to: self.lower_point(to, component)?,
                span,
            },
            ParsedPathVerbKindV2::QuadraticTo { control, to } => {
                SpatialPathVerbRecipeV2::QuadraticTo {
                    control: self.lower_point(control, component)?,
                    to: self.lower_point(to, component)?,
                    span,
                }
            }
            ParsedPathVerbKindV2::CubicTo {
                control1,
                control2,
                to,
            } => SpatialPathVerbRecipeV2::CubicTo {
                control1: self.lower_point(control1, component)?,
                control2: self.lower_point(control2, component)?,
                to: self.lower_point(to, component)?,
                span,
            },
            ParsedPathVerbKindV2::Close => SpatialPathVerbRecipeV2::Close { span },
        })
    }

    pub(super) fn lower_brushes(
        &self,
        owner: usize,
        component: u32,
    ) -> Result<Vec<SpatialBrushDeclarationV2>, AuthoringDiagnosticV2> {
        self.nodes[owner]
            .node
            .brushes
            .iter()
            .enumerate()
            .map(|(index, brush)| self.lower_brush(owner, component, index, brush))
            .collect()
    }

    fn lower_brush(
        &self,
        owner: usize,
        component: u32,
        index: usize,
        brush: &ParsedBrushV2,
    ) -> Result<SpatialBrushDeclarationV2, AuthoringDiagnosticV2> {
        if self.owner_symbols[owner].brushes.is_duplicate(index) {
            return Err(self.error(
                AuthoringDiagnosticKindV2::DuplicateSpatialBrushName,
                brush.symbol.anchor,
            ));
        }
        let symbol = self.brush_symbol_field(&brush.symbol, index)?;
        let content = match &brush.content {
            ParsedBrushContentV2::Solid(color) => SpatialBrushContentV2::Solid {
                color: self.binding_field(color, component, ValueType::Rgba8)?,
            },
            ParsedBrushContentV2::LinearGradient { start, end, stops } => {
                SpatialBrushContentV2::LinearGradient {
                    start: self.lower_point(start, component)?,
                    end: self.lower_point(end, component)?,
                    stops: stops
                        .iter()
                        .map(|stop| {
                            Ok(SpatialGradientStopV2::new(
                                self.literal_field(&stop.offset)?,
                                self.binding_field(&stop.color, component, ValueType::Rgba8)?,
                                logical_span(stop.anchor),
                            ))
                        })
                        .collect::<Result<Vec<_>, AuthoringDiagnosticV2>>()?,
                }
            }
        };
        Ok(SpatialBrushDeclarationV2::new(
            symbol,
            content,
            logical_span(brush.anchor),
        ))
    }

    pub(super) fn resolve_shape(
        &self,
        owner: usize,
        field: &ParsedNameFieldV2,
    ) -> Result<SpatialFieldV2<SpatialShapeSymbolV2>, AuthoringDiagnosticV2> {
        let index = self.owner_symbols[owner]
            .shapes
            .get(&field.value)
            .ok_or_else(|| {
                self.error(
                    AuthoringDiagnosticKindV2::UnknownSpatialShapeName,
                    field.anchor,
                )
            })?;
        self.shape_symbol_field(field, index)
    }

    pub(super) fn resolve_brush(
        &self,
        owner: usize,
        field: &ParsedNameFieldV2,
    ) -> Result<SpatialFieldV2<SpatialBrushSymbolV2>, AuthoringDiagnosticV2> {
        let index = self.owner_symbols[owner]
            .brushes
            .get(&field.value)
            .ok_or_else(|| {
                self.error(
                    AuthoringDiagnosticKindV2::UnknownSpatialBrushName,
                    field.anchor,
                )
            })?;
        self.brush_symbol_field(field, index)
    }

    fn shape_symbol_field(
        &self,
        field: &ParsedNameFieldV2,
        index: usize,
    ) -> Result<SpatialFieldV2<SpatialShapeSymbolV2>, AuthoringDiagnosticV2> {
        let value = self.dense_symbol(index, AuthoringLimitKindV2::Shapes, field.anchor)?;
        Ok(self.field_value(field.anchor, SpatialShapeSymbolV2::new(value)))
    }

    fn brush_symbol_field(
        &self,
        field: &ParsedNameFieldV2,
        index: usize,
    ) -> Result<SpatialFieldV2<SpatialBrushSymbolV2>, AuthoringDiagnosticV2> {
        let value = self.dense_symbol(index, AuthoringLimitKindV2::Brushes, field.anchor)?;
        Ok(self.field_value(field.anchor, SpatialBrushSymbolV2::new(value)))
    }
}
