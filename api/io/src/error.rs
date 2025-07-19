use core::num::TryFromIntError;

#[derive(Debug, Clone, Copy)]
pub enum IoError {
    Other,
}

impl From<TryFromIntError> for IoError {
    fn from(_value: TryFromIntError) -> Self {
        Self::Other
    }
}

pub type Result<T> = core::result::Result<T, IoError>;
