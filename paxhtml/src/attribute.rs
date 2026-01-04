use bumpalo::collections::String as BumpString;
use bumpalo::collections::Vec as BumpVec;
use bumpalo::Bump;

/// Represents a typed attribute value.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "value"))]
pub enum AttributeValue<'bump> {
    /// A string value.
    String(BumpString<'bump>),
    /// An integer value.
    Int(i128),
    /// A floating-point value.
    Float(f64),
    /// A boolean value.
    Bool(bool),
}

impl<'bump> AttributeValue<'bump> {
    /// Returns the value as a string slice if this is a String variant.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            AttributeValue::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Convert the attribute value to its string representation for HTML output.
    ///
    /// This allocates a new string in the bump allocator for non-string variants.
    pub fn to_str(&self, bump: &'bump Bump) -> BumpString<'bump> {
        match self {
            AttributeValue::String(s) => {
                let mut result = BumpString::new_in(bump);
                result.push_str(s.as_str());
                result
            }
            AttributeValue::Int(i) => BumpString::from_str_in(&i.to_string(), bump),
            AttributeValue::Float(f) => BumpString::from_str_in(&f.to_string(), bump),
            AttributeValue::Bool(b) => BumpString::from_str_in(&b.to_string(), bump),
        }
    }

    /// Returns the integer value if this is an Int variant.
    pub fn as_int(&self) -> Option<i128> {
        match self {
            AttributeValue::Int(i) => Some(*i),
            _ => None,
        }
    }

    /// Returns the float value if this is a Float variant.
    pub fn as_float(&self) -> Option<f64> {
        match self {
            AttributeValue::Float(f) => Some(*f),
            _ => None,
        }
    }

    /// Returns the boolean value if this is a Bool variant.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            AttributeValue::Bool(b) => Some(*b),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// A key-value pair for an HTML attribute.
pub struct Attribute<'bump> {
    /// The key of the attribute.
    pub key: BumpString<'bump>,
    /// The value of the attribute.
    pub value: Option<AttributeValue<'bump>>,
}

impl<'bump> Attribute<'bump> {
    /// Create a new attribute with a string key and value.
    pub fn new(bump: &'bump Bump, key: &str, value: &str) -> Self {
        Attribute {
            key: BumpString::from_str_in(key, bump),
            value: Some(AttributeValue::String(BumpString::from_str_in(value, bump))),
        }
    }

    /// Create a new attribute with an integer value.
    pub fn new_int(bump: &'bump Bump, key: &str, value: i128) -> Self {
        Attribute {
            key: BumpString::from_str_in(key, bump),
            value: Some(AttributeValue::Int(value)),
        }
    }

    /// Create a new attribute with a float value.
    pub fn new_float(bump: &'bump Bump, key: &str, value: f64) -> Self {
        Attribute {
            key: BumpString::from_str_in(key, bump),
            value: Some(AttributeValue::Float(value)),
        }
    }

    /// Create a new attribute with a boolean value.
    pub fn new_bool(bump: &'bump Bump, key: &str, value: bool) -> Self {
        Attribute {
            key: BumpString::from_str_in(key, bump),
            value: Some(AttributeValue::Bool(value)),
        }
    }

    /// Create a boolean attribute (no value).
    pub fn boolean(bump: &'bump Bump, key: &str) -> Self {
        Attribute {
            key: BumpString::from_str_in(key, bump),
            value: None,
        }
    }

    /// Create an attribute with an optional string value.
    pub fn with_optional_value(bump: &'bump Bump, key: &str, value: Option<&str>) -> Self {
        Attribute {
            key: BumpString::from_str_in(key, bump),
            value: value.map(|v| AttributeValue::String(BumpString::from_str_in(v, bump))),
        }
    }

    /// Get the value as a string slice, if present and is a String variant.
    ///
    /// For non-string values, this returns `None`. Use `value_to_str()` to get
    /// the string representation of any value type.
    pub fn value_as_str(&self) -> Option<&str> {
        self.value.as_ref().and_then(|v| v.as_str())
    }

