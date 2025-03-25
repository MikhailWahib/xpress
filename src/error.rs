use std::fmt;

#[derive(Debug)]
pub enum XpressError {
    IoError(std::io::Error),
    JsonError(serde_json::Error),
    MutexError(String),
    FileNotFound(String),
    ConnectionError(String),
    Custom(String),
}

impl fmt::Display for XpressError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            XpressError::IoError(err) => write!(f, "I/O Error: {}", err),
            XpressError::JsonError(err) => write!(f, "JSON Error: {}", err),
            XpressError::MutexError(msg) => write!(f, "Mutex Lock Error: {}", msg),
            XpressError::FileNotFound(path) => write!(f, "File Not Found: {}", path),
            XpressError::ConnectionError(msg) => write!(f, "Connection Error: {}", msg),
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
