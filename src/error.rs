use std::error::Error;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum XpressError {
    IoError(io::Error),
    ParseError(String),
    SendError(String),
    JsonError(serde_json::Error),
    FileNotFound(String),
}

impl fmt::Display for XpressError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            XpressError::IoError(err) => write!(f, "IO Error: {}", err),
            XpressError::ParseError(msg) => write!(f, "Parse Error: {}", msg),
            XpressError::SendError(msg) => write!(f, "Send Error: {}", msg),
            XpressError::JsonError(err) => write!(f, "JSON Error: {}", err),
            XpressError::FileNotFound(path) => write!(f, "File not found: {}", path),
        }
    }
}

impl Error for XpressError {}

impl From<io::Error> for XpressError {
    fn from(err: io::Error) -> Self {
        XpressError::IoError(err)
    }
}
