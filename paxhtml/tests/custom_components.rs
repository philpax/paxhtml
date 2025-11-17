use paxhtml::{html, Element};

#[derive(Default)]
struct MyCustomElementProps {
    cool: i32,
    test: String,
    children: Vec<Element>,
    // Extra field to demonstrate ..Default::default() usefulness
    #[allow(dead_code)]
    extra: Option<String>,
}

#[allow(non_snake_case)]
fn MyCustomElement(props: MyCustomElementProps) -> Element {
    Element::Tag {
        name: "div".to_string(),
        attributes: vec![],
        children: vec![
            Element::Tag {
                name: "p".to_string(),
                attributes: vec![],
                children: vec![Element::Text {
                    text: format!("cool: {}, test: {}", props.cool, props.test),
                }],
                void: false,
            },
            Element::Tag {
                name: "div".to_string(),
                attributes: vec![],
                children: props.children,
                void: false,
            },
        ],
        void: false,
    }
}

#[derive(Default)]
struct SimpleProps {
    enabled: bool,
    // Extra field to demonstrate ..Default::default() usefulness
    #[allow(dead_code)]
    style: Option<String>,
}

#[allow(non_snake_case)]
fn Simple(props: SimpleProps) -> Element {
    Element::Tag {
        name: "div".to_string(),
        attributes: vec![],
        children: vec![Element::Text {
            text: format!("enabled: {}", props.enabled),
        }],
        void: false,
    }
}

#[test]
fn test_component_with_attributes_and_children() {
    let result = html! {
        <MyCustomElement cool={5} test={"hello!"}>
            <h1>"Wow!"</h1>
            <p>"Second child"</p>
        </MyCustomElement>
    };

    let expected = Element::Tag {
        name: "div".to_string(),
        attributes: vec![],
        children: vec![
            Element::Tag {
                name: "p".to_string(),
                attributes: vec![],
                children: vec![Element::Text {
                    text: "cool: 5, test: hello!".to_string(),
                }],
                void: false,
            },
            Element::Tag {
                name: "div".to_string(),
                attributes: vec![],
                children: vec![
                    Element::Tag {
                        name: "h1".to_string(),
                        attributes: vec![],
                        children: vec![Element::Text {
                            text: "Wow!".to_string(),
                        }],
                        void: false,
                    },
                    Element::Tag {
                        name: "p".to_string(),
                        attributes: vec![],
                        children: vec![Element::Text {
                            text: "Second child".to_string(),
                        }],
                        void: false,
                    },
                ],
                void: false,
            },
        ],
        void: false,
    };

    assert_eq!(result, expected);
}

#[test]
fn test_component_with_valueless_attribute() {
    let result = html! {
        <Simple enabled />
    };

    let expected = Element::Tag {
        name: "div".to_string(),
        attributes: vec![],
        children: vec![Element::Text {
            text: "enabled: true".to_string(),
        }],
        void: false,
    };

    assert_eq!(result, expected);
}

#[test]
fn test_component_with_explicit_false() {
    let result = html! {
        <Simple enabled={false} />
    };

    let expected = Element::Tag {
        name: "div".to_string(),
        attributes: vec![],
        children: vec![Element::Text {
            text: "enabled: false".to_string(),
        }],
        void: false,
    };

    assert_eq!(result, expected);
}

#[test]
fn test_mix_of_regular_html_and_custom_components() {
    let result = html! {
        <div>
            <h1>"Regular HTML"</h1>
            <Simple enabled={true} />
            <p>"More regular HTML"</p>
        </div>
    };

    // Regular HTML elements are just tags
    if let Element::Tag { name, children, .. } = result {
        assert_eq!(name, "div");
        assert_eq!(children.len(), 3);

        // First child is h1
        if let Element::Tag { name, .. } = &children[0] {
            assert_eq!(name, "h1");
        } else {
            panic!("Expected h1 tag");
        }

        // Second child is the Simple component result
        if let Element::Tag { name, children, .. } = &children[1] {
            assert_eq!(name, "div");
            assert_eq!(children.len(), 1);
            if let Element::Text { text } = &children[0] {
                assert_eq!(text, "enabled: true");
            } else {
                panic!("Expected text node");
            }
        } else {
            panic!("Expected div tag from Simple component");
        }

        // Third child is p
        if let Element::Tag { name, .. } = &children[2] {
            assert_eq!(name, "p");
        } else {
            panic!("Expected p tag");
        }
    } else {
        panic!("Expected root div tag");
    }
}

#[test]
fn test_component_with_kebab_case_attribute() {
    #[derive(Default)]
    struct KebabComponentProps {
        my_attribute: String,
        // Extra field to demonstrate ..Default::default() usefulness
        #[allow(dead_code)]
        other: Option<i32>,
    }

    #[allow(non_snake_case)]
    fn KebabComponent(props: KebabComponentProps) -> Element {
        Element::Tag {
            name: "div".to_string(),
            attributes: vec![],
            children: vec![Element::Text {
                text: props.my_attribute,
            }],
            void: false,
        }
    }

    // Write the attribute as myAttribute (camelCase) - it will be converted to my-attribute (kebab-case)
    // and then to my_attribute (snake_case) for the struct field
    let result = html! {
        <KebabComponent myAttribute={"test-value"} />
    };

    let expected = Element::Tag {
        name: "div".to_string(),
        attributes: vec![],
        children: vec![Element::Text {
            text: "test-value".to_string(),
        }],
        void: false,
    };

    assert_eq!(result, expected);
}

#[test]
fn test_component_without_children() {
    let result = html! {
        <Simple enabled={true} />
    };

    let expected = Element::Tag {
        name: "div".to_string(),
        attributes: vec![],
        children: vec![Element::Text {
            text: "enabled: true".to_string(),
        }],
        void: false,
    };

    assert_eq!(result, expected);
}
