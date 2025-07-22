use core::fmt::Debug;
use file_io::{FileIoError, FilePos, Result};
use io::IoError;
use zerocopy::{
    FromBytes, Immutable, IntoBytes, KnownLayout,
    little_endian::{U16, U32},
};

use crate::{source::Ext4Source, utils::{u32_from_hi_lo, u64_from_hi_lo}};

pub(crate) const BLOCK_GROUP_DESCRIPTOR_SIZE: usize = core::mem::size_of::<BlockGroupDescriptor>();

#[repr(C, packed)]
#[derive(Clone, IntoBytes, FromBytes, Immutable, KnownLayout)]
pub(crate) struct BlockGroupDescriptor {
    /// Blocks bitmap block
    bg_block_bitmap_lo: U32,
    /// Inodes bitmap block
    bg_inode_bitmap_lo: U32,
    /// Inodes table block
    bg_inode_table_lo: U32,
    /// Free blocks count
    bg_free_blocks_count_lo: U16,
    /// Free inodes count
    bg_free_inodes_count_lo: U16,
    /// Directories count
    bg_used_dirs_count_lo: U16,
    /// EXT4_BG_flags (INODE_UNINIT, etc)
    bg_flags: U16,
    /// Exclude bitmap for snapshots
    bg_exclude_bitmap_lo: U32,
    /// crc32c(s_uuid+grp_num+bbitmap) LE
    bg_block_bitmap_csum_lo: U16,
    /// crc32c(s_uuid+grp_num+ibitmap) LE
    bg_inode_bitmap_csum_lo: U16,
    /// Unused inodes count
    bg_itable_unused_lo: U16,
    /// crc16(sb_uuid+group+desc)
    bg_checksum: U16,
    /// Blocks bitmap block MSB
    bg_block_bitmap_hi: U32,
    /// Inodes bitmap block MSB
    bg_inode_bitmap_hi: U32,
    /// Inodes table block MSB
    bg_inode_table_hi: U32,
    /// Free blocks count MSB
    bg_free_blocks_count_hi: U16,
    /// Free inodes count MSB
    bg_free_inodes_count_hi: U16,
    /// Directories count MSB
    bg_used_dirs_count_hi: U16,
    /// Unused inodes count MSB
    bg_itable_unused_hi: U16,
    /// Exclude bitmap block MSB
    bg_exclude_bitmap_hi: U32,
    /// crc32c(s_uuid+grp_num+bbitmap) BE
    bg_block_bitmap_csum_hi: U16,
    /// crc32c(s_uuid+grp_num+ibitmap) BE
    bg_inode_bitmap_csum_hi: U16,
    bg_reserved: U32,
}

impl BlockGroupDescriptor {
    pub(crate) fn read<T: Ext4Source>(source: &T, file_pos: &FilePos) -> Result<(Self, FilePos)> {
        let mut buf = [0; BLOCK_GROUP_DESCRIPTOR_SIZE];
        source.read(file_pos, &mut buf)?;
        let bgd = BlockGroupDescriptor::read_from_bytes(&buf).map_err(|err| {
            FileIoError::IoError(IoError::from_zerocopy_err(
                "failed to read block group descriptor from bytes",
                err,
            ))
        })?;

        Ok((bgd, *file_pos + BLOCK_GROUP_DESCRIPTOR_SIZE))
    }

    pub fn bg_block_bitmap(&self) -> u64 {
        u64_from_hi_lo(self.bg_block_bitmap_hi.get(), self.bg_block_bitmap_lo.get())
    }

    pub fn bg_inode_bitmap(&self) -> u64 {
        u64_from_hi_lo(self.bg_inode_bitmap_hi.get(), self.bg_inode_bitmap_lo.get())
    }

    pub fn bg_inode_table(&self) -> u64 {
        u64_from_hi_lo(self.bg_inode_table_hi.get(), self.bg_inode_table_lo.get())
    }

    pub fn bg_free_blocks_count(&self) -> u32 {
        u32_from_hi_lo(
            self.bg_free_blocks_count_hi.get(),
            self.bg_free_blocks_count_lo.get(),
        )
    }

    pub fn bg_free_inodes_count(&self) -> u32 {
        u32_from_hi_lo(
            self.bg_free_inodes_count_hi.get(),
            self.bg_free_inodes_count_lo.get(),
        )
    }

    pub fn bg_used_dirs_count(&self) -> u32 {
        u32_from_hi_lo(
            self.bg_used_dirs_count_hi.get(),
            self.bg_used_dirs_count_lo.get(),
        )
    }

    pub fn bg_exclude_bitmap(&self) -> u64 {
        u64_from_hi_lo(
            self.bg_exclude_bitmap_hi.get(),
            self.bg_exclude_bitmap_lo.get(),
        )
    }

    pub fn bg_block_bitmap_csum(&self) -> u32 {
        u32_from_hi_lo(
            self.bg_block_bitmap_csum_hi.get(),
            self.bg_block_bitmap_csum_lo.get(),
        )
    }

    pub fn bg_inode_bitmap_csum(&self) -> u32 {
        u32_from_hi_lo(
            self.bg_inode_bitmap_csum_hi.get(),
            self.bg_inode_bitmap_csum_lo.get(),
        )
    }

    pub fn bg_itable_unused(&self) -> u32 {
        u32_from_hi_lo(
            self.bg_itable_unused_hi.get(),
            self.bg_itable_unused_lo.get(),
        )
    }
}

impl Debug for BlockGroupDescriptor {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("BlockGroupDescriptor")
            .field("bg_block_bitmap", &self.bg_block_bitmap())
            .field("bg_inode_bitmap", &self.bg_inode_bitmap())
            .field("bg_inode_table", &self.bg_inode_table())
            .field("bg_free_blocks_count", &self.bg_free_blocks_count())
            .field("bg_free_inodes_count", &self.bg_free_inodes_count())
            .field("bg_used_dirs_count", &self.bg_used_dirs_count())
            .field("bg_flags", &self.bg_flags.get())
            .field("bg_exclude_bitmap", &self.bg_exclude_bitmap())
            .field("bg_block_bitmap_csum", &self.bg_block_bitmap_csum())
            .field("bg_inode_bitmap_csum", &self.bg_inode_bitmap_csum())
            .field("bg_itable_unused", &self.bg_itable_unused())
            .field("bg_checksum", &self.bg_checksum.get())
            .finish()
    }
}