    /// Get the string representation of the value for HTML output.
    ///
    /// This allocates a new string in the bump allocator for non-string variants.
    pub fn value_to_str(&self, bump: &'bump Bump) -> Option<BumpString<'bump>> {
        self.value.as_ref().map(|v| v.to_str(bump))
    }
}

/// Trait for types that can be converted into an Attribute with a bump allocator.
pub trait IntoAttribute<'bump> {
    /// Convert this value into an Attribute using the given bump allocator.
    fn into_attribute(self, bump: &'bump Bump) -> Attribute<'bump>;
}
impl<'bump> IntoAttribute<'bump> for Attribute<'bump> {
    fn into_attribute(self, _bump: &'bump Bump) -> Attribute<'bump> {
        self
    }
}
impl<'bump> IntoAttribute<'bump> for &str {
    fn into_attribute(self, bump: &'bump Bump) -> Attribute<'bump> {
        Attribute::boolean(bump, self)
    }
}
impl<'bump> IntoAttribute<'bump> for (&str, &str) {
    fn into_attribute(self, bump: &'bump Bump) -> Attribute<'bump> {
        Attribute::new(bump, self.0, self.1)
    }
}
impl<'bump> IntoAttribute<'bump> for (&str, String) {
    fn into_attribute(self, bump: &'bump Bump) -> Attribute<'bump> {
        Attribute::new(bump, self.0, &self.1)
    }
}
impl<'bump> IntoAttribute<'bump> for (String, &str) {
    fn into_attribute(self, bump: &'bump Bump) -> Attribute<'bump> {
        Attribute::new(bump, &self.0, self.1)
    }
}
impl<'bump> IntoAttribute<'bump> for (String, String) {
    fn into_attribute(self, bump: &'bump Bump) -> Attribute<'bump> {
        Attribute::new(bump, &self.0, &self.1)
    }
}

/// Error type for attribute parsing failures
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttributeParseError {
    /// The attribute value has an unclosed quote
    ///
    /// Contains:
    /// - The quote character that was not closed
    /// - The position of the unclosed quote
    /// - The partial attribute value that was parsed
    UnclosedQuote {
        /// The quote character that was not closed (either ' or ")
        quote: char,
        /// The position of the unclosed quote in the input string
        position: usize,
        /// The partial attribute value that was parsed
        partial_value: String,
    },
    /// The attribute syntax is invalid
    ///
    /// Contains:
    /// - The unexpected character that caused the error
    /// - The position where the error occurred
    /// - The context of what was being parsed
    InvalidSyntax {
        /// The unexpected character that caused the error
        unexpected: char,
        /// The position where the error occurred
        position: usize,
        /// What was being parsed when the error occurred
        context: ParseContext,
    },
}
/// Context of what was being parsed when an error occurred
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub enum ParseContext {
    /// Expected an attribute name but found something else
    ExpectedAttributeName,
    /// Expected an attribute value but found something else
    ExpectedAttributeValue,
    /// Expected a quote or valid unquoted value character but found something else
    ExpectedQuoteOrValue,
}
impl std::fmt::Display for AttributeParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttributeParseError::UnclosedQuote {
                quote,
                position,
                partial_value,
            } => {
                write!(
                    f,
                    "Unclosed quote '{quote}' at position {position} with partial value '{partial_value}'"
                )
            }
            AttributeParseError::InvalidSyntax {
                unexpected,
                position,
                context,
            } => {
                write!(
                    f,
                    "Invalid syntax at position {}: unexpected character '{}' while {}",
                    position,
                    unexpected,
                    match context {
                        ParseContext::ExpectedAttributeName => "expecting attribute name",
                        ParseContext::ExpectedAttributeValue => "expecting attribute value",
                        ParseContext::ExpectedQuoteOrValue =>
                            "expecting quote or valid value character",
                    }
                )
            }
        }
    }
}
impl std::error::Error for AttributeParseError {}

impl<'bump> Attribute<'bump> {
    /// Parse a string of attributes into a vector of attributes.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use paxhtml::{bumpalo::Bump, Attribute};
    ///
    /// let bump = Bump::new();
    /// let attributes = Attribute::parse_from_str(&bump, r#"id="my-id" class="my-class my-class-2" some-attr"#).unwrap();
    /// assert_eq!(attributes.len(), 3);
    /// assert_eq!(attributes[0].key.as_str(), "id");
    /// assert_eq!(attributes[0].value_as_str(), Some("my-id"));
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns an error if the string does not respect the HTML attribute syntax.
    /// The error will contain detailed information about what caused the parsing failure,
    /// including the position of the error and the context of what was being parsed.
    pub fn parse_from_str(
        bump: &'bump Bump,
        s: &str,
    ) -> Result<BumpVec<'bump, Self>, AttributeParseError> {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        enum ParseState {
            BeforeAttribute,
            InName,
            BeforeEquals,
            AfterEquals,
            InQuotedValue,
            InUnquotedValue,
        }

