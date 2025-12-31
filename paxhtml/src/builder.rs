//! Implements a builder DSL for creating HTML documents through a series of methods.

use bumpalo::collections::String as BumpString;
use bumpalo::collections::Vec as BumpVec;
use bumpalo::Bump;

use crate::{Attribute, Element, IntoAttribute, IntoElement};

/// A builder for creating HTML elements using a bump allocator.
///
/// # Example
///
/// ```
/// use paxhtml::{bumpalo::Bump, builder::Builder};
///
/// let bump = Bump::new();
/// let b = Builder::new(&bump);
/// let doc = b.document([
///     b.div([b.attr(("class", "container"))])(
///         b.h1([])(b.text("Hello, World!"))
///     )
/// ]);
/// ```
#[derive(Clone, Copy)]
pub struct Builder<'bump> {
    bump: &'bump Bump,
}
impl<'bump> Builder<'bump> {
    /// Create a new builder with the given bump allocator.
    pub fn new(bump: &'bump Bump) -> Self {
        Self { bump }
    }

    /// Get a reference to the bump allocator.
    pub fn bump(&self) -> &'bump Bump {
        self.bump
    }

    /// Create an attribute from a value that implements [IntoAttribute].
    pub fn attr(&self, value: impl IntoAttribute<'bump>) -> Attribute<'bump> {
        value.into_attribute(self.bump)
    }

    /// Create a text element from a string.
    pub fn text(&self, text: &str) -> Element<'bump> {
        Element::Text {
            text: BumpString::from_str_in(text, self.bump),
        }
    }

    /// Create a raw HTML element from a string.
    pub fn raw(&self, html: &str) -> Element<'bump> {
        Element::Raw {
            html: BumpString::from_str_in(html, self.bump),
        }
    }

    /// Create a tag element from a name, attributes, and a boolean indicating whether the tag is a void
    /// element (i.e. doesn't have a closing tag).
    ///
    /// The children are passed in as a single argument to the returned function.
    pub fn tag<E: IntoElement<'bump>>(
        &self,
        name: &str,
        attributes: impl IntoIterator<Item = Attribute<'bump>>,
        void: bool,
    ) -> impl FnOnce(E) -> Element<'bump> {
        let bump = self.bump;
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
    pub fn doctype(
        &self,
        attributes: impl IntoIterator<Item = Attribute<'bump>>,
    ) -> Element<'bump> {
        Element::Tag {
            name: BumpString::from_str_in("!DOCTYPE", self.bump),
            attributes: BumpVec::from_iter_in(attributes, self.bump),
            children: BumpVec::new_in(self.bump),
            void: true,
        }
    }

    /// Create a fragment element from an iterator of elements.
    ///
    /// This is a convenience wrapper around [`Element::from_iter`].
    pub fn fragment(&self, children: impl IntoIterator<Item = Element<'bump>>) -> Element<'bump> {
        Element::from_iter(self.bump, children)
    }

    /// Create a document from an iterator of elements.
    ///
    /// This is a convenience wrapper around [`Document::new`].
    pub fn document(
        &self,
        children: impl IntoIterator<Item = Element<'bump>>,
    ) -> crate::Document<'bump> {
        crate::Document::new(self.bump, children)
    }

    /// Create a document with a doctype declaration followed by the given element.
    ///
    /// This is a convenience wrapper around [`Document::new_with_doctype`].
    pub fn document_with_doctype(&self, element: Element<'bump>) -> crate::Document<'bump> {
        crate::Document::new_with_doctype(self.bump, element)
    }
}

macro_rules! non_void_builders {
    ($($tag_ident:ident),*) => {
        impl<'bump> Builder<'bump> {
            $(
                #[doc = concat!("Create a non-void element with the tag name `", stringify!($tag_ident), "` and a list of attributes.\n\nThe children are passed in as a single argument to the returned function.")]
                pub fn $tag_ident<E: IntoElement<'bump>>(
                    &self,
                    attributes: impl IntoIterator<Item = Attribute<'bump>>,
                ) -> impl FnOnce(E) -> Element<'bump> {
                    self.tag(stringify!($tag_ident), attributes, false)
                }
            )*
        }
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
        impl<'bump> Builder<'bump> {
            $(
                #[doc = concat!("Create a void element with the tag name `", stringify!($tag_ident), "` and a list of attributes.")]
                pub fn $tag_ident(
                    &self,
                    attributes: impl IntoIterator<Item = Attribute<'bump>>,
                ) -> Element<'bump> {
                    self.tag(stringify!($tag_ident), attributes, true)(Element::Empty)
                }
            )*
        }
        /// A list of all void tags.
        pub const VOID_TAGS: &[&str] = &[$(stringify!($tag_ident)),*];
    };
}
void_builders! {
    area, base, br, col, embed, hr, input, link, meta,
    param, source, track, wbr, img
}
