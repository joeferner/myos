use file_io::{FileIoError, Result};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

use crate::{DataBlockIndex, INodeBlockIndex, inode::INode};

pub const MAGIC: [u8; 4] = *b"vsfs";
pub const BLOCK_SIZE: usize = 4 * 1024;
pub(crate) const PHYSICAL_INODE_SIZE: usize = core::mem::size_of::<PhysicalINode>();
pub(crate) const PHYSICAL_INODES_PER_BLOCK: u32 = (BLOCK_SIZE / PHYSICAL_INODE_SIZE) as u32;
/// Number of block offsets stored in the inode itself, if the number of
/// blocks exceeds this amount additional blocks will be stored in
/// the indirect_block data
pub(crate) const IMMEDIATE_BLOCK_COUNT: usize = 12;
pub(crate) const BLOCK_NOT_SET: u32 = 0;

#[repr(C, packed)]
#[derive(Debug, Clone, IntoBytes, FromBytes, Immutable, KnownLayout)]
pub(crate) struct PhysicalINode {
    pub uid: u32,
    pub gid: u32,
    pub mode: u16,
    /// size of the file
    pub size: u64,
    /// what time was this file last accessed?
    pub time: u64,
    /// what time was this file created?
    pub ctime: u64,
    /// what time was this file last modified?
    pub mtime: u64,
    /// index into the blocks where the first x blocks of data can be found, 0 indicates unused block
    pub blocks: [u32; IMMEDIATE_BLOCK_COUNT],
    /// if not 0, indicates an index into the block table where you will find more block addresses
    pub indirect_block_idx: u32,
}

impl From<INode> for PhysicalINode {
    fn from(value: INode) -> Self {
        let mut blocks = [0; IMMEDIATE_BLOCK_COUNT];
        for i in 0..IMMEDIATE_BLOCK_COUNT {
            blocks[i] = value.blocks[i].unwrap_or(DataBlockIndex(BLOCK_NOT_SET)).0;
        }

        Self {
            uid: value.uid.0,
            gid: value.gid.0,
            mode: value.mode.0,
            size: value.size.0,
            time: value.time.0,
            ctime: value.ctime.0,
            mtime: value.mtime.0,
            blocks,
            indirect_block_idx: value.indirect_block_idx.unwrap_or(DataBlockIndex(BLOCK_NOT_SET)).0,
        }
    }
}

#[repr(C, packed)]
#[derive(Debug, Clone, IntoBytes, FromBytes, Immutable, KnownLayout)]
pub(crate) struct PhysicalSuperBlock {
    pub magic: [u8; 4],
    pub inode_count: u32,
    pub data_block_count: u32,
}

/// Data stored on the file system for each entry in a directory.
#[repr(C, packed)]
#[derive(Debug, Clone, IntoBytes, FromBytes, Immutable, KnownLayout)]
pub(crate) struct PhysicalDirectoryEntry {
    pub inode_idx: u32,
    pub name_len: u16,
    // name: [u8; < MAX_FILE_NAME_LEN]
}

pub(crate) const BASE_PHYSICAL_DIRECTORY_ENTRY_SIZE: usize =
    core::mem::size_of::<PhysicalDirectoryEntry>();
pub(crate) const MAX_FILE_NAME_LEN: usize = BLOCK_SIZE - BASE_PHYSICAL_DIRECTORY_ENTRY_SIZE;

impl PhysicalDirectoryEntry {
    pub(crate) fn write<'a>(
        inode_idx: INodeBlockIndex,
        name: &str,
        dest_buf: &'a mut [u8],
    ) -> Result<&'a [u8]> {
        let name_bytes = name.as_bytes();
        if name_bytes.len() > MAX_FILE_NAME_LEN {
            return Err(FileIoError::FilenameTooLong);
        }
        let name_len: u16 = name_bytes
            .len()
            .try_into()
            .map_err(|_| FileIoError::FilenameTooLong)?;

        let entry = PhysicalDirectoryEntry {
            inode_idx: inode_idx.0,
            name_len,
        };
        let entry_bytes = entry.as_bytes();
        let total_len = entry_bytes.len() + name_bytes.len();
        if total_len > dest_buf.len() {
            return Err(FileIoError::BufferTooSmall);
        }

        dest_buf
            .get_mut(0..entry_bytes.len())
            .ok_or(FileIoError::BufferTooSmall)?
            .copy_from_slice(entry_bytes);
        dest_buf
            .get_mut(entry_bytes.len()..entry_bytes.len() + name_bytes.len())
            .ok_or(FileIoError::BufferTooSmall)?
            .copy_from_slice(name_bytes);

        Ok(&dest_buf[0..total_len])
    }
}
