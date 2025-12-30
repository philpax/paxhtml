use std::io::Write;

use bumpalo::collections::String as BumpString;
use bumpalo::collections::Vec as BumpVec;
use bumpalo::Bump;

use crate::{Attribute, Element};

#[derive(Debug, Clone, PartialEq, Eq)]
/// A renderable element in an HTML document.
///
/// These are constructed from [`Element`]s using [`RenderElement::from_elements`].
/// This will process the tree to remove any extraneous nodes during conversion.
pub enum RenderElement<'bump> {
    /// A tag element.
    Tag {
        /// The name of the tag.
        name: BumpString<'bump>,
        /// The attributes of the tag.
        attributes: BumpVec<'bump, Attribute<'bump>>,
        /// The children of the tag.
        children: BumpVec<'bump, RenderElement<'bump>>,
        /// Whether the tag is void.
        void: bool,
    },
    /// A text element.
    Text {
        /// The text of the element.
        text: BumpString<'bump>,
    },
    /// A raw element.
    Raw {
        /// The raw HTML of the element.
        html: BumpString<'bump>,
    },
}
impl<'bump> RenderElement<'bump> {
    /// Convert a list of [`Element`]s into a list of [`RenderElement`]s.
    ///
    /// This will process the tree to remove any extraneous nodes during conversion.
    pub fn from_elements(
        bump: &'bump Bump,
        elements: impl IntoIterator<Item = Element<'bump>>,
    ) -> BumpVec<'bump, Self> {
        let mut result = BumpVec::new_in(bump);
        for e in elements {
            match e {
                Element::Empty => {}
                Element::Tag {
                    name,
                    attributes,
                    children,
                    void,
                } => {
                    result.push(Self::Tag {
                        name,
                        attributes,
                        children: Self::from_elements(bump, children),
                        void,
                    });
                }
                Element::Fragment { children } => {
                    result.extend(Self::from_elements(bump, children));
                }
                Element::Text { text } if text.as_str() == "\n" => {}
                Element::Text { text } => {
                    result.push(Self::Text { text });
                }
                Element::Raw { html } => {
                    result.push(Self::Raw { html });
                }
            }
        }
        result
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
                write!(writer, "<{}", name.as_str())?;
                for Attribute { key, value } in attributes.iter() {
                    match value {
                        Some(value) => write!(
                            writer,
                            " {}=\"{}\"",
                            key.as_str(),
                            html_escape::encode_quoted_attribute(value.as_str())
                        )?,
                        None => write!(writer, " {}", key.as_str())?,
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

                let did_indent = Self::write_many(writer, children.as_slice(), depth + 1)?;

                // end tag
                if did_indent {
                    writeln!(writer)?;
                    for _ in 0..depth {
                        write!(writer, "  ")?;
                    }
                }
                write!(writer, "</{}>", name.as_str())?;
                Ok(())
            }
            RenderElement::Text { text } => {
                let text = html_escape::encode_text(text.as_str());
                for (idx, line) in text.lines().enumerate() {
                    if idx > 0 {
                        writeln!(writer)?;
                    }
                    write!(writer, "{line}")?;
                }
                Ok(())
            }
            RenderElement::Raw { html } => {
                write!(writer, "{}", html.as_str())?;
                Ok(())
            }
        }
    }

    /// Write a list of [`RenderElement`]s to a writer.
    ///
    /// Returns whether or not the result was indented.
    pub fn write_many(
        writer: &mut dyn Write,
        elements: &[RenderElement<'bump>],
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
    pub fn write_many_to_string(elements: &[RenderElement<'bump>]) -> std::io::Result<String> {
        let mut output = vec![];
        Self::write_many(&mut output, elements, 0)?;
        Ok(String::from_utf8(output).unwrap())
    }

    /// Get the tag name of the element if it is a [`Tag`].
    pub fn tag(&self) -> Option<&str> {
        match self {
            RenderElement::Tag { name, .. } => Some(name.as_str()),
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
        let bump = Bump::new();
        let element = h3(&bump, [])([
            small(&bump, [])(text(&bump, "test ")),
            text(&bump, "tested"),
            small(&bump, [])(text(&bump, "!")),
        ]);
        let render_elements = RenderElement::from_elements(&bump, [element]);
        let output = RenderElement::write_many_to_string(render_elements.as_slice()).unwrap();
        assert_eq!(
            output,
            r#"<h3><small>test </small>tested<small>!</small></h3>"#.trim()
        );
    }

    #[test]
    pub fn wont_indent_inline_elements() {
        let bump = Bump::new();
        let elements = [
            text(&bump, "test "),
            a(&bump, [attr(&bump, ("href", "https://example.com"))])([text(&bump, "tested")]),
            text(&bump, "!"),
        ];
        let render_elements = RenderElement::from_elements(&bump, elements);
        let output = RenderElement::write_many_to_string(render_elements.as_slice()).unwrap();
        assert_eq!(output, r#"test <a href="https://example.com">tested</a>!"#);
    }
}
