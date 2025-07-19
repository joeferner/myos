#![cfg_attr(all(not(feature = "std"), not(test)), no_std)]
#![allow(clippy::new_without_default)]

mod directory;
mod error;
mod file;
mod format;
pub mod io;
mod layout;

pub use directory::{CreateFileOptions, Directory, DirectoryIterator};
pub use error::{Error, Result};
pub use file::File;
pub use format::{FormatVolumeOptions, format_volume};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

use crate::{
    io::{ReadWriteSeek, SeekFrom},
    layout::Layout,
};

pub struct FsOptions {
    pub(crate) read_root_inode: bool,
}

impl FsOptions {
    pub fn new() -> Self {
        Self {
            read_root_inode: true,
        }
    }
}

pub const MAGIC: [u8; 4] = *b"vsfs";
pub const BLOCK_SIZE: usize = 4 * 1024;
pub const MODE_DIRECTORY: u16 = 0o40000;
pub(crate) type INodeIndex = u32;
pub(crate) type BlockIndex = u32;
pub(crate) type Uid = u32;
pub(crate) type Time = u64;
pub(crate) type Mode = u16;
pub(crate) type Addr = u64;
pub(crate) type FileSize = u64;
pub(crate) type SignedFileSize = i64;
pub(crate) type FileNameLen = u16;
pub(crate) const INODE_SIZE: usize = core::mem::size_of::<INode>();
pub(crate) const INODES_PER_BLOCK: BlockIndex = (BLOCK_SIZE / INODE_SIZE) as BlockIndex;
pub const ROOT_UID: Uid = 0;
pub(crate) const IMMEDIATE_BLOCK_COUNT: usize = 12;
pub(crate) const ROOT_INODE_IDX: INodeIndex = 2;

#[repr(C, packed)]
#[derive(Debug, Clone, IntoBytes, FromBytes, Immutable, KnownLayout)]
pub(crate) struct INode {
    uid: Uid,
    gid: Uid,
    mode: Mode,
    /// size of the file
    size: FileSize,
    /// what time was this file last accessed?
    time: Time,
    /// what time was this file created?
    ctime: Time,
    /// what time was this file last modified?
    mtime: Time,
    /// index into the blocks where the first x blocks of data can be found, 0 indicates unused block
    blocks: [BlockIndex; IMMEDIATE_BLOCK_COUNT],
    /// if not 0, indicates an index into the block table where you will find more block addresses
    indirect_block: BlockIndex,
}

