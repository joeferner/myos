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

use file_io::{FileIoError, FilePos, Result};

use crate::{
    directory::Directory,
    source::Ext4Source,
    types::{
        INodeIndex,
        bitmap::Bitmap,
        block_group_descriptor::{BLOCK_GROUP_DESCRIPTOR_SIZE, BlockGroupDescriptor},
        inode::INode,
        super_block::{SUPER_BLOCK_POS, SUPER_BLOCK_SIZE, SuperBlock},
    },
};

mod directory;
mod source;
mod types;
mod utils;

pub const MAX_BLOCK_SIZE: usize = 0x10000;

pub struct Ext4<T: Ext4Source> {
    source: T,
    super_block: SuperBlock,
}

impl<T: Ext4Source> Ext4<T> {
    pub fn new(source: T) -> Result<Self> {
        let super_block = SuperBlock::read(&source)?;

        #[cfg(test)]
        println!("super_block {:?}", super_block);

        let mut file_pos = SUPER_BLOCK_POS + SUPER_BLOCK_SIZE;
        for _ in 0..super_block.block_group_descriptor_count() {
            let bgd = BlockGroupDescriptor::read(&source, &file_pos)?;
            file_pos += BLOCK_GROUP_DESCRIPTOR_SIZE;

            #[cfg(test)]
            println!("bgd {:?}", bgd);
        }

        Ok(Self {
            source,
            super_block,
        })
    }

    pub fn root_dir(&self) -> Result<Directory> {
        let inode = self.read_inode(&INodeIndex::root())?;
        if let Some(inode) = inode {
            Ok(Directory::new(INodeIndex::root(), inode))
        } else {
            Err(FileIoError::Other("could not read root inode"))
        }
    }

    /// returns None if the given inode is not filled/readable
    fn read_inode(&self, inode_idx: &INodeIndex) -> Result<Option<INode>> {
        let bgd = self.read_bgd_for_inode_index(&inode_idx)?;
        let bitmap = Bitmap::read(
            &self.source,
            &bgd.block_bitmap(),
            self.super_block.block_size(),
        )?;
        let relative_inode_idx =
            INodeIndex::new(inode_idx.real_index() % self.super_block.blocks_per_group());
        if !bitmap.is_readable(relative_inode_idx) {
            return Ok(None);
        }

        let inode = INode::read(
            &self.source,
            &(bgd.inode_table()),
            &relative_inode_idx,
            self.super_block.block_size(),
            self.super_block.inode_size(),
        )?;

        #[cfg(test)]
        println!("inode {:?}", inode);

        Ok(Some(inode))
    }

    pub(crate) fn read(&self, inode: &INode, offset: &FilePos, buf: &mut [u8]) -> Result<()> {
        let (data_block_idx, block_count) = inode.get_data_extent(offset, self.super_block.block_size())?;

         #[cfg(test)]
        println!("data_block_idx {data_block_idx:?}, block_count {block_count}");
        // self.source.read(&file_pos, buf)?;

        todo!();
    }

    fn read_bgd_for_inode_index(&self, inode_idx: &INodeIndex) -> Result<BlockGroupDescriptor> {
        let bgd_file_pos = self.super_block.get_bgd_file_pos_for_inode_index(inode_idx);
        BlockGroupDescriptor::read(&self.source, &bgd_file_pos)
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use std::fs::File;

    use crate::source::FileExt4Source;

    use super::*;

    #[test]
    fn test_read() {
        let source = FileExt4Source::new(File::open("test-data/simple.ext4").unwrap());
        let ext4 = Ext4::new(source).unwrap();

        let root = ext4.root_dir().unwrap();
        for entry in root.iter(&ext4).unwrap() {
            println!("{:?}", entry);
        }
    }
}
