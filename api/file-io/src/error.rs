use io::IoError;

#[derive(Debug, Clone, Copy)]
pub enum FileIoError {
    IoError(IoError),
    FilenameTooLong,
    BufferTooSmall,
    FileAlreadyExists,
    Other(&'static str),
}

impl From<IoError> for FileIoError {
    fn from(err: IoError) -> Self {
        FileIoError::IoError(err)
    }
}

pub type Result<T> = core::result::Result<T, FileIoError>;
