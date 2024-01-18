use htmxpress::{Element, HtmxElement};

#[derive(Element)]
struct Test {
    #[element("p")]
    foo: Option<String>,
    #[element("p")]
    #[format("bar: {}")]
    bar: Option<u64>,
}

#[test]
fn works() {
    let test = Test {
        foo: None,
        bar: Some(420),
    };
    let html = r#"<p>bar: 420</p>"#;

    assert_eq!(html, test.to_htmx())
}
