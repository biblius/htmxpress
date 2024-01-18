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

#[test]
fn works() {
    let test = Test {
        foo: None,
        bar: Some(420),
        qux: None,
        qua: None,
    };
    let html = r#"<p>bar: 420</p><p>foo</p><p>quack</p>"#;

    println!("{}", test.to_htmx());

    assert_eq!(html, test.to_htmx())
}
