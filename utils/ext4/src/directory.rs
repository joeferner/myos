use myos_api::filesystem::{FilePos, Result};

use crate::{
    Ext4,
    source::Ext4Source,
    types::{INodeIndex, directory_entry::DirEntry2, inode::INode},
};

pub struct Directory {
    _inode_idx: INodeIndex,
    inode: INode,
}

impl Directory {
    pub(crate) fn new(inode_idx: INodeIndex, inode: INode) -> Self {
        Self {
            _inode_idx: inode_idx,
            inode,
        }
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
        loop {
            if self.offset.0 >= self.size.0 {
                return None;
            }

            let dir_entry = match DirEntry2::read(self.fs, self.inode, self.offset) {
                Ok(dir_entry) => dir_entry,
                Err(err) => {
                    return Some(Err(err));
                }
            };
            self.offset += dir_entry.record_length;

            if !dir_entry.inode.is_valid() {
                continue;
            }

            return Some(Ok(DirectoryEntry::new(dir_entry)));
        }
    }
}

#[derive(Debug)]
pub struct DirectoryEntry {
    dir_entry: DirEntry2,
}

impl DirectoryEntry {
    fn new(dir_entry: DirEntry2) -> Self {
        Self { dir_entry }
    }

    pub fn name(&self) -> &str {
        self.dir_entry.name()
    }
}
