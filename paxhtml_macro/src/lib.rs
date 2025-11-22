use paxhtml_parser::{AstAttribute, AstNode, AttributeValue, SynAstNode};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};

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

// Local wrapper to implement ToTokens (avoids orphan rule)
struct HtmlNode(SynAstNode);

impl ToTokens for HtmlNode {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        ast_node_to_tokens(&self.0.0, tokens);
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
    let node = HtmlNode(syn::parse_macro_input!(input as SynAstNode));
    quote! { #node }.into()
}