        let mut attributes = BumpVec::new_in(bump);
        let mut chars = s.chars().enumerate().peekable();
        let mut current_key = String::new();
        let mut current_value: Option<String> = None;
        let mut in_quotes = false;
        let mut quote_char = None;
        let mut quote_start_pos = 0;
        let mut state = ParseState::BeforeAttribute;

        while let Some((pos, c)) = chars.next() {
            match state {
                ParseState::BeforeAttribute => {
                    match c {
                        ' ' | '\t' | '\n' => continue, // Skip whitespace between attributes
                        '=' => {
                            return Err(AttributeParseError::InvalidSyntax {
                                unexpected: c,
                                position: pos,
                                context: ParseContext::ExpectedAttributeName,
                            })
                        }
                        _ => {
                            current_key.push(c);
                            state = ParseState::InName;
                        }
                    }
                }
                ParseState::InName => {
                    match c {
                        ' ' | '\t' | '\n' => {
                            // Look ahead to see if there's an equals sign
                            let temp_iter = chars.clone();
                            let mut found_equals = false;
                            for (_, next_c) in temp_iter {
                                if next_c == '=' {
                                    found_equals = true;
                                    break;
                                }
                                if !next_c.is_whitespace() {
                                    break;
                                }
                            }
                            if found_equals {
                                state = ParseState::BeforeEquals;
                            } else {
                                // This is a boolean attribute
                                attributes.push(Attribute::boolean(bump, &current_key));
                                current_key.clear();
                                state = ParseState::BeforeAttribute;
                            }
                        }
                        '=' => {
                            state = ParseState::AfterEquals;
                            current_value = Some(String::new());
                        }
                        _ => current_key.push(c),
                    }
                }
                ParseState::BeforeEquals => {
                    match c {
                        ' ' | '\t' | '\n' => continue, // Skip whitespace before equals
                        '=' => {
                            state = ParseState::AfterEquals;
                            current_value = Some(String::new());
                        }
                        _ => {
                            return Err(AttributeParseError::InvalidSyntax {
                                unexpected: c,
                                position: pos,
                                context: ParseContext::ExpectedAttributeValue,
                            })
                        }
                    }
                }
                ParseState::AfterEquals => {
                    match c {
                        ' ' | '\t' | '\n' => continue, // Skip whitespace after equals
                        '"' | '\'' => {
                            quote_char = Some(c);
                            quote_start_pos = pos;
                            in_quotes = true;
                            state = ParseState::InQuotedValue;
                        }
                        _ => {
                            if let Some(ref mut value) = current_value {
                                if !c.is_alphanumeric() && c != '-' && c != '_' {
                                    return Err(AttributeParseError::InvalidSyntax {
                                        unexpected: c,
                                        position: pos,
                                        context: ParseContext::ExpectedQuoteOrValue,
                                    });
                                }
                                value.push(c);
                                state = ParseState::InUnquotedValue;
                            }
                        }
                    }
                }
                ParseState::InQuotedValue => {
                    if Some(c) == quote_char {
                        in_quotes = false;
                        attributes.push(Attribute::with_optional_value(
                            bump,
                            &current_key,
                            current_value.as_deref(),
                        ));
                        current_key.clear();
                        current_value = None;
                        state = ParseState::BeforeAttribute;
                    } else if let Some(ref mut value) = current_value {
                        value.push(c);
                    }
                }
                ParseState::InUnquotedValue => match c {
                    ' ' | '\t' | '\n' => {
                        attributes.push(Attribute::with_optional_value(
                            bump,
                            &current_key,
                            current_value.as_deref(),
                        ));
                        current_key.clear();
                        current_value = None;
                        state = ParseState::BeforeAttribute;
                    }
                    _ => {
                        if !c.is_alphanumeric() && c != '-' && c != '_' {
                            return Err(AttributeParseError::InvalidSyntax {
                                unexpected: c,
                                position: pos,
                                context: ParseContext::ExpectedQuoteOrValue,
                            });
                        }
                        if let Some(ref mut value) = current_value {
                            value.push(c);
                        }
                    }
                },
            }
        }

