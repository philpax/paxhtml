use bumpalo::collections::String as BumpString;
use bumpalo::collections::Vec as BumpVec;
use bumpalo::Bump;

use crate::Attribute;

/// An element in an HTML document. This is optimised for authoring, and supports both
/// [Element::Empty] and [Element::Fragment] for convenience.
///
/// These will be removed when converted to [crate::RenderElement]s.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Element<'bump> {
    /// An empty element.
    #[default]
    Empty,
    /// A tag element.
    Tag {
        /// The name of the tag.
        name: BumpString<'bump>,
        /// The attributes of the tag.
        attributes: BumpVec<'bump, Attribute<'bump>>,
        /// The children of the tag.
        children: BumpVec<'bump, Element<'bump>>,
        /// Whether the tag is void.
        void: bool,
    },
    /// A fragment element.
    Fragment {
        /// The children of the fragment.
        children: BumpVec<'bump, Element<'bump>>,
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

/// Trait for types that can be converted into an Element with a bump allocator.
pub trait IntoElement<'bump> {
    /// Convert this value into an Element using the given bump allocator.
    fn into_element(self, bump: &'bump Bump) -> Element<'bump>;
}
impl<'bump> IntoElement<'bump> for Element<'bump> {
    fn into_element(self, _bump: &'bump Bump) -> Element<'bump> {
        self
    }
}
impl<'bump> IntoElement<'bump> for &str {
    fn into_element(self, bump: &'bump Bump) -> Element<'bump> {
        Element::Text {
            text: BumpString::from_str_in(self, bump),
        }
    }
}
impl<'bump> IntoElement<'bump> for String {
    fn into_element(self, bump: &'bump Bump) -> Element<'bump> {
        Element::Text {
            text: BumpString::from_str_in(&self, bump),
        }
    }
}
impl<'bump> IntoElement<'bump> for &String {
    fn into_element(self, bump: &'bump Bump) -> Element<'bump> {
        Element::Text {
            text: BumpString::from_str_in(self, bump),
        }
    }
}
impl<'bump, T: IntoElement<'bump>> IntoElement<'bump> for Option<T> {
    fn into_element(self, bump: &'bump Bump) -> Element<'bump> {
        match self {
            Some(e) => e.into_element(bump),
            None => Element::Empty,
        }
    }
}
impl<'bump, const N: usize> IntoElement<'bump> for [Element<'bump>; N] {
    fn into_element(self, bump: &'bump Bump) -> Element<'bump> {
        if N == 0 {
            Element::Empty
        } else if N == 1 {
            self.into_iter().next().unwrap()
        } else {
            Element::Fragment {
                children: BumpVec::from_iter_in(self, bump),
            }
        }
    }
}
impl<'bump> Element<'bump> {
    /// Create an element from an iterator of elements.
    pub fn from_iter(
        bump: &'bump Bump,
        iter: impl IntoIterator<Item = Element<'bump>>,
    ) -> Element<'bump> {
        let children: BumpVec<'bump, Element<'bump>> = BumpVec::from_iter_in(iter, bump);
        if children.is_empty() {
            Element::Empty
        } else if children.len() == 1 {
            children.into_iter().next().unwrap()
        } else {
            Element::Fragment { children }
        }
    }

    /// Create a text element.
    pub fn text(bump: &'bump Bump, text: &str) -> Element<'bump> {
        Element::Text {
            text: BumpString::from_str_in(text, bump),
        }
    }

    /// Create a raw HTML element.
    pub fn raw(bump: &'bump Bump, html: &str) -> Element<'bump> {
        Element::Raw {
            html: BumpString::from_str_in(html, bump),
        }
    }

    /// Get the tag name of the element if it is a [`Tag`].
    pub fn tag(&self) -> Option<&str> {
        match self {
            Element::Tag { name, .. } => Some(name.as_str()),
            _ => None,
        }
    }

    /// Get the attributes of the element if it is a [`Tag`].
    pub fn attrs(&self) -> Option<&[Attribute<'bump>]> {
        match self {
            Element::Tag { attributes, .. } => Some(attributes.as_slice()),
            _ => None,
        }
    }

    /// Get the inner text of the element.
    ///
    /// This will return an empty string if no inner text exists.
    pub fn inner_text(&self, bump: &'bump Bump) -> BumpString<'bump> {
        match self {
            Element::Empty => BumpString::new_in(bump),
            Element::Tag { children, .. } => {
                let mut result = BumpString::new_in(bump);
                for child in children.iter() {
                    result.push_str(child.inner_text(bump).as_str());
                }
                result
            }
            Element::Fragment { children } => {
                let mut result = BumpString::new_in(bump);
                for child in children.iter() {
                    result.push_str(child.inner_text(bump).as_str());
                }
                result
            }
            Element::Text { text } => {
                let mut result = BumpString::new_in(bump);
                result.push_str(text.as_str());
                result
            }
            Element::Raw { .. } => BumpString::new_in(bump),
        }
    }

    /// Returns `true` if the element is [`Empty`].
    ///
    /// [`Empty`]: Element::Empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    /// Returns `true` if the element is [`Tag`].
    ///
    /// [`Tag`]: Element::Tag
    #[must_use]
    pub fn is_tag(&self) -> bool {
        matches!(self, Self::Tag { .. })
    }

    /// Returns `true` if the element is [`Fragment`].
    ///
    /// [`Fragment`]: Element::Fragment
    #[must_use]
    pub fn is_fragment(&self) -> bool {
        matches!(self, Self::Fragment { .. })
    }

    /// Returns `true` if the element is [`Text`].
    ///
    /// [`Text`]: Element::Text
    #[must_use]
    pub fn is_text(&self) -> bool {
        matches!(self, Self::Text { .. })
    }

    /// Returns `true` if the element is [`Raw`].
    ///
    /// [`Raw`]: Element::Raw
    #[must_use]
    pub fn is_raw(&self) -> bool {
        matches!(self, Self::Raw { .. })
    }
}
