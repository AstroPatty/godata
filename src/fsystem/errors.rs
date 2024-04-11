use std::error::Error;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum GodataErrorType {
    NotFound,
    AlreadyExists,
    InvalidPath,
    NotPermitted,
    IOError,
}

#[derive(Debug)]
pub(crate) struct GodataError {
    pub(crate) error_type: GodataErrorType,
    pub(crate) message: String,
}

impl GodataError {
    pub(crate) fn new(error_type: GodataErrorType, message: String) -> Self {
        Self {
            error_type,
            message,
        }
    }

    pub(crate) fn kind(&self) -> GodataErrorType {
        self.error_type
    }
}

impl std::fmt::Display for GodataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.error_type, self.message)
    }
}

impl From<std::io::Error> for GodataError {
    fn from(error: std::io::Error) -> Self {
        Self {
            error_type: GodataErrorType::IOError,
            message: error.to_string(),
        }
    }
}

impl From<sled::Error> for GodataError {
    fn from(error: sled::Error) -> Self {
        Self {
            error_type: GodataErrorType::IOError,
            message: error.to_string(),
        }
    }
}

impl Error for GodataError {}

pub(crate) type Result<T> = std::result::Result<T, GodataError>;