        // Handle the last attribute if any
        if in_quotes {
            return Err(AttributeParseError::UnclosedQuote {
                quote: quote_char.unwrap(),
                position: quote_start_pos,
                partial_value: current_value.unwrap_or_default(),
            });
        }

        if !current_key.is_empty() {
            attributes.push(Attribute::with_optional_value(
                bump,
                &current_key,
                current_value.as_deref(),
            ));
        }

        Ok(attributes)
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_attribute() {
        let bump = Bump::new();
        let attributes = Attribute::parse_from_str(&bump, "id=\"test\"").unwrap();
        assert_eq!(attributes.len(), 1);
        assert_eq!(attributes[0].key.as_str(), "id");
        assert_eq!(attributes[0].value_as_str(), Some("test"));
    }

    #[test]
    fn test_parse_multiple_attributes() {
        let bump = Bump::new();
        let attributes =
            Attribute::parse_from_str(&bump, "id=\"test\" class=\"btn btn-primary\"").unwrap();
        assert_eq!(attributes.len(), 2);
        assert_eq!(attributes[0].key.as_str(), "id");
        assert_eq!(attributes[0].value_as_str(), Some("test"));
        assert_eq!(attributes[1].key.as_str(), "class");
        assert_eq!(attributes[1].value_as_str(), Some("btn btn-primary"));
    }

    #[test]
    fn test_parse_boolean_attribute() {
        let bump = Bump::new();
        let attributes = Attribute::parse_from_str(&bump, "disabled").unwrap();
        assert_eq!(attributes.len(), 1);
        assert_eq!(attributes[0].key.as_str(), "disabled");
        assert_eq!(attributes[0].value, None);
    }

    #[test]
    fn test_parse_mixed_attributes() {
        let bump = Bump::new();
        let attributes =
            Attribute::parse_from_str(&bump, "id=\"test\" disabled class=\"btn\"").unwrap();
        assert_eq!(attributes.len(), 3);
        assert_eq!(attributes[0].key.as_str(), "id");
        assert_eq!(attributes[0].value_as_str(), Some("test"));
        assert_eq!(attributes[1].key.as_str(), "disabled");
        assert_eq!(attributes[1].value, None);
        assert_eq!(attributes[2].key.as_str(), "class");
        assert_eq!(attributes[2].value_as_str(), Some("btn"));
    }

