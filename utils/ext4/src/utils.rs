use chrono::{DateTime, NaiveDateTime};
use file_io::{FileIoError, Result};

pub(crate) fn u64_from_hi_lo(hi: u32, lo: u32) -> u64 {
    ((hi as u64) << 4) | lo as u64
}

pub(crate) fn u32_from_hi_lo(hi: u16, lo: u16) -> u32 {
    ((hi as u32) << 2) | lo as u32
}

pub(crate) fn hi_low_to_date_time(hi: u32, lo: u32) -> Result<Option<NaiveDateTime>> {
    let ms: i64 = (u64_from_hi_lo(hi, lo) * 1000)
        .try_into()
        .map_err(|_| FileIoError::Other("invalid time"))?;
    if ms == 0 {
        Ok(None)
    } else {
        Ok(Some(
            DateTime::from_timestamp_millis(ms)
                .ok_or_else(|| FileIoError::Other("invalid time"))?
                .naive_utc(),
        ))
    }
}
