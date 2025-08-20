use std::fmt;

#[derive(Debug)]
pub enum XpressError {
    IoError(std::io::Error),
    JsonError(serde_json::Error),
    MutexError(String),
    NotFound(String),
    FileNotFound(String),
    ConnectionError(String),
    ParsingError(String),
    Custom(String),
}

impl fmt::Display for XpressError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            XpressError::IoError(err) => write!(f, "I/O Error: {}", err),
            XpressError::JsonError(err) => write!(f, "JSON Error: {}", err),
            XpressError::MutexError(msg) => write!(f, "Mutex Lock Error: {}", msg),
            XpressError::NotFound(path) => write!(f, "Route Not Found: {}", path),
            XpressError::FileNotFound(path) => write!(f, "File Not Found: {}", path),
            XpressError::ConnectionError(msg) => write!(f, "Connection Error: {}", msg),
            XpressError::ParsingError(msg) => write!(f, "Parsing Error: {}", msg),
            XpressError::Custom(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl From<std::io::Error> for XpressError {
    fn from(err: std::io::Error) -> Self {
        XpressError::IoError(err)
    }
}

impl From<serde_json::Error> for XpressError {
    fn from(err: serde_json::Error) -> Self {
        XpressError::JsonError(err)
    }
}

impl XpressError {
    pub fn status_code(&self) -> u16 {
        match self {
            XpressError::NotFound(_) | XpressError::FileNotFound(_) => 404,
            XpressError::ParsingError(_) => 400,
            XpressError::ConnectionError(_) => 502,
            XpressError::MutexError(_) => 500,
            XpressError::IoError(_) | XpressError::JsonError(_) | XpressError::Custom(_) => 500,
        }
    }
}
