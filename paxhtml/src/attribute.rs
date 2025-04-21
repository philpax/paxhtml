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
                    "Unclosed quote '{}' at position {} with partial value '{}'",
                    quote, position, partial_value
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

impl Attribute {
    /// Parse a string of attributes into a vector of attributes.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use paxhtml::Attribute;
    ///
    /// let attributes = Attribute::parse_from_str(r#"id="my-id" class="my-class my-class-2" some-attr"#).unwrap();
    /// assert_eq!(attributes, vec![
    ///     ("id", "my-id").into(),
    ///     ("class", "my-class my-class-2").into(),
    ///     "some-attr".into()
    /// ]);
    /// ```
    ///
    /// ## Errors
    ///
    /// Returns an error if the string does not respect the HTML attribute syntax.
    /// The error will contain detailed information about what caused the parsing failure,
    /// including the position of the error and the context of what was being parsed.
    pub fn parse_from_str(s: &str) -> Result<Vec<Self>, AttributeParseError> {
        let mut attributes = Vec::new();
        let mut chars = s.chars().enumerate().peekable();
        let mut current_key = String::new();
        let mut current_value = None;
        let mut in_quotes = false;
        let mut quote_char = None;
        let mut quote_start_pos = 0;

        while let Some((pos, c)) = chars.next() {
            match c {
                ' ' | '\t' | '\n' if !in_quotes => {
                    if !current_key.is_empty() {
                        attributes.push(Attribute {
                            key: current_key.clone(),
                            value: current_value.take(),
                        });
                        current_key.clear();
                    }
                }
                '=' if !in_quotes => {
                    if current_key.is_empty() {
                        return Err(AttributeParseError::InvalidSyntax {
                            unexpected: '=',
                            position: pos,
                            context: ParseContext::ExpectedAttributeName,
                        });
                    }
                    current_value = Some(String::new());
                }
                '"' | '\'' if !in_quotes && current_value.is_some() => {
                    in_quotes = true;
                    quote_char = Some(c);
                    quote_start_pos = pos;
                }
                c if in_quotes => {
                    if Some(c) == quote_char {
                        in_quotes = false;
                        quote_char = None;
                    } else if let Some(ref mut value) = current_value {
                        value.push(c);
                    }
                }
                c if !current_key.is_empty() && current_value.is_some() && !in_quotes => {
                    // For unquoted values, only allow alphanumeric characters, hyphens, and underscores
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
                c => {
                    if let Some(ref mut value) = current_value {
                        value.push(c);
                    } else {
                        current_key.push(c);
                    }
                }
            }
        }

        if in_quotes {
            return Err(AttributeParseError::UnclosedQuote {
                quote: quote_char.unwrap(),
                position: quote_start_pos,
                partial_value: current_value.unwrap_or_default(),
            });
        }

        if !current_key.is_empty() {
            attributes.push(Attribute {
                key: current_key,
                value: current_value,
            });
        }

        Ok(attributes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_attribute() {
        let attributes = Attribute::parse_from_str("id=\"test\"").unwrap();
        assert_eq!(attributes.len(), 1);
        assert_eq!(attributes[0].key, "id");
        assert_eq!(attributes[0].value, Some("test".to_string()));
    }

    #[test]
    fn test_parse_multiple_attributes() {
        let attributes =
            Attribute::parse_from_str("id=\"test\" class=\"btn btn-primary\"").unwrap();
        assert_eq!(attributes.len(), 2);
        assert_eq!(attributes[0].key, "id");
        assert_eq!(attributes[0].value, Some("test".to_string()));
        assert_eq!(attributes[1].key, "class");
        assert_eq!(attributes[1].value, Some("btn btn-primary".to_string()));
    }

    #[test]
    fn test_parse_boolean_attribute() {
        let attributes = Attribute::parse_from_str("disabled").unwrap();
        assert_eq!(attributes.len(), 1);
        assert_eq!(attributes[0].key, "disabled");
        assert_eq!(attributes[0].value, None);
    }

    #[test]
    fn test_parse_mixed_attributes() {
        let attributes = Attribute::parse_from_str("id=\"test\" disabled class=\"btn\"").unwrap();
        assert_eq!(attributes.len(), 3);
        assert_eq!(attributes[0].key, "id");
        assert_eq!(attributes[0].value, Some("test".to_string()));
        assert_eq!(attributes[1].key, "disabled");
        assert_eq!(attributes[1].value, None);
        assert_eq!(attributes[2].key, "class");
        assert_eq!(attributes[2].value, Some("btn".to_string()));
    }

    #[test]
    fn test_parse_error_unclosed_quote() {
        let result = Attribute::parse_from_str("id=\"test");
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
        let result = Attribute::parse_from_str("=value");
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
        let attributes = Attribute::parse_from_str("id='test'").unwrap();
        assert_eq!(attributes.len(), 1);
        assert_eq!(attributes[0].key, "id");
        assert_eq!(attributes[0].value, Some("test".to_string()));
    }

    #[test]
    fn test_parse_with_whitespace() {
        let attributes = Attribute::parse_from_str("  id=\"test\"  \n  class=\"btn\"  ").unwrap();
        assert_eq!(attributes.len(), 2);
        assert_eq!(attributes[0].key, "id");
        assert_eq!(attributes[0].value, Some("test".to_string()));
        assert_eq!(attributes[1].key, "class");
        assert_eq!(attributes[1].value, Some("btn".to_string()));
    }

    #[test]
    fn test_parse_error_unclosed_quote_with_context() {
        let result = Attribute::parse_from_str("id=\"test class='value");
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
        let result = Attribute::parse_from_str("=invalid");
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
        let result = Attribute::parse_from_str("id=test!");
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
}
