use crate::ast::{AstAttribute, AstNode, AttributeValue};
use convert_case::{Case, Casing};
use std::fmt;
use syn::{
    parse::{Parse, ParseStream},
    token, Expr, Ident, Lit, LitStr, Result as SynResult, Token,
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

/// Wrapper for parsing an [AstNode] from a token stream.
pub struct AstNodeParser(pub AstNode);

impl Parse for AstNodeParser {
    fn parse(input: ParseStream) -> SynResult<Self> {
        Ok(AstNodeParser(parse_node(input)?))
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
                // Parse literal (string, integer, float, or bool)
                let lit = input.parse::<Lit>()?;
                let attr_value = match lit {
                    Lit::Str(s) => AttributeValue::LiteralString(s.value()),
                    Lit::Int(i) => {
                        let value: i128 = i
                            .base10_parse()
                            .map_err(|_| input.error("Invalid integer literal"))?;
                        AttributeValue::LiteralInt(value)
                    }
                    Lit::Float(f) => {
                        let value: f64 = f
                            .base10_parse()
                            .map_err(|_| input.error("Invalid float literal"))?;
                        AttributeValue::LiteralFloat(value)
                    }
                    Lit::Bool(b) => AttributeValue::LiteralBool(b.value()),
                    _ => {
                        return Err(input.error("Expected string, integer, float, or bool literal"))
                    }
                };
                Some(attr_value)
            };

            Ok(AstAttribute::Named { name, value })
        } else {
            Ok(AstAttribute::Named { name, value: None })
        }
    }
}

/// The parsed opening portion of a tag: name, attributes, and whether it is void (self-closing).
struct TagHead {
    name: String,
    attributes: Vec<AstAttribute>,
    void: bool,
}

/// Parse the tag name, attributes, and closing `>` or `/>`.
/// Assumes `<` has already been consumed.
fn parse_tag_head(input: ParseStream) -> SynResult<TagHead> {
    let name = input.parse::<Ident>()?.to_string();
    let name = name.strip_prefix("r#").unwrap_or(&name).to_string();

    let mut attributes = Vec::new();
    while !input.peek(Token![>]) && !input.peek(Token![/]) {
        attributes.push(parse_attribute(input)?);
    }

    let void = if input.peek(Token![/]) {
        input.parse::<Token![/]>()?;
        input.parse::<Token![>]>()?;
        true
    } else {
        input.parse::<Token![>]>()?;
        false
    };

    Ok(TagHead {
        name,
        attributes,
        void,
    })
}

fn parse_children(input: ParseStream) -> SynResult<Vec<AstNode>> {
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
    Ok(children)
}

