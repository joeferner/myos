use core::fmt::Debug;

use chrono::{DateTime, NaiveDateTime};
use file_io::{FileIoError, FilePos, Result};
use io::IoError;
use zerocopy::{
    FromBytes, Immutable, IntoBytes, KnownLayout,
    little_endian::{U16, U32, U64},
};

use crate::{source::Ext4Source, utils::from_hi_lo};

pub(crate) const SUPER_BLOCK_SIZE: usize = core::mem::size_of::<SuperBlock>();
pub(crate) const SUPER_BLOCK_POS: FilePos = FilePos(0x400);
pub(crate) const EXT4_MAGIC: u16 = 0xef53;

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
    log_cluster_size: U32,  /* Allocation cluster size */
    /*20*/
    blocks_per_group: U32,   /* # Blocks per group */
    clusters_per_group: U32, /* # Clusters per group */
    inodes_per_group: U32,   /* # Inodes per group */
    mtime: U32,              /* Mount time */
    /*30*/
    wtime: U32,           /* Write time */
    mnt_count: U16,       /* Mount count */
    max_mnt_count: U16,   /* Maximal mount count */
    magic: U16,           /* Magic signature */
    state: U16,           /* File system state */
    errors: U16,          /* Behaviour when detecting errors */
    minor_rev_level: U16, /* minor revision level */
    /*40*/
    lastcheck: U32,     /* time of last check */
    checkinterval: U32, /* max. time between checks */
    creator_os: U32,    /* OS */
    rev_level: U32,     /* Revision level */
    /*50*/
    def_resuid: U16, /* Default uid for reserved blocks */
    def_resgid: U16, /* Default gid for reserved blocks */
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
    first_ino: U32,      /* First non-reserved inode */
    inode_size: U16,     /* size of inode structure */
    block_group_nr: U16, /* block group # of this Superblock */
    feature_compat: U32, /* compatible feature set */
    /*60*/
    feature_incompat: U32,  /* incompatible feature set */
    feature_ro_compat: U32, /* readonly-compatible feature set */
    /*68*/
    uuid: [u8; 16], /* 128-bit uuid for volume */
    /*78*/
    volume_name: [u8; 16], /* volume name */
    /*88*/
    last_mounted: [u8; 64], /* directory where last mounted */
    /*C8*/
    algorithm_usage_bitmap: U32, /* For compression */
    /*
     * Performance hints.  Directory preallocation should only
     * happen if the EXT4_FEATURE_COMPAT_DIR_PREALLOC flag is on.
     */
    prealloc_blocks: u8,      /* Nr of blocks to try to preallocate*/
    prealloc_dir_blocks: u8,  /* Nr to preallocate for dirs */
    reserved_gdt_blocks: U16, /* Per group desc for online growth */
    /*
     * Journaling support valid if EXT4_FEATURE_COMPAT_HAS_JOURNAL set.
     */
    /*D0*/
    journal_uuid: [u8; 16], /* uuid of journal Superblock */
    /*E0*/
    journal_inum: U32,    /* inode number of journal file */
    journal_dev: U32,     /* device number of journal file */
    last_orphan: U32,     /* start of list of inodes to delete */
    hash_seed: [U32; 4],  /* HTREE hash seed */
    def_hash_version: u8, /* Default hash version to use */
    jnl_backup_type: u8,
    desc_size: U16, /* size of group descriptor */
    /*100*/
    default_mount_opts: U32,
    first_meta_bg: U32,    /* First metablock block group */
    mkfs_time: U32,        /* When the filesystem was created */
    jnl_blocks: [U32; 17], /* Backup of the journal inode */
    /* 64bit support valid if EXT4_FEATURE_COMPAT_64BIT */
    /*150*/
    blocks_count_hi: U32,      /* Blocks count */
    r_blocks_count_hi: U32,    /* Reserved blocks count */
    free_blocks_count_hi: U32, /* Free blocks count */
    min_extra_isize: U16,      /* All inodes have at least # bytes */
    want_extra_isize: U16,     /* New inodes should reserve # bytes */
    flags: U32,                /* Miscellaneous flags */
    raid_stride: U16,          /* RAID stride */
    mmp_update_interval: U16,  /* # seconds to wait in MMP checking */
    mmp_block: U64,            /* Block for multi-mount protection */
    raid_stripe_width: U32,    /* blocks on all data disks (N*stride)*/
    log_groups_per_flex: u8,   /* FLEX_BG group size */
    checksum_type: u8,         /* metadata checksum algorithm used */
    encryption_level: u8,      /* versioning level for encryption */
    reserved_pad: u8,          /* Padding to next 32bits */
    kbytes_written: U64,       /* nr of lifetime kilobytes written */
    snapshot_inum: U32,        /* Inode number of active snapshot */
    snapshot_id: U32,          /* sequential ID of active snapshot */
    snapshot_r_blocks_count: U64, /* reserved blocks for active
                               snapshot's future use */
    snapshot_list: U32, /* inode number of the head of the on-disk snapshot list */
    // #define EXT4_S_ERR_START offsetof(struct ext4_super_block,error_count)
    error_count: U32,           /* number of fs errors */
    first_error_time: U32,      /* first time an error happened */
    first_error_ino: U32,       /* inode involved in first error */
    first_error_block: U64,     /* block involved of first error */
    first_error_func: [u8; 32], /* function where the error happened */
    first_error_line: U32,      /* line number where error happened */
    last_error_time: U32,       /* most recent time of an error */
    last_error_ino: U32,        /* inode involved in last error */
    last_error_line: U32,       /* line number where error happened */
    last_error_block: U64,      /* block involved of last error */
    last_error_func: [u8; 32],  /* function where the error happened */
    // #define EXT4_S_ERR_END offsetof(struct ext4_super_block,mount_opts)
    mount_opts: [u8; 64],
    usr_quota_inum: U32,       /* inode for tracking user quota */
    grp_quota_inum: U32,       /* inode for tracking group quota */
    overhead_clusters: U32,    /* overhead blocks/clusters in fs */
    backup_bgs: [U32; 2],      /* groups with sparse_super2 SBs */
    encrypt_algos: [u8; 4],    /* Encryption algorithms in use  */
    encrypt_pw_salt: [u8; 16], /* Salt used for string2key algorithm */
    lpf_ino: U32,              /* Location of the lost+found inode */
    prj_quota_inum: U32,       /* inode for tracking project quota */
    checksum_seed: U32,        /* crc32c(uuid) if csum_seed set */
    wtime_hi: u8,
    mtime_hi: u8,
    mkfs_time_hi: u8,
    lastcheck_hi: u8,
    first_error_time_hi: u8,
    last_error_time_hi: u8,
    pad: [u8; 2],
    encoding: U16,       /* Filename charset encoding */
    encoding_flags: U16, /* Filename charset encoding flags */
    reserved: [U32; 95], /* Padding to the end of the block */
    checksum: U32,       /* crc32c(Superblock) */
}

