use htmxpress::HtmxElement;

#[derive(Debug, htmxpress::Element)]
#[element(div)]
#[hx_post("/somewhere/{}", some_property)]
struct Parent {
    some_property: String,

    #[element(p)]
    #[hx_get("/somewhere/something")]
    #[format("I am a p! {}")]
    my_p: String,

    #[nest]
    child: Child,
}

#[derive(Debug, htmxpress::Element)]
#[element(div)]
#[id("child")]
#[class("child-class")]
#[hx_get("/elsewhere")]
struct Child {
    #[element(p)]
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

    println!("{}", parent.to_htmx());
}
