use core::fmt::Debug;
use myos_api::filesystem::{FileIoError, FilePos, Result};
use nostdio::NoStdIoError;
use zerocopy::{
    FromBytes, Immutable, IntoBytes, KnownLayout,
    little_endian::{U16, U32},
};

use crate::{
    source::Ext4Source, types::BlockIndex, utils::{u32_from_hi_lo, u64_from_hi_lo}
};

pub(crate) const BLOCK_GROUP_DESCRIPTOR_SIZE: usize = core::mem::size_of::<BlockGroupDescriptor>();

#[repr(C, packed)]
#[derive(Clone, IntoBytes, FromBytes, Immutable, KnownLayout)]
pub(crate) struct BlockGroupDescriptor {
    /// Blocks bitmap block
    block_bitmap_lo: U32,
    /// Inodes bitmap block
    inode_bitmap_lo: U32,
    /// Inodes table block
    inode_table_lo: U32,
    /// Free blocks count
    free_blocks_count_lo: U16,
    /// Free inodes count
    free_inodes_count_lo: U16,
    /// Directories count
    used_dirs_count_lo: U16,
    /// EXT4_BG_flags (INODE_UNINIT, etc)
    flags: U16,
    /// Exclude bitmap for snapshots
    exclude_bitmap_lo: U32,
    /// crc32c(s_uuid+grp_num+bbitmap) LE
    block_bitmap_csum_lo: U16,
    /// crc32c(s_uuid+grp_num+ibitmap) LE
    inode_bitmap_csum_lo: U16,
    /// Unused inodes count
    itable_unused_lo: U16,
    /// crc16(sb_uuid+group+desc)
    checksum: U16,
    /// Blocks bitmap block MSB
    block_bitmap_hi: U32,
    /// Inodes bitmap block MSB
    inode_bitmap_hi: U32,
    /// Inodes table block MSB
    inode_table_hi: U32,
    /// Free blocks count MSB
    free_blocks_count_hi: U16,
    /// Free inodes count MSB
    free_inodes_count_hi: U16,
    /// Directories count MSB
    used_dirs_count_hi: U16,
    /// Unused inodes count MSB
    itable_unused_hi: U16,
    /// Exclude bitmap block MSB
    exclude_bitmap_hi: U32,
    /// crc32c(s_uuid+grp_num+bbitmap) BE
    block_bitmap_csum_hi: U16,
    /// crc32c(s_uuid+grp_num+ibitmap) BE
    inode_bitmap_csum_hi: U16,
    reserved: U32,
}

impl BlockGroupDescriptor {
    pub(crate) fn read<T: Ext4Source>(source: &T, file_pos: FilePos) -> Result<Self> {
        let mut buf = [0; BLOCK_GROUP_DESCRIPTOR_SIZE];
        source.read(file_pos, &mut buf)?;
        let bgd = BlockGroupDescriptor::read_from_bytes(&buf).map_err(|err| {
            FileIoError::IoError(NoStdIoError::from_zerocopy_err(
                "failed to read block group descriptor from bytes",
                err,
            ))
        })?;

        Ok(bgd)
    }

    pub fn block_bitmap_block_index(&self) -> BlockIndex {
        BlockIndex(u64_from_hi_lo(self.block_bitmap_hi.get(), self.block_bitmap_lo.get()))
    }

    pub fn inode_bitmap_block_index(&self) -> BlockIndex {
        BlockIndex(u64_from_hi_lo(self.inode_bitmap_hi.get(), self.inode_bitmap_lo.get()))
    }

    pub fn inode_table_block_index(&self) -> BlockIndex {
        BlockIndex(u64_from_hi_lo(self.inode_table_hi.get(), self.inode_table_lo.get()))
    }

    pub fn free_blocks_count(&self) -> u32 {
        u32_from_hi_lo(
            self.free_blocks_count_hi.get(),
            self.free_blocks_count_lo.get(),
        )
    }

    pub fn free_inodes_count(&self) -> u32 {
        u32_from_hi_lo(
            self.free_inodes_count_hi.get(),
            self.free_inodes_count_lo.get(),
        )
    }

    pub fn used_dirs_count(&self) -> u32 {
        u32_from_hi_lo(self.used_dirs_count_hi.get(), self.used_dirs_count_lo.get())
    }

    pub fn exclude_bitmap(&self) -> BlockIndex {
        BlockIndex(u64_from_hi_lo(self.exclude_bitmap_hi.get(), self.exclude_bitmap_lo.get()))
    }

    pub fn block_bitmap_csum(&self) -> u32 {
        u32_from_hi_lo(
            self.block_bitmap_csum_hi.get(),
            self.block_bitmap_csum_lo.get(),
        )
    }

    pub fn inode_bitmap_csum(&self) -> u32 {
        u32_from_hi_lo(
            self.inode_bitmap_csum_hi.get(),
            self.inode_bitmap_csum_lo.get(),
        )
    }

    pub fn itable_unused(&self) -> u32 {
        u32_from_hi_lo(self.itable_unused_hi.get(), self.itable_unused_lo.get())
    }
}

impl Debug for BlockGroupDescriptor {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("BlockGroupDescriptor")
            .field("block_bitmap", &self.block_bitmap_block_index())
            .field("inode_bitmap", &self.inode_bitmap_block_index())
            .field("inode_table", &self.inode_table_block_index())
            .field("free_blocks_count", &self.free_blocks_count())
            .field("free_inodes_count", &self.free_inodes_count())
            .field("used_dirs_count", &self.used_dirs_count())
            .field("flags", &self.flags.get())
            .field("exclude_bitmap", &self.exclude_bitmap())
            .field("block_bitmap_csum", &self.block_bitmap_csum())
            .field("inode_bitmap_csum", &self.inode_bitmap_csum())
            .field("itable_unused", &self.itable_unused())
            .field("checksum", &self.checksum.get())
            .finish()
    }
}
