use crate::ast::{AstAttribute, AstNode, AttributeValue};
use convert_case::{Case, Casing};
use std::fmt;

/// Error type for parsing HTML
#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub position: usize,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Parse error at position {}: {}", self.position, self.message)
    }
}

impl std::error::Error for ParseError {}

/// Parser for HTML strings
pub struct Parser {
    input: Vec<char>,
    pos: usize,
}

impl Parser {
    fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            pos: 0,
        }
    }

    fn current(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    fn peek(&self, offset: usize) -> Option<char> {
        self.input.get(self.pos + offset).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.current();
        if c.is_some() {
            self.pos += 1;
        }
        c
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.current() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn error(&self, message: impl Into<String>) -> ParseError {
        ParseError {
            message: message.into(),
            position: self.pos,
        }
    }

    fn expect(&mut self, expected: char) -> Result<(), ParseError> {
        match self.advance() {
            Some(c) if c == expected => Ok(()),
            Some(c) => Err(self.error(format!("Expected '{}', found '{}'", expected, c))),
            None => Err(self.error(format!("Expected '{}', found end of input", expected))),
        }
    }

    fn parse_identifier(&mut self) -> Result<String, ParseError> {
        let mut ident = String::new();

        // First character must be alphabetic or underscore
        match self.current() {
            Some(c) if c.is_alphabetic() || c == '_' => {
                ident.push(c);
                self.advance();
            }
            Some(c) => return Err(self.error(format!("Expected identifier, found '{}'", c))),
            None => return Err(self.error("Expected identifier, found end of input")),
        }

        // Subsequent characters can be alphanumeric, underscore, or hyphen
        while let Some(c) = self.current() {
            if c.is_alphanumeric() || c == '_' || c == '-' {
                ident.push(c);
                self.advance();
            } else {
                break;
            }
        }

        Ok(ident)
    }

    fn parse_string_literal(&mut self) -> Result<String, ParseError> {
        self.expect('"')?;
        let mut value = String::new();

        loop {
            match self.current() {
                Some('"') => {
                    self.advance();
                    break;
                }
                Some('\\') => {
                    self.advance();
                    match self.current() {
                        Some('n') => {
                            value.push('\n');
                            self.advance();
                        }
                        Some('r') => {
                            value.push('\r');
                            self.advance();
                        }
                        Some('t') => {
                            value.push('\t');
                            self.advance();
                        }
                        Some('\\') => {
                            value.push('\\');
                            self.advance();
                        }
                        Some('"') => {
                            value.push('"');
                            self.advance();
                        }
                        Some(c) => {
                            value.push(c);
                            self.advance();
                        }
                        None => return Err(self.error("Unterminated string literal")),
                    }
                }
                Some(c) => {
                    value.push(c);
                    self.advance();
                }
                None => return Err(self.error("Unterminated string literal")),
            }
        }

        Ok(value)
    }

    fn parse_attribute(&mut self) -> Result<AstAttribute, ParseError> {
        // Check for interpolation (not supported in runtime parser)
        if let Some('{') = self.current() {
            return Err(self.error("Interpolation is not supported in runtime HTML parsing"));
        }

        let name = self.parse_identifier()?;
        let name = name.to_case(Case::Kebab);

        self.skip_whitespace();

        // Check for attribute value
        if let Some('=') = self.current() {
            self.advance();
            self.skip_whitespace();

            // Check for interpolation in value
            if let Some('{') = self.current() {
                return Err(self.error("Interpolation is not supported in runtime HTML parsing"));
            }

            let value = self.parse_string_literal()?;
            Ok(AstAttribute::Named {
                name,
                value: Some(AttributeValue::Literal(value)),
            })
        } else {
            // Valueless attribute
            Ok(AstAttribute::Named { name, value: None })
        }
    }

    fn parse_text(&mut self) -> Result<String, ParseError> {
        let mut text = String::new();

        loop {
            match self.current() {
                Some('<') | Some('{') | None => break,
                Some(c) => {
                    text.push(c);
                    self.advance();
                }
            }
        }

        Ok(text)
    }

    fn parse_node(&mut self) -> Result<AstNode, ParseError> {
        self.skip_whitespace();

        // Check for interpolation
        if let Some('{') = self.current() {
            return Err(self.error("Interpolation is not supported in runtime HTML parsing"));
        }

        if let Some('#') = self.current() {
            if let Some('{') = self.peek(1) {
                return Err(self.error("Interpolation is not supported in runtime HTML parsing"));
            }
        }

        // Check for element
        if let Some('<') = self.current() {
            self.advance();

            // Check for fragment or closing tag
            if let Some('>') = self.current() {
                // Fragment opening tag: <>
                self.advance();
                let children = self.parse_children(None)?;
                // parse_children already consumed the closing tag
                return Ok(AstNode::Fragment(children));
            } else if let Some('/') = self.current() {
                return Err(self.error("Unexpected closing tag"));
            }

            // Parse tag name
            let name = self.parse_identifier()?;
            self.skip_whitespace();

            // Parse attributes
            let mut attributes = Vec::new();
            while let Some(c) = self.current() {
                if c == '>' || c == '/' {
                    break;
                }
                attributes.push(self.parse_attribute()?);
                self.skip_whitespace();
            }

            // Check for void element
            let void = if let Some('/') = self.current() {
                self.advance();
                self.skip_whitespace();
                self.expect('>')?;
                true
            } else {
                self.expect('>')?;
                false
            };

            if void {
                return Ok(AstNode::Element {
                    name,
                    attributes,
                    children: vec![],
                    void: true,
                });
            }

            // Parse children
            let children = self.parse_children(Some(&name))?;

            Ok(AstNode::Element {
                name,
                attributes,
                children,
                void: false,
            })
        } else {
            // Parse text
            let text = self.parse_text()?;
            if text.is_empty() {
                Err(self.error("Expected element or text"))
            } else {
                Ok(AstNode::Text(text))
            }
        }
    }

    fn parse_children(&mut self, parent_tag: Option<&str>) -> Result<Vec<AstNode>, ParseError> {
        let mut children = Vec::new();

        loop {
            self.skip_whitespace();

            // Check for closing tag
            if let Some('<') = self.current() {
                if let Some('/') = self.peek(1) {
                    self.advance(); // consume '<'
                    self.advance(); // consume '/'

                    // For fragments, expect '>'
                    if parent_tag.is_none() {
                        self.skip_whitespace();
                        self.expect('>')?;
                    } else {
                        // For regular elements, expect the tag name
                        self.skip_whitespace();
                        let close_name = self.parse_identifier()?;
                        if Some(close_name.as_str()) != parent_tag {
                            return Err(self.error(format!(
                                "Mismatched closing tag: expected {:?}, found '{}'",
                                parent_tag, close_name
                            )));
                        }
                        self.skip_whitespace();
                        self.expect('>')?;
                    }
                    break;
                }
            }

            if self.current().is_none() {
                if parent_tag.is_some() {
                    return Err(self.error(format!(
                        "Unclosed tag: expected closing tag for '{}'",
                        parent_tag.unwrap()
                    )));
                }
                break;
            }

            children.push(self.parse_node()?);
        }

        Ok(children)
    }
}

/// Parse an HTML string into an AST
pub fn parse_html(input: &str) -> Result<AstNode, ParseError> {
    let mut parser = Parser::new(input);
    let node = parser.parse_node()?;
    parser.skip_whitespace();

    // Ensure we've consumed all input
    if parser.current().is_some() {
        return Err(parser.error("Unexpected content after root element"));
    }

    Ok(node)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_element() {
        let html = r#"<div class="container">Hello</div>"#;
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
        let html = r#"<input type="text" />"#;
        let result = parse_html(html).unwrap();

        match result {
            AstNode::Element { name, void, attributes, .. } => {
                assert_eq!(name, "input");
                assert!(void);
                assert_eq!(attributes.len(), 1);
            }
            _ => panic!("Expected element"),
        }
    }

    #[test]
    fn test_parse_nested_elements() {
        let html = r#"<div><p>Hello</p><span>World</span></div>"#;
        let result = parse_html(html).unwrap();

        match result {
            AstNode::Element { name, children, .. } => {
                assert_eq!(name, "div");
                assert_eq!(children.len(), 2);
            }
            _ => panic!("Expected element"),
        }
    }

    #[test]
    fn test_parse_fragment() {
        let html = r#"<><div>First</div><div>Second</div></>"#;
        let result = parse_html(html).unwrap();

        match result {
            AstNode::Fragment(children) => {
                assert_eq!(children.len(), 2);
            }
            _ => panic!("Expected fragment"),
        }
    }

    #[test]
    fn test_interpolation_error() {
        let html = r#"<div>{some_expr}</div>"#;
        let result = parse_html(html);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Interpolation"));
    }

    #[test]
    fn test_attribute_without_value() {
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
