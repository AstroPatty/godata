use std::error::Error;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum GodataErrorType {
    NotFound,
    AlreadyExists,
    InvalidPath,
    NotPermitted,
    IOError,
    InternalError,
}

impl Into<warp::http::StatusCode> for GodataErrorType {
    fn into(self) -> warp::http::StatusCode {
        match self {
            GodataErrorType::NotFound => warp::http::StatusCode::NOT_FOUND,
            GodataErrorType::AlreadyExists => warp::http::StatusCode::CONFLICT,
            GodataErrorType::InvalidPath => warp::http::StatusCode::BAD_REQUEST,
            GodataErrorType::NotPermitted => warp::http::StatusCode::FORBIDDEN,
            _ => warp::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(Debug)]
pub(crate) struct GodataError {
    pub(crate) error_type: GodataErrorType,
    pub(crate) message: String,
}

impl warp::Reply for GodataError {
    fn into_response(self) -> warp::reply::Response {
        warp::reply::with_status(warp::reply::json(&self.message), self.error_type.into())
            .into_response()
    }
}

impl GodataError {
    pub(crate) fn new(error_type: GodataErrorType, message: String) -> Self {
        Self {
            error_type,
            message,
        }
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

impl From<regex::Error> for GodataError {
    fn from(error: regex::Error) -> Self {
        Self {
            error_type: GodataErrorType::InvalidPath,
            message: error.to_string(),
        }
    }
}

impl Error for GodataError {}

pub(crate) type Result<T> = std::result::Result<T, GodataError>;
