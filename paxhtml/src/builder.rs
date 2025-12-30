//! Implements a builder DSL for creating HTML documents through a series of functions.

use bumpalo::collections::String as BumpString;
use bumpalo::collections::Vec as BumpVec;
use bumpalo::Bump;

pub use super::{Attribute, Document, Element, IntoElement};

/// Create a text element from a string.
pub fn text<'bump>(bump: &'bump Bump, text: &str) -> Element<'bump> {
    Element::Text {
        text: BumpString::from_str_in(text, bump),
    }
}

/// Create a raw HTML element from a string.
pub fn raw<'bump>(bump: &'bump Bump, html: &str) -> Element<'bump> {
    Element::Raw {
        html: BumpString::from_str_in(html, bump),
    }
}

/// Create a tag element from a name, attributes, and a boolean indicating whether the tag is a void
/// element (i.e. doesn't have a closing tag).
///
/// The children are passed in as a single argument to the returned function.
pub fn tag<'bump, E: IntoElement<'bump>>(
    bump: &'bump Bump,
    name: &str,
    attributes: impl IntoIterator<Item = Attribute<'bump>>,
    void: bool,
) -> impl FnOnce(E) -> Element<'bump> {
    let name = BumpString::from_str_in(name, bump);
    let attributes: BumpVec<'bump, Attribute<'bump>> = BumpVec::from_iter_in(attributes, bump);

    move |children: E| {
        let children = children.into_element(bump);
        let children = match children {
            Element::Fragment { children } => children,
            Element::Empty => BumpVec::new_in(bump),
            other => {
                let mut v = BumpVec::with_capacity_in(1, bump);
                v.push(other);
                v
            }
        };
        Element::Tag {
            name,
            attributes,
            children,
            void,
        }
    }
}

/// Create a doctype element with a list of attributes.
pub fn doctype<'bump>(
    bump: &'bump Bump,
    attributes: impl IntoIterator<Item = Attribute<'bump>>,
) -> Element<'bump> {
    Element::Tag {
        name: BumpString::from_str_in("!DOCTYPE", bump),
        attributes: BumpVec::from_iter_in(attributes, bump),
        children: BumpVec::new_in(bump),
        void: true,
    }
}

macro_rules! non_void_builders {
    ($($tag_ident:ident),*) => {
        $(
            #[doc = concat!("Create a non-void element with the tag name `", stringify!($tag_ident), "` and a list of attributes.\n\nThe children are passed in as a single argument to the returned function.")]
            pub fn $tag_ident<'bump, E: IntoElement<'bump>>(
                bump: &'bump Bump,
                attributes: impl IntoIterator<Item = Attribute<'bump>>,
            ) -> impl FnOnce(E) -> Element<'bump> {
                tag(bump, stringify!($tag_ident), attributes, false)
            }

        )*
        /// A list of all non-void tags.
        pub const NON_VOID_TAGS: &[&str] = &[$(stringify!($tag_ident)),*];
    };
}
non_void_builders! {
    head, body, main, p, code, div, pre, header, nav,
    ol, ul, li, strong, em, blockquote, article, section,
    aside, span, script, title, time, html, a,
    h1, h2, h3, h4, h5, h6, small, sup, sub, label, q, s,
    table, tr, td, th, tbody, thead, tfoot, colgroup, video
}

macro_rules! void_builders {
    ($($tag_ident:ident),*) => {
        $(
            #[doc = concat!("Create a void element with the tag name `", stringify!($tag_ident), "` and a list of attributes.")]
            pub fn $tag_ident<'bump>(
                bump: &'bump Bump,
                attributes: impl IntoIterator<Item = Attribute<'bump>>,
            ) -> Element<'bump> {
                tag(bump, stringify!($tag_ident), attributes, true)(Element::Empty)
            }
        )*
        /// A list of all void tags.
        pub const VOID_TAGS: &[&str] = &[$(stringify!($tag_ident)),*];
    };
}
void_builders! {
    area, base, br, col, embed, hr, input, link, meta,
    param, source, track, wbr, img
}
