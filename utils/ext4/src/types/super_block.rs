use core::{ffi::CStr, fmt::Debug};

use chrono::NaiveDateTime;
use file_io::{FileIoError, FilePos, Result};
use io::IoError;
use uuid::Uuid;
use zerocopy::{
    FromBytes, Immutable, IntoBytes, KnownLayout,
    little_endian::{U16, U32, U64},
};

use crate::{
    source::Ext4Source,
    utils::{u64_from_hi_lo, hi_low_to_date_time},
};

pub(crate) const SUPER_BLOCK_SIZE: usize = core::mem::size_of::<SuperBlock>();
pub(crate) const SUPER_BLOCK_POS: FilePos = FilePos(0x400);
pub(crate) const EXT4_MAGIC: u16 = 0xef53;

// Reference
//   https://blogs.oracle.com/linux/post/understanding-ext4-disk-layout-part-1
//   https://thiscouldbebetter.wordpress.com/2021/10/23/creating-an-ext4-filesystem-image-file/
//   https://docs.kernel.org/filesystems/ext4/

#[repr(C, packed)]
#[derive(Clone, IntoBytes, FromBytes, Immutable, KnownLayout)]
pub(crate) struct SuperBlock {
    /*00*/
    /// Inodes count
    inodes_count: U32,
    /// Blocks count
    blocks_count_lo: U32,
    /// Reserved blocks count
    r_blocks_count_lo: U32,
    /// Free blocks count
    free_blocks_count_lo: U32,
    /*10*/
    /// Free inodes count
    free_inodes_count: U32,
    /// First Data Block
    first_data_block: U32,
    /// Block size
    log_block_size: U32,
    /// Allocation cluster size
    log_cluster_size: U32,
    /*20*/
    /// # Blocks per group
    blocks_per_group: U32,
    /// # Clusters per group
    clusters_per_group: U32,
    /// # Inodes per group
    inodes_per_group: U32,
    /// Mount time
    mtime: U32,
    /*30*/
    /// Write time
    wtime: U32,
    /// Mount count
    mnt_count: U16,
    /// Maximal mount count
    max_mnt_count: U16,
    /// Magic signature
    magic: U16,
    /// File system state
    state: U16,
    /// Behavior when detecting errors
    errors: U16,
    /// minor revision level
    minor_rev_level: U16,
    /*40*/
    /// time of last check
    lastcheck: U32,
    /// max. time between checks
    checkinterval: U32,
    /// OS
    creator_os: U32,
    /// Revision level
    rev_level: U32,
    /*50*/
    /// Default uid for reserved blocks
    def_resuid: U16,
    ///Default gid for reserved blocks
    def_resgid: U16,
    /*
     * These fields are for EXT4_DYNAMIC_REV Superblocks only.
     *
     * Note: the difference between the compatible feature set and
     * the incompatible feature set is that if there is a bit set
     * in the incompatible feature set that the kernel doesn't
     * know about, it should refuse to mount the filesystem.
     *
     * e2fsck's requirements are more strict, if it doesn't know
     * about a feature in either the compatible or incompatible
     * feature set, it must abort and not try to meddle with
     * things it doesn't understand...
     */
    /// First non-reserved inode
    first_ino: U32,
    /// size of inode structure
    inode_size: U16,
    /// block group # of this Superblock
    block_group_nr: U16,
    /// compatible feature set
    feature_compat: U32,
    /*60*/
    /// incompatible feature set
    feature_incompat: U32,
    /// readonly-compatible feature set
    feature_ro_compat: U32,
    /*68*/
    /// 128-bit uuid for volume
    uuid: [u8; 16],
    /*78*/
    /// volume name
    volume_name: [u8; 16],
    /*88*/
    /// directory where last mounted
    last_mounted: [u8; 64],
    /*C8*/
    /// For compression
    algorithm_usage_bitmap: U32,
    /*
     * Performance hints.  Directory preallocation should only
     * happen if the EXT4_FEATURE_COMPAT_DIR_PREALLOC flag is on.
     */
    /// Nr of blocks to try to preallocate
    prealloc_blocks: u8,
    /// Nr to preallocate for dirs
    prealloc_dir_blocks: u8,
    /// Per group desc for online growth
    reserved_gdt_blocks: U16,
    /*
     * Journaling support valid if EXT4_FEATURE_COMPAT_HAS_JOURNAL set.
     */
    /*D0*/
    /// uuid of journal Superblock
    journal_uuid: [u8; 16],
    /*E0*/
    /// inode number of journal file
    journal_inum: U32,
    /// device number of journal file
    journal_dev: U32,
    /// start of list of inodes to delete
    last_orphan: U32,
    /// HTREE hash seed
    hash_seed: [U32; 4],
    /// Default hash version to use
    def_hash_version: u8,
    jnl_backup_type: u8,
    /// size of group descriptor
    desc_size: U16,
    /*100*/
    default_mount_opts: U32,
    /// First metablock block group
    first_meta_bg: U32,
    /// When the filesystem was created
    mkfs_time: U32,
    /// Backup of the journal inode
    jnl_blocks: [U32; 17],
    /* 64bit support valid if EXT4_FEATURE_COMPAT_64BIT */
    /*150*/
    /// Blocks count
    blocks_count_hi: U32,
    /// Reserved blocks count
    r_blocks_count_hi: U32,
    /// Free blocks count
    free_blocks_count_hi: U32,
    /// All inodes have at least # bytes
    min_extra_isize: U16,
    /// New inodes should reserve # bytes
    want_extra_isize: U16,
    /// Miscellaneous flags
    flags: U32,
    /// RAID stride
    raid_stride: U16,
    /// # seconds to wait in MMP checking
    mmp_update_interval: U16,
    /// Block for multi-mount protection
    mmp_block: U64,
    /// blocks on all data disks (N*stride)
    raid_stripe_width: U32,
    /// FLEX_BG group size
    log_groups_per_flex: u8,
    /// metadata checksum algorithm used
    checksum_type: u8,
    /// versioning level for encryption
    encryption_level: u8,
    /// Padding to next 32bits
    reserved_pad: u8,
    /// nr of lifetime kilobytes written
    kbytes_written: U64,
    /// Inode number of active snapshot
    snapshot_inum: U32,
    /// sequential ID of active snapshot
    snapshot_id: U32,
    /// reserved blocks for active snapshot's future use
    snapshot_r_blocks_count: U64,
    /// inode number of the head of the on-disk snapshot list
    snapshot_list: U32,
    // #define EXT4_S_ERR_START offsetof(struct ext4_super_block,error_count)
    /// number of fs errors
    error_count: U32,
    /// first time an error happened
    first_error_time: U32,
    /// inode involved in first error
    first_error_ino: U32,
    /// block involved of first error
    first_error_block: U64,
    /// function where the error happened
    first_error_func: [u8; 32],
    /// line number where error happened
    first_error_line: U32,
    /// most recent time of an error
    last_error_time: U32,
    /// inode involved in last error
    last_error_ino: U32,
    /// line number where error happened
    last_error_line: U32,
    /// block involved of last error
    last_error_block: U64,
    /// function where the error happened
    last_error_func: [u8; 32],
    // #define EXT4_S_ERR_END offsetof(struct ext4_super_block,mount_opts)
    mount_opts: [u8; 64],
    /// inode for tracking user quota
    usr_quota_inum: U32,
    /// inode for tracking group quota
    grp_quota_inum: U32,
    /// overhead blocks/clusters in fs
    overhead_clusters: U32,
    /// groups with sparse_super2 SBs
    backup_bgs: [U32; 2],
    /// Encryption algorithms in use
    encrypt_algos: [u8; 4],
    /// Salt used for string2key algorithm
    encrypt_pw_salt: [u8; 16],
    /// Location of the lost+found inode
    lpf_ino: U32,
    /// inode for tracking project quota
    prj_quota_inum: U32,
    /// crc32c(uuid) if csum_seed set
    checksum_seed: U32,
    wtime_hi: u8,
    mtime_hi: u8,
    mkfs_time_hi: u8,
    lastcheck_hi: u8,
    first_error_time_hi: u8,
    last_error_time_hi: u8,
    pad: [u8; 2],
    /// Filename charset encoding
    encoding: U16,
    /// Filename charset encoding flags
    encoding_flags: U16,
    /// Padding to the end of the block
    reserved: [U32; 95],
    /// crc32c(Superblock)
    checksum: U32,
}

