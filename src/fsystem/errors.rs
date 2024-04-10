use std::error::Error;

#[derive(Debug)]
pub(crate) enum FSErrorType {
    NotFound,
    AlreadyExists,
    InvalidPath,
    IOError,
}

#[derive(Debug)]
pub(crate) struct FileSystemError {
    pub(crate) error_type: FSErrorType,
    pub(crate) message: String,
}

impl FileSystemError {
    pub(crate) fn new(error_type: FSErrorType, message: String) -> Self {
        Self {
            error_type,
            message,
        }
    }
}

impl std::fmt::Display for FileSystemError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.error_type, self.message)
    }
}

impl Error for FileSystemError {}

pub(crate) type Result<T> = std::result::Result<T, FileSystemError>;
