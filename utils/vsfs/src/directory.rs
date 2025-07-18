use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

use crate::{
    BLOCK_SIZE, Error, File, FileNameLen, FileSize, FileSystem, INode, INodeId, Result, Uid,
    io::ReadWriteSeek,
};

#[repr(C, packed)]
#[derive(Debug, Clone, IntoBytes, FromBytes, Immutable, KnownLayout)]
pub(crate) struct PhysicalDirectoryEntry {
    pub inode: INodeId,
    pub name_len: FileNameLen,
    // name: [u8; < MAX_FILE_NAME_LEN]
}

pub(crate) const BASE_PHYSICAL_DIRECTORY_ENTRY_SIZE: usize =
    core::mem::size_of::<PhysicalDirectoryEntry>();
pub(crate) const MAX_FILE_NAME_LEN: usize = BLOCK_SIZE - BASE_PHYSICAL_DIRECTORY_ENTRY_SIZE;

impl PhysicalDirectoryEntry {
    pub(crate) fn write(buf: &mut [u8], inode: INodeId, name: &str) -> Result<usize> {
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
    inode_id: INodeId,
    inode: INode,
}

impl Directory {
    pub(crate) fn new(inode_id: INodeId, inode: INode) -> Self {
        Self { inode_id, inode }
    }

    pub fn create_file(&mut self, _file_name: &str) -> Result<File> {
        todo!();
    }

    pub fn iter<'a, T: ReadWriteSeek>(
        &self,
        fs: &'a mut FileSystem<'a, T>,
    ) -> Result<DirectoryIterator<'a, T>> {
        DirectoryIterator::new(fs, self.inode.clone())
    }

    pub fn uid(&self) -> Uid {
        self.inode.uid
    }

    pub fn gid(&self) -> Uid {
        self.inode.gid
    }

    pub fn mode(&self) -> u16 {
        self.inode.mode & 0o777
    }

    pub fn inode_id(&self) -> INodeId {
        self.inode_id
    }
}

pub struct DirectoryIterator<'a, T: ReadWriteSeek> {
    fs: &'a mut FileSystem<'a, T>,
    inode: INode,
    offset: FileSize,
    block_offset: usize,
    block: [u8; BLOCK_SIZE],
}

impl<'a, T: ReadWriteSeek> DirectoryIterator<'a, T> {
    pub(crate) fn new(fs: &'a mut FileSystem<'a, T>, inode: INode) -> Result<Self> {
        let offset = 0;
        let mut block = [0; BLOCK_SIZE];
        fs.read(&inode, offset, &mut block)?;

        Ok(Self {
            fs,
            inode,
            offset,
            block_offset: 0,
            block,
        })
    }

    fn read_next_physical_directory_entry(
        &mut self,
    ) -> Result<Option<(PhysicalDirectoryEntry, [u8; MAX_FILE_NAME_LEN])>> {
        loop {
            if self.offset + self.block_offset as FileSize >= self.inode.size {
                return Ok(None);
            }

            if BLOCK_SIZE - self.block_offset < BASE_PHYSICAL_DIRECTORY_ENTRY_SIZE {
                self.offset += BLOCK_SIZE as FileSize;
                self.block_offset = 0;
                self.fs.read(&self.inode, self.offset, &mut self.block)?;
            }
            let buf = &self
                .block
                .get(self.block_offset..self.block_offset + BASE_PHYSICAL_DIRECTORY_ENTRY_SIZE)
                .ok_or(Error::BlockOutOfRange)?;
            self.block_offset += BASE_PHYSICAL_DIRECTORY_ENTRY_SIZE;

            let entry =
                PhysicalDirectoryEntry::read_from_bytes(buf).map_err(|_| Error::SizeError)?;

            if entry.inode == 0 {
                continue;
            }

            let mut file_name_result = [0; MAX_FILE_NAME_LEN];
            let file_name = self
                .block
                .get(self.block_offset..self.block_offset + entry.name_len as usize)
                .ok_or(Error::BlockOutOfRange)?;
            self.block_offset += entry.name_len as usize;
            file_name_result
                .get_mut(0..file_name.len())
                .ok_or(Error::SizeError)?
                .copy_from_slice(file_name);

            return Ok(Some((entry, file_name_result)));
        }
    }

    fn read_next(&mut self) -> Result<Option<DirectoryEntry>> {
        let entry = self.read_next_physical_directory_entry()?;
        match entry {
            Some(entry) => {
                let entry_inode = self.fs.read_inode(entry.0.inode)?;
                Ok(Some(DirectoryEntry::new(entry.0, entry.1, entry_inode)))
            }
            None => Ok(None),
        }
    }
}

impl<'a, T: ReadWriteSeek> Iterator for DirectoryIterator<'a, T> {
    type Item = Result<DirectoryEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.read_next();
        match result {
            Ok(entry) => match entry {
                Some(entry) => Some(Ok(entry)),
                None => None,
            },
            Err(err) => Some(Err(err)),
        }
    }
}

#[derive(Debug)]
pub struct DirectoryEntry {
    physical_directory_entry: PhysicalDirectoryEntry,
    file_name: [u8; MAX_FILE_NAME_LEN],
    inode: INode,
}

impl DirectoryEntry {
    pub(crate) fn new(
        physical_directory_entry: PhysicalDirectoryEntry,
        file_name: [u8; MAX_FILE_NAME_LEN],
        inode: INode,
    ) -> Self {
        Self {
            physical_directory_entry,
            file_name,
            inode,
        }
    }
}
