use std::fmt::Debug;

use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::quote;
use syn::{spanned::Spanned, Attribute, Data, DeriveInput, Ident, LitStr, Token};

const ELEMENT_ATTR: &str = "element";
const NEST_ATTR: &str = "nest";
const HX_GET_ATTR: &str = "hx_get";
const HX_POST_ATTR: &str = "hx_post";
const HX_PATCH_ATTR: &str = "hx_patch";
const HX_PUT_ATTR: &str = "hx_put";
const HX_DELETE_ATTR: &str = "hx_delete";
const HX_TRIGGER_ATTR: &str = "hx_trigger";
const FORMAT_ATTR: &str = "format";
const ID_ATTR: &str = "id";
const CLASS_ATTR: &str = "class";

const HTMX_METHODS: [&str; 5] = [
    HX_GET_ATTR,
    HX_POST_ATTR,
    HX_PUT_ATTR,
    HX_DELETE_ATTR,
    HX_PATCH_ATTR,
];

#[proc_macro_derive(
    Element,
    attributes(
        element, nest, format, wrap, hx_get, hx_post, hx_put, hx_patch, hx_delete, hx_target, id,
        class,
    )
)]
pub fn derive_element(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let strct: DeriveInput = syn::parse(input).expect("invalid input");

    let HtmxStruct {
        self_element,
        inner_tokens,
    } = HtmxStruct::collect_from(&strct);

    let ident = &strct.ident;
    let (im, ty, wh) = strct.generics.split_for_impl();

    let parent_open = self_element.as_ref().map(HtmxStructElement::open);
    let parent_close = self_element.as_ref().map(HtmxStructElement::close);

    quote::quote!(
        impl #im htmxpress::HtmxElement for #ident #ty #wh {
            fn to_htmx(&self) -> String {
                use std::fmt::Write;
                let mut html = String::new();
                #parent_open
                #inner_tokens
                #parent_close
                html
            }
        }
    )
    .into()
}

#[derive(Debug, Default)]
struct HtmxStruct {
    /// Self html element
    self_element: Option<HtmxStructElement>,

    /// Contains tokens for the HtmxElement impl
    /// obtained from fields and nested htmx structs
    inner_tokens: TokenStream,
}

impl HtmxStruct {
    fn collect_from(strct: &DeriveInput) -> Self {
        let mut this = Self {
            self_element: HtmxStructElement::collect_from(&strct.attrs),
            ..Default::default()
        };

        let Data::Struct(ref strct) = strct.data else {
            abort!(strct.span(), "Element can only be derived on structs");
        };

        for field in strct.fields.iter() {
            if let Some(element) =
                HtmxFieldElement::collect_from(field.ident.as_ref().unwrap(), &field.attrs)
            {
                this.inner_tokens.extend(element.to_tokens())
            }

            for attr in field.attrs.iter() {
                let Some(id) = attr.meta.path().get_ident() else {
                    continue;
                };

                if id == NEST_ATTR {
                    let syn::Type::Path(ref path) = field.ty else {
                        abort!(
                            field.ty.span(),
                            "nest can only be used on fields that implement Element"
                        );
                    };

                    let path = path.path.clone();
                    let field_name = field.ident.as_ref().unwrap_or_else(|| {
                        abort!(
                            field.span(),
                            "HtmxElement only works on structs with named fields"
                        )
                    });

                    let tokens = quote!(
                        {
                            let nested = <#path as htmxpress::HtmxElement>::to_htmx(&self.#field_name);
                            let _ = write!(html, "{nested}");
                        }
                    );

                    this.inner_tokens.extend(tokens);
                }
            }
        }

        this
    }
}

#[derive(Debug)]
struct HtmxFieldElement {
    /// Name of the field annotated with `element`
    field_name: Ident,

    html_element: String,

    attrs: HtmlAttributes,
}

/// Tokens used to create HTML element attributes
#[derive(Debug)]
struct AttributeTokens {
    id: TokenStream,

    class: TokenStream,

    /// The hx-method tokens
    request: TokenStream,
}

