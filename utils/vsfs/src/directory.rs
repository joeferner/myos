use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

use crate::{Error, File, INode, Result};

#[repr(C, packed)]
#[derive(Debug, Clone, IntoBytes, FromBytes, Immutable, KnownLayout)]
pub(crate) struct PhysicalDirectoryEntry {
    pub inode: u32,
    pub name_len: u16,
}

impl PhysicalDirectoryEntry {
    pub(crate) fn write(buf: &mut [u8], inode: u32, name: &str) -> Result<usize> {
        let name_bytes = name.as_bytes();
        let name_len: u16 = name_bytes.len().try_into().map_err(|_| Error::SizeError)?;

        let entry = PhysicalDirectoryEntry { inode, name_len };
        let entry_bytes = entry.as_bytes();
        let total_len = entry_bytes.len() + name_bytes.len();
        if total_len > buf.len() {
            return Err(Error::SizeError);
        }

        buf[0..entry_bytes.len()].copy_from_slice(entry_bytes);
        buf[entry_bytes.len()..entry_bytes.len() + name_bytes.len()].copy_from_slice(name_bytes);

        Ok(total_len)
    }
}

pub struct Directory {
    inode: INode,
}

impl Directory {
    pub(crate) fn new(inode: INode) -> Self {
        Self { inode }
    }

    pub fn create_file(&mut self, _file_name: &str) -> Result<File> {
        todo!();
    }

    pub fn iter(&self) -> DirectoryIterator {
        todo!();
    }

    pub fn uid(&self) -> u32 {
        self.inode.uid
    }

    pub fn gid(&self) -> u32 {
        self.inode.gid
    }

    pub fn mode(&self) -> u16 {
        self.inode.mode & 0o777
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
