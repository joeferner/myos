use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

use crate::{
    BLOCK_SIZE, Error, File, FileNameLen, Addr, FileSystem, INode, INodeIndex, MODE_DIRECTORY,
    Mode, Result, Uid, io::ReadWriteSeek,
};

/// Data stored on the file system for each entry in a directory.
#[repr(C, packed)]
#[derive(Debug, Clone, IntoBytes, FromBytes, Immutable, KnownLayout)]
pub(crate) struct PhysicalDirectoryEntry {
    pub inode_idx: INodeIndex,
    pub name_len: FileNameLen,
    // name: [u8; < MAX_FILE_NAME_LEN]
}

pub(crate) const BASE_PHYSICAL_DIRECTORY_ENTRY_SIZE: usize =
    core::mem::size_of::<PhysicalDirectoryEntry>();
pub(crate) const MAX_FILE_NAME_LEN: usize = BLOCK_SIZE - BASE_PHYSICAL_DIRECTORY_ENTRY_SIZE;

impl PhysicalDirectoryEntry {
    pub(crate) fn write(buf: &mut [u8], inode_idx: INodeIndex, name: &str) -> Result<usize> {
        let name_bytes = name.as_bytes();
        let name_len: u16 = name_bytes.len().try_into().map_err(|_| Error::SizeError)?;

        let entry = PhysicalDirectoryEntry {
            inode_idx,
            name_len,
        };
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
    inode_idx: INodeIndex,
    inode: INode,
}

impl Directory {
    pub(crate) fn new(inode_idx: INodeIndex, inode: INode) -> Self {
        Self { inode_idx, inode }
    }

    pub fn create_file<'a, T: ReadWriteSeek>(
        &mut self,
        fs: &'a mut FileSystem<T>,
        options: CreateFileOptions,
    ) -> Result<File> {
        let file_name_bytes = options.file_name.as_bytes();

        if file_name_bytes.len() >= MAX_FILE_NAME_LEN {
            return Err(Error::FileNameTooLong);
        }

        if self.exists(fs, options.file_name)? {
            return Err(Error::FileExists);
        }

        #[cfg(not(feature = "std"))]
        let time = options.time;

        #[cfg(feature = "std")]
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| Error::TimeError)?
            .as_secs();

        let mut file_inode = INode::new(0o755 | MODE_DIRECTORY, time);
        file_inode.uid = options.uid;
        file_inode.gid = options.gid;
        file_inode.size = 0;
        let file_inode_id = fs.create_inode(file_inode.clone())?;

        let mut buf = [0; BLOCK_SIZE];
        let dir_entry = PhysicalDirectoryEntry {
            inode_idx: file_inode_id,
            name_len: options.file_name.len() as u16,
        };
        dir_entry
            .write_to_prefix(&mut buf)
            .map_err(|_| Error::SizeError)?;
        let file_name_buf = buf
            .get_mut(
                BASE_PHYSICAL_DIRECTORY_ENTRY_SIZE
                    ..BASE_PHYSICAL_DIRECTORY_ENTRY_SIZE + file_name_bytes.len(),
            )
            .ok_or(Error::SizeError)?;
        file_name_buf.copy_from_slice(file_name_bytes);

        fs.append(self.inode_idx, &buf)?;

        Ok(File::new(file_inode_id, file_inode))
    }

    pub fn exists<'a, T: ReadWriteSeek>(
        &mut self,
        fs: &'a mut FileSystem<T>,
        file_name: &str,
    ) -> Result<bool> {
        for entry in self.iter(fs)? {
            let entry = entry?;
            if entry.file_name()? == file_name {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn iter<'a, T: ReadWriteSeek>(
        &self,
        fs: &'a mut FileSystem<T>,
    ) -> Result<DirectoryIterator<'a, T>> {
        DirectoryIterator::new(fs, self.inode_idx)
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

    pub fn inode_idx(&self) -> INodeIndex {
        self.inode_idx
    }
}

pub struct CreateFileOptions<'a> {
    pub uid: Uid,
    pub gid: Uid,
    pub mode: Mode,
    pub file_name: &'a str,
    #[cfg(not(feature = "std"))]
    pub time: crate::Time,
}

pub struct DirectoryIterator<'a, T: ReadWriteSeek> {
    fs: &'a mut FileSystem<T>,
    inode_idx: INodeIndex,
    offset: Addr,
    block_offset: usize,
    block_size: usize,
    block: [u8; BLOCK_SIZE],
}

impl<'a, T: ReadWriteSeek> DirectoryIterator<'a, T> {
    pub(crate) fn new(fs: &'a mut FileSystem<T>, inode_idx: INodeIndex) -> Result<Self> {
        let offset = 0;
        let mut block = [0; BLOCK_SIZE];
        let block_size = fs.read_block(inode_idx, offset, &mut block)?;

        Ok(Self {
            fs,
            inode_idx,
            offset,
            block_offset: 0,
            block_size,
            block,
        })
    }

    fn read_next_physical_directory_entry(
        &mut self,
    ) -> Result<Option<(PhysicalDirectoryEntry, [u8; MAX_FILE_NAME_LEN])>> {
        loop {
            if BLOCK_SIZE - self.block_offset < BASE_PHYSICAL_DIRECTORY_ENTRY_SIZE {
                self.offset += BLOCK_SIZE as Addr;
                self.block_offset = 0;
                self.block_size = self.fs.read_block(self.inode_idx, self.offset, &mut self.block)?;
            }
            let end_offset = self.block_offset + BASE_PHYSICAL_DIRECTORY_ENTRY_SIZE;
            if end_offset > self.block_size {
                return Ok(None);
            }
            let buf = &self
                .block
                .get(self.block_offset..end_offset)
                .ok_or(Error::BlockOutOfRange)?;
            self.block_offset += BASE_PHYSICAL_DIRECTORY_ENTRY_SIZE;

            let entry =
                PhysicalDirectoryEntry::read_from_bytes(buf).map_err(|_| Error::SizeError)?;

            if entry.inode_idx == 0 {
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
                let entry_inode = self.fs.read_inode(entry.0.inode_idx)?;
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
            Ok(entry) => entry.map(Ok),
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

    pub fn is_dir(&self) -> bool {
        (self.inode.mode & MODE_DIRECTORY) == MODE_DIRECTORY
    }

    pub fn to_dir(&self) -> Option<Directory> {
        if self.is_dir() {
            Some(Directory::new(
                self.physical_directory_entry.inode_idx,
                self.inode.clone(),
            ))
        } else {
            None
        }
    }

    pub fn file_name(&self) -> Result<&str> {
        let file_name = self
            .file_name
            .get(0..self.physical_directory_entry.name_len as usize)
            .ok_or(Error::SizeError)?;
        str::from_utf8(file_name).map_err(|_| Error::Utf8Error)
    }
}
