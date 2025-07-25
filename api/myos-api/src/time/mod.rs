pub enum TimeError {
    ToEpochError,
}

pub type Result<T> = core::result::Result<T, TimeError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TimeSeconds(pub u64);

impl TimeSeconds {
    #[cfg(feature = "std")]
    pub fn now() -> Result<Self> {
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| TimeError::ToEpochError)?
            .as_secs();
        Ok(TimeSeconds(time))
    }
}
