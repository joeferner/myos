#[cfg(feature = "std")]
use file_io::TimeSeconds;
use file_io::{FileIoError, FilePos, Mode, Result};
use io::{IoError, ReadWriteSeek};
use myos_api::Uid;
use zerocopy::FromBytes;

use crate::{
    INodeBlockIndex, ReadWritePos, Vsfs,
    file::File,
    inode::INode,
    physical::{
        BASE_PHYSICAL_DIRECTORY_ENTRY_SIZE, BLOCK_SIZE, MAX_FILE_NAME_LEN, PhysicalDirectoryEntry,
    },
};

pub struct Directory {
    inode_idx: INodeBlockIndex,
    inode: INode,
}

impl Directory {
    pub(crate) fn new(inode_idx: INodeBlockIndex, inode: INode) -> Self {
        Self { inode_idx, inode }
    }

    pub fn uid(&self) -> Uid {
        self.inode.uid
    }

    pub fn gid(&self) -> Uid {
        self.inode.gid
    }

    pub fn mode(&self) -> Mode {
        self.inode.mode & Mode(0o777)
    }

    pub(crate) fn inode_idx(&self) -> INodeBlockIndex {
        self.inode_idx
    }

    pub fn create_file<'a, T: ReadWriteSeek>(
        &mut self,
        fs: &'a mut Vsfs<T>,
        options: CreateFileOptions,
    ) -> Result<File> {
        let file_name_bytes = options.file_name.as_bytes();

        if file_name_bytes.len() >= MAX_FILE_NAME_LEN {
            return Err(FileIoError::FilenameTooLong);
        }

        if self.exists(fs, options.file_name)? {
            return Err(FileIoError::FileAlreadyExists);
        }

        #[cfg(not(feature = "std"))]
        let time = options.time;

        #[cfg(feature = "std")]
        let time = TimeSeconds::now();

        let mut file_inode = INode::new(Mode(0o755) | Mode::directory(), time);
        file_inode.uid = options.uid;
        file_inode.gid = options.gid;
        file_inode.size = FilePos(0);
        let file_inode_id = fs.create_inode(file_inode.clone())?;

        let mut buf = [0; BLOCK_SIZE];
        let dir_entry_buf =
            PhysicalDirectoryEntry::write(file_inode_id, options.file_name, &mut buf)?;

        fs.write(self.inode_idx, ReadWritePos::End(0), &dir_entry_buf)?;

        Ok(File::new(file_inode_id, file_inode))
    }

    pub fn exists<'a, T: ReadWriteSeek>(
        &mut self,
        fs: &'a mut Vsfs<T>,
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
        fs: &'a mut Vsfs<T>,
    ) -> Result<DirectoryIterator<'a, T>> {
        DirectoryIterator::new(fs, self.inode_idx)
    }
}

pub struct CreateFileOptions<'a> {
    pub uid: Uid,
    pub gid: Uid,
    pub mode: Mode,
    pub file_name: &'a str,
    #[cfg(not(feature = "std"))]
    pub time: crate::TimeSeconds,
}

pub struct DirectoryIterator<'a, T: ReadWriteSeek> {
    fs: &'a mut Vsfs<T>,
    inode_idx: INodeBlockIndex,
    offset: FilePos,
}

impl<'a, T: ReadWriteSeek> DirectoryIterator<'a, T> {
    pub(crate) fn new(fs: &'a mut Vsfs<T>, inode_idx: INodeBlockIndex) -> Result<Self> {
        Ok(Self {
            fs,
            inode_idx,
            offset: FilePos(0),
        })
    }

    fn read_next(&mut self) -> Result<Option<DirectoryEntry>> {
        let mut dir_entry_buf = [0; BASE_PHYSICAL_DIRECTORY_ENTRY_SIZE];
        let mut file_name_buf = [0; MAX_FILE_NAME_LEN];
        loop {
            let read = self.fs.read(
                self.inode_idx,
                ReadWritePos::Start(self.offset.0),
                &mut dir_entry_buf,
            )?;
            if read != dir_entry_buf.len() {
                return Ok(None);
            }
            self.offset += read;

            let dir_entry = PhysicalDirectoryEntry::read_from_bytes(&dir_entry_buf)
                .map_err(|_| FileIoError::BufferTooSmall)?;
            if dir_entry.inode_idx == 0 {
                self.offset += dir_entry.name_len;
                continue;
            }

            let file_name = file_name_buf
                .get_mut(0..dir_entry.name_len as usize)
                .ok_or(FileIoError::BufferTooSmall)?;
            let read = self.fs.read(
                self.inode_idx,
                ReadWritePos::Start(self.offset.0),
                file_name,
            )?;
            if read < dir_entry.name_len as usize {
                return Err(FileIoError::IoError(IoError::ReadError));
            }

            let entry_inode = self.fs.read_inode(INodeBlockIndex(dir_entry.inode_idx))?;
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
        self.inode.mode.is_directory()
    }

    pub(crate) fn inode_idx(&self) -> INodeBlockIndex {
        INodeBlockIndex(self.physical_directory_entry.inode_idx)
    }

    pub fn to_dir(&self) -> Option<Directory> {
        if self.is_dir() {
            Some(Directory::new(self.inode_idx(), self.inode.clone()))
        } else {
            None
        }
    }

    pub fn file_name(&self) -> Result<&str> {
        let file_name = self
            .file_name
            .get(0..self.physical_directory_entry.name_len as usize)
            .ok_or(FileIoError::BufferTooSmall)?;
        str::from_utf8(file_name).map_err(|_| FileIoError::Other("failed to decode utf8"))
    }
}
