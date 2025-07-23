use core::fmt::Debug;

use bitflags::bitflags;
use chrono::NaiveDateTime;
use file_io::{FileIoError, Result};
use io::IoError;
use zerocopy::{
    FromBytes, Immutable, IntoBytes, KnownLayout,
    little_endian::{U16, U32},
};

use crate::{
    source::Ext4Source,
    types::{BlockIndex, INodeIndex},
    utils::{hi_low_to_date_time, u32_from_hi_lo, u64_from_hi_lo},
};

pub(crate) const INODE_SIZE: usize = core::mem::size_of::<INode>();
const EXT4_N_BLOCKS: usize = 15;

#[repr(C, packed)]
#[derive(Clone, IntoBytes, FromBytes, Immutable, KnownLayout)]
pub(crate) struct INode {
    /// File mode
    mode: U16,
    /// Low 16 bits of Owner Uid
    i_uid: U16,
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
    i_gid: U16,
    /// Links count
    links_count: U16,
    /// Blocks count
    blocks_lo: U32,
    /// File flags
    i_flags: U32,
    // version
    i_version: U32,

    /// Pointers to blocks
    block: [U32; EXT4_N_BLOCKS],
    /// File version (for NFS)
    generation: U32,
    /// File ACL
    file_acl_lo: U32,
    size_high: U32,
    /// Obsoleted fragment address
    obso_faddr: U32,

    /// werereserved1
    blocks_high: U16,
    file_acl_high: U16,
    /// these 2 fields
    uid_high: U16,
    /// were reserved2[0]
    gid_high: U16,
    /// crc32c(uuid+inum+inode) LE
    checksum_lo: U16,
    reserved: U16,

    extra_isize: U16,
    /// crc32c(uuid+inum+inode) BE
    checksum_hi: U16,
    /// extra Change time (nsec << 2 | epoch)
    ctime_extra: U32,
    /// extra Modification time (nsec << 2 | epoch)
    mtime_extra: U32,
    /// extra Access time (nsec << 2 | epoch)
    atime_extra: U32,
    /// File Creation time
    crtime: U32,
    /// extra FileCreationtime (nsec << 2 | epoch)
    crtime_extra: U32,
    /// high 32 bits for 64-bit version
    version_hi: U32,
    /// Project ID
    projid: U32,
}

bitflags! {
    /// see https://www.kernel.org/doc/html/latest/filesystems/ext4/inodes.html#i-flags
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct INodeFileFlags: u32 {
        /// This file requires secure deletion
        const SECRM = 0x0001;
        /// This file should be preserved, should undeletion be desired
        const UNRM = 0x0002;
        /// File is compressed
        const COMPR = 0x0004;
        /// All writes to the file must be synchronous
        const SYNC = 0x0008;
        /// File is immutable
        const IMMUTABLE = 0x0010;
        /// File can only be appended
        const APPEND = 0x0020;
        /// The dump(1) utility should not dump this file ().
        const NODUMP = 0x40;
        /// Do not update access time ().
        const NOATIME = 0x80;
        /// Dirty compressed file (). (not used)
        const DIRTY = 0x100;
        /// File has one or more compressed clusters (). (not used)
        const COMPRBLK = 0x200;
        /// Do not compress file (). (not used)
        const NOCOMPR = 0x400;
        /// Encrypted inode (). This bit value previously was EXT4_ECOMPR_FL (compression error), which was never used.
        const ENCRYPT = 0x800;
        /// Directory has hashed indexes ().
        const INDEX = 0x1000;
        /// AFS magic directory ().
        const IMAGIC = 0x2000;
        /// File data must always be written through the journal ().
        const JOURNAL_DATA = 0x4000;
        /// File tail should not be merged (). (not used by ext4)
        const NOTAIL = 0x8000;
        /// All directory entry data should be written synchronously (see dirsync) ().
        const DIRSYNC = 0x10000;
        /// Top of directory hierarchy ().
        const TOPDIR = 0x20000;
        /// This is a huge file ().
        const HUGE_FILE = 0x40000;
        /// Inode uses extents ().
        const EXTENTS = 0x80000;
        /// Verity protected file ().
        const VERITY = 0x100000;
        /// Inode stores a large extended attribute value in its data blocks ().
        const EA_INODE = 0x200000;
        /// This file has blocks allocated past EOF (). (deprecated)
        const EOFBLOCKS = 0x400000;
        /// Inode is a snapshot (). (not in mainline)
        const SNAPFILE = 0x01000000;
        /// Snapshot is being deleted (). (not in mainline)
        const SNAPFILE_DELETED = 0x04000000;
        /// Snapshot shrink has completed (). (not in mainline)
        const SNAPFILE_SHRUNK = 0x08000000;
        /// Inode has inline data ().
        const INLINE_DATA = 0x10000000;
        /// Create children with the same project ID ().
        const PROJINHERIT = 0x20000000;
        /// Reserved for ext4 library ().
        const RESERVED = 0x80000000;
    }
}

