use bumpalo::collections::String as BumpString;
use bumpalo::collections::Vec as BumpVec;
use bumpalo::Bump;
use paxhtml::{html, DefaultIn, Element};

struct MyCustomElementProps<'bump> {
    cool: i32,
    test: String,
    children: BumpVec<'bump, Element<'bump>>,
}
impl<'bump> DefaultIn<'bump> for MyCustomElementProps<'bump> {
    fn default_in(bump: &'bump Bump) -> Self {
        Self {
            cool: 0,
            test: String::new(),
            children: BumpVec::new_in(bump),
        }
    }
}

#[allow(non_snake_case)]
fn MyCustomElement<'bump>(bump: &'bump Bump, props: MyCustomElementProps<'bump>) -> Element<'bump> {
    let mut p_children = BumpVec::new_in(bump);
    p_children.push(Element::Text {
        text: BumpString::from_str_in(&format!("cool: {}, test: {}", props.cool, props.test), bump),
    });

    let mut div_children = BumpVec::new_in(bump);
    div_children.push(Element::Tag {
        name: BumpString::from_str_in("p", bump),
        attributes: BumpVec::new_in(bump),
        children: p_children,
        void: false,
    });
    div_children.push(Element::Tag {
        name: BumpString::from_str_in("div", bump),
        attributes: BumpVec::new_in(bump),
        children: props.children,
        void: false,
    });

    Element::Tag {
        name: BumpString::from_str_in("div", bump),
        attributes: BumpVec::new_in(bump),
        children: div_children,
        void: false,
    }
}

struct SimpleProps {
    enabled: bool,
}
impl DefaultIn<'_> for SimpleProps {
    fn default_in(_bump: &Bump) -> Self {
        Self { enabled: false }
    }
}

#[allow(non_snake_case)]
fn Simple<'bump>(bump: &'bump Bump, props: SimpleProps) -> Element<'bump> {
    let mut children = BumpVec::new_in(bump);
    children.push(Element::Text {
        text: BumpString::from_str_in(&format!("enabled: {}", props.enabled), bump),
    });

    Element::Tag {
        name: BumpString::from_str_in("div", bump),
        attributes: BumpVec::new_in(bump),
        children,
        void: false,
    }
}

#[test]
fn test_component_with_attributes_and_children() {
    let bump = Bump::new();

    let result = html! { in &bump;
        <MyCustomElement cool={5} test={"hello!"}>
            <h1>"Wow!"</h1>
            <p>"Second child"</p>
        </MyCustomElement>
    };

    // Just check the structure matches
    if let Element::Tag { name, children, .. } = &result {
        assert_eq!(name.as_str(), "div");
        assert_eq!(children.len(), 2);

        // First child should be p with the cool/test message
        if let Element::Tag {
            name,
            children: p_children,
            ..
        } = &children[0]
        {
            assert_eq!(name.as_str(), "p");
            if let Element::Text { text } = &p_children[0] {
                assert_eq!(text.as_str(), "cool: 5, test: hello!");
            }
        }

        // Second child should be div with h1 and p children
        if let Element::Tag {
            name,
            children: div_children,
            ..
        } = &children[1]
        {
            assert_eq!(name.as_str(), "div");
            assert_eq!(div_children.len(), 2);
        }
    } else {
        panic!("Expected Tag element");
    }
}

#[test]
fn test_component_with_valueless_attribute() {
    let bump = Bump::new();

    let result = html! { in &bump;
        <Simple enabled />
    };

    if let Element::Tag { name, children, .. } = result {
        assert_eq!(name.as_str(), "div");
        assert_eq!(children.len(), 1);
        if let Element::Text { text } = &children[0] {
            assert_eq!(text.as_str(), "enabled: true");
        }
    } else {
        panic!("Expected Tag element");
    }
}

#[test]
fn test_component_with_explicit_false() {
    let bump = Bump::new();

    let result = html! { in &bump;
        <Simple enabled={false} />
    };

    if let Element::Tag { name, children, .. } = result {
        assert_eq!(name.as_str(), "div");
        if let Element::Text { text } = &children[0] {
            assert_eq!(text.as_str(), "enabled: false");
        }
    } else {
        panic!("Expected Tag element");
    }
}

