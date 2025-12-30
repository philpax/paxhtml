//! Owned types that use standard heap allocation.
//!
//! These types are useful for interoperability with systems that require
//! owned data (like serde serialization or Lua bindings). They can be
//! converted to bump-allocated types using the `into_bump` methods.

use bumpalo::collections::String as BumpString;
use bumpalo::collections::Vec as BumpVec;
use bumpalo::Bump;

use crate::{Attribute, Element};

/// An owned attribute using standard heap allocation.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OwnedAttribute {
    /// The key of the attribute.
    pub key: String,
    /// The value of the attribute.
    pub value: Option<String>,
}
impl OwnedAttribute {
    /// Create a new owned attribute with a key and value.
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        OwnedAttribute {
            key: key.into(),
            value: Some(value.into()),
        }
    }

    /// Create a boolean attribute (no value).
    pub fn boolean(key: impl Into<String>) -> Self {
        OwnedAttribute {
            key: key.into(),
            value: None,
        }
    }

    /// Convert this owned attribute into a bump-allocated attribute.
    pub fn into_bump<'bump>(self, bump: &'bump Bump) -> Attribute<'bump> {
        Attribute {
            key: BumpString::from_str_in(&self.key, bump),
            value: self.value.map(|v| BumpString::from_str_in(&v, bump)),
        }
    }
}
impl From<&str> for OwnedAttribute {
    fn from(s: &str) -> Self {
        OwnedAttribute::boolean(s)
    }
}
impl From<String> for OwnedAttribute {
    fn from(s: String) -> Self {
        OwnedAttribute::boolean(s)
    }
}
impl From<(&str, &str)> for OwnedAttribute {
    fn from((key, value): (&str, &str)) -> Self {
        OwnedAttribute::new(key, value)
    }
}
impl From<(String, String)> for OwnedAttribute {
    fn from((key, value): (String, String)) -> Self {
        OwnedAttribute::new(key, value)
    }
}

/// An owned element using standard heap allocation.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type"))]
pub enum OwnedElement {
    #[default]
    /// An empty element.
    Empty,
    /// A tag element.
    Tag {
        /// The name of the tag.
        name: String,
        /// The attributes of the tag.
        attributes: Vec<OwnedAttribute>,
        /// The children of the tag.
        children: Vec<OwnedElement>,
        /// Whether the tag is void.
        void: bool,
    },
    /// A fragment element.
    Fragment {
        /// The children of the fragment.
        children: Vec<OwnedElement>,
    },
    /// A text element.
    Text {
        /// The text of the element.
        text: String,
    },
    /// A raw element.
    Raw {
        /// The raw HTML of the element.
        html: String,
    },
}
impl OwnedElement {
    /// Convert this owned element into a bump-allocated element.
    pub fn into_bump<'bump>(self, bump: &'bump Bump) -> Element<'bump> {
        match self {
            OwnedElement::Empty => Element::Empty,
            OwnedElement::Tag {
                name,
                attributes,
                children,
                void,
            } => Element::Tag {
                name: BumpString::from_str_in(&name, bump),
                attributes: BumpVec::from_iter_in(
                    attributes.into_iter().map(|a| a.into_bump(bump)),
                    bump,
                ),
                children: BumpVec::from_iter_in(
                    children.into_iter().map(|c| c.into_bump(bump)),
                    bump,
                ),
                void,
            },
            OwnedElement::Fragment { children } => Element::Fragment {
                children: BumpVec::from_iter_in(
                    children.into_iter().map(|c| c.into_bump(bump)),
                    bump,
                ),
            },
            OwnedElement::Text { text } => Element::Text {
                text: BumpString::from_str_in(&text, bump),
            },
            OwnedElement::Raw { html } => Element::Raw {
                html: BumpString::from_str_in(&html, bump),
            },
        }
    }
}
impl From<String> for OwnedElement {
    fn from(s: String) -> Self {
        OwnedElement::Text { text: s }
    }
}
impl From<&str> for OwnedElement {
    fn from(s: &str) -> Self {
        OwnedElement::Text {
            text: s.to_string(),
        }
    }
}
impl From<Vec<OwnedElement>> for OwnedElement {
    fn from(children: Vec<OwnedElement>) -> Self {
        if children.is_empty() {
            OwnedElement::Empty
        } else if children.len() == 1 {
            children.into_iter().next().unwrap()
        } else {
            OwnedElement::Fragment { children }
        }
    }
}
impl FromIterator<OwnedElement> for OwnedElement {
    fn from_iter<I: IntoIterator<Item = OwnedElement>>(iter: I) -> Self {
        iter.into_iter().collect::<Vec<_>>().into()
    }
}
