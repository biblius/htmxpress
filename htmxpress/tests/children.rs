use htmxpress::HtmxElement;
use htmxpress_macros::Element;

#[derive(Debug, Element)]
struct Test {
    #[element("p")]
    #[before("<span>Hello</span>")]
    #[after("<span>World</span>")]
    el: String,
}

#[test]
fn children_works() {
    let test = Test {
        el: "My".to_string(),
    };

    let html = r#"<p><span>Hello</span>My<span>World</span></p>"#;

    assert_eq!(html, test.to_htmx());
}
