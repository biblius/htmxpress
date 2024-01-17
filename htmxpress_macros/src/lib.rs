use std::fmt::Debug;

use proc_macro2::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use quote::{format_ident, quote};
use syn::{
    parse::ParseStream, punctuated::Punctuated, spanned::Spanned, Attribute, Data, DeriveInput,
    Ident, LitStr, MetaList, MetaNameValue, Token,
};

const ELEMENT_ATTR: &str = "element";
const NEST_ATTR: &str = "nest";
const FORMAT_ATTR: &str = "format";
const ATTRS_ATTR: &str = "attrs";
const ATTR_ATTR: &str = "attr";
const HX_GET_ATTR: &str = "hx_get";
const HX_POST_ATTR: &str = "hx_post";
const HX_PATCH_ATTR: &str = "hx_patch";
const HX_PUT_ATTR: &str = "hx_put";
const HX_DELETE_ATTR: &str = "hx_delete";

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
        element, attrs, attr, format, nest, hx, hx_get, hx_post, hx_put, hx_patch, hx_delete,
    )
)]
#[proc_macro_error]
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
            // Extract element from attributes
            if let Some(element) =
                HtmxFieldElement::collect_from(field.ident.as_ref().unwrap(), &field.attrs)
            {
                this.inner_tokens.extend(element.to_tokens())
            }

            // Handle nested structs
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

