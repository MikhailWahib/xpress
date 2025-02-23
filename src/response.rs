use crate::error::XpressError;
use std::collections::HashMap;
use tokio::fs;

#[derive(Debug)]
pub struct Response {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub sent: bool,
}

impl Response {
    pub fn new() -> Self {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "text/plain".to_string());
        headers.insert("Server".to_string(), "Xpress/0.1".to_string());

        Self {
            status: 200,
            headers,
            body: Vec::new(),
            sent: false,
        }
    }

    pub fn status(&mut self, status: u16) -> &mut Self {
        self.status = status;
        self
    }

    pub async fn send(&mut self, body: impl Into<Vec<u8>>) -> Result<(), XpressError> {
        self.body = body.into();
        self.sent = true;
        Ok(())
    }

    pub async fn json<T: serde::Serialize>(&mut self, body: &T) -> Result<(), XpressError> {
        self.headers
            .insert("Content-Type".to_string(), "application/json".to_string());
        self.body = serde_json::to_vec(body)?;
        self.sent = true;
        Ok(())
    }

    pub async fn html(&mut self, path: &str) -> Result<(), XpressError> {
        self.headers
            .insert("Content-Type".to_string(), "text/html".to_string());

        match fs::read(path).await {
            Ok(content) => {
                self.body = content;
                self.sent = true;
                Ok(())
            }
            Err(_) => {
                self.status(404);
                self.send(format!("File not found: {}", path)).await?;
                Err(XpressError::FileNotFound(path.to_string()))
            }
        }
    }

    pub fn set_header(&mut self, key: &str, value: &str) -> &mut Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }
}
