use paxhtml::{parse_html, Bump, Document};

#[test]
fn test_runtime_parse_simple_html() {
    let bump = Bump::new();
    let html = r#"<div class="container"><p>"Hello, world!"</p></div>"#;
    let element = parse_html(&bump, html).unwrap();
    let doc = Document::new(&bump, [element]);
    let output = doc.write_to_string().unwrap();

    assert!(output.contains(r#"<div class="container">"#));
    assert!(output.contains("<p>Hello, world!</p>"));
}

#[test]
fn test_runtime_parse_nested_structure() {
    let bump = Bump::new();
    let html = r#"<ul><li>"First"</li><li>"Second"</li><li>"Third"</li></ul>"#;
    let element = parse_html(&bump, html).unwrap();
    let doc = Document::new(&bump, [element]);
    let output = doc.write_to_string().unwrap();

    assert!(output.contains("<ul>"));
    assert!(output.contains("<li>First</li>"));
    assert!(output.contains("<li>Second</li>"));
    assert!(output.contains("<li>Third</li>"));
}

#[test]
fn test_runtime_parse_void_elements() {
    let bump = Bump::new();
    let html = r#"<input r#type="text" placeholder="Enter name" />"#;
    let element = parse_html(&bump, html).unwrap();
    let doc = Document::new(&bump, [element]);
    let output = doc.write_to_string().unwrap();

    // Check for the essential parts - void elements are rendered without self-closing slash
    assert!(output.contains("<input"));
    assert!(output.contains(r#"type="text""#));
    assert!(output.contains(r#"placeholder="Enter name""#));
    assert_eq!(output, r#"<input type="text" placeholder="Enter name">"#);
}

#[test]
fn test_runtime_parse_fragment() {
    let bump = Bump::new();
    let html = r#"<><div>"First"</div><div>"Second"</div></>"#;
    let element = parse_html(&bump, html).unwrap();
    let doc = Document::new(&bump, [element]);
    let output = doc.write_to_string().unwrap();

    assert!(output.contains("<div>First</div>"));
    assert!(output.contains("<div>Second</div>"));
}

#[test]
fn test_runtime_parse_attributes_without_values() {
    let bump = Bump::new();
    let html = r#"<input disabled checked />"#;
    let element = parse_html(&bump, html).unwrap();
    let doc = Document::new(&bump, [element]);
    let output = doc.write_to_string().unwrap();

    assert!(output.contains("disabled"));
    assert!(output.contains("checked"));
}

#[test]
fn test_runtime_parse_custom_element_name() {
    let bump = Bump::new();
    // Custom elements (uppercase) should still be parsed as regular tags at runtime
    let html = r#"<MyComponent foo="bar">"content"</MyComponent>"#;
    let element = parse_html(&bump, html).unwrap();
    let doc = Document::new(&bump, [element]);
    let output = doc.write_to_string().unwrap();

    // Custom components are left as regular HTML elements at runtime
    assert!(output.contains(r#"<MyComponent foo="bar">"#));
    assert!(output.contains("content"));
    assert!(output.contains("</MyComponent>"));
}

#[test]
fn test_runtime_parse_rejects_interpolation() {
    let bump = Bump::new();
    let html = r#"<div>{some_expr}</div>"#;
    let result = parse_html(&bump, html);

    // Interpolation syntax will cause a parse error at runtime
    assert!(result.is_err());
}
