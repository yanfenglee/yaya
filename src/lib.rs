
#[derive(Clone,Debug)]
pub struct Payload {
    pub host: String,
    pub path: String,
    pub method: http::Method,
    pub body: String,
}

pub mod simple_http;