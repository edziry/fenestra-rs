use std::collections::BTreeMap;

use fenestra_ui_ir::prototype::{InputPolicy, PropertyId, PropertyValue, TemplateNodeId};

use crate::error::{HarnessError, HarnessErrorKind, HarnessLimitKind};
use crate::fixture::HarnessLimitsV1;
use crate::model::clean_rebuild_v1;
use crate::semantic::{NodePathV1, NormalizedNodeV1};

use super::RebuildInput;
use super::types::{
    NormalizedHeadlessComputedStyleV1, NormalizedHeadlessGeometryV1, NormalizedHeadlessHitRegionV1,
    NormalizedHeadlessProjectionV1, NormalizedHeadlessSceneRectangleV1,
    NormalizedHeadlessSemanticV1, ProjectionRect, rect, semantic_action, semantic_role,
};

#[derive(Clone)]
struct Frame {
    bounds: ProjectionRect,
    clip: ProjectionRect,
    visible: bool,
}

pub(super) fn rebuild(
    input: RebuildInput<'_>,
    limits: HarnessLimitsV1,
) -> Result<NormalizedHeadlessProjectionV1, HarnessError> {
    if input.surface.width() < 0 || input.surface.height() < 0 {
        return Err(state_error());
    }
    let logical = clean_rebuild_v1(input.style.construction(), input.desired, limits)?;
    let count = logical.node_count();
    let capacity = input.spec.capacity();
    ensure_limit(
        count,
        capacity.computed_styles(),
        HarnessLimitKind::NormalizedNodes,
    )?;
    ensure_limit(
        count,
        capacity.geometry(),
        HarnessLimitKind::NormalizedNodes,
    )?;

    let mut computed_styles = Vec::with_capacity(count);
    for node in logical.nodes() {
        computed_styles.push(materialize(input, node)?);
    }
    if computed_styles
        .iter()
        .any(|record| record.width < 0 || record.height < 0)
    {
        return Err(state_error());
    }

    let (geometries, effective_visibility) = layout(input, logical.nodes(), &computed_styles)?;
    let mut semantic_count = 0_usize;
    let mut hit_count = 0_usize;
    let mut scene_count = 0_usize;
    for (((node, computed), geometry), visible) in logical
        .nodes()
        .iter()
        .zip(&computed_styles)
        .zip(&geometries)
        .zip(&effective_visibility)
    {
        if !*visible || !non_empty(geometry.clip) {
            continue;
        }
        scene_count = increment(scene_count)?;
        if node.template() == input.spec.semantic_template() {
            semantic_count = increment(semantic_count)?;
        }
        if computed.input == InputPolicy::Accept {
            hit_count = increment(hit_count)?;
        }
    }
    ensure_derived(semantic_count, capacity.semantics())?;
    ensure_derived(hit_count, capacity.hit_regions())?;
    ensure_derived(scene_count, capacity.scene_rectangles())?;

    let mut semantics = Vec::with_capacity(semantic_count);
    let mut hit_regions = Vec::with_capacity(hit_count);
    let mut scene_rectangles = Vec::with_capacity(scene_count);
    for (((node, computed), geometry), visible) in logical
        .nodes()
        .iter()
        .zip(&computed_styles)
        .zip(&geometries)
        .zip(&effective_visibility)
    {
        if !*visible || !non_empty(geometry.clip) {
            continue;
        }
        if node.template() == input.spec.semantic_template() {
            semantics.push(NormalizedHeadlessSemanticV1 {
                path: node.path().clone(),
                role: semantic_role(),
                label: input.spec.semantic_label(),
                action: semantic_action(),
            });
        }
        if computed.input == InputPolicy::Accept {
            hit_regions.push(NormalizedHeadlessHitRegionV1 {
                path: node.path().clone(),
                clip: geometry.clip,
            });
        }
        scene_rectangles.push(NormalizedHeadlessSceneRectangleV1 {
            path: node.path().clone(),
            rectangle: geometry.clip,
            color: computed.color,
        });
    }
    Ok(NormalizedHeadlessProjectionV1 {
        surface: input.surface,
        computed_styles,
        geometries,
        semantics,
        hit_regions,
        scene_rectangles,
    })
}

fn materialize(
    input: RebuildInput<'_>,
    node: &NormalizedNodeV1,
) -> Result<NormalizedHeadlessComputedStyleV1, HarnessError> {
    Ok(NormalizedHeadlessComputedStyleV1 {
        path: node.path().clone(),
        width: scalar(value(
            input,
            node.path(),
            node.template(),
            input.spec.width(),
        )?)?,
        height: scalar(value(
            input,
            node.path(),
            node.template(),
            input.spec.height(),
        )?)?,
        color: color(value(
            input,
            node.path(),
            node.template(),
            input.spec.color(),
        )?)?,
        visible: boolean(value(
            input,
            node.path(),
            node.template(),
            input.spec.visible(),
        )?)?,
        input: policy(value(
            input,
            node.path(),
            node.template(),
            input.spec.input(),
        )?)?,
    })
}

