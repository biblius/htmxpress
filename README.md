# htmxpress

Procedural macros for quickly generating htmx from rust structs.

## Example

The following attributes:

```rust
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
```

generate the following html:

```html
<div hx-post="/somewhere/something">
  <p hx-get="/somewhere/else">I am a p! Hello World!</p>
  <div hx-get="/elsewhere" id="child" class="child-class">
    <p id="keepit69">Always keep it 69</p>
  </div>
</div>
```

Todo List:

- [x] Basic HTML
- [x] Ajax attributes
- [x] Attributes for collections for ez lists
- [] Response trait
- [] hx headers for response trait
