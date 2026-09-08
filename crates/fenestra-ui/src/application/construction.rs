use fenestra_ui_runtime::prototype::UiRuntime;

use super::{Application, NamedNode, TextState, layout, text, validate_size, viewport};
use crate::{Error, Limits, Size, TextEngine, View, lower};

impl Application {
    /// Constructs a view with default resource limits and no text engine.
    pub fn new(view: View, size: Size) -> Result<Self, Error> {
        Self::with_limits(view, size, Limits::default())
    }

    /// Constructs a view within explicit resource limits and no text engine.
    pub fn with_limits(view: View, size: Size, limits: Limits) -> Result<Self, Error> {
        Self::construct(view, size, limits, None)
    }

    /// Constructs text views using an application-owned, replaceable engine.
    pub fn with_text_engine(
        view: View,
        size: Size,
        engine: impl TextEngine + 'static,
    ) -> Result<Self, Error> {
        Self::with_limits_and_text_engine(view, size, Limits::default(), engine)
    }

    /// Constructs bounded text views without exposing any engine-specific types.
    pub fn with_limits_and_text_engine(
        view: View,
        size: Size,
        limits: Limits,
        engine: impl TextEngine + 'static,
    ) -> Result<Self, Error> {
        Self::construct(view, size, limits, Some(Box::new(engine)))
    }

    fn construct(
        view: View,
        size: Size,
        limits: Limits,
        mut text_engine: Option<Box<dyn TextEngine>>,
    ) -> Result<Self, Error> {
        validate_size(size, limits)?;
        let flat = lower::prepare(&view, limits)?;
        let mut texts = flat
            .nodes
            .iter()
            .map(|node| {
                node.element.text.as_deref().map(|content| {
                    TextState::new(content, node.element.text_style.unwrap_or_default())
                })
            })
            .collect::<Vec<_>>();
        let descriptions = flat
            .nodes
            .iter()
            .map(|node| crate::layout::LayoutNode {
                name: &node.element.name,
                kind: node.element.kind,
                style: node.element.style,
                children: &node.children,
            })
            .collect::<Vec<_>>();
        let sizes = layout::resolve(&descriptions, &mut texts, &mut text_engine, size, limits)?;
        let lowered = lower::lower_prepared(&flat, &sizes, limits)?;
        let runtime = UiRuntime::new_spatial_ir(
            lowered.program,
            viewport(size),
            lowered.spatial_limits,
            lowered.capacity,
        )
        .map_err(|error| Error::Runtime(format!("{:?}", error.kind())))?;
        let committed = runtime.committed();
        let mut logical = vec![committed.root()];
        let mut by_template = vec![None; lowered.names.len()];
        while let Some(node) = logical.pop() {
            let template = committed
                .template(node)
                .ok_or_else(|| Error::InvalidProgram("missing node template".into()))?;
            let slot = by_template
                .get_mut(template.get() as usize)
                .ok_or_else(|| Error::InvalidProgram("unknown node template".into()))?;
            *slot = Some(node);
            logical.extend(committed.children(node).unwrap_or(&[]).iter().copied());
        }
        let mut nodes = Vec::with_capacity(lowered.names.len());
        for (index, ((flat_node, text), resolved)) in
            flat.nodes.iter().zip(texts).zip(sizes).enumerate()
        {
            let element = flat_node.element;
            let id = by_template[index]
                .ok_or_else(|| Error::InvalidProgram("missing logical element".into()))?;
            let mut node = NamedNode {
                name: element.name.clone(),
                id,
                kind: element.kind,
                style: element.style,
                resolved,
                children: flat_node.children.clone(),
                text,
            };
            text::prepare_node(&mut node, &mut text_engine, limits.text())?;
            nodes.push(node);
        }
        let text_frame = text::prepare_frame(&committed, &nodes, lowered.spatial_limits, limits)?;
        Ok(Self {
            runtime,
            nodes,
            size,
            limits,
            revision: 0,
            spatial_limits: lowered.spatial_limits,
            text_engine,
            text_frame,
        })
    }
}