impl SuperBlock {
    pub(crate) fn read<T: Ext4Source>(source: &T) -> Result<(Self, FilePos)> {
        let mut buf = [0; SUPER_BLOCK_SIZE];
        source.read(&SUPER_BLOCK_POS, &mut buf)?;
        let super_block = SuperBlock::read_from_bytes(&buf).map_err(|err| {
            FileIoError::IoError(IoError::from_zerocopy_err(
                "failed to read super block from bytes",
                err,
            ))
        })?;

        if super_block.magic.get() != EXT4_MAGIC {
            return Err(FileIoError::Other("ext4 magic mismatch"));
        }

        Ok((super_block, SUPER_BLOCK_POS + SUPER_BLOCK_SIZE))
    }

    pub fn blocks_count(&self) -> u64 {
        u64_from_hi_lo(self.blocks_count_hi.get(), self.blocks_count_lo.get())
    }

    pub fn reserved_blocks_count(&self) -> u64 {
        u64_from_hi_lo(self.r_blocks_count_hi.get(), self.r_blocks_count_lo.get())
    }

    pub fn free_blocks_count(&self) -> u64 {
        u64_from_hi_lo(
            self.free_blocks_count_hi.get(),
            self.free_blocks_count_lo.get(),
        )
    }

    pub fn mount_time(&self) -> Result<Option<NaiveDateTime>> {
        hi_low_to_date_time(self.mtime_hi as u32, self.mtime.get())
    }

