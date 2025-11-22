mod ast;
mod parser;

pub use ast::{AstAttribute, AstNode, AttributeValue};
pub use parser::{parse_html, ParseError};
