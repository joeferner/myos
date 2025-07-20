#![cfg_attr(all(not(feature = "std"), not(test)), no_std)]
#![allow(clippy::new_without_default)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unimplemented,
    clippy::unreachable,
    clippy::indexing_slicing,
    clippy::cast_possible_truncation
)]

mod error;

use core::fmt::Debug;

pub use error::{FileIoError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FilePos(pub u64);

impl core::ops::Add for FilePos {
    type Output = FilePos;

    fn add(self, rhs: Self) -> Self::Output {
        FilePos(self.0 + rhs.0)
    }
}

impl core::ops::AddAssign for FilePos {
    fn add_assign(&mut self, rhs: Self) {
        self.0 = self.0 + rhs.0
    }
}

impl core::ops::AddAssign<usize> for FilePos {
    fn add_assign(&mut self, rhs: usize) {
        self.0 = self.0 + rhs as u64
    }
}

impl core::ops::AddAssign<u16> for FilePos {
    fn add_assign(&mut self, rhs: u16) {
        self.0 = self.0 + rhs as u64
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SignedFilePos(pub i64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TimeSeconds(pub u64);

impl TimeSeconds {
    #[cfg(feature = "std")]
    pub fn now() -> Self {
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| Error::TimeError)?
            .as_secs();
        TimeSeconds(time)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Mode(pub u16);

impl Mode {
    pub fn directory() -> Self {
        Mode(0o40000)
    }

    pub fn is_directory(&self) -> bool {
        (*self & Mode::directory()) == Mode::directory()
    }
}

impl core::ops::BitOr<Mode> for Mode {
    type Output = Mode;

    fn bitor(self, rhs: Mode) -> Self::Output {
        Mode(self.0 | rhs.0)
    }
}

impl core::ops::BitAnd<Mode> for Mode {
    type Output = Mode;

    fn bitand(self, rhs: Mode) -> Self::Output {
        Mode(self.0 & rhs.0)
    }
}

impl Debug for Mode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("Mode")
            .field(&format_args!("{:o}", self.0))
            .finish()
    }
}
