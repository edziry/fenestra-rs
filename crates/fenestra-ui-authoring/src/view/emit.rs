use proc_macro2::TokenStream;

use super::parser::{Document, Element};
use super::{CompiledView, Diagnostic, Limits};

pub(super) fn emit(document: &Document, limits: Limits) -> Result<CompiledView, Diagnostic> {
    let mut output = Output {
        text: String::new(),
        maximum: limits.generated_bytes,
        document,
    };
    output.push(&format!("::fenestra_ui::View::new(\"{}\",", document.name))?;
    let mut pending = vec![Part::Finish, Part::Element(0)];
    while let Some(part) = pending.pop() {
        match part {
            Part::Finish => output.push(")")?,
            Part::Child(index) => {
                output.push(".child(")?;
                pending.push(Part::Finish);
                pending.push(Part::Element(index));
            }
            Part::Element(index) => {
                let element = &document.elements[index];
                output.element(element)?;
                pending.extend(element.children.iter().rev().copied().map(Part::Child));
            }
        }
    }
    let tokens: TokenStream = output
        .text
        .parse()
        .map_err(|_| Diagnostic::new("failed to construct public View tokens", document.origin))?;
    let mut source = tokens.to_string();
    source.push('\n');
    if source.len() > limits.generated_bytes {
        return Err(output.limit_error());
    }
    Ok(CompiledView { source, tokens })
}

enum Part {
    Element(usize),
    Child(usize),
    Finish,
}

struct Output<'a> {
    text: String,
    maximum: usize,
    document: &'a Document,
}

impl Output<'_> {
    fn element(&mut self, element: &Element) -> Result<(), Diagnostic> {
        self.push(&format!(
            "::fenestra_ui::Element::{}(\"{}\").style(::fenestra_ui::Style::new()",
            element.kind.name(),
            element.name,
        ))?;
        let props = &element.properties;
        for (name, value) in [
            ("width", props.width),
            ("height", props.height),
            ("padding", props.padding),
            ("gap", props.gap),
        ] {
            if let Some(value) = value {
                self.push(&format!(".{name}({value}i32)"))?;
            }
        }
        if let Some([red, green, blue, alpha]) = props.background {
            self.push(&format!(
                ".background(::fenestra_ui::Color::rgba8({red}u8,{green}u8,{blue}u8,{alpha}u8))"
            ))?;
        }
        if let Some(value) = props.input {
            self.push(&format!(".input({value})"))?;
        }
        self.push(")")
    }

    fn push(&mut self, value: &str) -> Result<(), Diagnostic> {
        if value.len() > self.maximum.saturating_sub(self.text.len()) {
            return Err(self.limit_error());
        }
        self.text.push_str(value);
        Ok(())
    }

    fn limit_error(&self) -> Diagnostic {
        Diagnostic::new(
            "authoring limit exceeded: generated Rust bytes",
            self.document.origin,
        )
    }
}
