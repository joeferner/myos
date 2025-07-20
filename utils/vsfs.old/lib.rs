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
    
    fn write_data_block(
        &mut self,
        data_block_idx: DataBlockIndex,
        block: [u8; BLOCK_SIZE],
    ) -> Result<()> {
        // write data
        let addr = self.layout.calc_data_addr(data_block_idx)?;
        self.file.seek(SeekFrom::Start(addr.0))?;
        self.file.write(&block)?;

        // update bitmap
        let (addr, offset, bit) = self.layout.calc_data_bitmap_addr(data_block_idx)?;
        self.file.seek(SeekFrom::Start(addr.0))?;
        self.file.read(&mut self.block)?;
        self.block[offset] = 1 << bit;
        self.file.seek(SeekFrom::Start(addr.0))?;
        self.file.write(&self.block)?;

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

 






}