impl HtmlAttributes {
    pub fn attr_tokens(&self) -> AttributeTokens {
        let id = self
            .id
            .as_ref()
            .map(|id| quote!(let id = format!(r#" id="{}""#, #id);))
            .unwrap_or(quote!(let id = String::new();));

        let class = self
            .class
            .as_ref()
            .map(|class| quote!(let class = format!(r#" class="{}""#, #class);))
            .unwrap_or(quote!(let class = String::new();));

        let request = self
            .hx_req
            .as_ref()
            .map(|hx_req| {
                let method = match hx_req.method {
                    HtmxMethod::Get => "hx-get",
                    HtmxMethod::Post => "hx-post",
                    HtmxMethod::Put => "hx-put",
                    HtmxMethod::Delete => "hx-delete",
                    HtmxMethod::Patch => "hx-patch",
                };

                let path = &hx_req.params.path;
                let args = &hx_req.params.args;
                if args.is_empty() {
                    let path = format!(r#" {method}="{}""#, path.value());

                    quote!(
                        let request = #path;
                    )
                } else {
                    let args = args.iter().map(|field| quote!(self.#field));
                    let path = format!(r#" {method}="{}""#, path.value());

                    quote!(
                        let request = format!(#path, #(#args),*);
                    )
                }
            })
            .unwrap_or(quote!(let request = String::new();));

        AttributeTokens { id, class, request }
    }
}

impl HtmxFieldElement {
    fn to_tokens(&self) -> proc_macro2::TokenStream {
        let Self {
            field_name,
            html_element,
            attrs,
        } = self;

        let AttributeTokens { id, class, request } = attrs.attr_tokens();

        let content = attrs
            .format_str
            .as_ref()
            .map(|fmt| quote!(let content = format!(#fmt, self.#field_name);))
            .unwrap_or_else(|| quote!(let content = format!("{}", self.#field_name);));

        let element = quote!(let element = #html_element;);

        quote!(
            {
                #id
                #class
                #request
                #content
                #element
                let _ = write!(html, r#"<{element}{request}{id}{class}>{content}</{element}>"#);
            }
        )
    }

    /// Collect all attributes related to HTML
    ///
    /// Ignores the `nest` attribute
    fn collect_from(field_name: &Ident, attrs: &[Attribute]) -> Option<Self> {
        let mut element = Self {
            field_name: field_name.clone(),
            html_element: String::new(),
            attrs: HtmlAttributes::collect_from(attrs),
        };

        let mut _attrs = attrs
            .iter()
            .filter_map(|attr| Some(attr.path().get_ident()?.to_string()))
            .collect::<Vec<_>>();

        let is_element = _attrs.contains(&ELEMENT_ATTR.to_string());

        if !is_element {
            return None;
        }

        for attr in attrs {
            let Some(id) = attr.meta.path().get_ident() else {
                continue;
            };
            if id == ELEMENT_ATTR {
                element.html_element = parse_element(attr).to_string()
            }
        }

        Some(element)
    }
}

#[derive(Debug, Default)]
struct HtmxStructElement {
    html_element: Option<String>,

    attrs: HtmlAttributes,
}

#[derive(Debug, Default)]
struct HtmlAttributes {
    /// HTML id attribute
    id: Option<String>,

    /// HTML class attribute
    class: Option<String>,

    /// Format string for the inner content.
    format_str: Option<LitStr>,

    /// hx-method attribute
    hx_req: Option<HtmxRequest>,

    /// hx-target attribute
    hx_target: Option<String>,

    /// hx-trigger attribute
    hx_trigger: Option<String>,
}

impl HtmlAttributes {
    fn collect_from(attrs: &[Attribute]) -> Self {
        let mut this = Self::default();

        for attr in attrs {
            let Some(id) = attr.meta.path().get_ident() else {
                continue;
            };

            if HTMX_METHODS.contains(&id.to_string().as_str()) {
                if this.hx_req.is_some() {
                    abort!(
                        attr.span(),
                        "cannot have more than one htmx method on element"
                    )
                }
                let hx_req = parse_htmx_request(attr);
                this.hx_req = Some(hx_req);
                continue;
            }

            if id == FORMAT_ATTR {
                if this.format_str.is_some() {
                    abort!(
                        attr.span(),
                        "cannot have more than one format str on element"
                    )
                }
                let format = parse_format(attr);
                this.format_str = Some(format);
                continue;
            }

            if id == CLASS_ATTR {
                let class = parse_str(attr);
                this.class = Some(class);
            }

            if id == ID_ATTR {
                let id = parse_str(attr);
                this.id = Some(id);
            }
        }

        this
    }
}

impl HtmxStructElement {
    fn open(&self) -> Option<proc_macro2::TokenStream> {
        let Self {
            html_element,
            attrs,
        } = self;

        let element = html_element.as_ref().map(|el| quote!(let element = #el;))?;
        let AttributeTokens { id, class, request } = attrs.attr_tokens();

        Some(quote!(
            {
                #id
                #class
                #request
                #element
                let _ = write!(html, r#"<{element}{request}{id}{class}>"#);
            }
        ))
    }

    fn close(&self) -> Option<TokenStream> {
        let element = self
            .html_element
            .as_ref()
            .map(|el| quote!(let element = #el;))?;
        Some(quote!(
            {
                #element
                let _ = write!(html, "</{element}>");
            }
        ))
    }

    /// Collect all attributes related to HTML
    ///
    /// Ignores the `nest` attribute
    fn collect_from(attrs: &[Attribute]) -> Option<Self> {
        let mut element = Self {
            html_element: None,
            attrs: HtmlAttributes::collect_from(attrs),
        };

        for attr in attrs {
            let Some(id) = attr.meta.path().get_ident() else {
                continue;
            };
            if id == ELEMENT_ATTR {
                element.html_element = Some(parse_element(attr).to_string())
            }
        }

        Some(element)
    }
}

fn parse_str(attr: &Attribute) -> String {
    let list = attr
        .meta
        .require_list()
        .unwrap_or_else(|_| abort!(attr.meta.span(), r#"expected str list, e.g. `foo("bar")`"#));

    list.parse_args::<LitStr>()
        .unwrap_or_else(|_| abort!(attr.meta.span(), "expected ident, e.g. `element(div)`"))
        .value()
}

fn parse_element(attr: &Attribute) -> Ident {
    let list = attr
        .meta
        .require_list()
        .unwrap_or_else(|_| abort!(attr.meta.span(), "expected list, e.g. `element(div)`"));
    list.parse_args::<Ident>()
        .unwrap_or_else(|_| abort!(attr.meta.span(), "expected ident, e.g. `element(div)`"))
}

fn parse_format(attr: &Attribute) -> LitStr {
    let list = attr
        .meta
        .require_list()
        .unwrap_or_else(|_| abort!(attr.meta.span(), r#"expected list, e.g. htmx_get("/path")"#));
    list.parse_args::<LitStr>()
        .unwrap_or_else(|e| abort!(attr.meta.span(), format!("{e}")))
}

fn parse_htmx_request(attr: &Attribute) -> HtmxRequest {
    let list = attr
        .meta
        .require_list()
        .unwrap_or_else(|_| abort!(attr.meta.span(), r#"expected list, e.g. htmx_get("/path")"#));

    let ident = attr
        .path()
        .get_ident()
        .unwrap_or_else(|| abort!(attr.path().span(), "htmx_* attributes must have ident",));

    let method = match ident.to_string().as_str() {
        HX_DELETE_ATTR => HtmxMethod::Delete,
        HX_GET_ATTR => HtmxMethod::Get,
        HX_POST_ATTR => HtmxMethod::Post,
        HX_PUT_ATTR => HtmxMethod::Put,
        HX_PATCH_ATTR => HtmxMethod::Patch,
        _ => abort!(ident.span(), "unrecognized htmx attr"),
    };

    let params: HtmxRequestParams = list
        .parse_args()
        .unwrap_or_else(|e| abort!(list.span(), &format!("{e}")));

    HtmxRequest { method, params }
}

#[derive(Debug)]
struct HtmxRequest {
    method: HtmxMethod,
    params: HtmxRequestParams,
}

#[derive(Debug)]
enum HtmxMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

/// Arguments for hx_method attributes
#[derive(Debug)]
struct HtmxRequestParams {
    /// Request path
    path: LitStr,

    /// Optional args for the path, i.e. fields on this struct.
    args: Vec<Ident>,
}

impl syn::parse::Parse for HtmxRequestParams {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let path = input.parse::<LitStr>()?;

        if input.is_empty() {
            return Ok(Self { path, args: vec![] });
        }

        let mut args = vec![];

        loop {
            if input.is_empty() {
                break;
            }

            input.parse::<Token![,]>()?;

            if input.is_empty() {
                break;
            }

            let ident = input.parse()?;
            args.push(ident);
        }

        Ok(Self { path, args })
    }
}