impl SuperBlock {
    pub(crate) fn read<T: Ext4Source>(source: &T) -> Result<Self> {
        let mut buf = [0; SUPER_BLOCK_SIZE];
        source.read(SUPER_BLOCK_POS, &mut buf)?;
        let super_block = SuperBlock::read_from_bytes(&buf).map_err(|err| {
            FileIoError::IoError(IoError::from_zerocopy_err(
                "failed to read super block from bytes",
                err,
            ))
        })?;

        if super_block.magic.get() != EXT4_MAGIC {
            return Err(FileIoError::Other("ext4 magic mismatch"));
        }

        let super_block: SuperBlock = super_block.into();
        Ok(super_block)
    }

    pub fn blocks_count(&self) -> u64 {
        from_hi_lo(self.blocks_count_hi.get(), self.blocks_count_lo.get())
    }

    pub fn reserved_blocks_count(&self) -> u64 {
        from_hi_lo(self.r_blocks_count_hi.get(), self.r_blocks_count_lo.get())
    }

    pub fn free_blocks_count(&self) -> u64 {
        from_hi_lo(
            self.free_blocks_count_hi.get(),
            self.free_blocks_count_lo.get(),
        )
    }

    pub fn mount_time(&self) -> Result<NaiveDateTime> {
        hi_low_to_date_time(self.mtime_hi as u32, self.mtime.get())
    }

    pub fn write_time(&self) -> Result<NaiveDateTime> {
        hi_low_to_date_time(self.wtime_hi as u32, self.wtime.get())
    }
}

fn hi_low_to_date_time(hi: u32, lo: u32) -> Result<NaiveDateTime> {
    let ms: i64 = (from_hi_lo(hi, lo) * 1000)
        .try_into()
        .map_err(|_| FileIoError::Other("invalid time"))?;
    Ok(DateTime::from_timestamp_millis(ms)
        .ok_or_else(|| FileIoError::Other("invalid time"))?
        .naive_utc())
}

