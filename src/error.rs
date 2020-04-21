
use std::io;

#[derive(Debug)]
pub enum JvmsError {
    IoError(io::Error),
    InvalidConfiguration(String),
    SerdeJsonError(serde_json::Error)
}

impl From<io::Error> for JvmsError {

    fn from(error: io::Error) -> Self {
        JvmsError::IoError(error)
    }

}

impl From<serde_json::Error> for JvmsError {

    fn from(error: serde_json::Error) -> Self {
        JvmsError::SerdeJsonError(error)
    }

}

pub type Result<T> = std::result::Result<T, JvmsError>;
