use crate::{File, Result};

pub struct Directory {}

impl Directory {
    pub fn create_file(&mut self, _file_name: &str) -> Result<File> {
        todo!();
    }

    pub fn iter(&self) -> DirectoryIterator {
        todo!();
    }
}

pub struct DirectoryIterator {}

impl Iterator for DirectoryIterator {
    type Item = DirectoryEntry;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

#[derive(Debug)]
pub struct DirectoryEntry {}
