use htmxpress::HtmxElement;

#[derive(Debug, htmxpress::Element)]
#[element("div")]
#[hx_post("/somewhere/{}", some_property)]
struct Parent {
    some_property: String,

    #[element("p")]
    #[hx_get("/somewhere/else")]
    #[format("I am a p! {}")]
    my_p: String,

    #[nest]
    child: Child,
}

#[derive(Debug, htmxpress::Element)]
#[element("div")]
#[attrs(id = "child", class = "child-class")]
#[hx_get("/elsewhere")]
struct Child {
    #[element("p")]
    #[attr("id" = "keepit{}", meaning_of_life)]
    #[format("Always keep it {}")]
    meaning_of_life: usize,
}

#[test]
fn works() {
    let parent = Parent {
        some_property: "something".to_string(),
        my_p: "Hello World!".to_string(),
        child: Child {
            meaning_of_life: 69,
        },
    };

    let htmx = r#"<div hx-post="/somewhere/something"><p hx-get="/somewhere/else">I am a p! Hello World!</p><div hx-get="/elsewhere" id="child" class="child-class"><p id="keepit69">Always keep it 69</p></div></div>"#;

    println!("{}", parent.to_htmx());
    assert_eq!(htmx, parent.to_htmx());
}
