use crate::Attribute;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// An element in an HTML document. This is optimised for authoring, and supports both
/// [Element::Empty] and [Element::Fragment] for convenience.
///
/// These will be removed when converted to [crate::RenderElement]s.
pub enum Element {
    #[default]
    /// An empty element.
    Empty,
    /// A tag element.
    Tag {
        /// The name of the tag.
        name: String,
        /// The attributes of the tag.
        attributes: Vec<Attribute>,
        /// The children of the tag.
        children: Vec<Element>,
        /// Whether the tag is void.
        void: bool,
    },
    /// A fragment element.
    Fragment {
        /// The children of the fragment.
        children: Vec<Element>,
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
impl From<String> for Element {
    fn from(s: String) -> Self {
        Element::Text { text: s }
    }
}
impl From<&str> for Element {
    fn from(s: &str) -> Self {
        s.to_string().into()
    }
}
impl From<&String> for Element {
    fn from(s: &String) -> Self {
        s.clone().into()
    }
}
impl From<Vec<Element>> for Element {
    fn from(children: Vec<Element>) -> Self {
        if children.is_empty() {
            Element::Empty
        } else if children.len() == 1 {
            children[0].clone()
        } else {
            Element::Fragment { children }
        }
    }
}
impl From<&[Element]> for Element {
    fn from(children: &[Element]) -> Self {
        children.to_vec().into()
    }
}
impl<const N: usize> From<[Element; N]> for Element {
    fn from(children: [Element; N]) -> Self {
        children.to_vec().into()
    }
}
impl From<Option<Element>> for Element {
    fn from(element: Option<Element>) -> Self {
        element.unwrap_or(Element::Empty)
    }
}
impl FromIterator<Element> for Element {
    fn from_iter<I: IntoIterator<Item = Element>>(iter: I) -> Self {
        iter.into_iter().collect::<Vec<_>>().into()
    }
}
impl Element {
    /// Get the tag name of the element if it is a [`Tag`].
    pub fn tag(&self) -> Option<&str> {
        match self {
            Element::Tag { name, .. } => Some(name),
            _ => None,
        }
    }

    /// Get the attributes of the element if it is a [`Tag`].
    pub fn attrs(&self) -> Option<&[Attribute]> {
        match self {
            Element::Tag { attributes, .. } => Some(attributes),
            _ => None,
        }
    }

    /// Get the inner text of the element.
    ///
    /// This will return an empty string if no inner text exists.
    pub fn inner_text(&self) -> String {
        match self {
            Element::Empty => String::new(),
            Element::Tag { children, .. } => children.iter().map(Element::inner_text).collect(),
            Element::Fragment { children } => children.iter().map(Element::inner_text).collect(),
            Element::Text { text } => text.clone(),
            Element::Raw { .. } => String::new(),
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
