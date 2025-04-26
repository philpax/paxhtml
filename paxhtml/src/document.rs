use std::path::Path;

use crate::{routing::RoutePath, Element, RenderElement};

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// A document is a collection of elements that will be rendered to HTML.
pub struct Document {
    /// The children of the document. [Element]s are converted to [RenderElement]s when the document
    /// is created.
    pub children: Vec<RenderElement>,
}
impl From<Vec<Element>> for Document {
    fn from(children: Vec<Element>) -> Self {
        Document {
            children: RenderElement::from_elements(children),
        }
    }
}
impl Document {
    /// Create a new document with a list of children. [Element]s are converted to [RenderElement]s
    /// when the document is created.
    pub fn new(children: impl IntoIterator<Item = Element>) -> Self {
        Document {
            children: RenderElement::from_elements(children),
        }
    }

    /// Write the document to a writer.
    pub fn write(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        RenderElement::write_many(writer, &self.children, 0)?;
        Ok(())
    }

    /// Write the document to a file in the given route.
    ///
    /// If `dump_tree` is enabled, the document's element tree will also be written to a JSON file
    /// in the same directory.
    pub fn write_to_route(
        &self,
        output_dir: &Path,
        route_path: impl Into<RoutePath>,
    ) -> std::io::Result<()> {
        let route_path: RoutePath = route_path.into();
        #[cfg(feature = "dump_tree")]
        {
            route_path
                .clone()
                .with_filename(route_path.filename().replace(".html", ".json"))
                .write(output_dir, serde_json::to_string_pretty(&self).unwrap())?;
        }
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
    use crate::builder::*;

    #[test]
    fn test_inline_code() {
        let input = Document::new([p([])([
            text("This is an example of "),
            code([])("inline code"),
            text(" in a paragraph."),
        ])]);

        let output = input.write_to_string().unwrap();
        assert_eq!(
            output,
            "<p>This is an example of <code>inline code</code> in a paragraph.</p>"
        );
    }

    #[test]
    fn test_empty_ul_with_tags_class() {
        let input = Document::new([ul([("class", "tags").into()])([])]);
        let output = input.write_to_string().unwrap();
        assert_eq!(output, "<ul class=\"tags\"></ul>");
    }

    #[test]
    fn test_void_element() {
        let input = Document::new([br([])]);
        let output = input.write_to_string().unwrap();
        assert_eq!(output, "<br>");
    }

    #[test]
    fn should_indent_successive_p_tags_in_a_fragment() {
        let input_elements = Document::new([div([])([p([])(text("Hello")), p([])(text("World"))])]);
        let output = input_elements.write_to_string().unwrap();
        assert_eq!(output, "<div>\n  <p>Hello</p>\n  <p>World</p>\n</div>");
    }
}
