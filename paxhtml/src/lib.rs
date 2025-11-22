#![deny(missing_docs)]
//! A crate for generating formatted HTML from a runtime-generated tree of elements.
//!
//! Elements are created through either [builder] or [html], and placed in a [Document],
//! which will implicitly convert them to [RenderElement]s for rendering. A HTML string
//! can then be generated through [Document::write_to_string] or similar methods.

pub mod builder;
pub mod util;

mod attribute;
pub use attribute::{attr, Attribute, AttributeParseError};

mod document;
pub use document::Document;

mod element;
pub use element::Element;

mod eval;
pub use eval::{eval_node, EvalError};

mod render_element;
pub use render_element::RenderElement;

mod routing;
pub use routing::RoutePath;

#[cfg(feature = "macros")]
pub use paxhtml_macro::html;

// Re-export parser types for convenience
pub use paxhtml_parser::{parse_html as parse_html_ast, AstNode, ParseError};

/// Parse an HTML string into a runtime [Element] tree.
///
/// This function parses HTML at runtime and returns an [Element] tree that can be
/// used with [Document] or other paxhtml APIs.
///
/// # Example
///
/// ```
/// use paxhtml::{parse_html, Document};
///
/// let element = parse_html(r#"<div class="container"><p>"Hello, world!"</p></div>"#).unwrap();
/// let doc = Document::new([element]);
/// let html = doc.write_to_string();
/// ```
///
/// # Errors
///
/// Returns a [ParseError] if the HTML is malformed, or an [EvalError] if the HTML
/// contains features not supported at runtime (like interpolation syntax).
pub fn parse_html(html: &str) -> Result<Element, Box<dyn std::error::Error>> {
    let ast = paxhtml_parser::parse_html(html)?;
    let element = eval::eval_node(&ast)?;
    Ok(element)
}
