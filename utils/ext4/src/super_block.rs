use file_io::{FileIoError, FilePos, Result};
use io::IoError;
use zerocopy::{
    FromBytes, Immutable, IntoBytes, KnownLayout,
    little_endian::{U16, U32, U64},
};

use crate::{source::Ext4Source, utils::from_hi_lo};

pub(crate) const SUPER_BLOCK_SIZE: usize = core::mem::size_of::<PhysicalSuperBlock>();
pub(crate) const SUPER_BLOCK_POS: FilePos = FilePos(0x400);
pub(crate) const EXT4_MAGIC: u16 = 0xef53;

#[repr(C, packed)]
#[derive(Debug, Clone, IntoBytes, FromBytes, Immutable, KnownLayout)]
pub(crate) struct PhysicalSuperBlock {
    /*00*/
    s_inodes_count: U32,         /* Inodes count */
    s_blocks_count_lo: U32,      /* Blocks count */
    s_r_blocks_count_lo: U32,    /* Reserved blocks count */
    s_free_blocks_count_lo: U32, /* Free blocks count */
    /*10*/
    s_free_inodes_count: U32, /* Free inodes count */
    s_first_data_block: U32,  /* First Data Block */
    s_log_block_size: U32,    /* Block size */
    s_log_cluster_size: U32,  /* Allocation cluster size */
    /*20*/
    s_blocks_per_group: U32,   /* # Blocks per group */
    s_clusters_per_group: U32, /* # Clusters per group */
    s_inodes_per_group: U32,   /* # Inodes per group */
    s_mtime: U32,              /* Mount time */
    /*30*/
    s_wtime: U32,           /* Write time */
    s_mnt_count: U16,       /* Mount count */
    s_max_mnt_count: U16,   /* Maximal mount count */
    s_magic: U16,           /* Magic signature */
    s_state: U16,           /* File system state */
    s_errors: U16,          /* Behaviour when detecting errors */
    s_minor_rev_level: U16, /* minor revision level */
    /*40*/
    s_lastcheck: U32,     /* time of last check */
    s_checkinterval: U32, /* max. time between checks */
    s_creator_os: U32,    /* OS */
    s_rev_level: U32,     /* Revision level */
    /*50*/
    s_def_resuid: U16, /* Default uid for reserved blocks */
    s_def_resgid: U16, /* Default gid for reserved blocks */
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
    s_first_ino: U32,      /* First non-reserved inode */
    s_inode_size: U16,     /* size of inode structure */
    s_block_group_nr: U16, /* block group # of this Superblock */
    s_feature_compat: U32, /* compatible feature set */
    /*60*/
    s_feature_incompat: U32,  /* incompatible feature set */
    s_feature_ro_compat: U32, /* readonly-compatible feature set */
    /*68*/
    s_uuid: [u8; 16], /* 128-bit uuid for volume */
    /*78*/
    s_volume_name: [u8; 16], /* volume name */
    /*88*/
    s_last_mounted: [u8; 64], /* directory where last mounted */
    /*C8*/
    s_algorithm_usage_bitmap: U32, /* For compression */
    /*
     * Performance hints.  Directory preallocation should only
     * happen if the EXT4_FEATURE_COMPAT_DIR_PREALLOC flag is on.
     */
    s_prealloc_blocks: u8,      /* Nr of blocks to try to preallocate*/
    s_prealloc_dir_blocks: u8,  /* Nr to preallocate for dirs */
    s_reserved_gdt_blocks: U16, /* Per group desc for online growth */
    /*
     * Journaling support valid if EXT4_FEATURE_COMPAT_HAS_JOURNAL set.
     */
    /*D0*/
    s_journal_uuid: [u8; 16], /* uuid of journal Superblock */
    /*E0*/
    s_journal_inum: U32,    /* inode number of journal file */
    s_journal_dev: U32,     /* device number of journal file */
    s_last_orphan: U32,     /* start of list of inodes to delete */
    s_hash_seed: [U32; 4],  /* HTREE hash seed */
    s_def_hash_version: u8, /* Default hash version to use */
    s_jnl_backup_type: u8,
    s_desc_size: U16, /* size of group descriptor */
    /*100*/
    s_default_mount_opts: U32,
    s_first_meta_bg: U32,    /* First metablock block group */
    s_mkfs_time: U32,        /* When the filesystem was created */
    s_jnl_blocks: [U32; 17], /* Backup of the journal inode */
    /* 64bit support valid if EXT4_FEATURE_COMPAT_64BIT */
    /*150*/
    s_blocks_count_hi: U32,      /* Blocks count */
    s_r_blocks_count_hi: U32,    /* Reserved blocks count */
    s_free_blocks_count_hi: U32, /* Free blocks count */
    s_min_extra_isize: U16,      /* All inodes have at least # bytes */
    s_want_extra_isize: U16,     /* New inodes should reserve # bytes */
    s_flags: U32,                /* Miscellaneous flags */
    s_raid_stride: U16,          /* RAID stride */
    s_mmp_update_interval: U16,  /* # seconds to wait in MMP checking */
    s_mmp_block: U64,            /* Block for multi-mount protection */
    s_raid_stripe_width: U32,    /* blocks on all data disks (N*stride)*/
    s_log_groups_per_flex: u8,   /* FLEX_BG group size */
    s_checksum_type: u8,         /* metadata checksum algorithm used */
    s_encryption_level: u8,      /* versioning level for encryption */
    s_reserved_pad: u8,          /* Padding to next 32bits */
    s_kbytes_written: U64,       /* nr of lifetime kilobytes written */
    s_snapshot_inum: U32,        /* Inode number of active snapshot */
    s_snapshot_id: U32,          /* sequential ID of active snapshot */
    s_snapshot_r_blocks_count: U64, /* reserved blocks for active
                                 snapshot's future use */
    s_snapshot_list: U32, /* inode number of the head of the on-disk snapshot list */
    // #define EXT4_S_ERR_START offsetof(struct ext4_super_block, s_error_count)
    s_error_count: U32,           /* number of fs errors */
    s_first_error_time: U32,      /* first time an error happened */
    s_first_error_ino: U32,       /* inode involved in first error */
    s_first_error_block: U64,     /* block involved of first error */
    s_first_error_func: [u8; 32], /* function where the error happened */
    s_first_error_line: U32,      /* line number where error happened */
    s_last_error_time: U32,       /* most recent time of an error */
    s_last_error_ino: U32,        /* inode involved in last error */
    s_last_error_line: U32,       /* line number where error happened */
    s_last_error_block: U64,      /* block involved of last error */
    s_last_error_func: [u8; 32],  /* function where the error happened */
    // #define EXT4_S_ERR_END offsetof(struct ext4_super_block, s_mount_opts)
    s_mount_opts: [u8; 64],
    s_usr_quota_inum: U32,       /* inode for tracking user quota */
    s_grp_quota_inum: U32,       /* inode for tracking group quota */
    s_overhead_clusters: U32,    /* overhead blocks/clusters in fs */
    s_backup_bgs: [U32; 2],      /* groups with sparse_super2 SBs */
    s_encrypt_algos: [u8; 4],    /* Encryption algorithms in use  */
    s_encrypt_pw_salt: [u8; 16], /* Salt used for string2key algorithm */
    s_lpf_ino: U32,              /* Location of the lost+found inode */
    s_prj_quota_inum: U32,       /* inode for tracking project quota */
    s_checksum_seed: U32,        /* crc32c(uuid) if csum_seed set */
    s_wtime_hi: u8,
    s_mtime_hi: u8,
    s_mkfs_time_hi: u8,
    s_lastcheck_hi: u8,
    s_first_error_time_hi: u8,
    s_last_error_time_hi: u8,
    s_pad: [u8; 2],
    s_encoding: U16,       /* Filename charset encoding */
    s_encoding_flags: U16, /* Filename charset encoding flags */
    s_reserved: [U32; 95], /* Padding to the end of the block */
    s_checksum: U32,       /* crc32c(Superblock) */
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct SuperBlock {
    pub inodes_count: u32,
    pub blocks_count: u64,
    pub reserved_blocks_count: u64,
    pub free_blocks_count: u64,
    pub free_inodes_count: u32,
    pub first_data_block: u32,
    pub log_block_size: u32,
    pub log_cluster_size: u32,
    pub blocks_per_group: u32,
    pub clusters_per_group: u32,
    pub inodes_per_group: u32,
    pub mount_time: u32,
}

impl SuperBlock {
    pub(crate) fn read<T: Ext4Source>(source: &T) -> Result<SuperBlock> {
        let mut buf = [0; SUPER_BLOCK_SIZE];
        source.read(SUPER_BLOCK_POS, &mut buf)?;
        let super_block = PhysicalSuperBlock::read_from_bytes(&buf).map_err(|err| {
            FileIoError::IoError(IoError::from_zerocopy_err(
                "failed to read super block from bytes",
                err,
            ))
        })?;

        if super_block.s_magic.get() != EXT4_MAGIC {
            return Err(FileIoError::Other("ext4 magic mismatch"));
        }

        let super_block: SuperBlock = super_block.into();
        Ok(super_block)
    }
}

impl From<PhysicalSuperBlock> for SuperBlock {
    fn from(value: PhysicalSuperBlock) -> Self {
        Self {
            inodes_count: value.s_inodes_count.get(),
            blocks_count: from_hi_lo(value.s_blocks_count_hi.get(), value.s_blocks_count_lo.get()),
            reserved_blocks_count: from_hi_lo(
                value.s_r_blocks_count_hi.get(),
                value.s_r_blocks_count_lo.get(),
            ),
            free_blocks_count: from_hi_lo(
                value.s_free_blocks_count_hi.get(),
                value.s_free_blocks_count_lo.get(),
            ),
            free_inodes_count: value.s_free_inodes_count.get(),
            first_data_block: value.s_first_data_block.get(),
            log_block_size: value.s_log_block_size.get(),
            log_cluster_size: value.s_log_cluster_size.get(),
            blocks_per_group: value.s_blocks_per_group.get(),
            clusters_per_group: value.s_clusters_per_group.get(),
            inodes_per_group: value.s_inodes_per_group.get(),
            mount_time: value.s_mtime.get(),
        }
    }
}
