use std::path::Path;

use bumpalo::collections::Vec as BumpVec;
use bumpalo::Bump;

use crate::{builder::Builder, routing::RoutePath, Element, RenderElement};

#[derive(Debug)]
/// A document is a collection of elements that will be rendered to HTML.
pub struct Document<'bump> {
    /// The children of the document. [Element]s are converted to [RenderElement]s when the document
    /// is created.
    pub children: BumpVec<'bump, RenderElement<'bump>>,
}
impl<'bump> Document<'bump> {
    /// Create a new document with a list of children. [Element]s are converted to [RenderElement]s
    /// when the document is created.
    pub fn new(bump: &'bump Bump, children: impl IntoIterator<Item = Element<'bump>>) -> Self {
        Document {
            children: RenderElement::from_elements(bump, children),
        }
    }

    /// Create a new document with a doctype declaration followed by the given element.
    ///
    /// This is equivalent to `Document::new(bump, [b.doctype([b.attr("html")]), element])`.
    pub fn new_with_doctype(bump: &'bump Bump, element: Element<'bump>) -> Self {
        let b = Builder::new(bump);
        Self::new(bump, [b.doctype([b.attr("html")]), element])
    }

    /// Write the document to a writer.
    pub fn write(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        RenderElement::write_many(writer, self.children.as_slice(), 0)?;
        Ok(())
    }

    /// Write the document to a file in the given route.
    pub fn write_to_route(&self, output_dir: &Path, route_path: &RoutePath) -> std::io::Result<()> {
        self.write(&mut route_path.writer(output_dir)?)
    }

    /// Write the document to a string.
    pub fn write_to_string(&self) -> std::io::Result<String> {
        let mut output = vec![];
        self.write(&mut output)?;
        Ok(String::from_utf8(output).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inline_code() {
        let bump = Bump::new();
        let b = Builder::new(&bump);
        let input = b.document([b.p([])([
            b.text("This is an example of "),
            b.code([])(b.text("inline code")),
            b.text(" in a paragraph."),
        ])]);

        let output = input.write_to_string().unwrap();
        assert_eq!(
            output,
            "<p>This is an example of <code>inline code</code> in a paragraph.</p>"
        );
    }

    #[test]
    fn test_empty_ul_with_tags_class() {
        let bump = Bump::new();
        let b = Builder::new(&bump);
        let input = b.document([b.ul([b.attr(("class", "tags"))])([])]);
        let output = input.write_to_string().unwrap();
        assert_eq!(output, "<ul class=\"tags\"></ul>");
    }

    #[test]
    fn test_void_element() {
        let bump = Bump::new();
        let b = Builder::new(&bump);
        let input = b.document([b.br([])]);
        let output = input.write_to_string().unwrap();
        assert_eq!(output, "<br>");
    }

    #[test]
    fn should_indent_successive_p_tags_in_a_fragment() {
        let bump = Bump::new();
        let b = Builder::new(&bump);
        let input_elements = b.document([b.div([])([
            b.p([])(b.text("Hello")),
            b.p([])(b.text("World")),
        ])]);
        let output = input_elements.write_to_string().unwrap();
        assert_eq!(output, "<div>\n  <p>Hello</p>\n  <p>World</p>\n</div>");
    }

    #[test]
    fn test_new_with_doctype() {
        let bump = Bump::new();
        let b = Builder::new(&bump);
        let input = b.document_with_doctype(b.html([])(b.body([])(b.text("Hello"))));
        let output = input.write_to_string().unwrap();
        assert_eq!(
            output,
            "<!DOCTYPE html><html>\n  <body>Hello</body>\n</html>"
        );
    }
}
