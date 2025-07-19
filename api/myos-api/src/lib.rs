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

pub struct Uid(pub u32);
pub const ROOT_UID: Uid = Uid(0);
