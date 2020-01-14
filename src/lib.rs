
#[derive(Clone,Debug)]
pub struct Payload {
    pub host: String,
    pub path: String,
    pub method: String,
    pub body: String,
}

pub mod simple_http;