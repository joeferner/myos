#[derive(Debug)]
pub enum Error {
    #[cfg(feature = "std")]
    StdIoError(std::io::Error),
    SizeError,
    SuperBlockError,
}

pub type Result<T> = core::result::Result<T, Error>;
