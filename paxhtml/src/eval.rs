use bumpalo::collections::String as BumpString;
use bumpalo::collections::Vec as BumpVec;
use bumpalo::Bump;

use crate::{Attribute, Element};
use paxhtml_parser::{AstAttribute, AstNode, AttributeValue, ParseError};
use std::fmt;

/// Error type for AST evaluation
#[derive(Debug, Clone)]
pub enum EvalError {
    /// Interpolation is not supported at runtime
    InterpolationNotSupported,
    /// Expression attributes are not supported at runtime
    ExpressionAttributeNotSupported,
}
impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvalError::InterpolationNotSupported => {
                write!(
                    f,
                    "Interpolation is not supported in runtime HTML evaluation"
                )
            }
            EvalError::ExpressionAttributeNotSupported => {
                write!(
                    f,
                    "Expression attributes are not supported in runtime HTML evaluation"
                )
            }
        }
    }
}
impl std::error::Error for EvalError {}

/// Error type for runtime HTML parsing
#[derive(Debug)]
pub enum ParseHtmlError {
    /// Error parsing the HTML syntax
    Parse(ParseError),
    /// Error evaluating the AST (e.g., interpolation not supported)
    Eval(EvalError),
}
impl std::fmt::Display for ParseHtmlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseHtmlError::Parse(e) => write!(f, "Parse error: {}", e),
            ParseHtmlError::Eval(e) => write!(f, "Evaluation error: {}", e),
        }
    }
}
impl std::error::Error for ParseHtmlError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ParseHtmlError::Parse(e) => Some(e),
            ParseHtmlError::Eval(e) => Some(e),
        }
    }
}
impl From<ParseError> for ParseHtmlError {
    fn from(e: ParseError) -> Self {
        ParseHtmlError::Parse(e)
    }
}

impl From<EvalError> for ParseHtmlError {
    fn from(e: EvalError) -> Self {
        ParseHtmlError::Eval(e)
    }
}

/// Parse an HTML string into a runtime [Element] tree.
///
/// This function parses HTML at runtime and returns an [Element] tree that can be
/// used with [Document] or other paxhtml APIs.
///
/// # Example
///
/// ```
/// use paxhtml::{parse_html, Document, Bump};
///
/// let bump = Bump::new();
/// let element = parse_html(&bump, r#"<div class="container"><p>"Hello, world!"</p></div>"#).unwrap();
/// let doc = Document::new(&bump, [element]);
/// let html = doc.write_to_string();
/// ```
///
/// # Errors
///
/// Returns a [ParseHtmlError] if the HTML is malformed or contains features not
/// supported at runtime (like interpolation syntax).
pub fn parse_html<'bump>(bump: &'bump Bump, html: &str) -> Result<Element<'bump>, ParseHtmlError> {
    let ast = paxhtml_parser::parse_html(html)?;
    let element = eval_node(bump, &ast)?;
    Ok(element)
}

/// Convert an AST node to a runtime Element
pub fn eval_node<'bump>(bump: &'bump Bump, node: &AstNode) -> Result<Element<'bump>, EvalError> {
    match node {
        AstNode::Element {
            name,
            attributes,
            children,
            void,
        } => {
            let mut attrs = BumpVec::new_in(bump);
            for attr in attributes {
                attrs.push(eval_attribute(bump, attr)?);
            }

            let mut child_elements = BumpVec::new_in(bump);
            for child in children {
                child_elements.push(eval_node(bump, child)?);
            }

            Ok(Element::Tag {
                name: BumpString::from_str_in(name, bump),
                attributes: attrs,
                children: child_elements,
                void: *void,
            })
        }
        AstNode::Fragment(children) => {
            let mut child_elements = BumpVec::new_in(bump);
            for child in children {
                child_elements.push(eval_node(bump, child)?);
            }

            Ok(Element::Fragment {
                children: child_elements,
            })
        }
        AstNode::Expression { .. } => Err(EvalError::InterpolationNotSupported),
        AstNode::Text(text) => Ok(Element::Text {
            text: BumpString::from_str_in(text, bump),
        }),
    }
}

/// Convert an AST attribute to a runtime Attribute
fn eval_attribute<'bump>(
    bump: &'bump Bump,
    attr: &AstAttribute,
) -> Result<Attribute<'bump>, EvalError> {
    match attr {
        AstAttribute::Named { name, value } => {
            let val = match value {
                None => None,
                Some(AttributeValue::Literal(lit)) => Some(BumpString::from_str_in(lit, bump)),
                Some(AttributeValue::Expression(_)) => {
                    return Err(EvalError::ExpressionAttributeNotSupported)
                }
            };

            Ok(Attribute {
                key: BumpString::from_str_in(name, bump),
                value: val,
            })
        }
        AstAttribute::Interpolated(_) => Err(EvalError::InterpolationNotSupported),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use paxhtml_parser::parse_html;

    #[test]
    fn test_eval_simple_element() {
        let bump = Bump::new();
        let html = r#"<div class="container">"Hello"</div>"#;
        let ast = parse_html(html).unwrap();
        let element = eval_node(&bump, &ast).unwrap();

        match element {
            Element::Tag {
                name,
                attributes,
                children,
                void,
            } => {
                assert_eq!(name.as_str(), "div");
                assert!(!void);
                assert_eq!(attributes.len(), 1);
                assert_eq!(attributes[0].key.as_str(), "class");
                assert_eq!(
                    attributes[0].value.as_ref().map(|s| s.as_str()),
                    Some("container")
                );
                assert_eq!(children.len(), 1);
                match &children[0] {
                    Element::Text { text } => assert_eq!(text.as_str(), "Hello"),
                    _ => panic!("Expected text element"),
                }
            }
            _ => panic!("Expected tag element"),
        }
    }

    #[test]
    fn test_eval_void_element() {
        let bump = Bump::new();
        let html = r#"<input r#type="text" />"#;
        let ast = parse_html(html).unwrap();
        let element = eval_node(&bump, &ast).unwrap();

        match element {
            Element::Tag { name, void, .. } => {
                assert_eq!(name.as_str(), "input");
                assert!(void);
            }
            _ => panic!("Expected tag element"),
        }
    }

    #[test]
    fn test_eval_nested_elements() {
        let bump = Bump::new();
        let html = r#"<div><p>"Hello"</p><span>"World"</span></div>"#;
        let ast = parse_html(html).unwrap();
        let element = eval_node(&bump, &ast).unwrap();

        match element {
            Element::Tag { children, .. } => {
                assert_eq!(children.len(), 2);
            }
            _ => panic!("Expected tag element"),
        }
    }

    #[test]
    fn test_eval_fragment() {
        let bump = Bump::new();
        let html = r#"<><div>"First"</div><div>"Second"</div></>"#;
        let ast = parse_html(html).unwrap();
        let element = eval_node(&bump, &ast).unwrap();

        match element {
            Element::Fragment { children } => {
                assert_eq!(children.len(), 2);
            }
            _ => panic!("Expected fragment"),
        }
    }

    #[test]
    fn test_eval_attribute_without_value() {
        let bump = Bump::new();
        let html = r#"<input disabled />"#;
        let ast = parse_html(html).unwrap();
        let element = eval_node(&bump, &ast).unwrap();

        match element {
            Element::Tag { attributes, .. } => {
                assert_eq!(attributes.len(), 1);
                assert_eq!(attributes[0].key.as_str(), "disabled");
                assert_eq!(attributes[0].value, None);
            }
            _ => panic!("Expected tag element"),
        }
    }
}
