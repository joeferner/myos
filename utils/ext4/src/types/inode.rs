use core::fmt::Debug;

use chrono::NaiveDateTime;
use file_io::{FileIoError, Result};
use io::IoError;
use zerocopy::{
    FromBytes, Immutable, IntoBytes, KnownLayout,
    little_endian::{U16, U32},
};

use crate::{source::Ext4Source, types::BlockIndex, utils::hi_low_to_date_time};

pub(crate) const INODE_SIZE: usize = core::mem::size_of::<INode>();
const EXT4_N_BLOCKS: usize = 15;

#[repr(C, packed)]
#[derive(Clone, IntoBytes, FromBytes, Immutable, KnownLayout)]
pub(crate) struct INode {
    /// File mode
    mode: U16,
    /// Low 16 bits of Owner Uid
    uid: U16,
    /// Size in bytes
    size_lo: U32,
    /// Access time
    atime: U32,
    /// Inode Change time
    ctime: U32,
    /// Modification time
    mtime: U32,
    /// Deletion Time
    dtime: U32,
    /// Low 16 bits of Group Id
    gid: U16,
    /// Links count
    links_count: U16,
    /// Blocks count
    blocks_lo: U32,
    /// File flags
    flags: U32,
    // version
    version: U32,

    i_block: [U32; EXT4_N_BLOCKS], /* Pointers to blocks */
    i_generation: U32,             /* File version (for NFS) */
    i_file_acl_lo: U32,            /* File ACL */
    i_size_high: U32,
    i_obso_faddr: U32, /* Obsoleted fragment address */

    l_i_blocks_high: U16, /* were l_i_reserved1 */
    l_i_file_acl_high: U16,
    l_i_uid_high: U16,    /* these 2 fields */
    l_i_gid_high: U16,    /* were reserved2[0] */
    l_i_checksum_lo: U16, /* crc32c(uuid+inum+inode) LE */
    l_i_reserved: U16,

    i_extra_isize: U16,
    i_checksum_hi: U16,  /* crc32c(uuid+inum+inode) BE */
    i_ctime_extra: U32,  /* extra Change time      (nsec << 2 | epoch) */
    i_mtime_extra: U32,  /* extra Modification time(nsec << 2 | epoch) */
    i_atime_extra: U32,  /* extra Access time      (nsec << 2 | epoch) */
    i_crtime: U32,       /* File Creation time */
    i_crtime_extra: U32, /* extra FileCreationtime (nsec << 2 | epoch) */
    i_version_hi: U32,   /* high 32 bits for 64-bit version */
    i_projid: U32,       /* Project ID */
}

impl INode {
    pub(crate) fn read<T: Ext4Source>(
        source: &T,
        inode_table_block_idx: &BlockIndex,
        block_size: u32,
    ) -> Result<Self> {
        let mut buf = [0; INODE_SIZE];
        let file_pos = inode_table_block_idx.to_file_pos(block_size);
        source.read(&file_pos, &mut buf)?;
        let inode = INode::read_from_bytes(&buf).map_err(|err| {
            FileIoError::IoError(IoError::from_zerocopy_err(
                "failed to read inode from bytes",
                err,
            ))
        })?;

        Ok(inode)
    }

    pub fn access_time(&self) -> Result<Option<NaiveDateTime>> {
        hi_low_to_date_time(0, self.atime.get())
    }

    pub fn create_time(&self) -> Result<Option<NaiveDateTime>> {
        hi_low_to_date_time(0, self.ctime.get())
    }

    pub fn modified_time(&self) -> Result<Option<NaiveDateTime>> {
        hi_low_to_date_time(0, self.mtime.get())
    }

    pub fn deletion_time(&self) -> Result<Option<NaiveDateTime>> {
        hi_low_to_date_time(0, self.dtime.get())
    }
}

impl Debug for INode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("INode")
            .field("mode", &self.mode.get())
            .field("uid", &self.uid.get())
            .field("size_lo", &self.size_lo.get())
            .field("access_time", &self.access_time())
            .field("create_time", &self.create_time())
            .field("modified_time", &self.modified_time())
            .field("deletion_time", &self.deletion_time())
            .field("gid", &self.gid)
            .field("links_count", &self.links_count.get())
            .field("blocks_lo", &self.blocks_lo.get())
            .field("flags", &self.flags.get())
            .field("version", &self.version)
            .field("i_block", &self.i_block)
            .field("i_generation", &self.i_generation)
            .field("i_file_acl_lo", &self.i_file_acl_lo)
            .field("i_size_high", &self.i_size_high)
            .field("i_obso_faddr", &self.i_obso_faddr)
            .field("l_i_blocks_high", &self.l_i_blocks_high)
            .field("l_i_file_acl_high", &self.l_i_file_acl_high)
            .field("l_i_uid_high", &self.l_i_uid_high)
            .field("l_i_gid_high", &self.l_i_gid_high)
            .field("l_i_checksum_lo", &self.l_i_checksum_lo)
            .field("l_i_reserved", &self.l_i_reserved)
            .field("i_extra_isize", &self.i_extra_isize)
            .field("i_checksum_hi", &self.i_checksum_hi)
            .field("i_ctime_extra", &self.i_ctime_extra)
            .field("i_mtime_extra", &self.i_mtime_extra)
            .field("i_atime_extra", &self.i_atime_extra)
            .field("i_crtime", &self.i_crtime)
            .field("i_crtime_extra", &self.i_crtime_extra)
            .field("i_version_hi", &self.i_version_hi)
            .field("i_projid", &self.i_projid)
            .finish()
    }
}
