# htmxpress

Procedural macros for quickly generating htmx from rust structs.

## Attributes

Reference:

- [element](#element)
- [attrs](#attrs)
- [attr](#attr)
- [format](#format)
- [nest](#nest)
- [map](#map)
- [before/after](#before/after)
- [default](#default)
- [list](<#list-[(nest)]>)
- [hx](#hx,-hx_method)
- [urlencode](#urlencode)

### element

Include the field contents in the final HTML inside the specified element.

Only one element attribute is allowed per field/struct.

When applied on structs, all elements inside it will be wrapped in the specified element.

Fields that do not have this attribute will be ignored when generating the HTML. Subsequently, any other attributes related to the HTML element will also be ignored.

#### Example

```rust
use htmxpress::{Element, HtmxElement};

#[derive(Element)]
#[element("div")]
struct El {
  #[element("p")]
  foo: String
}
```

```html
<div><p>foo</p></div>
```

### attrs

Specify the HTML attributes for the element. Useful for commonly used static attributes.

Attributes are specified as `ident = "value"` pairs.

#### Example

```rust
use htmxpress::{Element, HtmxElement};

#[derive(Element)]
#[element("div")]
#[attrs(class = "container")]
struct El {
  #[element("p")]
  #[attrs(class = "inner", id = "foo")]
  foo: String
}
```

```html
<div class="container"><p class="inner" id="foo">foo</p></div>
```

### attr

Specify a single attribute for the element.

Useful either when the attribute must be dynamically created or when its key cannot be written as a valid rust ident.

#### Example

```rust
use htmxpress::{Element, HtmxElement};

#[derive(Element)]
#[element("div")]
#[attr("funky-attr" = "value")]
struct El {
  #[element("p")]
  #[attr("dynamic" = "{}", param)]
  foo: String,
  param: usize,
}
```

```html
<div funky-attr="value"><p dynamic="param">foo</p></div>
```

### format

Format the content of the element using the provided format string.

Valid only on fields.

#### Example

```rust
use htmxpress::{Element, HtmxElement};

#[derive(Element)]
#[element("div")]
struct El {
  #[element("p")]
  #[format("Hi, I'm {}")]
  foo: String
}
```

```html
<div><p>Hi, I'm foo</p></div>
```

### map

Map the value of this field using an expression before writing the HTML.

This attribute applies only to HTML generation. Any field annotated with this will
still use its original value when used in format strings in other attributes.

Valid syntax is `variable => { /* do stuff with variable */ }`

#### Example

```rust
use htmxpress::{Element, HtmxElement};

#[derive(Element)]
#[element("div")]
struct El {
  #[element("p")]
  #[attr("id" = "{}", foo)]
  #[map(var => var.is_empty())]
  #[format("Empty: {}")]
  foo: String
}

let el = El { foo: "foo".to_string() };
let html = r#"<div><p id="foo">Empty: false</p></div>"#;

assert_eq!(html, el.to_htmx());
```

```html
<div><p id="foo">Empty: false</p></div>
```

### default

Valid only on `Option`s and fields that are not annotated with `map`.

Normally, `Option`s annotated with `element` which are `None` during HTML generation will be completely ignored and no DOM object will get created.
This attribute ensures the element gets created with the specified content even when the field is `None`.

#### Example

```rust
use htmxpress::{Element, HtmxElement};

#[derive(Element)]
#[element("div")]
struct El {
  #[element("p")]
  #[default("foo")]
  foo: Option<String>,

  #[element("p")]
  bar: Option<String>
}

let el = El { foo: None, bar: None };
let html = r#"<div><p>foo</p></div>"#;

assert_eq!(html, el.to_htmx())
```

```html
<div><p>foo</p></div>
```

### before/after

Insert/append strings before/after the content of an element.

Useful for inserting elements inside the parent element, especially when dealing with lists.

When used with `list`, it inserts the given string before/after the first/last element.

#### Example

```rust
use htmxpress::{Element, HtmxElement};

#[derive(Element)]
#[element("div")]
struct El {
  #[element("section")]
  #[before("<h1>Win a million dollars</h1>")]
  #[after("<button>Do it</button>")]
  foo: String
}

let el = El { foo: "You are the chosen one".to_string() };
let html = r#"<div><section><h1>Win a million dollars</h1>You are the chosen one<button>Do it</button></section></div>"#;

assert_eq!(html, el.to_htmx())
```

```html
<div>
  <section>
    <h1>Win a million dollars</h1>
    You are the chosen one
    <button>Do it</button>
  </section>
</div>
```

### nest

Use on any field that's a struct implementing `HtmxElement`. Calls the underlying implementation in the context of the current struct.

#### Example

```rust
use htmxpress::{Element, HtmxElement};

#[derive(Element)]
#[element("div")]
struct El {
  #[element("section")]
  #[nest]
  foo: Qux
}

#[derive(Element)]
struct Qux {
  #[element("p")]
  qux: &'static str,
}

let el = El { foo: Qux { qux: "qux" } };
let html = r#"<div><section><p>qux</p></section></div>"#;

assert_eq!(html, el.to_htmx())
```

```html
<div>
  <section><p>qux</p></section>
</div>
```

### list [(nest)]

Use on list collections. Valid with any iterable whose item implements `Display`.

Create the specified element for each item in the list, using the item's value for its content.

When used as `list(nest)`, calls `to_htmx()` for each item in the list and writes it to the final HTML.

#### Example

```rust
use htmxpress::{Element, HtmxElement};

#[derive(Element)]
#[element("ul")]
struct El {
  #[element("li")]
  #[list]
  foo: Vec<&'static str>,

  #[list(nest)]
  #[element("li")]
  bar: Vec<Qux>,
}

#[derive(Element)]
struct Qux {
  #[element("p")]
  qux: &'static str,
}

let el = El { foo: vec!["foo1", "foo2"], bar: vec![Qux { qux: "qux" }] };
let html = r#"<ul><li>foo1</li><li>foo2</li><li><p>qux</p></li></ul>"#;

assert_eq!(html, el.to_htmx())
```

```html
<ul>
  <li>foo1</li>
  <li>foo2</li>
  <li>
    <p>qux</p>
  </li>
</ul>
```

### hx, hx_method

`hx_*` attributes correspond to the available AJAX methods in htmx. They also support format strings, i.e. can be dynamically generated using the fields of the struct in question.

`hx` is pretty much the same as [attrs](#attrs), except it prepends `hx` to every key.

If you need dynamic values, use [attr](#attr).

#### Example

```rust
use htmxpress::{Element, HtmxElement};

#[derive(Element)]
#[element("div")]
#[hx_get("/foo/{}", path)]
#[hx("swap" = "innerHtml")]
#[attr("hx-target" = "#{}", id)]
struct El {
  path: &'static str,

  #[element("p")]
  #[attr("id" = "{}", id)]
  #[format("Meaning of life: {}")]
  id: usize,
}

let el = El { id: 420, path: "bar" };
let html = r##"<div hx-get="/foo/bar" hx-target="#420" hx-swap="innerHtml"><p id="420">Meaning of life: 420</p></div>"##;

assert_eq!(html, el.to_htmx())
```

```html
<div hx-get="/foo/bar" hx-swap="innerHtml" hx-target="#420">
  <p id="420">Meaning of life: 420</p>
</div>
```

### urlencode

Use when you need to encode url parameters.

#### Example

```rust
use htmxpress::{Element, HtmxElement};

#[derive(Element)]
#[element("div")]
#[hx_get("/foo/{}", path)]
#[urlencode]
struct El {
  path: &'static str,
}

let el = El { path: "my bar" };
let html = r##"<div hx-get="/foo/my%20bar"></div>"##;

assert_eq!(html, el.to_htmx())
```

```html
<div hx-get="/foo/my%20bar"></div>
```

## More examples

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
- [] Self-closing elements
- [] Additional meta elements for existing ones for integrating with head-support
