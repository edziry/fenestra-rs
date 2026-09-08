use super::LayoutNode;
use crate::model::ElementKind;
use crate::{Dimension, Error, Size, Style};

#[derive(Clone, Copy)]
pub(super) enum Axis {
    Width,
    Height,
}

pub(super) struct Bounds {
    pub(super) minimum: u32,
    pub(super) maximum: u32,
}

impl Bounds {
    pub(super) fn clamp(&self, value: u64) -> u32 {
        // A container's natural sum can exceed signed coordinates even when
        // its authored maximum produces a small, representable final extent.
        value.clamp(u64::from(self.minimum), u64::from(self.maximum)) as u32
    }
}

impl Axis {
    pub(super) const fn index(self) -> usize {
        match self {
            Self::Width => 0,
            Self::Height => 1,
        }
    }

    pub(super) const fn dimension(self, style: Style) -> Dimension {
        match self {
            Self::Width => style.width,
            Self::Height => style.height,
        }
    }

    pub(super) const fn extent(self, size: Size) -> u32 {
        match self {
            Self::Width => size.width(),
            Self::Height => size.height(),
        }
    }

    pub(super) const fn is_main(self, kind: ElementKind) -> bool {
        matches!(
            (self, kind),
            (Self::Width, ElementKind::Row | ElementKind::Checkbox)
                | (Self::Height, ElementKind::Column | ElementKind::Button)
        )
    }

    pub(super) fn bounds(self, node: &LayoutNode<'_>) -> Result<Bounds, Error> {
        let (minimum, maximum) = match self {
            Self::Width => (node.style.min_width, node.style.max_width),
            Self::Height => (node.style.min_height, node.style.max_height),
        };
        let mut minimum = minimum as u32;
        if !matches!(self.dimension(node.style), Dimension::Px(_)) {
            let padding = padding(node.style);
            if padding > maximum as u64 {
                return Err(Error::InvalidElement {
                    node: node.name.into(),
                    reason: "auto and fill maxima must contain the element padding",
                });
            }
            minimum = minimum.max(padding as u32);
        }
        Ok(Bounds {
            minimum,
            maximum: maximum as u32,
        })
    }
}

pub(super) fn padding(style: Style) -> u64 {
    style.padding as u64 * 2
}

pub(super) fn gaps(node: &LayoutNode<'_>) -> Result<u64, Error> {
    let count = u64::try_from(node.children.len().saturating_sub(1))
        .map_err(|_| Error::CapacityOverflow)?;
    count
        .checked_mul(node.style.gap as u64)
        .ok_or(Error::CapacityOverflow)
}
