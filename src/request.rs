use std::collections::HashMap;
use std::fmt;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};

#[derive(Clone, Debug)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub http_version: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl Request {
    pub async fn from<R: AsyncReadExt + Unpin>(
        reader: &mut BufReader<R>,
    ) -> Result<Self, crate::error::XpressError> {
        // Read the request line
        let mut request_line = String::new();
        reader.read_line(&mut request_line).await?;

        // Parse the request line
        let parts: Vec<&str> = request_line.split_whitespace().collect();
        if parts.len() < 3 {
            return Err(crate::error::XpressError::ParseError(
                "Invalid request line".to_string(),
            ));
        }

        let method = parts[0].to_string();
        let path = parts[1].to_string();
        let http_version = parts[2].to_string();

        // Parse headers
        let mut headers = HashMap::new();
        loop {
            let mut header_line = String::new();
            let bytes_read = reader.read_line(&mut header_line).await?;

            // Check for end of headers (empty line)
            if bytes_read <= 2 || header_line.trim().is_empty() {
                break;
            }

            // Parse header
            if let Some((key, value)) = header_line.trim_end().split_once(": ") {
                headers.insert(key.to_lowercase(), value.to_string());
            }
        }

        // Read body if Content-Length header is present
        let mut body = Vec::new();
        if let Some(content_length) = headers.get("content-length") {
            if let Ok(length) = content_length.parse::<usize>() {
                let mut buffer = vec![0; length];
                reader.read_exact(&mut buffer).await?;
                body = buffer;
            }
        }

        Ok(Self {
            method,
            path,
            http_version,
            headers,
            body,
        })
    }

    pub fn body_as_string(&self) -> Result<String, crate::error::XpressError> {
        String::from_utf8(self.body.clone())
            .map_err(|e| crate::error::XpressError::ParseError(format!("Invalid UTF-8: {}", e)))
    }

    pub fn json<T: serde::de::DeserializeOwned>(&self) -> Result<T, crate::error::XpressError> {
        serde_json::from_slice(&self.body).map_err(crate::error::XpressError::JsonError)
    }

    pub fn get_header(&self, key: &str) -> Option<&String> {
        self.headers.get(&key.to_lowercase())
    }
}

impl fmt::Display for Request {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.method, self.path, self.http_version)
    }
}
