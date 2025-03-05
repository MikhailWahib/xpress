use crate::error::XpressError;
use std::collections::HashMap;

pub struct Response {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl Response {
    pub fn new() -> Self {
        Self {
            status: 200,
            body: Vec::new(),
            headers: HashMap::new(),
        }
    }

    pub fn status(mut self, code: u16) -> Self {
        self.status = code;
        self
    }

    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

    pub fn body(mut self, content: &str) -> Self {
        self.body = content.as_bytes().to_vec();
        self
    }

    pub fn html(mut self, content: &str) -> Self {
        self.headers
            .insert("Content-Type".to_string(), "text/html".to_string());

        self.body = content.as_bytes().to_vec();

        self
    }

    pub fn json<T: serde::Serialize>(mut self, body: &T) -> Result<Self, XpressError> {
        self.headers
            .insert("Content-Type".to_string(), "application/json".to_string());
        self.body = serde_json::to_vec(body).map_err(XpressError::JsonError)?;

        Ok(self)
    }
}
