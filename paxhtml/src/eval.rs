use crate::{Attribute, Element};
use paxhtml_parser::{AstAttribute, AstNode, AttributeValue};
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
                write!(f, "Interpolation is not supported in runtime HTML evaluation")
            }
            EvalError::ExpressionAttributeNotSupported => {
                write!(f, "Expression attributes are not supported in runtime HTML evaluation")
            }
        }
    }
}

impl std::error::Error for EvalError {}

/// Convert an AST node to a runtime Element
pub fn eval_node(node: &AstNode) -> Result<Element, EvalError> {
    match node {
        AstNode::Element {
            name,
            attributes,
            children,
            void,
        } => {
            let attrs = attributes
                .iter()
                .map(eval_attribute)
                .collect::<Result<Vec<_>, _>>()?;

            let child_elements = children
                .iter()
                .map(eval_node)
                .collect::<Result<Vec<_>, _>>()?;

            Ok(Element::Tag {
                name: name.clone(),
                attributes: attrs,
                children: child_elements,
                void: *void,
            })
        }
        AstNode::Fragment(children) => {
            let child_elements = children
                .iter()
                .map(eval_node)
                .collect::<Result<Vec<_>, _>>()?;

            Ok(Element::Fragment {
                children: child_elements,
            })
        }
        AstNode::Expression { .. } => Err(EvalError::InterpolationNotSupported),
        AstNode::Text(text) => Ok(Element::Text { text: text.clone() }),
    }
}

/// Convert an AST attribute to a runtime Attribute
fn eval_attribute(attr: &AstAttribute) -> Result<Attribute, EvalError> {
    match attr {
        AstAttribute::Named { name, value } => {
            let val = match value {
                None => None,
                Some(AttributeValue::Literal(lit)) => Some(lit.clone()),
                Some(AttributeValue::Expression(_)) => {
                    return Err(EvalError::ExpressionAttributeNotSupported)
                }
            };

            Ok(Attribute {
                key: name.clone(),
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
        let html = r#"<div class="container">Hello</div>"#;
        let ast = parse_html(html).unwrap();
        let element = eval_node(&ast).unwrap();

        match element {
            Element::Tag {
                name,
                attributes,
                children,
                void,
            } => {
                assert_eq!(name, "div");
                assert!(!void);
                assert_eq!(attributes.len(), 1);
                assert_eq!(attributes[0].key, "class");
                assert_eq!(attributes[0].value, Some("container".to_string()));
                assert_eq!(children.len(), 1);
                match &children[0] {
                    Element::Text { text } => assert_eq!(text, "Hello"),
                    _ => panic!("Expected text element"),
                }
            }
            _ => panic!("Expected tag element"),
        }
    }

    #[test]
    fn test_eval_void_element() {
        let html = r#"<input type="text" />"#;
        let ast = parse_html(html).unwrap();
        let element = eval_node(&ast).unwrap();

        match element {
            Element::Tag { name, void, .. } => {
                assert_eq!(name, "input");
                assert!(void);
            }
            _ => panic!("Expected tag element"),
        }
    }

    #[test]
    fn test_eval_nested_elements() {
        let html = r#"<div><p>Hello</p><span>World</span></div>"#;
        let ast = parse_html(html).unwrap();
        let element = eval_node(&ast).unwrap();

        match element {
            Element::Tag { children, .. } => {
                assert_eq!(children.len(), 2);
            }
            _ => panic!("Expected tag element"),
        }
    }

    #[test]
    fn test_eval_fragment() {
        let html = r#"<><div>First</div><div>Second</div></>"#;
        let ast = parse_html(html).unwrap();
        let element = eval_node(&ast).unwrap();

        match element {
            Element::Fragment { children } => {
                assert_eq!(children.len(), 2);
            }
            _ => panic!("Expected fragment"),
        }
    }

    #[test]
    fn test_eval_attribute_without_value() {
        let html = r#"<input disabled />"#;
        let ast = parse_html(html).unwrap();
        let element = eval_node(&ast).unwrap();

        match element {
            Element::Tag { attributes, .. } => {
                assert_eq!(attributes.len(), 1);
                assert_eq!(attributes[0].key, "disabled");
                assert_eq!(attributes[0].value, None);
            }
            _ => panic!("Expected tag element"),
        }
    }
}
