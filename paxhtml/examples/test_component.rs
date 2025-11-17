use paxhtml::{html, Document, Element};

#[derive(Default)]
struct MyCustomElementProps {
    cool: i32,
    test: String,
    children: Vec<Element>,
}

fn MyCustomElement(props: MyCustomElementProps) -> Element {
    html! {
        <div>
            <p>{format!("cool: {}, test: {}", props.cool, props.test)}</p>
            <div>{Element::from_iter(props.children)}</div>
        </div>
    }
}

#[derive(Default)]
struct SimpleProps {
    enabled: bool,
}

fn Simple(props: SimpleProps) -> Element {
    html! {
        <div>{format!("enabled: {}", props.enabled)}</div>
    }
}

fn main() {
    println!("Test 1 - Component with attributes and children:");
    let result = html! {
        <MyCustomElement cool={5} test={"hello!"}>
            <h1>"Wow!"</h1>
            <p>"Second child"</p>
        </MyCustomElement>
    };
    println!("{}", Document::new([result]).write_to_string().unwrap());
    println!();

    println!("Test 2 - Component with valueless attribute:");
    let result2 = html! {
        <Simple enabled />
    };
    println!("{}", Document::new([result2]).write_to_string().unwrap());
    println!();

    println!("Test 3 - Component with explicit false:");
    let result3 = html! {
        <Simple enabled={false} />
    };
    println!("{}", Document::new([result3]).write_to_string().unwrap());
    println!();

    println!("Test 4 - Mix of regular HTML and custom components:");
    let result4 = html! {
        <div>
            <h1>"Regular HTML"</h1>
            <Simple enabled={true} />
            <p>"More regular HTML"</p>
        </div>
    };
    println!("{}", Document::new([result4]).write_to_string().unwrap());
}
