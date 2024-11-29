use std::{collections::HashMap, io::Error};

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

        Self {
            status: 200,
            headers,
            body: Vec::new(),
            sent: false,
        }
    }

    pub fn send(&mut self, body: impl Into<Vec<u8>>) -> Result<(), Error> {
        self.body = body.into();
        self.sent = true;
        Ok(())
    }

    pub fn status(&mut self, status: u16) {
        self.status = status;
    }

    pub fn json<T: serde::Serialize>(&mut self, body: &T) -> Result<(), Error> {
        self.headers
            .insert("Content-Type".to_string(), "application/json".to_string());

        match serde_json::to_vec(body) {
            Ok(serialized) => {
                self.body = serialized;
                self.sent = true;
                Ok(())
            }
            Err(_) => Err(Error::new(
                std::io::ErrorKind::Other,
                "Serialization failed",
            )),
        }
    }

    pub fn html(&mut self, body: impl Into<Vec<u8>>) -> Result<(), Error> {
        self.headers
            .insert("Content-Type".to_string(), "text/html".to_string());

        self.body = body.into();
        self.sent = true;
        Ok(())
    }
}
