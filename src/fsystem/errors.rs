pub(crate) enum FSErrorType {
    NotFound,
    AlreadyExists,
}

pub(crate) struct FileSystemError {
    pub(crate) message: String,
}
