#![cfg_attr(all(not(feature = "std"), not(test)), no_std)]
#![allow(clippy::new_without_default)]

pub mod directory;
pub mod error;
pub mod file;
pub mod format;
pub mod io;
mod utils;

pub use directory::{Directory, DirectoryIterator};
pub use error::{Error, Result};
pub use file::File;
pub use format::{FormatVolumeOptions, format_volume};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

use crate::{
    io::{ReadWriteSeek, SeekFrom},
    utils::div_ceil,
};

pub struct FsOptions {}

impl FsOptions {
    pub fn new() -> Self {
        Self {}
    }
}

pub const MAGIC: [u8; 4] = *b"vsfs";
pub const BLOCK_SIZE: usize = 4 * 1024;
pub const MODE_DIRECTORY: u16 = 0o40000;
pub(crate) const INODE_SIZE: usize = core::mem::size_of::<INode>();
pub(crate) const INODES_PER_BLOCK: u32 = (BLOCK_SIZE / INODE_SIZE) as u32;
pub(crate) const ROOT_UID: u32 = 0;
pub(crate) const IMMEDIATE_BLOCK_COUNT: usize = 12;
pub(crate) const ROOT_INODE_ID: u32 = 2;

#[repr(C, packed)]
#[derive(Debug, Clone, IntoBytes, FromBytes, Immutable, KnownLayout)]
pub(crate) struct INode {
    uid: u32,
    gid: u32,
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
    blocks: [u32; IMMEDIATE_BLOCK_COUNT],
    /// if not 0, indicates an index into the block table where you will find more block addresses
    indirect_block: u32,
}

impl INode {
    pub(crate) fn new(mode: u16, time: u32) -> Self {
        Self {
            uid: ROOT_UID,
            gid: ROOT_UID,
            mode,
            size: 0,
            time,
            ctime: time,
            mtime: time,
            blocks: [0; IMMEDIATE_BLOCK_COUNT],
            indirect_block: 0,
        }
    }
}

#[repr(C, packed)]
#[derive(Debug, Clone, IntoBytes, FromBytes, Immutable, KnownLayout)]
pub struct SuperBlock {
    pub magic: [u8; 4],
    pub inode_count: u32,
    pub data_block_count: u32,
}

pub(crate) struct Layout {
    pub inode_bitmap_block_count: u32,
    pub data_bitmap_block_count: u32,
    pub inode_block_count: u32,
    pub inode_bitmap_offset: u64,
    pub data_bitmap_offset: u64,
    pub inode_offset: u64,
    pub data_offset: u64,
}

impl Layout {
    pub(crate) fn new(inode_count: u32, data_block_count: u32) -> Self {
        let inode_bitmap_block_count = div_ceil(div_ceil(inode_count, 8), BLOCK_SIZE as u32);
        let data_bitmap_block_count = div_ceil(div_ceil(data_block_count, 8), BLOCK_SIZE as u32);
        let inode_block_count = div_ceil(inode_count, INODES_PER_BLOCK as u32);

        let inode_bitmap_offset = BLOCK_SIZE as u64;
        let data_bitmap_offset =
            inode_bitmap_offset + (inode_bitmap_block_count as u64 * BLOCK_SIZE as u64);
        let inode_offset =
            data_bitmap_offset + (data_bitmap_block_count as u64 * BLOCK_SIZE as u64);
        let data_offset = inode_offset + (inode_block_count as u64 * BLOCK_SIZE as u64);

        Self {
            inode_bitmap_block_count,
            data_bitmap_block_count,
            inode_block_count,
            inode_bitmap_offset,
            data_bitmap_offset,
            inode_offset,
            data_offset,
        }
    }

    /// returns the address of the block containing the inode along with the offset
    /// within the block where to find the inode data
    pub(crate) fn calc_inode_block_addr(&self, inode: u32) -> Result<(u64, usize)> {
        let blocks_to_skip = inode / INODES_PER_BLOCK;
        let inode_count = inode - blocks_to_skip;
        let block_addr = self.inode_offset + (blocks_to_skip as u64 * BLOCK_SIZE as u64);
        let block_offset = inode_count as usize * INODE_SIZE;

        Ok((block_addr, block_offset))
    }
}

pub struct FileSystem<'a, T: ReadWriteSeek> {
    _file: &'a T,
    layout: Layout,
    root_inode: INode,
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

        let layout = Layout::new(super_block.inode_count, super_block.data_block_count);

        let root_inode = FileSystem::read_inode(file, ROOT_INODE_ID, &layout)?;

        Ok(Self {
            _file: file,
            layout,
            root_inode,
        })
    }

    pub fn root_dir(&self) -> Directory {
        Directory::new(self.root_inode.clone())
    }

    fn read_inode(file: &'a mut T, inode: u32, layout: &Layout) -> Result<INode> {
        let mut block = [0; BLOCK_SIZE];
        let (block_addr, inode_offset) = layout.calc_inode_block_addr(inode)?;
        file.seek(SeekFrom::Start(block_addr))?;
        if file.read(&mut block)? != BLOCK_SIZE {
            return Err(Error::SizeError);
        }
        let (inode, _) =
            INode::read_from_prefix(&block[inode_offset..]).map_err(|_| Error::SizeError)?;
        Ok(inode)
    }
}

#[cfg(test)]
mod tests {
    use crate::io::Cursor;

    use super::*;

    #[test]
    fn test_root_dir() {
        let mut data = [0; 100 * BLOCK_SIZE];
        let mut cursor = Cursor::new(&mut data);
        let mut options = FormatVolumeOptions::new(10, 10);
        options.time = 123;
        format_volume(&mut cursor, options).unwrap();

        let fs = FileSystem::new(&mut cursor, FsOptions::new()).unwrap();

        let root = fs.root_dir();
        assert_eq!(ROOT_UID, root.uid());
        assert_eq!(ROOT_UID, root.gid());
        assert_eq!(0o755, root.mode());
        assert_eq!(2, root.iter().collect::<Vec<_>>().len());
    }
}
