use std::collections::HashMap;

#[derive(Clone)]
pub struct Response {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl Response {
    pub fn new() -> Self {
        let mut headers = HashMap::new();
        headers.insert("Server".to_string(), "Xpress/1.0".to_string());

        Self {
            status: 200,
            headers,
            body: Vec::new(),
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
        if !self.headers.contains_key("Content-Type") {
            self.headers.insert(
                "Content-Type".to_string(),
                "text/plain; charset=utf-8".to_string(),
            );
        }

        self.body = content.as_bytes().to_vec();
        self
    }

    pub fn html(mut self, content: &str) -> Self {
        self.headers.insert(
            "Content-Type".to_string(),
            "text/html; charset=utf-8".to_string(),
        );
        self.body = content.as_bytes().to_vec();
        self
    }

    pub fn json<T: serde::Serialize>(
        mut self,
        value: &T,
    ) -> Result<Self, crate::error::XpressError> {
        self.headers
            .insert("Content-Type".to_string(), "application/json".to_string());

        let json_bytes = serde_json::to_vec(value).map_err(crate::error::XpressError::JsonError)?;

        self.body = json_bytes;
        Ok(self)
    }

    // Encode the response as bytes to send over the wire
    pub fn to_bytes(&self) -> Vec<u8> {
        let status_text = match self.status {
            100 => "Continue",
            200 => "OK",
            201 => "Created",
            204 => "No Content",
            400 => "Bad Request",
            401 => "Unauthorized",
            403 => "Forbidden",
            404 => "Not Found",
            405 => "Method Not Allowed",
            500 => "Internal Server Error",
            _ => "Unknown",
        };

        // Start with status line
        let mut response = format!("HTTP/1.1 {} {}\r\n", self.status, status_text).into_bytes();

        // Add Content-Length header
        let mut headers = self.headers.clone();
        headers.insert("Content-Length".to_string(), self.body.len().to_string());

        // Add headers
        for (key, value) in &headers {
            let header_line = format!("{}: {}\r\n", key, value);
            response.extend_from_slice(header_line.as_bytes());
        }

        // Add empty line to separate headers from body
        response.extend_from_slice(b"\r\n");

        // Add body
        response.extend_from_slice(&self.body);

        response
    }
}

impl Default for Response {
    fn default() -> Self {
        Self::new()
    }
}
