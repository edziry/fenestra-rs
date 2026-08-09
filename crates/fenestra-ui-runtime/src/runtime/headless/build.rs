use std::collections::HashSet;

use fenestra_ui_ir::prototype::{InputPolicy, PropertyId, PropertyValue};

use super::types::{
    ComputedProperty, ComputedStyleRecord, GeometryRecord, HeadlessProjectionFailure,
    HeadlessProjectionState, HeadlessRuntimeConfig, HitRegionRecord, SceneRectangleRecord,
    SemanticRecord,
};
use super::{
    HeadlessProjectionErrorKind, HeadlessProjectionLimitKind, HeadlessRect, HeadlessSemanticAction,
    HeadlessSemanticRole, HeadlessSurface,
};
use crate::logical_tree::NodeId;
use crate::runtime::state::{RuntimeNode, RuntimeState};

#[derive(Clone, Copy)]
struct ProjectionFrame {
    node: NodeId,
    bounds: HeadlessRect,
    clip: HeadlessRect,
    visible: bool,
}

#[derive(Clone, Copy, Default)]
struct DerivedCounts {
    semantics: usize,
    hit_regions: usize,
    scene_rectangles: usize,
}

struct DerivedRecords {
    semantics: Vec<SemanticRecord>,
    hit_regions: Vec<HitRegionRecord>,
    scene_rectangles: Vec<SceneRectangleRecord>,
}

impl RuntimeState {
    pub(crate) fn build_headless_projection(
        &self,
        config: &HeadlessRuntimeConfig,
        surface: HeadlessSurface,
    ) -> Result<HeadlessProjectionState, HeadlessProjectionFailure> {
        surface.validate().map_err(HeadlessProjectionFailure::new)?;
        config
            .preflight_fixed_records(Some(self.tree.len()))
            .map_err(HeadlessProjectionFailure::new)?;
        self.scan_negative_geometry(config)?;
        let (computed_styles, geometry, counts) = self.build_fixed_projection(config, surface)?;
        preflight_derived(config, counts)?;
        let derived = self.build_derived_projection(config, &geometry, counts)?;
        Ok(HeadlessProjectionState {
            surface,
            computed_styles,
            geometry,
            semantics: derived.semantics,
            hit_regions: derived.hit_regions,
            scene_rectangles: derived.scene_rectangles,
        })
    }

    pub(crate) fn rebuild_headless_projection(
        &mut self,
        config: &HeadlessRuntimeConfig,
        surface: HeadlessSurface,
    ) -> Result<(), HeadlessProjectionFailure> {
        let projection = self.build_headless_projection(config, surface)?;
        self.headless = Some(projection);
        Ok(())
    }

    pub(crate) fn headless_surface(&self) -> Option<HeadlessSurface> {
        self.headless.as_ref().map(|projection| projection.surface)
    }

    fn scan_negative_geometry(
        &self,
        config: &HeadlessRuntimeConfig,
    ) -> Result<(), HeadlessProjectionFailure> {
        self.visit_authored_nodes(|node, stored| {
            for property_id in [config.spec.width(), config.spec.height()] {
                if scalar_property(stored, property_id)? < 0 {
                    return Err(HeadlessProjectionFailure::negative(node, property_id));
                }
            }
            Ok(())
        })
    }

