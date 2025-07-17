#![cfg_attr(all(not(feature = "std"), not(test)), no_std)]
#![allow(clippy::new_without_default)]

pub mod directory;
pub mod error;
pub mod file;
pub mod format;
pub mod io;
mod utils;

pub use directory::{Directory, DirectoryEntry, DirectoryIterator};
pub use error::{Error, Result};
pub use file::File;
pub use format::{FormatVolumeOptions, format_volume};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

use crate::io::ReadWriteSeek;

pub struct FsOptions {}

impl FsOptions {
    pub fn new() -> Self {
        Self {}
    }
}

pub const MAGIC: [u8; 4] = *b"vsfs";
pub const BLOCK_SIZE: usize = 4 * 1024;
const INODE_SIZE: usize = core::mem::size_of::<INode>();
const INODES_PER_BLOCK: usize = BLOCK_SIZE / INODE_SIZE;

#[repr(C, packed)]
#[derive(Debug, Clone, IntoBytes, FromBytes, Immutable, KnownLayout)]
pub struct INode {
    uid: u16,
    gid: u16,
    mode: u16,
    /// size of the file
    size: u32,
    /// what time was this file last accessed?
    time: u32,
    /// what time was this file created?
    ctime: u32,
    /// what time was this file last modified?
    mtime: u32,
    /// index into the blocks where the first x blocks of data can be found, 0 indicates unused block
    blocks: [u32; 12],
    /// if not 0, indicates an index into the block table where you will find more block addresses
    indirect_block: u32,
}

#[repr(C, packed)]
#[derive(Debug, Clone, IntoBytes, FromBytes, Immutable, KnownLayout)]
pub struct SuperBlock {
    pub magic: [u8; 4],
    pub inode_count: u32,
    pub data_block_count: u32,
}

pub struct FileSystem<'a, T: ReadWriteSeek> {
    _file: &'a T,
}

impl<'a, T: ReadWriteSeek> FileSystem<'a, T> {
    pub fn new(file: &'a mut T, _options: FsOptions) -> Result<Self> {
        let mut block = [0; BLOCK_SIZE];
        file.seek(io::SeekFrom::Start(0))?;
        file.read(&mut block)?;
        let (super_block, _) =
            SuperBlock::read_from_prefix(&block).map_err(|_| Error::SuperBlockError)?;
        if super_block.magic != MAGIC {
            return Err(Error::SuperBlockError);
        }

        Ok(Self { _file: file })
    }

    pub fn root_dir(&self) -> Directory {
        todo!();
    }
}
