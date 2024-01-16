#![doc = include_str!(concat!("../", std::env!("CARGO_PKG_README")))]

pub use htmxpress_macros::Element;

pub trait HtmxElement {
    fn to_htmx(&self) -> String;
}
