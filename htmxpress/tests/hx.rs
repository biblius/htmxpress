use htmxpress::HtmxElement;

#[derive(Debug, htmxpress::Element)]
#[element("div")]
#[hx_post("/somewhere/{}", some_property)]
#[hx("push-url" = "hello")]
struct Parent {
    some_property: String,

    #[element("p")]
    #[hx("target" = "something", "swap" = "innerHtml")]
    #[hx_get("/somewhere/else")]
    my_p: String,
}

#[test]
fn works() {
    let test = Parent {
        some_property: "hello".to_string(),
        my_p: "myp".to_string(),
    };

    let html = r#"<div hx-post="/somewhere/hello" hx-push-url="hello"><p hx-get="/somewhere/else" hx-target="something" hx-swap="innerHtml">myp</p></div>"#;

    assert_eq!(html, test.to_htmx());
}
