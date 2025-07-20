use core::num::TryFromIntError;

#[derive(Debug, Clone, Copy)]
pub enum IoError {
    ReadError,
    Other(&'static str),
}

impl From<TryFromIntError> for IoError {
    fn from(_value: TryFromIntError) -> Self {
        Self::Other("could not convert number")
    }
}

pub type Result<T> = core::result::Result<T, IoError>;
