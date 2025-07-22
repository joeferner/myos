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

use file_io::{FileIoError, Result};

use crate::{
    directory::Directory,
    source::Ext4Source,
    types::{
        INodeIndex,
        block_group_descriptor::{BLOCK_GROUP_DESCRIPTOR_SIZE, BlockGroupDescriptor},
        inode::INode,
        super_block::{SUPER_BLOCK_POS, SUPER_BLOCK_SIZE, SuperBlock},
    },
};

mod directory;
mod source;
mod types;
mod utils;

pub struct Ext4<T: Ext4Source> {
    source: T,
    super_block: SuperBlock,
}

impl<T: Ext4Source> Ext4<T> {
    pub fn new(source: T) -> Result<Self> {
        let super_block = SuperBlock::read(&source)?;

        #[cfg(test)]
        println!("{:?}", super_block);

        let mut file_pos = SUPER_BLOCK_POS + SUPER_BLOCK_SIZE;
        for _ in 0..super_block.block_group_descriptor_count() {
            let bgd = BlockGroupDescriptor::read(&source, &file_pos)?;
            file_pos += BLOCK_GROUP_DESCRIPTOR_SIZE;

            #[cfg(test)]
            println!("{:?}", bgd);
        }

        Ok(Self {
            source,
            super_block,
        })
    }

    pub fn root_dir(&self) -> Result<Directory> {
        let inode = self.read_inode(INodeIndex::root())?;
        if let Some(inode) = inode {
            Ok(Directory::new(INodeIndex::root(), inode))
        } else {
            Err(FileIoError::Other("could not read root inode"))
        }
    }

    pub fn read_inode(&self, inode_idx: INodeIndex) -> Result<Option<INode>> {
        
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
