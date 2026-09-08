use crate::model::ElementKind;
use crate::{Dimension, Error, Size, Style, TextMetrics};

mod axis;
mod fill;

use axis::Axis;
use fill::Fill;

/// Borrowed, validated nodes in authored preorder with direct child indices.
pub(crate) struct LayoutNode<'a> {
    pub(crate) name: &'a str,
    pub(crate) kind: ElementKind,
    pub(crate) style: Style,
    pub(crate) children: &'a [usize],
}

pub(crate) fn resolve(
    nodes: &[LayoutNode<'_>],
    size: Size,
    measure: impl FnMut(usize, Option<u32>) -> Result<TextMetrics, Error>,
) -> Result<Vec<Size>, Error> {
    for node in nodes {
        node.style.validate(node.name, node.kind)?;
        Axis::Width.bounds(node)?;
        Axis::Height.bounds(node)?;
    }
    if nodes.is_empty() {
        return Ok(Vec::new());
    }
    let mut solver = Solver {
        nodes,
        measure,
        resolved: [vec![0; nodes.len()], vec![0; nodes.len()]],
        intrinsic: [vec![None; nodes.len()], vec![None; nodes.len()]],
    };
    // Widths are final before any wrapped-height measurement. Each axis lazily
    // measures intrinsic subtrees bottom-up, then allocates extents top-down.
    solver.resolve_axis(Axis::Width, size.width())?;
    solver.resolve_axis(Axis::Height, size.height())?;
    Ok(solver.resolved[0]
        .iter()
        .zip(&solver.resolved[1])
        .map(|(&width, &height)| Size::new(width, height))
        .collect())
}

struct Solver<'a, 'b, F> {
    nodes: &'a [LayoutNode<'b>],
    measure: F,
    resolved: [Vec<u32>; 2],
    intrinsic: [Vec<Option<u32>>; 2],
}

impl<F: FnMut(usize, Option<u32>) -> Result<TextMetrics, Error>> Solver<'_, '_, F> {
    fn resolve_axis(&mut self, axis: Axis, available: u32) -> Result<(), Error> {
        let root = &self.nodes[0];
        let extent = if matches!(axis.dimension(root.style), Dimension::Fill(_)) {
            axis.bounds(root)?.clamp(u64::from(available))
        } else {
            self.intrinsic(0, axis)?
        };
        self.resolved[axis.index()][0] = extent;
        for parent in 0..self.nodes.len() {
            self.resolve_children(parent, axis)?;
        }
        Ok(())
    }

    fn resolve_children(&mut self, parent: usize, axis: Axis) -> Result<(), Error> {
        let node = &self.nodes[parent];
        let children = node.children;
        let main = axis.is_main(node.kind);
        let inner = u64::from(self.resolved[axis.index()][parent])
            .saturating_sub(axis::padding(node.style));
        let mut occupied = if main { axis::gaps(node)? } else { 0 };
        let mut fills = Vec::new();
        for &index in children {
            let child = &self.nodes[index];
            let extent = match axis.dimension(child.style) {
                Dimension::Fill(weight) if main => {
                    let bounds = axis.bounds(child)?;
                    fills.push(Fill {
                        index,
                        minimum: bounds.minimum,
                        maximum: bounds.maximum,
                        weight,
                    });
                    continue;
                }
                Dimension::Fill(_) => axis.bounds(child)?.clamp(inner),
                _ => self.intrinsic(index, axis)?,
            };
            self.resolved[axis.index()][index] = extent;
            if main {
                occupied = occupied
                    .checked_add(u64::from(extent))
                    .ok_or(Error::CapacityOverflow)?;
            }
        }
        let available = inner.saturating_sub(occupied) as u32;
        for (fill, extent) in fills.iter().zip(fill::allocate(&fills, available)?) {
            self.resolved[axis.index()][fill.index] = extent;
        }
        Ok(())
    }

    fn intrinsic(&mut self, index: usize, axis: Axis) -> Result<u32, Error> {
        if let Some(extent) = self.intrinsic[axis.index()][index] {
            return Ok(extent);
        }
        let node = &self.nodes[index];
        let bounds = axis.bounds(node)?;
        let natural = match axis.dimension(node.style) {
            Dimension::Px(value) => value as u64,
            Dimension::Auto | Dimension::Fill(_) => match node.kind {
                ElementKind::Text => u64::from(self.text_extent(index, axis)?),
                ElementKind::Rect => 0,
                ElementKind::Row | ElementKind::Column => self.container_extent(index, axis)?,
            },
        };
        let extent = bounds.clamp(natural);
        self.intrinsic[axis.index()][index] = Some(extent);
        Ok(extent)
    }

    fn text_extent(&mut self, index: usize, axis: Axis) -> Result<u32, Error> {
        let width = match axis {
            Axis::Width => None,
            Axis::Height => {
                let width = self.resolved[Axis::Width.index()][index];
                if width == 0 {
                    return Ok(0);
                }
                Some(width)
            }
        };
        let measured = (self.measure)(index, width)?.ceil_size()?;
        Ok(axis.extent(measured))
    }

    fn container_extent(&mut self, index: usize, axis: Axis) -> Result<u64, Error> {
        let node = &self.nodes[index];
        let children = node.children;
        let main = axis.is_main(node.kind);
        let padding = axis::padding(node.style);
        let mut content = if main { axis::gaps(node)? } else { 0 };
        for &child in children {
            // Fill has an intrinsic contribution while its auto ancestor is
            // measured; allocation never asks an unresolved parent for space.
            let extent = u64::from(self.intrinsic(child, axis)?);
            content = if main {
                content.checked_add(extent).ok_or(Error::CapacityOverflow)?
            } else {
                content.max(extent)
            };
        }
        content.checked_add(padding).ok_or(Error::CapacityOverflow)
    }
}

#[cfg(test)]
mod tests;
