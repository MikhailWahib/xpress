use std::collections::HashMap;

#[derive(Debug)]
pub struct Request {
    pub path: String,
    pub method: String,
    pub headers: HashMap<String, String>,
    pub params: HashMap<String, String>,
    pub query: HashMap<String, String>,
    pub body: String,
}

impl Request {
    pub fn new() -> Self {
        Self {
            path: String::new(),
            method: String::new(),
            headers: HashMap::new(),
            params: HashMap::new(),
            query: HashMap::new(),
            body: String::new(),
        }
    }

    pub fn from(buffer: &str) -> Self {
        let mut request = Self::new();

        let request_line = buffer.split('\n').next().unwrap();
        let request_lines = request_line.split(' ').collect::<Vec<&str>>();

        request.path = request_lines[1].to_string();
        request.method = request_lines[0].to_string();

        request
    }
}
