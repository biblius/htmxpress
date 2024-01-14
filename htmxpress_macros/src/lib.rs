use syn::DeriveInput;

#[proc_macro_derive(Element, attributes(element, hx_get, hx_post, hx_target))]
pub fn derive_element(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let strct: DeriveInput = syn::parse(input).expect("invalid input");

    quote::quote!().into()
}

struct HtmxElement {
    /// The html element
    el: String,
    hx_target: Option<String>,
    class: Option<String>,
    id: Option<String>,
}
