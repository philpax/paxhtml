//! Implements a builder DSL for creating HTML documents through a series of functions.

pub use super::{Attribute, Document, Element};

/// Create a text element from a value that implements [Into<String>].
pub fn text(text: impl Into<String>) -> Element {
    Element::from(text.into())
}

/// Create a tag element from a name, attributes, and a boolean indicating whether the tag is a void
/// element (i.e. doesn't have a closing tag).
///
/// The children are passed in as a single argument to the returned function.
pub fn tag<E: Into<Element>>(
    name: impl Into<String>,
    attributes: impl IntoIterator<Item = Attribute>,
    void: bool,
) -> impl FnOnce(E) -> Element {
    move |children: E| {
        let children = children.into();
        let children = if let Element::Fragment { children } = children {
            children
        } else if !children.is_empty() {
            vec![children]
        } else {
            vec![]
        };
        Element::Tag {
            name: name.into(),
            attributes: attributes.into_iter().collect(),
            children,
            void,
        }
    }
}
/// Create a doctype element with a list of attributes.
pub fn doctype(attributes: impl IntoIterator<Item = Attribute>) -> Element {
    Element::Tag {
        name: "!DOCTYPE".into(),
        attributes: attributes.into_iter().collect(),
        children: vec![],
        void: true,
    }
}

macro_rules! non_void_builders {
    ($($tag_ident:ident),*) => { $(
        #[doc = concat!("Create a non-void element with the tag name `", stringify!($tag_ident), "` and a list of attributes.\n\nThe children are passed in as a single argument to the returned function.")]
        pub fn $tag_ident<E: Into<Element>>(attributes: impl IntoIterator<Item = Attribute>) -> impl FnOnce(E) -> Element {
            tag(stringify!($tag_ident), attributes, false)
        }
    )* };
}
non_void_builders! {
    head, body, main, p, code, div, pre, header, nav,
    ol, ul, li, strong, em, blockquote, article, section,
    aside, span, script, title, time, html, a,
    h1, h2, h3, h4, h5, h6, small, sup, sub, label, q, s,
    table, tr, td, th, tbody, thead, tfoot, colgroup
}

macro_rules! void_builders {
    ($($tag_ident:ident),*) => { $(
        #[doc = concat!("Create a void element with the tag name `", stringify!($tag_ident), "` and a list of attributes.")]
        pub fn $tag_ident(attributes: impl IntoIterator<Item = Attribute>) -> Element {
            tag(stringify!($tag_ident), attributes, true)([])
        }
    )* };
}
void_builders! {
    area, base, br, col, embed, hr, input, link, meta,
    param, source, track, wbr, img
}