    pub fn write_time(&self) -> Result<Option<NaiveDateTime>> {
        hi_low_to_date_time(self.wtime_hi as u32, self.wtime.get())
    }

    pub fn create_time(&self) -> Result<Option<NaiveDateTime>> {
        hi_low_to_date_time(self.mkfs_time_hi as u32, self.mkfs_time.get())
    }

    pub fn last_check_time(&self) -> Result<Option<NaiveDateTime>> {
        hi_low_to_date_time(self.lastcheck_hi as u32, self.lastcheck.get())
    }

    pub fn first_error_time(&self) -> Result<Option<NaiveDateTime>> {
        hi_low_to_date_time(self.first_error_time_hi as u32, self.first_error_time.get())
    }

    pub fn last_error_time(&self) -> Result<Option<NaiveDateTime>> {
        hi_low_to_date_time(self.last_error_time_hi as u32, self.last_error_time.get())
    }

    pub fn volume_name(&self) -> Result<&CStr> {
        CStr::from_bytes_until_nul(&self.volume_name)
            .map_err(|_| FileIoError::Other("string encoding error"))
    }

    pub fn last_mounted(&self) -> Result<&CStr> {
        CStr::from_bytes_until_nul(&self.last_mounted)
            .map_err(|_| FileIoError::Other("string encoding error"))
    }

    pub fn uuid(&self) -> Uuid {
        uuid::Builder::from_bytes(self.uuid).into_uuid()
    }

    pub fn journal_uuid(&self) -> Uuid {
        uuid::Builder::from_bytes(self.journal_uuid).into_uuid()
    }
}