#[test]
fn test_component_with_default_props() {
    let bump = Bump::new();

    // Only specify 'cool', let 'test' and 'children' use defaults
    let result = html! { in &bump;
        <MyCustomElement cool={42} />
    };

    if let Element::Tag { name, children, .. } = &result {
        assert_eq!(name.as_str(), "div");
        assert_eq!(children.len(), 2);

        // First child should be p with default test value (empty string)
        if let Element::Tag {
            name,
            children: p_children,
            ..
        } = &children[0]
        {
            assert_eq!(name.as_str(), "p");
            if let Element::Text { text } = &p_children[0] {
                assert_eq!(text.as_str(), "cool: 42, test: ");
            }
        }

        // Second child should be div with no children (default empty vec)
        if let Element::Tag {
            name,
            children: div_children,
            ..
        } = &children[1]
        {
            assert_eq!(name.as_str(), "div");
            assert_eq!(div_children.len(), 0);
        }
    } else {
        panic!("Expected Tag element");
    }
}

#[test]
fn test_mix_of_regular_html_and_custom_components() {
    let bump = Bump::new();

    let result = html! { in &bump;
        <div>
            <h1>"Regular HTML"</h1>
            <Simple enabled={true} />
            <p>"More regular HTML"</p>
        </div>
    };

    // Regular HTML elements are just tags
    if let Element::Tag { name, children, .. } = result {
        assert_eq!(name.as_str(), "div");
        assert_eq!(children.len(), 3);

        // First child is h1
        if let Element::Tag { name, .. } = &children[0] {
            assert_eq!(name.as_str(), "h1");
        } else {
            panic!("Expected h1 tag");
        }

        // Second child is the Simple component result
        if let Element::Tag { name, children, .. } = &children[1] {
            assert_eq!(name.as_str(), "div");
            assert_eq!(children.len(), 1);
            if let Element::Text { text } = &children[0] {
                assert_eq!(text.as_str(), "enabled: true");
            } else {
                panic!("Expected text node");
            }
        } else {
            panic!("Expected div tag from Simple component");
        }

        // Third child is p
        if let Element::Tag { name, .. } = &children[2] {
            assert_eq!(name.as_str(), "p");
        } else {
            panic!("Expected p tag");
        }
    } else {
        panic!("Expected root div tag");
    }
}

#[test]
fn test_component_with_kebab_case_attribute() {
    let bump = Bump::new();

    struct KebabComponentProps {
        my_attribute: String,
    }
    impl DefaultIn<'_> for KebabComponentProps {
        fn default_in(_bump: &Bump) -> Self {
            Self {
                my_attribute: String::new(),
            }
        }
    }

    #[allow(non_snake_case)]
    fn KebabComponent<'bump>(bump: &'bump Bump, props: KebabComponentProps) -> Element<'bump> {
        let mut children = BumpVec::new_in(bump);
        children.push(Element::Text {
            text: BumpString::from_str_in(&props.my_attribute, bump),
        });

        Element::Tag {
            name: BumpString::from_str_in("div", bump),
            attributes: BumpVec::new_in(bump),
            children,
            void: false,
        }
    }

    // Write the attribute as myAttribute (camelCase) - it will be converted to my-attribute (kebab-case)
    // and then to my_attribute (snake_case) for the struct field
    let result = html! { in &bump;
        <KebabComponent myAttribute={"test-value"} />
    };

    if let Element::Tag { name, children, .. } = result {
        assert_eq!(name.as_str(), "div");
        if let Element::Text { text } = &children[0] {
            assert_eq!(text.as_str(), "test-value");
        }
    } else {
        panic!("Expected Tag element");
    }
}

#[test]
fn test_component_without_children() {
    let bump = Bump::new();

    let result = html! { in &bump;
        <Simple enabled={true} />
    };

    if let Element::Tag { name, children, .. } = result {
        assert_eq!(name.as_str(), "div");
        if let Element::Text { text } = &children[0] {
            assert_eq!(text.as_str(), "enabled: true");
        }
    } else {
        panic!("Expected Tag element");
    }
}
