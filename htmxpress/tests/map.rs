use htmxpress::HtmxElement;

#[derive(Debug, htmxpress::Element)]
#[element("div")]
struct Test {
    #[element("p")]
    #[map(var => var.is_empty())]
    #[format("empty: {}")]
    some_property: String,
}

#[derive(Debug, htmxpress::Element)]
#[element("div")]
struct Test2 {
    #[element("p")]
    #[map(var => var.is_empty())]
    #[format("empty: {}")]
    some_property: Option<String>,
}

#[test]
fn works_option() {
    let test = Test2 {
        some_property: Some("bar".to_string()),
    };

    let html = r#"<div><p>empty: false</p></div>"#;

    assert_eq!(html, test.to_htmx());
}

#[test]
fn works() {
    let test = Test {
        some_property: "foo".to_string(),
    };

    let html = r#"<div><p>empty: false</p></div>"#;

    assert_eq!(html, test.to_htmx());

    let test = Test {
        some_property: "".to_string(),
    };

    let html = r#"<div><p>empty: true</p></div>"#;

    assert_eq!(html, test.to_htmx());
}