impl HtmxFieldElement {
    fn to_tokens(&self) -> proc_macro2::TokenStream {
        let Self {
            field_name,
            html_element,
            attrs,
        } = self;

        let AttributeTokens {
            static_attrs,
            dyn_attrs,
            request,
        } = attrs.attr_tokens();

        let content = attrs
            .format_str
            .as_ref()
            .map(|fmt| quote!(let content = format!(#fmt, self.#field_name);))
            .unwrap_or_else(|| quote!(let content = format!("{}", self.#field_name);));

        let element = quote!(let element = #html_element;);

        quote!(
            {
                let mut attributes = String::new();
                #dyn_attrs
                #static_attrs
                #request
                #content
                #element
                let _ = write!(html, r#"<{element}{request}{attributes}>{content}</{element}>"#);
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
                element.html_element = parse_str(attr)
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

impl HtmxStructElement {
    fn open(&self) -> Option<proc_macro2::TokenStream> {
        let Self {
            html_element,
            attrs,
        } = self;

        let element = html_element.as_ref().map(|el| quote!(let element = #el;))?;
        let AttributeTokens {
            static_attrs,
            request,
            dyn_attrs,
        } = attrs.attr_tokens();

        Some(quote!(
            {
                let mut attributes = String::new();
                #dyn_attrs
                #static_attrs
                #request
                #element
                let _ = write!(html, r#"<{element}{request}{attributes}>"#);
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
                element.html_element = Some(parse_str(attr))
            }
        }

        Some(element)
    }
}

#[derive(Debug, Default)]
struct HtmlAttributes {
    /// HTML key="value" attributes, along with
    /// any hx-*="*" attributes other than AJAX
    attributes: Vec<(String, String)>,

    /// HTML attributes from single `attr` and `hx` attributes
    dyn_attributes: Vec<DynamicAttr>,

    /// Format string for the inner content.
    format_str: Option<LitStr>,

    /// hx-method attribute
    hx_req: Option<HtmxRequest>,
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

            if id == ATTRS_ATTR {
                let attrs = parse_name_values(attr);
                this.attributes.extend(attrs);
            }

            if id == ATTR_ATTR {
                let attr = parse_dyn_attr(attr);
                this.dyn_attributes.push(attr);
            }
        }

        this
    }

    pub fn attr_tokens(&self) -> AttributeTokens {
        let static_attrs = self
            .attributes
            .iter()
            .map(|(key, val)| {
                let var = format_ident!("_{key}");
                quote!(
                    {
                        let #var = format!(r#" {}="{}""#, #key, #val);
                        let _ = write!(attributes, "{}", #var);
                    }
                )
            })
            .collect();

        let dyn_attrs = self
            .dyn_attributes
            .iter()
            .map(|DynamicAttr { key, params }| {
                let FormatParams { fmt, args } = params;
                let args = args.iter().map(|field| quote!(self.#field));
                quote!({
                    let _attr = format!(#fmt, #(#args),*);
                    let _attr = format!(r#" {}="{}""#, #key, _attr);
                    let _ = write!(attributes, "{}", _attr);
                })
            })
            .collect();

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
                hx_req.params.to_tokens(method)
            })
            .unwrap_or(quote!(let request = String::new();));

        AttributeTokens {
            static_attrs,
            dyn_attrs,
            request,
        }
    }
}

/// Tokens used to create HTML element attributes
#[derive(Debug)]
struct AttributeTokens {
    /// Attribute tokens that do not use the struct's fields. Obtained from `attrs`
    static_attrs: TokenStream,

    /// Attribute tokens that use struct fields. Obtained from `attr`
    dyn_attrs: TokenStream,

    /// The hx-method tokens
    request: TokenStream,
}

fn parse_name_values(attr: &Attribute) -> Vec<(String, String)> {
    let list = attr.meta.require_list().unwrap_or_else(|_| {
        abort!(
            attr.meta.span(),
            r#"expected name value list, e.g. `attrs(id = "foo")`"#
        )
    });

    list.parse_args_with(Punctuated::<MetaNameValue, Token![,]>::parse_terminated)
        .unwrap_or_else(|_| {
            abort!(
                list.span(),
                r#"expected name value list, e.g. `attrs(id = "foo")`"#
            )
        })
        .into_iter()
        .map(|p| {
            let key = p
                .path
                .require_ident()
                .unwrap_or_else(|_| abort!(p.span(), "attrs key-value must be ident-string"));
            let syn::Expr::Lit(lit) = p.value else {
                abort!(p.value.span(), "values in attrs must be string literals")
            };

            let syn::Lit::Str(str) = lit.lit else {
                abort!(lit.span(), "values in attrs must be string literals")
            };

            (key.to_string(), str.value())
        })
        .collect()
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

fn parse_format(attr: &Attribute) -> LitStr {
    let list = attr
        .meta
        .require_list()
        .unwrap_or_else(|_| abort!(attr.meta.span(), r#"expected list, e.g. htmx_get("/path")"#));
    list.parse_args::<LitStr>()
        .unwrap_or_else(|e| abort!(attr.meta.span(), format!("{e}")))
}

fn parse_htmx_request(attr: &Attribute) -> HtmxRequest {
    let (list, ident) = extract_list_and_args(attr);

    let method = match ident.to_string().as_str() {
        HX_DELETE_ATTR => HtmxMethod::Delete,
        HX_GET_ATTR => HtmxMethod::Get,
        HX_POST_ATTR => HtmxMethod::Post,
        HX_PUT_ATTR => HtmxMethod::Put,
        HX_PATCH_ATTR => HtmxMethod::Patch,
        _ => abort!(ident.span(), "unrecognized htmx attr"),
    };

    let params: FormatParams = list
        .parse_args()
        .unwrap_or_else(|e| abort!(list.span(), &format!("{e}")));

    HtmxRequest { method, params }
}

fn parse_dyn_attr(attr: &Attribute) -> DynamicAttr {
    let list = attr
        .meta
        .require_list()
        .unwrap_or_else(|_| abort!(attr.meta.span(), "malformed attribute"));

    list.parse_args_with(|input: ParseStream| {
        let key = input.parse::<LitStr>()?;

        input.parse::<Token![=]>()?;

        let fmt = input.parse::<FormatParams>()?;

        Ok(DynamicAttr {
            key: key.value(),
            params: fmt,
        })
    })
    .unwrap_or_else(|e| abort!(list.span(), &format!("{e}")))
}

fn extract_list_and_args(attr: &Attribute) -> (&MetaList, &Ident) {
    let list = attr
        .meta
        .require_list()
        .unwrap_or_else(|_| abort!(attr.meta.span(), "malformed attribute"));

    let ident = attr
        .path()
        .get_ident()
        .unwrap_or_else(|| abort!(attr.path().span(), "malformed attribute",));

    (list, ident)
}

#[derive(Debug)]
struct DynamicAttr {
    key: String,
    params: FormatParams,
}

#[derive(Debug)]
struct HtmxRequest {
    method: HtmxMethod,
    params: FormatParams,
}

#[derive(Debug)]
enum HtmxMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

/// Parameters for format strings for attributes
/// such as `hx_get("/{}", foo)`. Also used when
/// there are no substitutions.
#[derive(Debug)]
struct FormatParams {
    /// String literal used for formating. Also
    /// could just be a raw string without any substitutions.
    fmt: LitStr,

    /// Optional args for the fmt, i.e. fields on this struct.
    args: Vec<Ident>,
}

impl FormatParams {
    /// Create the tokens for this struct which format a string based
    /// on its args.
    ///
    /// The resulting string (the one created by the tokens) will be:
    ///
    /// ` attribute=format!(self.fmt, self.args)`
    fn to_tokens(&self, attribute: &str) -> TokenStream {
        let fmt = &self.fmt;
        let args = &self.args;

        if args.is_empty() {
            let path = format!(r#" {attribute}="{}""#, fmt.value());

            quote!(
                let request = #path;
            )
        } else {
            let args = args.iter().map(|field| quote!(self.#field));
            let path = format!(r#" {attribute}="{}""#, fmt.value());

            quote!(
                let request = format!(#path, #(#args),*);
            )
        }
    }
}

impl syn::parse::Parse for FormatParams {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let fmt = input.parse::<LitStr>()?;

        if input.is_empty() {
            return Ok(Self { fmt, args: vec![] });
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

        Ok(Self { fmt, args })
    }
}