    fn build_fixed_projection(
        &self,
        config: &HeadlessRuntimeConfig,
        surface: HeadlessSurface,
    ) -> Result<
        (Vec<ComputedStyleRecord>, Vec<GeometryRecord>, DerivedCounts),
        HeadlessProjectionFailure,
    > {
        let node_count = self.tree.len();
        let root = self.tree.root().ok_or_else(invariant)?;
        let root_node = self.tree.value(root).ok_or_else(invariant)?;
        let root_bounds = HeadlessRect::new(
            0,
            0,
            scalar_property(root_node, config.spec.width())?,
            scalar_property(root_node, config.spec.height())?,
        );
        let surface_bounds = HeadlessRect::new(0, 0, surface.width(), surface.height());
        let mut pending = vec![ProjectionFrame {
            node: root,
            bounds: root_bounds,
            clip: intersect(root_bounds, surface_bounds)?,
            visible: bool_property(root_node, config.spec.visible())?,
        }];
        let mut computed_styles = Vec::with_capacity(node_count);
        let mut geometry = Vec::with_capacity(node_count);
        let mut counts = DerivedCounts::default();
        while let Some(frame) = pending.pop() {
            let stored = self.tree.value(frame.node).ok_or_else(invariant)?;
            computed_styles.push(ComputedStyleRecord {
                node: frame.node,
                properties: stored
                    .properties
                    .iter()
                    .map(|property| ComputedProperty {
                        id: property.id,
                        value: property.value.clone(),
                    })
                    .collect(),
            });
            geometry.push(GeometryRecord {
                node: frame.node,
                bounds: frame.bounds,
                clip: frame.clip,
                effective_visible: frame.visible,
            });
            if frame.visible && frame.clip.is_non_empty() {
                counts.scene_rectangles = increment(counts.scene_rectangles)?;
                if stored.template == config.spec.semantic_template() {
                    counts.semantics = increment(counts.semantics)?;
                }
                if input_property(stored, config.spec.input())? == InputPolicy::Accept {
                    counts.hit_regions = increment(counts.hit_regions)?;
                }
            }

            let children = self.tree.children(frame.node).ok_or_else(invariant)?;
            let mut cursor = frame.bounds.y();
            let mut child_frames = Vec::with_capacity(children.len());
            for child in children {
                if self.tree.parent(*child) != Some(frame.node) {
                    return Err(invariant());
                }
                let child_node = self.tree.value(*child).ok_or_else(invariant)?;
                let height = scalar_property(child_node, config.spec.height())?;
                let bounds = HeadlessRect::new(
                    frame.bounds.x(),
                    cursor,
                    scalar_property(child_node, config.spec.width())?.min(frame.bounds.width()),
                    height,
                );
                child_frames.push(ProjectionFrame {
                    node: *child,
                    bounds,
                    clip: intersect(bounds, frame.clip)?,
                    visible: frame.visible && bool_property(child_node, config.spec.visible())?,
                });
                cursor = cursor.checked_add(height).ok_or_else(arithmetic)?;
            }
            pending.extend(child_frames.into_iter().rev());
        }
        if computed_styles.len() != node_count || geometry.len() != node_count {
            return Err(invariant());
        }
        Ok((computed_styles, geometry, counts))
    }

    fn build_derived_projection(
        &self,
        config: &HeadlessRuntimeConfig,
        geometry: &[GeometryRecord],
        counts: DerivedCounts,
    ) -> Result<DerivedRecords, HeadlessProjectionFailure> {
        let mut semantics = Vec::with_capacity(counts.semantics);
        let mut hit_regions = Vec::with_capacity(counts.hit_regions);
        let mut scene_rectangles = Vec::with_capacity(counts.scene_rectangles);
        for record in geometry {
            if !record.effective_visible || !record.clip.is_non_empty() {
                continue;
            }
            let stored = self.tree.value(record.node).ok_or_else(invariant)?;
            if stored.template == config.spec.semantic_template() {
                semantics.push(SemanticRecord {
                    node: record.node,
                    role: HeadlessSemanticRole::Control,
                    label: config.spec.semantic_label(),
                    action: HeadlessSemanticAction::Activate,
                });
            }
            if input_property(stored, config.spec.input())? == InputPolicy::Accept {
                hit_regions.push(HitRegionRecord {
                    node: record.node,
                    clip: record.clip,
                });
            }
            scene_rectangles.push(SceneRectangleRecord {
                node: record.node,
                rectangle: record.clip,
                color: color_property(stored, config.spec.color())?,
            });
        }
        if semantics.len() != counts.semantics
            || hit_regions.len() != counts.hit_regions
            || scene_rectangles.len() != counts.scene_rectangles
        {
            return Err(invariant());
        }
        Ok(DerivedRecords {
            semantics,
            hit_regions,
            scene_rectangles,
        })
    }

