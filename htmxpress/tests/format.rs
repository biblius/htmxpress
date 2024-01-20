use htmxpress::HtmxElement;
use htmxpress_macros::Element;

#[derive(Debug, Element)]
struct Test {
    #[element("p")]
    #[format("{} {} three", two)]
    one: String,

    two: String,
}

#[test]
fn works() {
    let test = Test {
        one: "one".to_string(),
        two: "two".to_string(),
    };

    let html = r#"<p>one two three</p>"#;

    assert_eq!(html, test.to_htmx());
}
