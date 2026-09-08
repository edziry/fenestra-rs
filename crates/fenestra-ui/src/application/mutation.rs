use std::sync::Arc;

use super::{Application, TextState, validate_size};
use crate::{Color, Error, Size, Style, TextMetrics, TextStyle};

impl Application {
    /// Returns the exact committed UTF-8 content of a text element.
    pub fn text(&self, name: &str) -> Result<&str, Error> {
        Ok(&self.text_node(name)?.content)
    }

    /// Returns the committed typography of a text element.
    pub fn text_style(&self, name: &str) -> Result<TextStyle, Error> {
        Ok(self.text_node(name)?.style)
    }

    /// Returns complete shaped measurements; zero-area text is not shaped.
    pub fn text_metrics(&self, name: &str) -> Result<TextMetrics, Error> {
        Ok(self
            .text_node(name)?
            .layout
            .as_ref()
            .map_or_else(TextMetrics::empty, |layout| layout.metrics()))
    }

    /// Publishes text, its measurements and paint together, or preserves all three.
    pub fn set_text(&mut self, name: &str, content: impl AsRef<str>) -> Result<(), Error> {
        let content = content.as_ref();
        let index = self.node_index(name)?;
        if self.text_node(name)?.content.as_ref() == content {
            return Ok(());
        }
        // Bound the whole view before copying the new text or invoking an engine.
        crate::text::validate_budget(
            self.nodes.iter().enumerate().flat_map(|(slot, node)| {
                [
                    node.text.as_ref().map(|text| {
                        if slot == index {
                            content
                        } else {
                            text.content.as_ref()
                        }
                    }),
                    node.control.as_ref().map(|control| control.label.as_ref()),
                ]
                .into_iter()
                .flatten()
                .map(|text| (text, Size::new(0, 0)))
            }),
            self.limits.text(),
        )?;
        let mut nodes = self.nodes.clone();
        let text = nodes[index]
            .text
            .as_mut()
            .expect("text element was checked");
        text.content = Arc::from(content);
        text.invalidate();
        self.commit_state(nodes, self.size)
    }

    /// Prepares and atomically publishes typography and the resulting text paint.
    pub fn set_text_style(&mut self, name: &str, style: TextStyle) -> Result<(), Error> {
        let index = self.node_index(name)?;
        let old = self.text_node(name)?;
        style.validate()?;
        if old.style == style {
            return Ok(());
        }
        let mut nodes = self.nodes.clone();
        let text = nodes[index]
            .text
            .as_mut()
            .expect("text element was checked");
        text.style = style;
        text.invalidate();
        self.commit_state(nodes, self.size)
    }

    /// Applies typed style, layout, text paint and hit geometry atomically.
    pub fn set_style(&mut self, name: &str, style: Style) -> Result<(), Error> {
        let index = self.node_index(name)?;
        let node = &self.nodes[index];
        style.validate(name, node.kind)?;
        if node.control_owner.is_some() && style.input {
            return Err(Error::InvalidElement {
                node: name.into(),
                reason: "control input is derived; descendants cannot enable independent input",
            });
        }
        if node.style == style {
            return Ok(());
        }
        let mut nodes = self.nodes.clone();
        nodes[index].style = style;
        self.commit_state(nodes, self.size)
    }

    /// Updates one element's background without changing its other properties.
    pub fn set_background(&mut self, name: &str, color: Color) -> Result<(), Error> {
        self.set_style(name, self.style(name)?.background(color))
    }

    /// Updates width and height together, including text, paint and hit geometry.
    pub fn set_size(&mut self, name: &str, width: i32, height: i32) -> Result<(), Error> {
        self.set_style(name, self.style(name)?.width(width).height(height))
    }

    /// Resizes the viewport after validating its pixel budget.
    pub fn resize(&mut self, size: Size) -> Result<(), Error> {
        validate_size(size, self.limits)?;
        if size == self.size {
            return Ok(());
        }
        self.commit_state(self.nodes.clone(), size)
    }

    fn text_node(&self, name: &str) -> Result<&TextState, Error> {
        self.nodes[self.node_index(name)?]
            .text
            .as_ref()
            .ok_or_else(|| Error::InvalidElement {
                node: name.into(),
                reason: "operation requires a text element",
            })
    }
}
