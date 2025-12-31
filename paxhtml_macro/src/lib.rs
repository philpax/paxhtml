use paxhtml_parser::{AstAttribute, AstNode, AttributeValue, SynAstNode};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{parse::Parse, parse::ParseStream, Expr, Token};

// Helper function to check if a name represents a custom component (starts with uppercase)
fn is_custom_component(name: &str) -> bool {
    name.chars().next().is_some_and(|c| c.is_uppercase())
}

/// Input format: `in <allocator>; <html>`
struct HtmlInput {
    allocator: Expr,
    node: SynAstNode,
}
impl Parse for HtmlInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Parse: in <allocator_expr> ;
        input.parse::<Token![in]>()?;
        let allocator = input.parse::<Expr>()?;
        input.parse::<Token![;]>()?;

        // Parse the HTML node
        let node = input.parse::<SynAstNode>()?;

        Ok(HtmlInput { allocator, node })
    }
}

// Wrapper to allow code generation with bump allocator
struct AstNodeWithBump<'a> {
    bump: &'a Expr,
    node: &'a AstNode,
}
impl<'a> ToTokens for AstNodeWithBump<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        ast_node_to_tokens_with_bump(self.bump, self.node, tokens);
    }
}

fn ast_node_to_tokens_with_bump(bump: &Expr, node: &AstNode, tokens: &mut TokenStream2) {
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

                // Add children if present (as Option<Element> using from_iter)
                if !children.is_empty() {
                    let children_tokens: Vec<_> = children
                        .iter()
                        .map(|c| AstNodeWithBump { bump, node: c })
                        .collect();
                    field_inits.push(quote! {
                        children: Some(paxhtml::Element::from_iter(#bump, [#(#children_tokens),*]))
                    });
                }

                // Use struct update syntax with default_in for unspecified fields
                tokens.extend(quote! {
                    #component_ident(#bump, #props_ident {
                        #(#field_inits,)*
                        ..paxhtml::DefaultIn::default_in(#bump)
                    })
                });
            } else {
                // Regular HTML element
                let attrs_code = if attributes.is_empty() {
                    quote! { bumpalo::collections::Vec::new_in(#bump) }
                } else {
                    let mut attr_statements = Vec::new();
                    for attr in attributes {
                        match attr {
                            AstAttribute::Named { name, value } => {
                                let attr_statement = match value {
                                    Some(AttributeValue::Expression(expr)) => quote! {
                                        __attrs.push(paxhtml::Attribute::new(
                                            #bump,
                                            #name,
                                            &(#expr).to_string()
                                        ));
                                    },
                                    Some(AttributeValue::Literal(lit)) => quote! {
                                        __attrs.push(paxhtml::Attribute::new(#bump, #name, #lit));
                                    },
                                    None => quote! {
                                        __attrs.push(paxhtml::Attribute::boolean(#bump, #name));
                                    },
                                };
                                attr_statements.push(attr_statement);
                            }
                            AstAttribute::Interpolated(expr) => {
                                attr_statements.push(quote! {
                                    for __a in #expr {
                                        __attrs.push(__a);
                                    }
                                });
                            }
                        }
                    }
                    quote! {{
                        let mut __attrs = bumpalo::collections::Vec::new_in(#bump);
                        #(#attr_statements)*
                        __attrs
                    }}
                };

                let children_code = if children.is_empty() {
                    quote! { bumpalo::collections::Vec::new_in(#bump) }
                } else {
                    let children_tokens: Vec<_> = children
                        .iter()
                        .map(|c| AstNodeWithBump { bump, node: c })
                        .collect();
                    quote! {{
                        let mut __children = bumpalo::collections::Vec::new_in(#bump);
                        #(__children.push(#children_tokens);)*
                        __children
                    }}
                };

                let name_str = name.as_str();
                tokens.extend(quote! {
                    paxhtml::Element::Tag {
                        name: bumpalo::collections::String::from_str_in(#name_str, #bump),
                        attributes: #attrs_code,
                        children: #children_code,
                        void: #void,
                    }
                });
            }
        }
        AstNode::Fragment(children) => {
            let children_tokens: Vec<_> = children
                .iter()
                .map(|c| AstNodeWithBump { bump, node: c })
                .collect();
            tokens.extend(quote! {{
                let mut __children = bumpalo::collections::Vec::new_in(#bump);
                #(__children.push(#children_tokens);)*
                paxhtml::Element::Fragment { children: __children }
            }});
        }
        AstNode::Expression { body, iterator } => {
            if *iterator {
                tokens.extend(quote! {
                    paxhtml::Element::from_iter(#bump, #body)
                });
            } else {
                tokens.extend(quote! {
                    paxhtml::IntoElement::into_element(#body, #bump)
                });
            }
        }
        AstNode::Text(text) => {
            tokens.extend(quote! {
                paxhtml::Element::Text {
                    text: bumpalo::collections::String::from_str_in(#text, #bump)
                }
            });
        }
    }
}

#[proc_macro]
/// Constructs a tree of [`paxhtml::Element`]s from (X)HTML-like syntax, similar to JSX.
///
/// # Syntax
///
/// ```ignore
/// html! { in <allocator>; <element>...</element> }
/// ```
///
/// The allocator is a reference to a [`bumpalo::Bump`] allocator that will be used
/// for all allocations.
///
/// Interpolation is supported using `{}` for expressions and `#{...}` for iterators.
///
/// Fragments are supported using `<>...</>` syntax.
///
/// # Example
///
/// ```ignore
/// use paxhtml::{html, Bump};
///
/// let bump = Bump::new();
/// let element = html! { in &bump;
///     <div class="container">
///         <h1>"Hello, World!"</h1>
///     </div>
/// };
/// ```
pub fn html(input: TokenStream) -> TokenStream {
    let HtmlInput { allocator, node } = syn::parse_macro_input!(input as HtmlInput);

    let wrapper = AstNodeWithBump {
        bump: &allocator,
        node: &node.0,
    };

    quote! { #wrapper }.into()
}