impl INode {
    pub(crate) fn read<T: Ext4Source>(
        source: &T,
        inode_table_block_idx: &BlockIndex,
        relative_inode_idx: &INodeIndex,
        block_size: u32,
        inode_size: u16,
    ) -> Result<Self> {
        let mut buf = [0; INODE_SIZE];

        #[cfg(test)]
        println!(
            "table 0x{:x} (size 0x{:x}) {}",
            inode_table_block_idx.to_file_pos(block_size).0,
            inode_size,
            relative_inode_idx.0
        );

        let file_pos = inode_table_block_idx.to_file_pos(block_size)
            + ((relative_inode_idx.0) as u64 * inode_size as u64);

        #[cfg(test)]
        println!("file_pos 0x{:x} (size 0x{:x})", file_pos.0, inode_size);

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
        // todo ctime_extra
        hi_low_to_date_time(0, self.ctime.get())
    }

    pub fn modified_time(&self) -> Result<Option<NaiveDateTime>> {
        // todo mtime_extra
        hi_low_to_date_time(0, self.mtime.get())
    }

    pub fn deletion_time(&self) -> Result<Option<NaiveDateTime>> {
        // todo atime_extra
        hi_low_to_date_time(0, self.dtime.get())
    }

    pub fn creation_time(&self) -> Result<Option<NaiveDateTime>> {
        // todo crtime_extra
        hi_low_to_date_time(0, self.crtime.get())
    }

    pub fn size(&self) -> u64 {
        u64_from_hi_lo(self.size_high.get(), self.size_lo.get())
    }

    pub fn blocks(&self) -> u64 {
        u64_from_hi_lo(self.blocks_high.get() as u32, self.blocks_lo.get())
    }

    pub fn file_acl(&self) -> u64 {
        u64_from_hi_lo(self.file_acl_high.get() as u32, self.file_acl_lo.get())
    }

    pub fn uid(&self) -> u32 {
        u32_from_hi_lo(self.uid_high.get(), self.i_uid.get())
    }

    pub fn gid(&self) -> u32 {
        u32_from_hi_lo(self.gid_high.get(), self.i_gid.get())
    }

    pub fn checksum(&self) -> u32 {
        u32_from_hi_lo(self.checksum_hi.get(), self.checksum_lo.get())
    }

    pub fn version(&self) -> u64 {
        u64_from_hi_lo(self.version_hi.get(), self.i_version.get())
    }

    pub fn flags(&self) -> INodeFileFlags {
        INodeFileFlags::from_bits_retain(self.i_flags.get())
    }
}

impl Debug for INode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("INode")
            .field("mode", &format_args!("0o{:o}", self.mode.get()))
            .field("uid", &self.uid())
            .field("gid", &self.gid())
            .field("size", &self.size())
            .field("access_time", &self.access_time())
            .field("create_time", &self.create_time())
            .field("modified_time", &self.modified_time())
            .field("deletion_time", &self.deletion_time())
            .field("links_count", &self.links_count.get())
            .field("blocks", &self.blocks())
            .field("flags", &self.flags())
            .field("version", &self.version())
            .field("block", &self.block)
            .field("generation", &self.generation.get())
            .field("file_acl", &self.file_acl())
            .field("obso_faddr", &self.obso_faddr.get())
            .field("checksum", &self.checksum())
            .field("extra_isize", &self.extra_isize.get())
            .field("crtime", &self.creation_time())
            .field("projid", &self.projid.get())
            .finish()
    }
}
