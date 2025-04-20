#![deny(missing_docs)]
//! A crate for generating formatted HTML from a runtime-generated tree of elements.
//!
//! Elements are created through either [builder] or [html], and placed in a [Document],
//! which will implicitly convert them to [RenderElement]s for rendering. A HTML string
//! can then be generated through [Document::write_to_string] or similar methods.

pub mod builder;
pub mod util;

mod attribute;
pub use attribute::{attr, Attribute};

mod document;
pub use document::Document;

mod element;
pub use element::Element;

mod render_element;
pub use render_element::RenderElement;

mod routing;
pub use routing::RoutePath;

#[cfg(feature = "macros")]
pub use paxhtml_macro::html;
