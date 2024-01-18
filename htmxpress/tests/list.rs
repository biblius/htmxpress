use htmxpress::{Element, HtmxElement};

#[derive(Debug, Element)]
#[element("ul")]
struct Test {
    #[list]
    #[element("li")]
    testi: Vec<String>,

    #[list(nest)]
    cles: Vec<Testicle>,
}

#[derive(Debug, Element)]
#[element("div")]
struct Testicle {
    #[element("p")]
    #[format("p {}")]
    p: String,
}

#[test]
fn works() {
    let test = Test {
        testi: vec!["foo".to_string(), "bar".to_string()],
        cles: vec![
            Testicle { p: "p".to_string() },
            Testicle { p: "f".to_string() },
        ],
    };

    let html = r#"<ul><li>foo</li><li>bar</li><div><p>p p</p></div><div><p>p f</p></div></ul>"#;
    assert_eq!(html, test.to_htmx());
}
