#![doc = include_str!(concat!("../", std::env!("CARGO_PKG_README")))]

pub use htmxpress_macros::Element;

pub trait HtmxElement {
    fn to_htmx(&self) -> String;

    fn to_htmx_response(&self) -> http::Response<String> {
        let htmx = self.to_htmx();
        let mut response = http::Response::new(htmx);
        *response.status_mut() = http::StatusCode::OK;
        response
            .headers_mut()
            .append("content-type", http::HeaderValue::from_static("text/html"));
        response
    }
}
