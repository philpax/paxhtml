use proc_macro2::TokenStream;

/// Represents an HTML attribute in the AST
#[derive(Debug, Clone)]
pub enum AstAttribute {
    /// A named attribute with an optional value
    Named {
        name: String,
        value: Option<AttributeValue>,
    },
    /// An interpolated attribute expression (macro only)
    Interpolated(TokenStream),
}

/// Represents an attribute value
#[derive(Debug, Clone)]
pub enum AttributeValue {
    /// A string literal value
    LiteralString(String),
    /// An integer literal value
    LiteralInt(i128),
    /// A floating-point literal value
    LiteralFloat(f64),
    /// A boolean literal value
    LiteralBool(bool),
    /// An expression (macro only)
    Expression(TokenStream),
}

impl AttributeValue {
    /// Convert the attribute value to its string representation
    pub fn to_string_value(&self) -> String {
        match self {
            AttributeValue::LiteralString(s) => s.clone(),
            AttributeValue::LiteralInt(i) => i.to_string(),
            AttributeValue::LiteralFloat(f) => f.to_string(),
            AttributeValue::LiteralBool(b) => b.to_string(),
            AttributeValue::Expression(_) => {
                panic!("Cannot convert expression to string at runtime")
            }
        }
    }
}

/// Represents a node in the HTML AST
#[derive(Debug, Clone)]
pub enum AstNode {
    /// An HTML element with tag name, attributes, and children
    Element {
        name: String,
        attributes: Vec<AstAttribute>,
        children: Vec<AstNode>,
        void: bool,
    },
    /// A fragment containing multiple children without a wrapper element
    Fragment(Vec<AstNode>),
    /// An interpolated expression (macro only)
    Expression { body: TokenStream, iterator: bool },
    /// Text content
    Text(String),
}

impl AstNode {
    /// Check if this node is a custom component (starts with uppercase letter)
    pub fn is_custom_component(&self) -> bool {
        match self {
            AstNode::Element { name, .. } => name.chars().next().is_some_and(|c| c.is_uppercase()),
            _ => false,
        }
    }

    /// Get the element name if this is an element node
    pub fn element_name(&self) -> Option<&str> {
        match self {
            AstNode::Element { name, .. } => Some(name),
            _ => None,
        }
    }
}
