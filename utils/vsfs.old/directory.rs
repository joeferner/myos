use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

use crate::{
    Addr, BLOCK_SIZE, Error, File, FileNameLen, FileSystem, INode, INodeIndex, MODE_DIRECTORY,
    Mode, Result, Uid, io::ReadWriteSeek,
};

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
}

impl<'a, T: ReadWriteSeek> DirectoryIterator<'a, T> {
    pub(crate) fn new(fs: &'a mut FileSystem<T>, inode_idx: INodeIndex) -> Result<Self> {
        Ok(Self {
            fs,
            inode_idx,
            offset: 0,
        })
    }

    fn read_next(&mut self) -> Result<Option<DirectoryEntry>> {
        let mut dir_entry_buf = [0; BASE_PHYSICAL_DIRECTORY_ENTRY_SIZE];
        let mut file_name_buf = [0; MAX_FILE_NAME_LEN];
        loop {
            let read = self
                .fs
                .read(self.inode_idx, self.offset, &mut dir_entry_buf)?;
            if read != dir_entry_buf.len() {
                return Ok(None);
            }
            self.offset += read as Addr;

            let dir_entry = PhysicalDirectoryEntry::read_from_bytes(&dir_entry_buf)
                .map_err(|_| Error::SizeError)?;
            if dir_entry.inode_idx == 0 {
                self.offset += dir_entry.name_len as Addr;
                continue;
            }

            let file_name = file_name_buf
                .get_mut(0..dir_entry.name_len as usize)
                .ok_or(Error::SizeError)?;
            let read = self.fs.read(self.inode_idx, self.offset, file_name)?;
            if read < dir_entry.name_len as usize {
                return Err(Error::ReadError);
            }

            let entry_inode = self.fs.read_inode(dir_entry.inode_idx)?;
            return Ok(Some(DirectoryEntry::new(
                dir_entry,
                file_name_buf,
                entry_inode,
            )));
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
