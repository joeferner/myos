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

use file_io::Result;

use crate::{directory::Directory, source::Ext4Source, super_block::SuperBlock};

mod directory;
mod source;
mod super_block;
mod utils;

pub struct Ext4<T: Ext4Source> {
    _source: T,
}

impl<T: Ext4Source> Ext4<T> {
    pub fn new(source: T) -> Result<Self> {
        let super_block = SuperBlock::read(&source)?;

        #[cfg(test)]
        println!("{:?}", super_block);

        Ok(Self { _source: source })
    }

    pub fn root_dir(&self) -> Result<Directory> {
        // let root_inode = self.read_inode(INodeBlockIndex::root())?;
        // Ok(Directory::new(INodeBlockIndex::root(), root_inode))
        todo!();
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
