use file_io::{FilePos, Result};

use crate::{
    Ext4,
    source::Ext4Source,
    types::{INodeIndex, directory_entry::DIR_ENTRY_2_SIZE, inode::INode},
};

pub struct Directory {
    _inode_idx: INodeIndex,
    inode: INode,
}

impl Directory {
    pub(crate) fn new(inode_idx: INodeIndex, inode: INode) -> Self {
        Self { _inode_idx: inode_idx, inode }
    }
}

impl Directory {
    pub fn iter<'a, T: Ext4Source>(&'a self, fs: &'a Ext4<T>) -> Result<DirectoryIterator<'a, T>> {
        Ok(DirectoryIterator {
            fs,
            inode: &self.inode,
            size: self.inode.size(),
            offset: FilePos(0),
        })
    }
}

pub struct DirectoryIterator<'a, T: Ext4Source> {
    fs: &'a Ext4<T>,
    inode: &'a INode,
    size: FilePos,
    offset: FilePos,
}

impl<'a, T: Ext4Source> Iterator for DirectoryIterator<'a, T> {
    type Item = Result<DirectoryEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset.0 >= self.size.0 {
            return None;
        }

        let mut buf = [0; DIR_ENTRY_2_SIZE];
        let read_result = self.fs.read(&self.inode, &self.offset, &mut buf);

        #[cfg(test)]
        println!("read_result {:?} {:?}", read_result, buf);

        self.offset += buf.len();

        todo!();
    }
}

#[derive(Debug)]
pub struct DirectoryEntry {}
