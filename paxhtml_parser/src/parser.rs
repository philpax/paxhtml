use crate::ast::{AstAttribute, AstNode, AttributeValue};
use convert_case::{Case, Casing};
use std::fmt;
use syn::{
    parse::{Parse, ParseStream},
    token, Expr, Ident, LitStr, Result as SynResult, Token,
};

/// Error type for parsing HTML
#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ParseError {}

impl From<syn::Error> for ParseError {
    fn from(err: syn::Error) -> Self {
        ParseError {
            message: err.to_string(),
        }
    }
}

/// Wrapper for parsing AstNode from syn
pub struct SynAstNode(pub AstNode);

impl Parse for SynAstNode {
    fn parse(input: ParseStream) -> SynResult<Self> {
        Ok(SynAstNode(parse_node(input)?))
    }
}

fn parse_attribute(input: ParseStream) -> SynResult<AstAttribute> {
    if input.peek(token::Brace) {
        // Parse interpolated attribute
        let content;
        syn::braced!(content in input);
        let expr = content.parse::<Expr>()?;
        Ok(AstAttribute::Interpolated(quote::quote! { #expr }))
    } else {
        let name = input.parse::<Ident>()?.to_string();
        let name = name
            .strip_prefix("r#")
            .unwrap_or(&name)
            .to_case(Case::Kebab);

        // Handle valueless attributes
        if input.peek(Token![=]) {
            input.parse::<Token![=]>()?;

            let value = if input.peek(token::Brace) {
                // Parse Rust expression in braces
                let content;
                syn::braced!(content in input);
                let expr = content.parse::<Expr>()?;
                Some(AttributeValue::Expression(quote::quote! { #expr }))
            } else {
                // Parse string literal
                Some(AttributeValue::Literal(input.parse::<LitStr>()?.value()))
            };

            Ok(AstAttribute::Named { name, value })
        } else {
            Ok(AstAttribute::Named { name, value: None })
        }
    }
}

fn parse_node(input: ParseStream) -> SynResult<AstNode> {
    if input.peek(token::Lt) {
        // Parse element
        input.parse::<Token![<]>()?;

        enum TagType {
            Fragment,
            Name(String),
        }

        impl TagType {
            pub fn is_fragment(&self) -> bool {
                matches!(self, TagType::Fragment)
            }
            pub fn unwrap_name_as_ref(&self) -> &str {
                match self {
                    TagType::Name(name) => name,
                    TagType::Fragment => panic!("Fragment cannot have a name"),
                }
            }
        }

        let tag = if input.peek(Token![>]) {
            TagType::Fragment
        } else {
            let name = input.parse::<Ident>()?.to_string();
            TagType::Name(name.strip_prefix("r#").unwrap_or(&name).to_string())
        };

        // Parse attributes
        let mut attributes = Vec::new();
        while !input.peek(Token![>]) && !input.peek(Token![/]) {
            attributes.push(parse_attribute(input)?);
        }

        // Handle void elements
        let void = if input.peek(Token![/]) {
            input.parse::<Token![/]>()?;
            input.parse::<Token![>]>()?;
            true
        } else {
            input.parse::<Token![>]>()?;
            false
        };

        if void {
            match tag {
                TagType::Name(name) => {
                    return Ok(AstNode::Element {
                        name,
                        attributes,
                        children: vec![],
                        void: true,
                    });
                }
                _ => return Err(input.error("Fragment cannot be void")),
            }
        }

        // Parse children
        let mut children = Vec::new();
        while !input.peek(Token![<]) || !input.peek2(Token![/]) {
            if input.peek(token::Brace) || (input.peek(Token![#]) && input.peek2(token::Brace)) {
                // Parse interpolated Rust expression
                let iterator = if input.peek(Token![#]) {
                    input.parse::<Token![#]>()?;
                    true
                } else {
                    false
                };
                let content;
                syn::braced!(content in input);
                let expr = content.parse::<Expr>()?;
                children.push(AstNode::Expression {
                    body: quote::quote! { #expr },
                    iterator,
                });
            } else if input.peek(Token![<]) {
                // Parse nested element
                children.push(parse_node(input)?);
            } else {
                // Parse text content
                let text = input.parse::<LitStr>()?.value();
                children.push(AstNode::Text(text));
            }

            if input.is_empty() {
                break;
            }
        }

        // Parse closing tag
        input.parse::<Token![<]>()?;
        input.parse::<Token![/]>()?;
        if !tag.is_fragment() {
            let close_name = input.parse::<Ident>()?.to_string();
            if close_name != tag.unwrap_name_as_ref() {
                return Err(input.error("Mismatched opening and closing tags"));
            }
        }
        input.parse::<Token![>]>()?;

        match tag {
            TagType::Fragment => Ok(AstNode::Fragment(children)),
            TagType::Name(name) => Ok(AstNode::Element {
                name,
                attributes,
                children,
                void: false,
            }),
        }
    } else if input.peek(token::Brace) || (input.peek(Token![#]) && input.peek2(token::Brace)) {
        // Parse interpolated Rust expression
        let iterator = if input.peek(Token![#]) {
            input.parse::<Token![#]>()?;
            true
        } else {
            false
        };
        let content;
        syn::braced!(content in input);
        let expr = content.parse::<Expr>()?;
        Ok(AstNode::Expression {
            body: quote::quote! { #expr },
            iterator,
        })
    } else {
        // Parse text content
        Ok(AstNode::Text(input.parse::<LitStr>()?.value()))
    }
}

/// Parse an HTML string into an AST using syn::parse_str
pub fn parse_html(html: &str) -> Result<AstNode, ParseError> {
    let node: SynAstNode = syn::parse_str(html)?;
    Ok(node.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_element() {
        let html = r#"<div class="container">"Hello"</div>"#;
        let result = parse_html(html).unwrap();

        match result {
            AstNode::Element { name, children, .. } => {
                assert_eq!(name, "div");
                assert_eq!(children.len(), 1);
                match &children[0] {
                    AstNode::Text(text) => assert_eq!(text, "Hello"),
                    _ => panic!("Expected text node"),
                }
            }
            _ => panic!("Expected element"),
        }
    }

    #[test]
    fn test_parse_void_element() {
        let html = r#"<input r#type="text" />"#;
        let result = parse_html(html).unwrap();

        match result {
            AstNode::Element {
                name,
                void,
                attributes,
                ..
            } => {
                assert_eq!(name, "input");
                assert!(void);
                assert_eq!(attributes.len(), 1);
            }
            _ => panic!("Expected element"),
        }
    }

    #[test]
    fn test_parse_nested_elements() {
        let html = r#"<div><p>"Hello"</p><span>"World"</span></div>"#;
        let result = parse_html(html).unwrap();

        match result {
            AstNode::Element { children, .. } => {
                assert_eq!(children.len(), 2);
            }
            _ => panic!("Expected element"),
        }
    }

    #[test]
    fn test_parse_fragment() {
        let html = r#"<><div>"First"</div><div>"Second"</div></>"#;
        let result = parse_html(html).unwrap();

        match result {
            AstNode::Fragment(children) => {
                assert_eq!(children.len(), 2);
            }
            _ => panic!("Expected fragment"),
        }
    }

    #[test]
    fn test_parse_attribute_without_value() {
        let html = r#"<input disabled />"#;
        let result = parse_html(html).unwrap();

        match result {
            AstNode::Element { attributes, .. } => {
                assert_eq!(attributes.len(), 1);
                match &attributes[0] {
                    AstAttribute::Named { name, value } => {
                        assert_eq!(name, "disabled");
                        assert!(value.is_none());
                    }
                    _ => panic!("Expected named attribute"),
                }
            }
            _ => panic!("Expected element"),
        }
    }
}
