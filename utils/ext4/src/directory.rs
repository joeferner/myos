use file_io::Result;

use crate::{Ext4, source::Ext4Source};

pub struct Directory {}

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
pub struct DirectoryEntry {
}
