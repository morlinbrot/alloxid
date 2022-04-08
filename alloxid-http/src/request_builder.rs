use tide::{Method, Request};

/// Request builder for convenience when firing requests in tests etc.
pub trait MakeRequestBuilder {
    fn build() -> RequestBuilder;
}

#[derive(Debug, Default)]
pub struct RequestBuilder {
    method: Option<Method>,
    url: Option<String>,
}

impl RequestBuilder {
    pub fn get(mut self) -> Self {
        self.method = Some(Method::Get);
        self
    }

    pub fn post(mut self) -> Self {
        self.method = Some(Method::Post);
        self
    }

    pub fn put(mut self) -> Self {
        self.method = Some(Method::Put);
        self
    }

    pub fn delete(mut self) -> Self {
        self.method = Some(Method::Delete);
        self
    }

    pub fn url(mut self, url: &str) -> Self {
        self.url = Some(url.to_string());
        self
    }
}

impl MakeRequestBuilder for RequestBuilder {
    fn build() -> RequestBuilder {
        RequestBuilder::default()
    }
}