impl Debug for SuperBlock {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SuperBlock")
            .field("inodes_count", &self.inodes_count)
            .field("blocks_count", &self.blocks_count())
            .field("reserved_blocks_count", &self.reserved_blocks_count())
            .field("free_blocks_count", &self.free_blocks_count())
            .field("free_inodes_count", &self.free_inodes_count)
            .field("first_data_block", &self.first_data_block)
            .field("log_block_size", &self.log_block_size)
            .field("log_cluster_size", &self.log_cluster_size)
            .field("blocks_per_group", &self.blocks_per_group)
            .field("clusters_per_group", &self.clusters_per_group)
            .field("inodes_per_group", &self.inodes_per_group)
            .field("mount_time", &self.mount_time())
            .field("write_time", &self.write_time())
            .field("mnt_count", &self.mnt_count)
            .field("max_mnt_count", &self.max_mnt_count)
            .field("magic", &self.magic)
            .field("state", &self.state)
            .field("errors", &self.errors)
            .field("minor_rev_level", &self.minor_rev_level)
            .field("lastcheck", &self.lastcheck)
            .field("checkinterval", &self.checkinterval)
            .field("creator_os", &self.creator_os)
            .field("rev_level", &self.rev_level)
            .field("def_resuid", &self.def_resuid)
            .field("def_resgid", &self.def_resgid)
            .field("first_ino", &self.first_ino)
            .field("inode_size", &self.inode_size)
            .field("block_group_nr", &self.block_group_nr)
            .field("feature_compat", &self.feature_compat)
            .field("feature_incompat", &self.feature_incompat)
            .field("feature_ro_compat", &self.feature_ro_compat)
            .field("uuid", &self.uuid)
            .field("volume_name", &self.volume_name)
            .field("last_mounted", &self.last_mounted)
            .field("algorithm_usage_bitmap", &self.algorithm_usage_bitmap)
            .field("prealloc_blocks", &self.prealloc_blocks)
            .field("prealloc_dir_blocks", &self.prealloc_dir_blocks)
            .field("reserved_gdt_blocks", &self.reserved_gdt_blocks)
            .field("journal_uuid", &self.journal_uuid)
            .field("journal_inum", &self.journal_inum)
            .field("journal_dev", &self.journal_dev)
            .field("last_orphan", &self.last_orphan)
            .field("hash_seed", &self.hash_seed)
            .field("def_hash_version", &self.def_hash_version)
            .field("jnl_backup_type", &self.jnl_backup_type)
            .field("desc_size", &self.desc_size)
            .field("default_mount_opts", &self.default_mount_opts)
            .field("first_meta_bg", &self.first_meta_bg)
            .field("mkfs_time", &self.mkfs_time)
            .field("jnl_blocks", &self.jnl_blocks)
            .field("min_extra_isize", &self.min_extra_isize)
            .field("want_extra_isize", &self.want_extra_isize)
            .field("flags", &self.flags)
            .field("raid_stride", &self.raid_stride)
            .field("mmp_update_interval", &self.mmp_update_interval)
            .field("mmp_block", &self.mmp_block)
            .field("raid_stripe_width", &self.raid_stripe_width)
            .field("log_groups_per_flex", &self.log_groups_per_flex)
            .field("checksum_type", &self.checksum_type)
            .field("encryption_level", &self.encryption_level)
            .field("reserved_pad", &self.reserved_pad)
            .field("kbytes_written", &self.kbytes_written)
            .field("snapshot_inum", &self.snapshot_inum)
            .field("snapshot_id", &self.snapshot_id)
            .field("snapshot_r_blocks_count", &self.snapshot_r_blocks_count)
            .field("snapshot_list", &self.snapshot_list)
            .field("error_count", &self.error_count)
            .field("first_error_time", &self.first_error_time)
            .field("first_error_ino", &self.first_error_ino)
            .field("first_error_block", &self.first_error_block)
            .field("first_error_func", &self.first_error_func)
            .field("first_error_line", &self.first_error_line)
            .field("last_error_time", &self.last_error_time)
            .field("last_error_ino", &self.last_error_ino)
            .field("last_error_line", &self.last_error_line)
            .field("last_error_block", &self.last_error_block)
            .field("last_error_func", &self.last_error_func)
            .field("mount_opts", &self.mount_opts)
            .field("usr_quota_inum", &self.usr_quota_inum)
            .field("grp_quota_inum", &self.grp_quota_inum)
            .field("overhead_clusters", &self.overhead_clusters)
            .field("backup_bgs", &self.backup_bgs)
            .field("encrypt_algos", &self.encrypt_algos)
            .field("encrypt_pw_salt", &self.encrypt_pw_salt)
            .field("lpf_ino", &self.lpf_ino)
            .field("prj_quota_inum", &self.prj_quota_inum)
            .field("checksum_seed", &self.checksum_seed)
            .field("mkfs_time_hi", &self.mkfs_time_hi)
            .field("lastcheck_hi", &self.lastcheck_hi)
            .field("first_error_time_hi", &self.first_error_time_hi)
            .field("last_error_time_hi", &self.last_error_time_hi)
            .field("pad", &self.pad)
            .field("encoding", &self.encoding)
            .field("encoding_flags", &self.encoding_flags)
            .field("reserved", &self.reserved)
            .field("checksum", &self.checksum)
            .finish()
    }
}
