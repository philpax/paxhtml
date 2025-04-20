#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// A key-value pair for an HTML attribute.
pub struct Attribute {
    /// The key of the attribute.
    pub key: String,
    /// The value of the attribute.
    pub value: Option<String>,
}
/// Create an attribute from a value that implements [Into<Attribute>].
pub fn attr(value: impl Into<Attribute>) -> Attribute {
    value.into()
}
impl From<&str> for Attribute {
    fn from(s: &str) -> Self {
        Attribute {
            key: s.to_string(),
            value: None,
        }
    }
}
impl From<String> for Attribute {
    fn from(s: String) -> Self {
        Attribute {
            key: s,
            value: None,
        }
    }
}
impl From<Attribute> for (String, Option<String>) {
    fn from(a: Attribute) -> Self {
        (a.key, a.value)
    }
}
impl From<(&str, &str)> for Attribute {
    fn from((key, value): (&str, &str)) -> Self {
        Attribute {
            key: key.to_string(),
            value: Some(value.to_string()),
        }
    }
}
impl From<(&str, String)> for Attribute {
    fn from((key, value): (&str, String)) -> Self {
        Attribute {
            key: key.to_string(),
            value: Some(value),
        }
    }
}
impl From<(String, &str)> for Attribute {
    fn from((key, value): (String, &str)) -> Self {
        Attribute {
            key,
            value: Some(value.to_string()),
        }
    }
}
impl From<(String, String)> for Attribute {
    fn from((key, value): (String, String)) -> Self {
        Attribute {
            key,
            value: Some(value),
        }
    }
}
