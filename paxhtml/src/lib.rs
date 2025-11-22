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

/// Error type for runtime HTML parsing
#[derive(Debug)]
pub enum ParseHtmlError {
    /// Error parsing the HTML syntax
    Parse(ParseError),
    /// Error evaluating the AST (e.g., interpolation not supported)
    Eval(EvalError),
}

impl std::fmt::Display for ParseHtmlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseHtmlError::Parse(e) => write!(f, "Parse error: {}", e),
            ParseHtmlError::Eval(e) => write!(f, "Evaluation error: {}", e),
        }
    }
}

impl std::error::Error for ParseHtmlError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ParseHtmlError::Parse(e) => Some(e),
            ParseHtmlError::Eval(e) => Some(e),
        }
    }
}

impl From<ParseError> for ParseHtmlError {
    fn from(e: ParseError) -> Self {
        ParseHtmlError::Parse(e)
    }
}

impl From<EvalError> for ParseHtmlError {
    fn from(e: EvalError) -> Self {
        ParseHtmlError::Eval(e)
    }
}

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
/// Returns a [ParseHtmlError] if the HTML is malformed or contains features not
/// supported at runtime (like interpolation syntax).
pub fn parse_html(html: &str) -> Result<Element, ParseHtmlError> {
    let ast = paxhtml_parser::parse_html(html)?;
    let element = eval::eval_node(&ast)?;
    Ok(element)
}
