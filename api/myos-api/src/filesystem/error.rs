use nostdio::NoStdIoError;

#[derive(Debug)]
pub enum FileIoError {
    IoError(NoStdIoError),
    FilenameTooLong,
    BufferTooSmall,
    FileAlreadyExists,
    OutOfDiskSpaceError,
    Other(&'static str),
}

impl From<NoStdIoError> for FileIoError {
    fn from(err: NoStdIoError) -> Self {
        FileIoError::IoError(err)
    }
}

pub type Result<T> = core::result::Result<T, FileIoError>;
