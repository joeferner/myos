use file_io::Result;

use crate::{
    Ext4,
    source::Ext4Source,
    types::{INodeIndex, inode::INode},
};

pub struct Directory {
    inode_idx: INodeIndex,
    inode: INode,
}

impl Directory {
    pub(crate) fn new(inode_idx: INodeIndex, inode: INode) -> Self {
        Self { inode_idx, inode }
    }
}

impl Directory {
    pub fn iter<'a, T: Ext4Source>(&self, fs: &'a Ext4<T>) -> Result<DirectoryIterator<'a, T>> {
        Ok(DirectoryIterator { _fs: fs })
    }
}

pub struct DirectoryIterator<'a, T: Ext4Source> {
    _fs: &'a Ext4<T>,
}

impl<'a, T: Ext4Source> Iterator for DirectoryIterator<'a, T> {
    type Item = Result<DirectoryEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

#[derive(Debug)]
pub struct DirectoryEntry {}
