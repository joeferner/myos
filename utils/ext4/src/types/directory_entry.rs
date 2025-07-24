use core::fmt::Debug;

use file_io::{FileIoError, Result};
use zerocopy::{
    FromBytes, Immutable, IntoBytes, KnownLayout, TryFromBytes,
    little_endian::{U16, U32},
};

use crate::types::INodeIndex;

pub(crate) const DIR_ENTRY_2_SIZE: usize = core::mem::size_of::<DirEntry2>();
pub(crate) const EXT4_NAME_LEN: usize = 255;

#[repr(C, packed)]
#[derive(Clone, IntoBytes, FromBytes, Immutable, KnownLayout)]
pub(crate) struct DirEntry2 {
    /// Number of the inode that this directory entry points to
    i_inode: U32,

    /// Length of this directory entry.
    rec_len: U16,

    /// Length of the file name
    name_len: u8,

    /// File type code, see ftype table below
    i_file_type: u8,
    // file name [u8; EXT4_NAME_LEN]
}

impl DirEntry2 {
    pub(crate) fn read<T: Ext4Source>(source: &T, file_pos: &FilePos) -> Result<Self> {
        let mut buf = [0; DIR_ENTRY_2_SIZE];
        if let Err(err) = self.fs.read(&self.inode, &file_pos, &mut buf) {
            return Some(Err(err));
        }
        self.offset += buf.len();

        let dir_entry = match DirEntry2::read_from_bytes(&buf) {
            Ok(dir_entry) => dir_entry,
            Err(err) => {
                return Some(Err(FileIoError::IoError(IoError::from_zerocopy_err(
                    "failed reading dir entry",
                    err,
                ))));
            }
        };
    }

    pub fn inode(&self) -> INodeIndex {
        INodeIndex(self.i_inode.get())
    }

    pub fn file_type(&self) -> DirEntryFileType {
        let buf = [self.i_file_type];
        DirEntryFileType::try_read_from_bytes(&buf).unwrap_or(DirEntryFileType::Unknown)
    }

    pub fn name<'a>(&'a self) -> Result<&'a str> {
        str::from_utf8(&self.i_name[0..self.name_len as usize])
            .map_err(|_| FileIoError::Other("utf encoding error"))
    }
}

impl Debug for DirEntry2 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DirEntry2")
            .field("inode", &self.inode())
            .field("rec_len", &self.rec_len.get())
            .field("name_len", &self.name_len)
            .field("file_type", &self.file_type())
            .field("name", &self.name())
            .finish()
    }
}

/// see https://docs.kernel.org/filesystems/ext4/dynamic.html#linear-classic-directories
#[allow(dead_code)]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, TryFromBytes)]
pub enum DirEntryFileType {
    Unknown = 0x0,
    RegularFile = 0x1,
    Directory = 0x2,
    CharacterDeviceFile = 0x3,
    BlockDeviceFile = 0x4,
    Fifo = 0x5,
    Socket = 0x6,
    SymbolicLink = 0x7,
}
