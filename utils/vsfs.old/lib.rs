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

pub(crate) type Addr = u64;
pub(crate) type SignedAddr = i64;

impl<T: ReadWriteSeek> FileSystem<T> {
    

    pub fn size(&self) -> Addr {
        self.layout.size()
    }

    pub fn root_dir(&self) -> Directory {
        Directory::new(ROOT_INODE_IDX, self.root_inode.clone())
    }

    pub(crate) fn create_inode(&mut self, inode: INode) -> Result<INodeIndex> {
        let mut inode_idx: Option<INodeIndex> = None;
        self.file
            .seek(SeekFrom::Start(self.layout.inode_bitmap_offset))?;
        let mut byte_offset = 0;
        let mut bit_offset = 0;
        for i in 0..self.layout.inode_count {
            if i.is_multiple_of(INODES_PER_BLOCK) {
                self.file.read(&mut self.block)?;
                byte_offset = 0;
                bit_offset = 0;
            }
            let byte = self.block[byte_offset];
            let bit = (byte >> bit_offset) & 1;
            if bit == 0 {
                inode_idx = Some(i);
            }
            bit_offset += 1;
            if bit_offset == 8 {
                bit_offset = 0;
                byte_offset += 1;
            }
        }

        if let Some(inode_idx) = inode_idx {
            self.write_inode(inode_idx, inode)?;
            Ok(inode_idx)
        } else {
            Err(Error::OutOfINodes)
        }
    }

    /// Reads an inode.
    ///
    /// This function will validate that the given index has data.
    fn read_inode(&mut self, inode_idx: INodeIndex) -> Result<INode> {
        if !self.is_inode_idx_readable(inode_idx)? {
            return Err(Error::INodeIndexEmpty);
        }

        let (block_addr, inode_offset) = self.layout.calc_inode_block_addr(inode_idx)?;
        self.file.seek(SeekFrom::Start(block_addr as Addr))?;
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

    /// Checks the inode bitmap to see if the given inode has data
    fn is_inode_idx_readable(&mut self, inode_idx: INodeIndex) -> Result<bool> {
        let (addr, offset, bit) = self.layout.calc_inode_bitmap_addr(inode_idx)?;
        self.file.seek(SeekFrom::Start(addr as Addr))?;
        self.file.read(&mut self.block)?;
        Ok((self.block[offset] >> bit) == 1)
    }

    /// Writes an inode at the given index.
    ///
    /// This function overwrites any existing inode data and updates the inode
    /// bitmap to indicate that the inode is now filled
    pub(crate) fn write_inode(&mut self, inode_idx: INodeIndex, inode: INode) -> Result<()> {
        // write inode
        let (addr, offset) = self.layout.calc_inode_block_addr(inode_idx)?;
        self.file.seek(SeekFrom::Start(addr as Addr))?;
        self.file.read(&mut self.block)?;
        let buf = self
            .block
            .get_mut(offset..offset + INODE_SIZE)
            .ok_or(Error::SizeError)?;
        inode.write_to(buf).map_err(|_| Error::SizeError)?;
        self.file.seek(SeekFrom::Start(addr as Addr))?;
        self.file.write(&self.block)?;

        // update bitmap
        let (addr, offset, bit) = self.layout.calc_inode_bitmap_addr(inode_idx)?;
        self.file.seek(SeekFrom::Start(addr as Addr))?;
        self.file.read(&mut self.block)?;
        self.block[offset] = 1 << bit;
        self.file.seek(SeekFrom::Start(addr as Addr))?;
        self.file.write(&self.block)?;

        if inode_idx == ROOT_INODE_IDX {
            self.root_inode = inode;
        }

        Ok(())
    }

    /// Reads a block from the given inode data. Returns the amount of data read.
    pub(crate) fn read_block(
        &mut self,
        inode_idx: INodeIndex,
        offset: Addr,
        block: &mut [u8; BLOCK_SIZE],
    ) -> Result<usize> {
        let inode = self.read_inode(inode_idx)?;
        if offset > inode.size {
            return Ok(0);
        }
        let data_block_idx = self.calc_data_block_idx(&inode, offset)?;
        let addr = self.layout.calc_data_addr(data_block_idx)?;
        self.file.seek(SeekFrom::Start(addr as Addr))?;
        let read_len = self.file.read(block)?;
        Ok((inode.size - offset).min(read_len as u64) as usize)
    }

    /// Reads data from the given inode data. Returns the amount of data read.
    pub(crate) fn read(
        &mut self,
        inode_idx: INodeIndex,
        offset: Addr,
        buf: &mut [u8],
    ) -> Result<usize> {
        todo!();
    }

    fn calc_data_block_idx(&self, inode: &INode, offset: Addr) -> Result<BlockIndex> {
        if !(offset as Addr).is_multiple_of(BLOCK_SIZE as Addr) {
            return Err(Error::InvalidOffset);
        }
        let block_idx = (offset as Addr / BLOCK_SIZE as Addr) as BlockIndex;

        if block_idx < IMMEDIATE_BLOCK_COUNT as BlockIndex {
            let data_block_idx = inode.blocks[block_idx as usize];
            return Ok(data_block_idx);
        }

        todo!();
    }

    pub(crate) fn write_data_block(
        &mut self,
        data_block_idx: BlockIndex,
        block: [u8; BLOCK_SIZE],
    ) -> Result<()> {
        // write data
        let addr = self.layout.calc_data_addr(data_block_idx)?;
        self.file.seek(SeekFrom::Start(addr as Addr))?;
        self.file.write(&block)?;

        // update bitmap
        let (addr, offset, bit) = self.layout.calc_data_bitmap_addr(data_block_idx)?;
        self.file.seek(SeekFrom::Start(addr as Addr))?;
        self.file.read(&mut self.block)?;
        self.block[offset] = 1 << bit;
        self.file.seek(SeekFrom::Start(addr as Addr))?;
        self.file.write(&self.block)?;

        Ok(())
    }

    pub(crate) fn append(&mut self, inode_idx: INodeIndex, buf: &[u8]) -> Result<()> {
        todo!();
    }
}

