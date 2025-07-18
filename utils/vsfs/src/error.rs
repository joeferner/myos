#[derive(Debug)]
pub enum Error {
    #[cfg(feature = "std")]
    StdIoError(std::io::Error),
    SizeError,
    SuperBlockError,
    INodeIndexOutOfRange,
    BlockOutOfRange,
    InvalidOffset,
    Utf8Error,
}

pub type Result<T> = core::result::Result<T, Error>;
