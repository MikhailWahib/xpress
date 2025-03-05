use std::collections::HashMap;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};

#[derive(Clone, Debug)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl Request {
    pub async fn from<R: AsyncReadExt + Unpin>(
        reader: &mut BufReader<R>,
    ) -> Result<Self, crate::error::XpressError> {
        let mut request_line = String::new();
        reader.read_line(&mut request_line).await?;

        let parts: Vec<&str> = request_line.split_whitespace().collect();
        if parts.len() < 3 {
            return Err(crate::error::XpressError::ParseError(
                "Invalid request line".to_string(),
            ));
        }

        let method = parts[0].to_string();
        let path = parts[1].to_string();

        let mut headers = HashMap::new();
        let mut body = Vec::new();

        loop {
            let mut header_line = String::new();
            reader.read_line(&mut header_line).await?;

            // Trim whitespace and check for end of headers
            let header_line = header_line.trim();
            if header_line.is_empty() {
                break;
            }

            if let Some((key, value)) = header_line.split_once(": ") {
                headers.insert(key.to_string(), value.to_string());
            }
        }

        // Read body if Content-Length header is present
        if let Some(content_length) = headers.get("Content-Length") {
            if let Ok(length) = content_length.parse::<usize>() {
                body.resize(length, 0);
                reader.read_exact(&mut body).await?;
            }
        }

        Ok(Self {
            method,
            path,
            headers,
            body: String::from_utf8_lossy(&body).to_string(),
        })
    }
}