fn value(
    input: RebuildInput<'_>,
    path: &NodePathV1,
    template: TemplateNodeId,
    property: PropertyId,
) -> Result<PropertyValue, HarnessError> {
    if let Some((_, value)) = input
        .desired
        .property_overrides
        .iter()
        .find(|((candidate, id), _)| candidate == path && *id == property)
    {
        return Ok(value.clone());
    }
    if let Some(assignment) = input.style.assignment(template, property) {
        return Ok(assignment.replacement().clone());
    }
    input
        .style
        .construction()
        .template(template)
        .and_then(|factory| factory.effective_value(property))
        .cloned()
        .ok_or_else(state_error)
}

fn layout(
    input: RebuildInput<'_>,
    nodes: &[NormalizedNodeV1],
    computed: &[NormalizedHeadlessComputedStyleV1],
) -> Result<(Vec<NormalizedHeadlessGeometryV1>, Vec<bool>), HarnessError> {
    let mut frames = BTreeMap::<NodePathV1, Frame>::new();
    let mut cursors = BTreeMap::<NodePathV1, i32>::new();
    let mut geometries = Vec::with_capacity(nodes.len());
    let mut effective_visibility = Vec::with_capacity(nodes.len());
    let surface = rect(0, 0, input.surface.width(), input.surface.height());
    for (node, style) in nodes.iter().zip(computed) {
        let frame = if let Some(parent) = node.parent() {
            let parent_frame = frames.get(parent).ok_or_else(state_error)?;
            let cursor = *cursors.get(parent).ok_or_else(state_error)?;
            let bounds = rect(
                parent_frame.bounds.x(),
                cursor,
                style.width.min(parent_frame.bounds.width()),
                style.height,
            );
            cursors.insert(
                parent.clone(),
                cursor
                    .checked_add(style.height)
                    .ok_or_else(arithmetic_error)?,
            );
            Frame {
                bounds,
                clip: intersect(bounds, parent_frame.clip)?,
                visible: parent_frame.visible && style.visible,
            }
        } else {
            let bounds = rect(0, 0, style.width, style.height);
            Frame {
                bounds,
                clip: intersect(bounds, surface)?,
                visible: style.visible,
            }
        };
        cursors.insert(node.path().clone(), frame.bounds.y());
        geometries.push(NormalizedHeadlessGeometryV1 {
            path: node.path().clone(),
            bounds: frame.bounds,
            clip: frame.clip,
        });
        effective_visibility.push(frame.visible);
        if frames.insert(node.path().clone(), frame).is_some() {
            return Err(state_error());
        }
    }
    Ok((geometries, effective_visibility))
}

fn intersect(
    first: ProjectionRect,
    second: ProjectionRect,
) -> Result<ProjectionRect, HarnessError> {
    let x = first.x().max(second.x());
    let y = first.y().max(second.y());
    let right = end(first.x(), first.width())?.min(end(second.x(), second.width())?);
    let bottom = end(first.y(), first.height())?.min(end(second.y(), second.height())?);
    let width = if right <= x {
        0
    } else {
        right.checked_sub(x).ok_or_else(arithmetic_error)?
    };
    let height = if bottom <= y {
        0
    } else {
        bottom.checked_sub(y).ok_or_else(arithmetic_error)?
    };
    Ok(rect(x, y, width, height))
}

fn scalar(value: PropertyValue) -> Result<i32, HarnessError> {
    match value {
        PropertyValue::ScalarI32(value) => Ok(value),
        _ => Err(state_error()),
    }
}

fn boolean(value: PropertyValue) -> Result<bool, HarnessError> {
    match value {
        PropertyValue::Bool(value) => Ok(value),
        _ => Err(state_error()),
    }
}

fn color(value: PropertyValue) -> Result<[u8; 4], HarnessError> {
    match value {
        PropertyValue::Rgba8(value) => Ok(value),
        _ => Err(state_error()),
    }
}

fn policy(value: PropertyValue) -> Result<InputPolicy, HarnessError> {
    match value {
        PropertyValue::InputPolicy(value) => Ok(value),
        _ => Err(state_error()),
    }
}

fn non_empty(rectangle: ProjectionRect) -> bool {
    rectangle.width() > 0 && rectangle.height() > 0
}

fn ensure_limit(count: usize, limit: usize, kind: HarnessLimitKind) -> Result<(), HarnessError> {
    if count > limit {
        Err(HarnessError::limit(kind))
    } else {
        Ok(())
    }
}

fn ensure_derived(count: usize, limit: usize) -> Result<(), HarnessError> {
    if count > limit {
        Err(state_error())
    } else {
        Ok(())
    }
}

fn end(origin: i32, extent: i32) -> Result<i32, HarnessError> {
    origin.checked_add(extent).ok_or_else(arithmetic_error)
}

fn increment(value: usize) -> Result<usize, HarnessError> {
    value.checked_add(1).ok_or_else(arithmetic_error)
}

fn state_error() -> HarnessError {
    HarnessError::new(HarnessErrorKind::StateMismatch)
}

fn arithmetic_error() -> HarnessError {
    HarnessError::new(HarnessErrorKind::ArithmeticExhausted)
}
