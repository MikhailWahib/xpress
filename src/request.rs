use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Read},
    net::TcpStream,
};

use crate::XpressError;

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

    pub fn from(buf_reader: &mut BufReader<&TcpStream>) -> Self {
        let mut request = Request::new();
        let mut lines = buf_reader.lines();

        // Parse req line
        if let Some(Ok(request_line)) = lines.next() {
            let parts: Vec<&str> = request_line.split_whitespace().collect();
            if parts.len() == 3 {
                request.method = parts[0].to_string();
                request.path = parts[1].to_string();

                // parse query params
                if let Some((path, query)) = request.path.clone().split_once('?') {
                    request.path = path.to_string();
                    request.query = query
                        .split('&')
                        .filter_map(|pair| {
                            let mut kv = pair.split('=');
                            Some((
                                kv.next()?.to_string(),
                                kv.next().unwrap_or_default().to_string(),
                            ))
                        })
                        .collect();
                }
            }
        }

        // Parse headers
        let mut headers = HashMap::new();
        for line in lines.by_ref() {
            let line = match line {
                Ok(line) => line,
                Err(_) => break,
            };

            if line.is_empty() {
                break;
            }

            if let Some((key, value)) = line.split_once(": ") {
                headers.insert(key.to_string(), value.to_string());
            }
        }
        request.headers = headers;

        // Parse body if Content-Length header is present
        if let Some(content_length) = request
            .headers
            .get("Content-Length")
            .and_then(|cl| cl.parse::<usize>().ok())
        {
            let mut body = vec![0; content_length];
            if buf_reader.read_exact(&mut body).is_ok() {
                request.body = String::from_utf8_lossy(&body).to_string();
            }
        }

        request
    }

    pub fn from_json<T: serde::de::DeserializeOwned>(&self) -> Result<T, XpressError> {
        if self.body.is_empty() {
            return Err(XpressError::Custom("Empty request body".to_string()));
        }

        match serde_json::from_str::<T>(&self.body) {
            Ok(user) => Ok(user),
            Err(e) => Err(XpressError::JsonError(e)),
        }
    }
}
