
#[derive(Clone,Debug)]
pub struct Payload {
    pub host: String,
    pub path: String,
    pub method: http::Method,
    pub body: String,
}

#[derive(PartialEq, Eq)]
pub enum ProcStatus {
    RUNNING = 0,
    TERMINATE
}

pub mod simple_http;