    #[test]
    fn test_parse_error_unclosed_quote() {
        let bump = Bump::new();
        let result = Attribute::parse_from_str(&bump, "id=\"test");
        match result {
            Err(AttributeParseError::UnclosedQuote {
                quote,
                position,
                partial_value,
            }) => {
                assert_eq!(quote, '"');
                assert_eq!(position, 3);
                assert_eq!(partial_value, "test");
            }
            _ => panic!("Expected UnclosedQuote error"),
        }
    }

    #[test]
    fn test_parse_error_invalid_syntax() {
        let bump = Bump::new();
        let result = Attribute::parse_from_str(&bump, "=value");
        match result {
            Err(AttributeParseError::InvalidSyntax {
                unexpected,
                position,
                context,
            }) => {
                assert_eq!(unexpected, '=');
                assert_eq!(position, 0);
                assert_eq!(context, ParseContext::ExpectedAttributeName);
            }
            _ => panic!("Expected InvalidSyntax error"),
        }
    }

    #[test]
    fn test_parse_with_single_quotes() {
        let bump = Bump::new();
        let attributes = Attribute::parse_from_str(&bump, "id='test'").unwrap();
        assert_eq!(attributes.len(), 1);
        assert_eq!(attributes[0].key.as_str(), "id");
        assert_eq!(attributes[0].value_as_str(), Some("test"));
    }

    #[test]
    fn test_parse_with_whitespace() {
        let bump = Bump::new();
        let attributes =
            Attribute::parse_from_str(&bump, "  id=\"test\"  \n  class=\"btn\"  ").unwrap();
        assert_eq!(attributes.len(), 2);
        assert_eq!(attributes[0].key.as_str(), "id");
        assert_eq!(attributes[0].value_as_str(), Some("test"));
        assert_eq!(attributes[1].key.as_str(), "class");
        assert_eq!(attributes[1].value_as_str(), Some("btn"));
    }

    #[test]
    fn test_parse_error_unclosed_quote_with_context() {
        let bump = Bump::new();
        let result = Attribute::parse_from_str(&bump, "id=\"test class='value");
        match result {
            Err(AttributeParseError::UnclosedQuote {
                quote,
                position,
                partial_value,
            }) => {
                assert_eq!(quote, '"');
                assert_eq!(position, 3);
                assert_eq!(partial_value, "test class='value");
            }
            _ => panic!("Expected UnclosedQuote error with context"),
        }
    }

    #[test]
    fn test_parse_error_invalid_syntax_with_context() {
        let bump = Bump::new();
        let result = Attribute::parse_from_str(&bump, "=invalid");
        match result {
            Err(AttributeParseError::InvalidSyntax {
                unexpected,
                position,
                context,
            }) => {
                assert_eq!(unexpected, '=');
                assert_eq!(position, 0);
                assert_eq!(context, ParseContext::ExpectedAttributeName);
            }
            _ => panic!("Expected InvalidSyntax error with context"),
        }
    }

    #[test]
    fn test_parse_error_invalid_unquoted_value() {
        let bump = Bump::new();
        let result = Attribute::parse_from_str(&bump, "id=test!");
        match result {
            Err(AttributeParseError::InvalidSyntax {
                unexpected,
                position,
                context,
            }) => {
                assert_eq!(unexpected, '!');
                assert_eq!(position, 7);
                assert_eq!(context, ParseContext::ExpectedQuoteOrValue);
            }
            _ => panic!("Expected InvalidSyntax error with context"),
        }
    }

    #[test]
    fn test_parse_space_after_key() {
        let bump = Bump::new();
        let attributes = Attribute::parse_from_str(&bump, r#"width ="150" height="80""#).unwrap();
        assert_eq!(attributes.len(), 2);
        assert_eq!(attributes[0].key.as_str(), "width");
        assert_eq!(attributes[0].value_as_str(), Some("150"));
        assert_eq!(attributes[1].key.as_str(), "height");
        assert_eq!(attributes[1].value_as_str(), Some("80"));
    }
}
