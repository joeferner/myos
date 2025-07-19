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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SignedFilePos(pub i64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TimeSeconds(pub u64);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Mode(pub u16);

impl core::ops::BitOr<Mode> for Mode {
    type Output = Mode;

    fn bitor(self, rhs: Mode) -> Self::Output {
        Mode(self.0 | rhs.0)
    }
}

impl Debug for Mode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("Mode")
            .field(&format_args!("{:o}", self.0))
            .finish()
    }
}

pub const MODE_DIRECTORY: Mode = Mode(0o40000);