impl INode {
    pub(crate) fn new(mode: u16, time: Time) -> Self {
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

pub struct FileSystem<T: ReadWriteSeek> {
    file: T,
    layout: Layout,
    root_inode: INode,
    block: [u8; BLOCK_SIZE],
}

impl<T: ReadWriteSeek> FileSystem<T> {
    pub fn new(mut file: T, options: FsOptions) -> Result<Self> {
        let mut block = [0; BLOCK_SIZE];
        file.seek(io::SeekFrom::Start(0))?;
        file.read(&mut block)?;
        let (super_block, _) =
            SuperBlock::read_from_prefix(&block).map_err(|_| Error::SuperBlockError)?;
        if super_block.magic != MAGIC {
            return Err(Error::SuperBlockError);
        }

        let layout = Layout::new(super_block.inode_count, super_block.data_block_count);

        let mut fs = Self {
            file,
            layout,
            root_inode: INode::new(0o755, 0),
            block: [0; BLOCK_SIZE],
        };

        if options.read_root_inode {
            fs.root_inode = fs.read_inode(ROOT_INODE_IDX)?
        };

        Ok(fs)
    }

    pub fn size(&self) -> Addr {
        self.layout.size()
    }

    pub fn root_dir(&self) -> Directory {
        Directory::new(ROOT_INODE_IDX, self.root_inode.clone())
    }

    fn read_inode(&mut self, inode_idx: INodeIndex) -> Result<INode> {
        let (block_addr, inode_offset) = self.layout.calc_inode_block_addr(inode_idx)?;
        self.file.seek(SeekFrom::Start(block_addr as FileSize))?;
        if self.file.read(&mut self.block)? != BLOCK_SIZE {
            return Err(Error::SizeError);
        }
        let buf = self
            .block
            .get(inode_offset..inode_offset + INODE_SIZE)
            .ok_or(Error::SizeError)?;
        let inode = INode::read_from_bytes(buf).map_err(|_| Error::SizeError)?;
        Ok(inode)
    }

    pub(crate) fn read(
        &mut self,
        inode: &INode,
        offset: FileSize,
        block: &mut [u8; BLOCK_SIZE],
    ) -> Result<()> {
        let addr = self.calc_data_block_addr(inode, offset)?;
        self.file.seek(SeekFrom::Start(addr as FileSize))?;
        self.file.read(block)?;
        Ok(())
    }

    fn calc_data_block_addr(&self, inode: &INode, offset: FileSize) -> Result<Addr> {
        if !(offset as Addr).is_multiple_of(BLOCK_SIZE as Addr) {
            return Err(Error::InvalidOffset);
        }
        let block_idx = (offset as Addr / BLOCK_SIZE as Addr) as BlockIndex;

        if block_idx < IMMEDIATE_BLOCK_COUNT as BlockIndex {
            let data_block_idx = inode.blocks[block_idx as usize];
            return self.layout.calc_data_addr(data_block_idx);
        }

        todo!();
    }

    pub(crate) fn write_inode(&mut self, inode_idx: INodeIndex, inode: INode) -> Result<()> {
        // write inode
        let (addr, offset) = self.layout.calc_inode_block_addr(inode_idx)?;
        self.file.seek(SeekFrom::Start(addr as FileSize))?;
        self.file.read(&mut self.block)?;
        let buf = self
            .block
            .get_mut(offset..offset + INODE_SIZE)
            .ok_or(Error::SizeError)?;
        inode.write_to(buf).map_err(|_| Error::SizeError)?;
        self.file.seek(SeekFrom::Start(addr as FileSize))?;
        self.file.write(&self.block)?;

        // update bitmap
        let (addr, offset, bit) = self.layout.calc_inode_bitmap_addr(inode_idx)?;
        self.file.seek(SeekFrom::Start(addr as FileSize))?;
        self.file.read(&mut self.block)?;
        self.block[offset] = 1 << bit;
        self.file.seek(SeekFrom::Start(addr as FileSize))?;
        self.file.write(&self.block)?;

        if inode_idx == ROOT_INODE_IDX {
            self.root_inode = inode;
        }

        Ok(())
    }

    pub(crate) fn read_data_block(
        &mut self,
        data_block_idx: BlockIndex,
        block: &mut [u8; BLOCK_SIZE],
    ) -> Result<()> {
        let addr = self.layout.calc_data_addr(data_block_idx)?;
        self.file.seek(SeekFrom::Start(addr as FileSize))?;
        self.file.read(block)?;
        Ok(())
    }

    pub(crate) fn write_data_block(
        &mut self,
        data_block_idx: BlockIndex,
        block: [u8; BLOCK_SIZE],
    ) -> Result<()> {
        // write data
        let addr = self.layout.calc_data_addr(data_block_idx)?;
        self.file.seek(SeekFrom::Start(addr as FileSize))?;
        self.file.write(&block)?;

        // update bitmap
        let (addr, offset, bit) = self.layout.calc_data_bitmap_addr(data_block_idx)?;
        self.file.seek(SeekFrom::Start(addr as FileSize))?;
        self.file.read(&mut self.block)?;
        self.block[offset] = 1 << bit;
        self.file.seek(SeekFrom::Start(addr as FileSize))?;
        self.file.write(&self.block)?;

        Ok(())
    }

    pub(crate) fn next_free_inode_id(&mut self) -> Result<INodeIndex> {
        todo!();
    }

    pub(crate) fn append(&mut self, inode: &INode, buf: &[u8]) -> Result<()> {
        todo!();
    }
}

#[cfg(test)]
mod tests {
    use crate::io::Cursor;

    use super::*;

    #[test]
    fn test_root_dir() {
        let mut data = [0; 100 * BLOCK_SIZE];
        let cursor = Cursor::new(&mut data);
        let mut options = FormatVolumeOptions::new(10, 10);
        options.time = 123;
        let mut fs = format_volume(cursor, options).unwrap();

        let root = fs.root_dir();
        assert_eq!(ROOT_UID, root.uid());
        assert_eq!(ROOT_UID, root.gid());
        assert_eq!(0o755, root.mode());

        let mut count = 0;
        for entry in root.iter(&mut fs).unwrap() {
            let entry = entry.unwrap();
            assert!(entry.is_dir());
            let dir = entry.to_dir().unwrap();
            assert_eq!(ROOT_UID, dir.uid());
            assert_eq!(ROOT_UID, dir.gid());
            assert_eq!(0o755, dir.mode());
            assert_eq!(ROOT_INODE_IDX, dir.inode_idx());

            if count == 0 {
                assert_eq!(".", entry.file_name().unwrap());
            } else if count == 1 {
                assert_eq!("..", entry.file_name().unwrap());
            }
            count += 1;
        }
        assert_eq!(2, count);
    }

    #[test]
    fn test_create_file() {
        let mut data = [0; 100 * BLOCK_SIZE];
        let cursor = Cursor::new(&mut data);
        let mut fs = format_volume(cursor, FormatVolumeOptions::new(10, 10)).unwrap();

        let mut root_dir = fs.root_dir();
        let mut file = root_dir
            .create_file(
                &mut fs,
                CreateFileOptions {
                    file_name: "hello.txt",
                    uid: ROOT_UID,
                    gid: ROOT_UID,
                    mode: 0o755,
                    time: 123,
                },
            )
            .unwrap();
        file.write_all(b"Hello World!").unwrap();
    }
}
