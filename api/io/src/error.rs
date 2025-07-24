use core::num::TryFromIntError;

#[cfg(feature = "std")]
use zerocopy::KnownLayout;
use zerocopy::SizeError;

#[derive(Debug)]
pub enum IoError {
    #[cfg(feature = "std")]
    StdIoError(std::io::Error),
    #[cfg(not(feature = "std"))]
    ReadError(&'static str),
    #[cfg(feature = "std")]
    ReadError(String),
    WriteError,
    EndOfFile,
    #[cfg(not(feature = "std"))]
    Other(&'static str),
    #[cfg(feature = "std")]
    Other(String),
}

impl IoError {
    #[cfg(not(feature = "std"))]
    pub fn from_zerocopy_err<Src, Dst: ?Sized>(
        message: &'static str,
        _err: SizeError<Src, Dst>,
    ) -> Self {
        IoError::Other(message)
    }

    #[cfg(feature = "std")]
    pub fn from_zerocopy_err<Src, Dst: ?Sized>(
        message: &'static str,
        err: SizeError<Src, Dst>,
    ) -> Self
    where
        Src: core::ops::Deref,
        Dst: KnownLayout,
    {
        IoError::Other(format!("{}: {}", message, err))
    }

    #[cfg(not(feature = "std"))]
    pub fn create_partial_read_error() -> Self {
        IoError::ReadError("partial read error")
    }

    #[cfg(feature = "std")]
    pub fn create_partial_read_error(file_pos: u64, read: usize, expected: usize) -> Self {
        IoError::ReadError(format!(
            "partial read error, expected {expected}, read {read} at file_pos {file_pos}"
        ))
    }
}

impl From<&'static str> for IoError {
    #[cfg(not(feature = "std"))]
    fn from(value: &'static str) -> Self {
        Self::Other(value)
    }

    #[cfg(feature = "std")]
    fn from(value: &'static str) -> Self {
        Self::Other(value.to_string())
    }
}

impl From<TryFromIntError> for IoError {
    #[cfg(not(feature = "std"))]
    fn from(_err: TryFromIntError) -> Self {
        Self::Other("could not convert number")
    }

    #[cfg(feature = "std")]
    fn from(err: TryFromIntError) -> Self {
        Self::Other(format!("could not convert number {}", err))
    }
}

pub type Result<T> = core::result::Result<T, IoError>;
