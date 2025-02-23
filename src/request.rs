use crate::error::XpressError;
use std::collections::HashMap;
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncReadExt};

#[derive(Debug, Clone)]
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

    pub async fn from_stream<T>(reader: &mut T) -> Result<Self, XpressError>
    where
        T: AsyncBufRead + Unpin,
    {
        let mut request = Request::new();
        let mut lines = reader.lines();

        // Parse request line
        if let Some(line) = lines.next_line().await? {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                request.method = parts[0].to_string();
                let full_path = parts[1].to_string();

                // Parse query parameters
                if let Some((path, query)) = full_path.split_once('?') {
                    request.path = path.to_string();
                    request.query = parse_query_string(query);
                } else {
                    request.path = full_path;
                }
            } else {
                return Err(XpressError::Custom("Invalid request line".to_string()));
            }
        }

        // Parse headers
        while let Some(line) = lines.next_line().await? {
            if line.is_empty() {
                break;
            }

            if let Some((key, value)) = line.split_once(": ") {
                request
                    .headers
                    .insert(key.to_lowercase(), value.trim().to_string());
            }
        }

        // Parse body if Content-Length header is present
        if let Some(length) = request
            .headers
            .get("content-length")
            .and_then(|cl| cl.parse::<usize>().ok())
        {
            let mut body = vec![0; length];
            reader.read_exact(&mut body).await?;
            request.body = String::from_utf8_lossy(&body).to_string();
        }

        Ok(request)
    }
}

fn parse_query_string(query: &str) -> HashMap<String, String> {
    query
        .split('&')
        .filter_map(|pair| {
            let mut parts = pair.split('=');
            Some((
                parts.next()?.to_string(),
                parts.next().unwrap_or_default().to_string(),
            ))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use tokio::io::BufReader;

    #[tokio::test]
    async fn test_request_parsing() {
        let raw_request = b"GET /hello HTTP/1.1\r\nHost: localhost\r\n\r\n";
        let mut reader = BufReader::new(Cursor::new(raw_request));

        let request = Request::from_stream(&mut reader).await.unwrap();

        assert_eq!(request.method, "GET");
        assert_eq!(request.path, "/hello");
        assert_eq!(request.headers.get("host").unwrap(), "localhost");
    }
}
