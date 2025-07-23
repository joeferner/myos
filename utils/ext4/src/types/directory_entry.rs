use zerocopy::{
    FromBytes, Immutable, IntoBytes, KnownLayout,
    little_endian::{U16, U32},
};

pub(crate) const DIR_ENTRY_2_SIZE: usize = core::mem::size_of::<DirEntry2>();
pub(crate) const EXT4_NAME_LEN: usize = 255;

#[repr(C, packed)]
#[derive(Clone, IntoBytes, FromBytes, Immutable, KnownLayout)]
pub(crate) struct DirEntry2 {
    /// Number of the inode that this directory entry points to
    inode: U32,

    /// Length of this directory entry.
    rec_len: U16,

    /// Length of the file name
    name_len: u8,

    /// File type code, see ftype table below
    file_type: u8,

    /// file name
    name: [u8; EXT4_NAME_LEN],
}
