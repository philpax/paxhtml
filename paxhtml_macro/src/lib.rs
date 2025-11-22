use convert_case::Casing;
use paxhtml_parser::{AstAttribute, AstNode, AttributeValue};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, token, Expr, Ident, LitStr, Result, Token,
};

// Custom keywords for parsing
mod kw {
    syn::custom_keyword!(r#async);
    syn::custom_keyword!(r#for);
    syn::custom_keyword!(r#type);
}

// Wrapper for parsing HtmlNode from syn
struct SynHtmlNode(AstNode);

// Implement parsing for attributes
impl Parse for SynHtmlNode {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(SynHtmlNode(parse_node(input)?))
    }
}

fn parse_attribute(input: ParseStream) -> Result<AstAttribute> {
    if input.peek(token::Brace) {
        // Parse interpolated attribute
        let content;
        syn::braced!(content in input);
        let expr = content.parse::<Expr>()?;
        Ok(AstAttribute::Interpolated(quote! { #expr }))
    } else {
        let name = input.parse::<Ident>()?.to_string();
        let name = name
            .strip_prefix("r#")
            .unwrap_or(&name)
            .to_case(convert_case::Case::Kebab);

        // Handle valueless attributes
        if input.peek(Token![=]) {
            input.parse::<Token![=]>()?;

            let value = if input.peek(token::Brace) {
                // Parse Rust expression in braces
                let content;
                syn::braced!(content in input);
                let expr = content.parse::<Expr>()?;
                Some(AttributeValue::Expression(quote! { #expr }))
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

fn parse_node(input: ParseStream) -> Result<AstNode> {
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
                    body: quote! { #expr },
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
            body: quote! { #expr },
            iterator,
        })
    } else {
        // Parse text content
        Ok(AstNode::Text(input.parse::<LitStr>()?.value()))
    }
}

// Helper function to check if a name represents a custom component (starts with uppercase)
fn is_custom_component(name: &str) -> bool {
    name.chars().next().is_some_and(|c| c.is_uppercase())
}

// Wrapper to allow AstNode to be used in quote! macros
struct AstNodeRef<'a>(&'a AstNode);

impl<'a> ToTokens for AstNodeRef<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        ast_node_to_tokens(self.0, tokens);
    }
}

// Convert AstNode to TokenStream
impl ToTokens for SynHtmlNode {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        ast_node_to_tokens(&self.0, tokens);
    }
}

fn ast_node_to_tokens(node: &AstNode, tokens: &mut TokenStream2) {
    match node {
        AstNode::Element {
            name,
            attributes,
            children,
            void,
        } => {
            // Check if this is a custom component
            if is_custom_component(name) {
                // Check for interpolated attributes (not supported for custom components)
                let has_interpolated = attributes
                    .iter()
                    .any(|attr| matches!(attr, AstAttribute::Interpolated(_)));
                if has_interpolated {
                    tokens.extend(quote! {
                        compile_error!("Interpolated attributes are not supported for custom components")
                    });
                    return;
                }

                // Generate custom component call
                let component_ident = syn::Ident::new(name, proc_macro2::Span::call_site());
                let props_type = format!("{}Props", name);
                let props_ident = syn::Ident::new(&props_type, proc_macro2::Span::call_site());

                // Convert attributes to struct fields
                let mut field_inits = Vec::new();
                for attr in attributes {
                    if let AstAttribute::Named { name, value } = attr {
                        // Convert kebab-case to snake_case for Rust struct fields
                        let field_name = name.replace('-', "_");
                        let field_ident =
                            syn::Ident::new(&field_name, proc_macro2::Span::call_site());

                        let value_expr = match value {
                            Some(AttributeValue::Expression(expr)) => quote! { #expr.into() },
                            Some(AttributeValue::Literal(lit)) => quote! { #lit.into() },
                            None => quote! { true.into() },
                        };

                        field_inits.push(quote! { #field_ident: #value_expr });
                    }
                }

                // Add children if present
                if !children.is_empty() {
                    let children_refs: Vec<_> = children.iter().map(AstNodeRef).collect();
                    field_inits.push(quote! { children: vec![#(#children_refs),*] });
                }

                tokens.extend(quote! {
                    #component_ident(#props_ident {
                        #(#field_inits,)*
                        ..Default::default()
                    })
                });
            } else {
                // Regular HTML element
                let attrs = if attributes.is_empty() {
                    quote! { vec![] }
                } else {
                    let mut attr_tokens = Vec::new();
                    for attr in attributes {
                        match attr {
                            AstAttribute::Named { name, value } => {
                                let attr_token = match value {
                                    Some(AttributeValue::Expression(expr)) => quote! {
                                        paxhtml::attr((#name.to_string(), #expr.to_string()))
                                    },
                                    Some(AttributeValue::Literal(lit)) => quote! {
                                        paxhtml::attr((#name.to_string(), #lit.to_string()))
                                    },
                                    None => quote! {
                                        paxhtml::attr(#name.to_string())
                                    },
                                };
                                attr_tokens.push(quote! { attrs.push(#attr_token); });
                            }
                            AstAttribute::Interpolated(expr) => {
                                attr_tokens.push(quote! { attrs.extend(#expr); });
                            }
                        }
                    }
                    quote! {{
                        let mut attrs = Vec::new();
                        #(#attr_tokens)*
                        attrs
                    }}
                };

                let children_tokens = if children.is_empty() {
                    quote! { vec![] }
                } else {
                    let children_refs: Vec<_> = children.iter().map(AstNodeRef).collect();
                    quote! { [#(#children_refs),*] }
                };

                tokens.extend(quote! {
                    paxhtml::builder::tag(#name, #attrs, #void)(#children_tokens)
                });
            }
        }
        AstNode::Fragment(children) => {
            let children_refs: Vec<_> = children.iter().map(AstNodeRef).collect();
            tokens.extend(quote! {
                paxhtml::Element::from_iter([#(#children_refs),*])
            });
        }
        AstNode::Expression { body, iterator } => {
            if *iterator {
                tokens.extend(quote! {
                    paxhtml::Element::from_iter(#body)
                });
            } else {
                tokens.extend(quote! {
                    paxhtml::Element::from(#body)
                });
            }
        }
        AstNode::Text(text) => {
            tokens.extend(quote! {
                paxhtml::Element::Text {
                    text: #text.to_string()
                }
            });
        }
    }
}

#[proc_macro]
/// Constructs a tree of [`paxhtml::Element`]s from (X)HTML-like syntax, similar to JSX.
///
/// Interpolation is supported using `{}` for expressions and `#{...}` for iterators.
///
/// Fragments are supported using `<>...</>` syntax.
pub fn html(input: TokenStream) -> TokenStream {
    let node = parse_macro_input!(input as SynHtmlNode);
    quote! { #node }.into()
}