fn parse_node(input: ParseStream) -> SynResult<AstNode> {
    if input.peek(token::Lt) {
        // Parse element
        input.parse::<Token![<]>()?;

        // Check for fragment: `<>...</>`
        if input.peek(Token![>]) {
            input.parse::<Token![>]>()?;
            let children = parse_children(input)?;

            // Parse closing `</>`
            input.parse::<Token![<]>()?;
            input.parse::<Token![/]>()?;
            input.parse::<Token![>]>()?;

            return Ok(AstNode::Fragment(children));
        }

        // Named element
        let TagHead {
            name,
            attributes,
            void,
        } = parse_tag_head(input)?;

        if void {
            return Ok(AstNode::Element {
                name,
                attributes,
                children: vec![],
                void: true,
            });
        }

        let children = parse_children(input)?;

        // Parse closing tag `</name>`
        input.parse::<Token![<]>()?;
        input.parse::<Token![/]>()?;
        let close_name = input.parse::<Ident>()?.to_string();
        if close_name != name {
            return Err(input.error("Mismatched opening and closing tags"));
        }
        input.parse::<Token![>]>()?;

        Ok(AstNode::Element {
            name,
            attributes,
            children,
            void: false,
        })
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

/// Parse an HTML string into an AST.
pub fn parse_html(html: &str) -> Result<AstNode, ParseError> {
    let node: AstNodeParser = syn::parse_str(html)?;
    Ok(node.0)
}

/// Wrapper for parsing just an opening tag from a token stream.
struct OpeningTagParser(TagHead);

impl Parse for OpeningTagParser {
    fn parse(input: ParseStream) -> SynResult<Self> {
        input.parse::<Token![<]>()?;
        Ok(OpeningTagParser(parse_tag_head(input)?))
    }
}

/// Parse an opening HTML tag (e.g. `<CityPoster image="foo.png">` or `<br />`)
/// and return the tag name, attributes, and whether it's void (self-closing).
pub fn parse_opening_tag(html: &str) -> Result<(String, Vec<AstAttribute>, bool), ParseError> {
    let OpeningTagParser(tag) = syn::parse_str(html)?;
    Ok((tag.name, tag.attributes, tag.void))
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

    #[test]
    fn test_parse_integer_attribute() {
        let html = r#"<input tabindex=1 />"#;
        let result = parse_html(html).unwrap();

        match result {
            AstNode::Element { attributes, .. } => {
                assert_eq!(attributes.len(), 1);
                match &attributes[0] {
                    AstAttribute::Named { name, value } => {
                        assert_eq!(name, "tabindex");
                        match value {
                            Some(AttributeValue::LiteralInt(v)) => assert_eq!(*v, 1),
                            _ => panic!("Expected integer literal value"),
                        }
                    }
                    _ => panic!("Expected named attribute"),
                }
            }
            _ => panic!("Expected element"),
        }
    }

    #[test]
    fn test_parse_float_attribute() {
        let html = r#"<div opacity=0.5 />"#;
        let result = parse_html(html).unwrap();

        match result {
            AstNode::Element { attributes, .. } => {
                assert_eq!(attributes.len(), 1);
                match &attributes[0] {
                    AstAttribute::Named { name, value } => {
                        assert_eq!(name, "opacity");
                        match value {
                            Some(AttributeValue::LiteralFloat(v)) => {
                                assert!((v - 0.5).abs() < f64::EPSILON)
                            }
                            _ => panic!("Expected float literal value"),
                        }
                    }
                    _ => panic!("Expected named attribute"),
                }
            }
            _ => panic!("Expected element"),
        }
    }

    #[test]
    fn test_parse_bool_attribute() {
        let html = r#"<input data_active=true data_disabled=false />"#;
        let result = parse_html(html).unwrap();

        match result {
            AstNode::Element { attributes, .. } => {
                assert_eq!(attributes.len(), 2);
                match &attributes[0] {
                    AstAttribute::Named { name, value } => {
                        assert_eq!(name, "data-active");
                        match value {
                            Some(AttributeValue::LiteralBool(v)) => assert!(*v),
                            _ => panic!("Expected bool literal value"),
                        }
                    }
                    _ => panic!("Expected named attribute"),
                }
                match &attributes[1] {
                    AstAttribute::Named { name, value } => {
                        assert_eq!(name, "data-disabled");
                        match value {
                            Some(AttributeValue::LiteralBool(v)) => assert!(!*v),
                            _ => panic!("Expected bool literal value"),
                        }
                    }
                    _ => panic!("Expected named attribute"),
                }
            }
            _ => panic!("Expected element"),
        }
    }

    #[test]
    fn test_parse_mixed_literal_attributes() {
        let html = r#"<input tabindex=0 r#type="text" maxlength=100 />"#;
        let result = parse_html(html).unwrap();

        match result {
            AstNode::Element { attributes, .. } => {
                assert_eq!(attributes.len(), 3);

                // tabindex=0
                match &attributes[0] {
                    AstAttribute::Named { name, value } => {
                        assert_eq!(name, "tabindex");
                        match value {
                            Some(AttributeValue::LiteralInt(v)) => assert_eq!(*v, 0),
                            _ => panic!("Expected integer literal value"),
                        }
                    }
                    _ => panic!("Expected named attribute"),
                }

                // type="text"
                match &attributes[1] {
                    AstAttribute::Named { name, value } => {
                        assert_eq!(name, "type");
                        match value {
                            Some(AttributeValue::LiteralString(v)) => assert_eq!(v, "text"),
                            _ => panic!("Expected string literal value"),
                        }
                    }
                    _ => panic!("Expected named attribute"),
                }

                // maxlength=100
                match &attributes[2] {
                    AstAttribute::Named { name, value } => {
                        assert_eq!(name, "maxlength");
                        match value {
                            Some(AttributeValue::LiteralInt(v)) => assert_eq!(*v, 100),
                            _ => panic!("Expected integer literal value"),
                        }
                    }
                    _ => panic!("Expected named attribute"),
                }
            }
            _ => panic!("Expected element"),
        }
    }
}
