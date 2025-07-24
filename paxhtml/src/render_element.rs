use std::io::Write;

use crate::{Attribute, Element};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// A renderable element in an HTML document.
///
/// These are constructed from [`Element`]s using [`RenderElement::from_elements`].
/// This will process the tree to remove any extraneous nodes during conversion.
pub enum RenderElement {
    /// A tag element.
    Tag {
        /// The name of the tag.
        name: String,
        /// The attributes of the tag.
        attributes: Vec<Attribute>,
        /// The children of the tag.
        children: Vec<RenderElement>,
        /// Whether the tag is void.
        void: bool,
    },
    /// A text element.
    Text {
        /// The text of the element.
        text: String,
    },
    /// A raw element.
    Raw {
        /// The raw HTML of the element.
        html: String,
    },
}
impl RenderElement {
    /// Convert a list of [`Element`]s into a list of [`RenderElement`]s.
    ///
    /// This will process the tree to remove any extraneous nodes during conversion.
    pub fn from_elements(elements: impl IntoIterator<Item = Element>) -> Vec<Self> {
        elements
            .into_iter()
            .flat_map(|e| match e {
                Element::Empty => vec![],
                Element::Tag {
                    name,
                    attributes,
                    children,
                    void,
                } => vec![Self::Tag {
                    name,
                    attributes,
                    children: Self::from_elements(children),
                    void,
                }],
                Element::Fragment { children } => Self::from_elements(children),
                Element::Text { text } if text == "\n" => vec![],
                Element::Text { text } => vec![Self::Text { text }],
                Element::Raw { html } => vec![Self::Raw { html }],
            })
            .collect()
    }

    /// Write the element to a string.
    pub fn write_to_string(&self) -> std::io::Result<String> {
        let mut output = vec![];
        self.write(&mut output, 0)?;
        Ok(String::from_utf8(output).unwrap())
    }

    /// Write the element to a writer.
    pub fn write(&self, writer: &mut dyn Write, depth: usize) -> std::io::Result<()> {
        match self {
            RenderElement::Tag {
                name,
                attributes,
                children,
                void,
            } => {
                // start tag
                write!(writer, "<{name}")?;
                for Attribute { key, value } in attributes {
                    match value {
                        Some(value) => write!(
                            writer,
                            " {key}=\"{}\"",
                            html_escape::encode_quoted_attribute(value)
                        )?,
                        None => write!(writer, " {key}")?,
                    }
                }
                write!(writer, ">")?;

                if *void {
                    if !children.is_empty() {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            format!("Void element ({self:?}) has children"),
                        ));
                    }
                    return Ok(());
                }

                let did_indent = Self::write_many(writer, children, depth + 1)?;

                // end tag
                if did_indent {
                    writeln!(writer)?;
                    for _ in 0..depth {
                        write!(writer, "  ")?;
                    }
                }
                write!(writer, "</{name}>")?;
                Ok(())
            }
            RenderElement::Text { text } => {
                let text = html_escape::encode_text(text);
                for (idx, line) in text.lines().enumerate() {
                    if idx > 0 {
                        writeln!(writer)?;
                    }
                    write!(writer, "{line}")?;
                }
                Ok(())
            }
            RenderElement::Raw { html } => {
                write!(writer, "{html}")?;
                Ok(())
            }
        }
    }

    /// Write a list of [`RenderElement`]s to a writer.
    ///
    /// Returns whether or not the result was indented.
    pub fn write_many(
        writer: &mut dyn Write,
        elements: &[RenderElement],
        depth: usize,
    ) -> std::io::Result<bool> {
        let should_indent = !elements.is_empty();
        let mut did_indent = false;
        let mut encountered_text_element = false;
        for element in elements {
            encountered_text_element |= matches!(element, Self::Text { .. });
            let should_indent_this_child = should_indent
                && !encountered_text_element
                && !element.is_inline_element()
                && !element.is_raw();
            if should_indent_this_child && depth > 0 {
                writeln!(writer)?;
                for _ in 0..depth {
                    write!(writer, "  ")?;
                }
                did_indent = true;
            }
            element.write(writer, depth)?;
        }
        Ok(did_indent)
    }

    /// Write a list of [`RenderElement`]s to a string.
    pub fn write_many_to_string(elements: &[RenderElement]) -> std::io::Result<String> {
        let mut output = vec![];
        Self::write_many(&mut output, elements, 0)?;
        Ok(String::from_utf8(output).unwrap())
    }

    /// Get the tag name of the element if it is a [`Tag`].
    pub fn tag(&self) -> Option<&str> {
        match self {
            RenderElement::Tag { name, .. } => Some(name),
            _ => None,
        }
    }

    /// Returns `true` if the element-with-tag is an inline element.
    pub fn is_inline_element(&self) -> bool {
        self.tag().is_some_and(|t| {
            [
                "a", "abbr", "acronym", "b", "bdo", "big", "br", "button", "cite", "code", "dfn",
                "em", "i", "img", "input", "kbd", "label", "map", "pre", "object", "output", "q",
                "samp", "script", "select", "small", "span", "strong", "sub", "sup", "textarea",
                "time", "tt", "var",
            ]
            .contains(&t)
        })
    }

    /// Returns `true` if the element is [`Raw`].
    ///
    /// [`Raw`]: RenderElement::Raw
    #[must_use]
    pub fn is_raw(&self) -> bool {
        matches!(self, Self::Raw { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{attr, builder::*};

    #[test]
    pub fn wont_indent_text_surrounded_by_tags() {
        let element = h3([])([small([])("test "), text("tested"), small([])("!")]);
        let render_elements = RenderElement::from_elements([element]);
        let output = RenderElement::write_many_to_string(&render_elements).unwrap();
        assert_eq!(
            output,
            r#"<h3><small>test </small>tested<small>!</small></h3>"#.trim()
        );
    }

    #[test]
    pub fn wont_indent_inline_elements() {
        let elements = [
            text("test "),
            a([attr(("href", "https://example.com"))])([text("tested")]),
            text("!"),
        ];
        let render_elements = RenderElement::from_elements(elements);
        let output = RenderElement::write_many_to_string(&render_elements).unwrap();
        assert_eq!(output, r#"test <a href="https://example.com">tested</a>!"#);
    }
}
