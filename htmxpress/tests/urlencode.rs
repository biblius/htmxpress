use htmxpress::*;

#[derive(Element)]
#[element("div")]
#[hx_get("/foo?bar={}", fo)]
#[urlencode]
struct Test {
    #[element("p")]
    #[hx_get("/{}", fo)]
    #[urlencode]
    fo: String,
}

#[test]
fn works() {
    let test = Test {
        fo: "crazy param".to_string(),
    };

    let html =
        r#"<div hx-get="/foo?bar=crazy%20param"><p hx-get="/crazy%20param">crazy param</p></div>"#;

    assert_eq!(html, test.to_htmx())
}
