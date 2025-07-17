#![cfg_attr(all(not(feature = "std"), not(test)), no_std)]
#![allow(clippy::new_without_default)]

pub mod directory;
pub mod file;
pub mod format;
pub mod io;

pub use directory::{Directory, DirectoryEntry, DirectoryIterator};
pub use file::File;
pub use format::{FormatVolumeOptions, format_volume};

use crate::io::ReadWriteSeek;

#[derive(Debug)]
pub struct Error {}

pub type Result<T> = core::result::Result<T, Error>;

pub struct FsOptions {}

impl FsOptions {
    pub fn new() -> Self {
        Self {}
    }
}

pub struct FileSystem<'a, T: ReadWriteSeek> {
    file: &'a T,
}

impl<'a, T: ReadWriteSeek> FileSystem<'a, T> {
    pub fn new(file: &'a T, options: FsOptions) -> Result<Self> {
        Ok(Self { file })
    }

    pub fn root_dir(&self) -> Directory {
        todo!();
    }
}