    fn visit_authored_nodes(
        &self,
        mut visit: impl FnMut(NodeId, &RuntimeNode) -> Result<(), HeadlessProjectionFailure>,
    ) -> Result<(), HeadlessProjectionFailure> {
        let root = self.tree.root().ok_or_else(invariant)?;
        let mut pending = vec![root];
        let mut seen = HashSet::with_capacity(self.tree.len());
        while let Some(node) = pending.pop() {
            if !seen.insert(node) {
                return Err(invariant());
            }
            let stored = self.tree.value(node).ok_or_else(invariant)?;
            visit(node, stored)?;
            let children = self.tree.children(node).ok_or_else(invariant)?;
            for child in children.iter().rev() {
                if self.tree.parent(*child) != Some(node) {
                    return Err(invariant());
                }
                pending.push(*child);
            }
        }
        if seen.len() != self.tree.len() {
            return Err(invariant());
        }
        Ok(())
    }
}

fn preflight_derived(
    config: &HeadlessRuntimeConfig,
    counts: DerivedCounts,
) -> Result<(), HeadlessProjectionFailure> {
    let capacity = config.spec.capacity();
    for (count, limit, kind) in [
        (
            counts.semantics,
            capacity.semantics(),
            HeadlessProjectionLimitKind::Semantics,
        ),
        (
            counts.hit_regions,
            capacity.hit_regions(),
            HeadlessProjectionLimitKind::HitRegions,
        ),
        (
            counts.scene_rectangles,
            capacity.scene_rectangles(),
            HeadlessProjectionLimitKind::SceneRectangles,
        ),
    ] {
        if count > limit {
            return Err(HeadlessProjectionFailure::new(
                HeadlessProjectionErrorKind::CapacityExceeded(kind),
            ));
        }
    }
    Ok(())
}

fn property(
    node: &RuntimeNode,
    property: PropertyId,
) -> Result<&PropertyValue, HeadlessProjectionFailure> {
    node.properties
        .iter()
        .find(|candidate| candidate.id == property)
        .map(|candidate| &candidate.value)
        .ok_or_else(invariant)
}

fn scalar_property(
    node: &RuntimeNode,
    property_id: PropertyId,
) -> Result<i32, HeadlessProjectionFailure> {
    match property(node, property_id)? {
        PropertyValue::ScalarI32(value) => Ok(*value),
        _ => Err(invariant()),
    }
}

fn bool_property(
    node: &RuntimeNode,
    property_id: PropertyId,
) -> Result<bool, HeadlessProjectionFailure> {
    match property(node, property_id)? {
        PropertyValue::Bool(value) => Ok(*value),
        _ => Err(invariant()),
    }
}

fn color_property(
    node: &RuntimeNode,
    property_id: PropertyId,
) -> Result<[u8; 4], HeadlessProjectionFailure> {
    match property(node, property_id)? {
        PropertyValue::Rgba8(value) => Ok(*value),
        _ => Err(invariant()),
    }
}

fn input_property(
    node: &RuntimeNode,
    property_id: PropertyId,
) -> Result<InputPolicy, HeadlessProjectionFailure> {
    match property(node, property_id)? {
        PropertyValue::InputPolicy(value) => Ok(*value),
        _ => Err(invariant()),
    }
}

fn intersect(
    first: HeadlessRect,
    second: HeadlessRect,
) -> Result<HeadlessRect, HeadlessProjectionFailure> {
    let x = first.x().max(second.x());
    let y = first.y().max(second.y());
    let right =
        rectangle_end(first.x(), first.width())?.min(rectangle_end(second.x(), second.width())?);
    let bottom =
        rectangle_end(first.y(), first.height())?.min(rectangle_end(second.y(), second.height())?);
    let width = if right <= x {
        0
    } else {
        right.checked_sub(x).ok_or_else(arithmetic)?
    };
    let height = if bottom <= y {
        0
    } else {
        bottom.checked_sub(y).ok_or_else(arithmetic)?
    };
    Ok(HeadlessRect::new(x, y, width, height))
}

fn rectangle_end(origin: i32, extent: i32) -> Result<i32, HeadlessProjectionFailure> {
    origin.checked_add(extent).ok_or_else(arithmetic)
}

fn increment(value: usize) -> Result<usize, HeadlessProjectionFailure> {
    value.checked_add(1).ok_or_else(arithmetic)
}

const fn invariant() -> HeadlessProjectionFailure {
    HeadlessProjectionFailure::new(HeadlessProjectionErrorKind::InvariantViolation)
}

const fn arithmetic() -> HeadlessProjectionFailure {
    HeadlessProjectionFailure::new(HeadlessProjectionErrorKind::ArithmeticExhausted)
}