impl Debug for SuperBlock {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SuperBlock")
            .field("inodes_count", &self.inodes_count.get())
            .field("blocks_count", &self.blocks_count())
            .field("reserved_blocks_count", &self.reserved_blocks_count())
            .field("free_blocks_count", &self.free_blocks_count())
            .field("free_inodes_count", &self.free_inodes_count.get())
            .field("first_data_block", &self.first_data_block.get())
            .field("log_block_size", &self.log_block_size.get())
            .field("log_cluster_size", &self.log_cluster_size.get())
            .field("blocks_per_group", &self.blocks_per_group.get())
            .field("clusters_per_group", &self.clusters_per_group.get())
            .field("inodes_per_group", &self.inodes_per_group.get())
            .field("mount_time", &self.mount_time())
            .field("write_time", &self.write_time())
            .field("mnt_count", &self.mnt_count.get())
            .field("max_mnt_count", &self.max_mnt_count.get())
            .field("magic", &self.magic.get())
            .field("state", &self.state.get())
            .field("errors", &self.errors.get())
            .field("minor_rev_level", &self.minor_rev_level.get())
            .field("last_check_time", &self.last_check_time())
            .field("checkinterval", &self.checkinterval.get())
            .field("creator_os", &self.creator_os.get())
            .field("rev_level", &self.rev_level.get())
            .field("def_resuid", &self.def_resuid)
            .field("def_resgid", &self.def_resgid.get())
            .field("first_ino", &self.first_ino.get())
            .field("inode_size", &self.inode_size.get())
            .field("block_group_nr", &self.block_group_nr.get())
            .field("feature_compat", &self.feature_compat.get())
            .field("feature_incompat", &self.feature_incompat.get())
            .field("feature_ro_compat", &self.feature_ro_compat.get())
            .field("uuid", &self.uuid())
            .field("volume_name", &self.volume_name())
            .field("last_mounted", &self.last_mounted())
            .field("algorithm_usage_bitmap", &self.algorithm_usage_bitmap.get())
            .field("prealloc_blocks", &self.prealloc_blocks)
            .field("prealloc_dir_blocks", &self.prealloc_dir_blocks)
            .field("reserved_gdt_blocks", &self.reserved_gdt_blocks.get())
            .field("journal_uuid", &self.journal_uuid())
            .field("journal_inum", &self.journal_inum.get())
            .field("journal_dev", &self.journal_dev.get())
            .field("last_orphan", &self.last_orphan.get())
            .field("hash_seed", &self.hash_seed)
            .field("def_hash_version", &self.def_hash_version)
            .field("jnl_backup_type", &self.jnl_backup_type)
            .field("desc_size", &self.desc_size.get())
            .field("default_mount_opts", &self.default_mount_opts.get())
            .field("first_meta_bg", &self.first_meta_bg.get())
            .field("create_time", &self.create_time())
            .field("jnl_blocks", &self.jnl_blocks)
            .field("min_extra_isize", &self.min_extra_isize.get())
            .field("want_extra_isize", &self.want_extra_isize.get())
            .field("flags", &self.flags.get())
            .field("raid_stride", &self.raid_stride.get())
            .field("mmp_update_interval", &self.mmp_update_interval.get())
            .field("mmp_block", &self.mmp_block.get())
            .field("raid_stripe_width", &self.raid_stripe_width.get())
            .field("log_groups_per_flex", &self.log_groups_per_flex)
            .field("checksum_type", &self.checksum_type)
            .field("encryption_level", &self.encryption_level)
            .field("reserved_pad", &self.reserved_pad)
            .field("kbytes_written", &self.kbytes_written.get())
            .field("snapshot_inum", &self.snapshot_inum.get())
            .field("snapshot_id", &self.snapshot_id.get())
            .field(
                "snapshot_r_blocks_count",
                &self.snapshot_r_blocks_count.get(),
            )
            .field("snapshot_list", &self.snapshot_list.get())
            .field("error_count", &self.error_count.get())
            .field("first_error_time", &self.first_error_time())
            .field("first_error_ino", &self.first_error_ino.get())
            .field("first_error_block", &self.first_error_block.get())
            .field("first_error_func", &self.first_error_func)
            .field("first_error_line", &self.first_error_line.get())
            .field("last_error_time", &self.last_error_time())
            .field("last_error_ino", &self.last_error_ino.get())
            .field("last_error_line", &self.last_error_line.get())
            .field("last_error_block", &self.last_error_block.get())
            .field("last_error_func", &self.last_error_func)
            .field("mount_opts", &self.mount_opts)
            .field("usr_quota_inum", &self.usr_quota_inum.get())
            .field("grp_quota_inum", &self.grp_quota_inum.get())
            .field("overhead_clusters", &self.overhead_clusters.get())
            .field("backup_bgs", &self.backup_bgs)
            .field("encrypt_algos", &self.encrypt_algos)
            .field("encrypt_pw_salt", &self.encrypt_pw_salt)
            .field("lpf_ino", &self.lpf_ino.get())
            .field("prj_quota_inum", &self.prj_quota_inum.get())
            .field("checksum_seed", &self.checksum_seed.get())
            .field("pad", &self.pad)
            .field("encoding", &self.encoding.get())
            .field("encoding_flags", &self.encoding_flags.get())
            .field("checksum", &self.checksum.get())
            .finish()
    }
}
