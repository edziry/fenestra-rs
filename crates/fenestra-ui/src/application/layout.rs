use super::{NamedNode, TextState};
use crate::layout::{self, LayoutNode};
use crate::{Error, Limits, Size, TextEngine, TextError, TextMeasureRequest, TextMetrics};

pub(super) fn resolve(
    nodes: &[LayoutNode<'_>],
    texts: &mut [Option<TextState>],
    engine: &mut Option<Box<dyn TextEngine>>,
    size: Size,
    limits: Limits,
) -> Result<Vec<Size>, Error> {
    layout::resolve(nodes, size, |index, width| {
        let text = texts[index].as_mut().ok_or_else(|| {
            Error::InvalidProgram("text measurement requested for a non-text node".into())
        })?;
        measure(text, engine, width, limits)
    })
}

fn measure(
    text: &mut TextState,
    engine: &mut Option<Box<dyn TextEngine>>,
    width: Option<u32>,
    limits: Limits,
) -> Result<TextMetrics, Error> {
    let cached = match width {
        None => text.natural,
        Some(width) => text
            .wrapped
            .filter(|(cached, _)| *cached == width)
            .map(|(_, metrics)| metrics),
    };
    if let Some(metrics) = cached {
        return Ok(metrics);
    }
    let request = TextMeasureRequest::new(&text.content, text.style, width, limits.text())?;
    let metrics = engine
        .as_mut()
        .ok_or(TextError::EngineUnavailable)?
        .measure(request)?;
    metrics.validate_measurement(request)?;
    match width {
        None => text.natural = Some(metrics),
        Some(width) => text.wrapped = Some((width, metrics)),
    }
    Ok(metrics)
}

pub(super) fn resolve_nodes(
    nodes: &mut [NamedNode],
    engine: &mut Option<Box<dyn TextEngine>>,
    size: Size,
    limits: Limits,
) -> Result<(), Error> {
    // Only the staged candidate is borrowed here; failure drops its caches and
    // leaves the application's accepted node vector and images intact.
    let mut texts = nodes
        .iter_mut()
        .map(|node| node.text.take())
        .collect::<Vec<_>>();
    let descriptions = nodes
        .iter()
        .map(|node| LayoutNode {
            name: &node.name,
            kind: node.kind,
            style: node.style,
            children: &node.children,
        })
        .collect::<Vec<_>>();
    let sizes = resolve(&descriptions, &mut texts, engine, size, limits)?;
    crate::text::validate_budget(
        texts
            .iter()
            .zip(&sizes)
            .filter_map(|(text, &size)| text.as_ref().map(|text| (text.content.as_ref(), size))),
        limits.text(),
    )?;
    for ((node, text), size) in nodes.iter_mut().zip(texts).zip(sizes) {
        node.text = text;
        node.resolved = size;
        finish_measurement(node, engine, limits)?;
    }
    Ok(())
}

pub(super) fn finish_measurement(
    node: &mut NamedNode,
    engine: &mut Option<Box<dyn TextEngine>>,
    limits: Limits,
) -> Result<(), Error> {
    let size = node.resolved;
    if let Some(text) = &mut node.text {
        // Intrinsic width may have been measured without wrapping. Obtain
        // matching authored-color metrics for the final raster width too.
        if text.natural.is_some() && size.width() > 0 && size.height() > 0 {
            measure(text, engine, Some(size.width()), limits)?;
        }
    }
    Ok(())
}
