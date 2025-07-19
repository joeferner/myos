#[derive(Debug)]
pub enum Error {
    #[cfg(feature = "std")]
    StdIoError(std::io::Error),
    #[cfg(feature = "std")]
    TimeError,
    SizeError,
    SuperBlockError,
    INodeIndexOutOfRange,
    BlockOutOfRange,
    InvalidOffset,
    Utf8Error,
    FileExists,
    FileNameTooLong,
    OutOfINodes,
    /// indicates that the given inode index is empty and has not been written
    /// to or has been deleted
    INodeIndexEmpty,
}

pub type Result<T> = core::result::Result<T, Error>;

#[cfg(feature = "std")]
impl From<Error> for std::io::Error {
    fn from(value: Error) -> Self {
        match value {
            crate::Error::StdIoError(err) => err,
            other => std::io::Error::other(format!("{other:?}")),
        }
    }
}
