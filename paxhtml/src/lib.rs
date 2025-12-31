#![deny(missing_docs)]
//! A crate for generating formatted HTML from a runtime-generated tree of elements.
//!
//! Elements are created through either [builder] or [html], and placed in a [Document],
//! which will implicitly convert them to [RenderElement]s for rendering. A HTML string
//! can then be generated through [Document::write_to_string] or similar methods.
//!
//! All allocations are done through a bump allocator ([bumpalo::Bump]) which must be passed
//! to all element-creating functions.
//!
//! # Example
//!
//! ```
//! use paxhtml::{bumpalo::Bump, html, Document};
//!
//! let bump = Bump::new();
//! let element = html! { in &bump;
//!     <div class="container">
//!         <h1>"Hello, World!"</h1>
//!     </div>
//! };
//! let doc = Document::new(&bump, [element]);
//! let html_string = doc.write_to_string().unwrap();
//! ```

pub mod builder;
pub mod util;

// Re-export bumpalo for convenience
pub use bumpalo;

mod attribute;
pub use attribute::{attr, Attribute, AttributeParseError, IntoAttribute};

mod document;
pub use document::Document;

mod element;
pub use element::{DefaultIn, Element, IntoElement};

#[cfg(feature = "parser")]
mod eval;
#[cfg(feature = "parser")]
pub use eval::{eval_node, parse_html, EvalError, ParseHtmlError};

mod render_element;
pub use render_element::RenderElement;

mod owned;
pub use owned::{OwnedAttribute, OwnedElement};

mod routing;
pub use routing::RoutePath;

#[cfg(feature = "macros")]
pub use paxhtml_macro::html;

// Re-export parser types for convenience
#[cfg(feature = "parser")]
pub use paxhtml_parser::{parse_html as parse_html_ast, AstNode, ParseError};
