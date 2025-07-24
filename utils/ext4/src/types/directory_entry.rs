use core::fmt::Debug;

use file_io::{FileIoError, FilePos, Result};
use io::IoError;
use zerocopy::{
    FromBytes, Immutable, IntoBytes, KnownLayout, TryFromBytes,
    little_endian::{U16, U32},
};

use crate::{Ext4, source::Ext4Source, types::INodeIndex, types::inode::INode};

const DIR_ENTRY_2_HEADER_SIZE: usize = core::mem::size_of::<DirEntry2Header>();
pub(crate) const EXT4_NAME_LEN: usize = 255;

#[repr(C, packed)]
#[derive(Clone, IntoBytes, FromBytes, Immutable, KnownLayout)]
pub(crate) struct DirEntry2Header {
    /// Number of the inode that this directory entry points to
    inode: U32,

    /// Length of this directory entry.
    rec_len: U16,

    /// Length of the file name
    name_len: u8,

    /// File type code, see ftype table below
    file_type: u8,
    // file name [u8; EXT4_NAME_LEN]
}

pub(crate) struct DirEntry2 {
    pub inode: INodeIndex,
    pub file_type: DirEntryFileType,
    pub record_length: usize,
    name_len: usize,
    name_buf: [u8; EXT4_NAME_LEN],
}

impl DirEntry2 {
    pub(crate) fn read<T: Ext4Source>(
        source: &Ext4<T>,
        inode: &INode,
        file_pos: FilePos,
    ) -> Result<Self> {
        let mut buf = [0; DIR_ENTRY_2_HEADER_SIZE];
        source.read(inode, file_pos, &mut buf)?;

        let dir_entry_header = match DirEntry2Header::read_from_bytes(&buf) {
            Ok(dir_entry) => dir_entry,
            Err(err) => {
                return Err(FileIoError::IoError(IoError::from_zerocopy_err(
                    "failed reading dir entry",
                    err,
                )));
            }
        };

        let file_type = {
            let buf = [dir_entry_header.file_type];
            DirEntryFileType::try_read_from_bytes(&buf).unwrap_or(DirEntryFileType::Unknown)
        };

        let mut name_buf = [0; EXT4_NAME_LEN];
        let partial_name_buf = name_buf
            .get_mut(0..dir_entry_header.name_len as usize)
            .ok_or(FileIoError::BufferTooSmall)?;
        source.read(
            inode,
            file_pos + DIR_ENTRY_2_HEADER_SIZE,
            partial_name_buf,
        )?;

        let name_len = dir_entry_header.name_len as usize;

        Ok(Self {
            inode: INodeIndex(dir_entry_header.inode.get()),
            file_type,
            record_length: dir_entry_header.rec_len.get() as usize,
            name_len,
            name_buf,
        })
    }

    pub fn name(&self) -> Result<&str> {
        // verified to work in new
        let partial_name_buf = self
            .name_buf
            .get(0..self.name_len)
            .ok_or(FileIoError::BufferTooSmall)?;
        str::from_utf8(partial_name_buf).map_err(|_| FileIoError::Other("string encoding error"))
    }
}

impl Debug for DirEntry2 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DirEntry2")
            .field("inode", &self.inode)
            .field("file_type", &self.file_type)
            .field("record_length", &self.record_length)
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
