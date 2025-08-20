use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Read},
    net::TcpStream,
};

use crate::{parser::parse_query, XpressError};

#[derive(Debug)]
pub struct Request {
    pub(crate) path: String,
    pub(crate) method: String,
    pub(crate) headers: HashMap<String, String>,
    pub params: HashMap<String, String>,
    pub query: HashMap<String, String>,
    pub(crate) body: String,
}

impl Request {
    pub fn default() -> Self {
        Self {
            path: String::new(),
            method: String::new(),
            headers: HashMap::new(),
            params: HashMap::new(),
            query: HashMap::new(),
            body: String::new(),
        }
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

impl<'a> TryFrom<&'a mut BufReader<&TcpStream>> for Request {
    type Error = XpressError;

    fn try_from(buf_reader: &'a mut BufReader<&TcpStream>) -> Result<Self, Self::Error> {
        let mut request = Request::default();
        let mut lines = buf_reader.lines();

        let request_line = lines
            .next()
            .ok_or_else(|| XpressError::ParsingError("Missing request line".into()))??;

        let mut parts = request_line.split_whitespace();
        request.method = parts.next().unwrap_or("").to_string();
        let raw_path = parts.next().unwrap_or("");
        if request.method.is_empty() || raw_path.is_empty() {
            return Err(XpressError::ParsingError(format!(
                "Malformed request line: {}",
                request_line
            )));
        }

        if let Some((path, query)) = raw_path.split_once('?') {
            request.path = path.to_string();
            request.query = parse_query(query);
        } else {
            request.path = raw_path.to_string();
        }

        for line in lines.by_ref() {
            let line = line?;
            if line.is_empty() {
                break;
            }
            if let Some((key, value)) = line.split_once(": ") {
                request.headers.insert(key.to_string(), value.to_string());
            }
        }

        if let Some(content_length) = request
            .headers
            .get("Content-Length")
            .and_then(|cl| cl.parse::<usize>().ok())
        {
            let mut body = vec![0; content_length];
            buf_reader.read_exact(&mut body)?;
            request.body = String::from_utf8(body)
                .map_err(|_| XpressError::ParsingError("Invalid UTF-8 in body".into()))?;
        }

        Ok(request)
    }
}
