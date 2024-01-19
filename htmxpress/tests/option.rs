use htmxpress::{Element, HtmxElement};

#[derive(Element)]
struct Test {
    #[element("p")]
    foo: Option<String>,

    #[element("p")]
    #[format("bar: {}")]
    bar: Option<u64>,

    #[element("p")]
    #[default("foo")]
    qux: Option<String>,

    #[element("p")]
    #[default("qua")]
    #[format("{}ck")]
    qua: Option<String>,
}

#[derive(Element)]
struct TestTwo {
    #[nest]
    _foo: Option<Child>,
}

#[derive(Element)]
struct Child {
    #[element("p")]
    #[format("c: {}")]
    #[default("foo")]
    c: Option<String>,
}

#[test]
fn works() {
    let test = Test {
        foo: None,
        bar: Some(420),
        qux: None,
        qua: None,
    };
    let html = r#"<p>bar: 420</p><p>foo</p><p>quack</p>"#;

    assert_eq!(html, test.to_htmx())
}

#[test]
fn works_nested() {
    let test = TestTwo {
        _foo: Some(Child {
            c: Some("s".to_string()),
        }),
    };

    let html = r#"<p>c: s</p>"#;

    assert_eq!(html, test.to_htmx())
}
