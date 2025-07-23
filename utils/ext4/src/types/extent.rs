use core::fmt::Debug;
use zerocopy::{
    FromBytes, Immutable, IntoBytes, KnownLayout,
    little_endian::{U16, U32},
};

use crate::utils::u64_from_hi_lo;

pub(crate) const EXTENT_HEADER_SIZE: usize = core::mem::size_of::<ExtentHeader>();
pub(crate) const EXTENT_HEADER_MAGIC: u16 = 0xf30a;

#[repr(C, packed)]
#[derive(Clone, IntoBytes, FromBytes, Immutable, KnownLayout)]
pub(crate) struct ExtentHeader {
    /// Magic number, 0xF30A
    pub magic: U16,
    /// Number of valid entries following the header
    pub entries: U16,
    /// Maximum number of entries that could follow the header
    pub max: U16,
    /// Depth of this extent node in the extent tree. 0 = this extent node
    /// points to data blocks; otherwise, this extent node points to other
    /// extent nodes. The extent tree can be at most 5 levels deep: a logical
    /// block number can be at most 2^32, and the smallest n that satisfies
    /// 4*(((blocksize - 12)/12)^n) >= 2^32 is 5.
    pub depth: U16,
    /// Generation of the tree. (Used by Lustre, but not standard ext4).
    pub generation: U32,
}

impl Debug for ExtentHeader {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ExtentHeader")
            .field("magic", &self.magic.get())
            .field("entries", &self.entries.get())
            .field("max", &self.max.get())
            .field("depth", &self.depth.get())
            .field("generation", &self.generation.get())
            .finish()
    }
}

pub(crate) const EXTENT_SIZE: usize = core::mem::size_of::<Extent>();

#[repr(C, packed)]
#[derive(Clone, IntoBytes, FromBytes, Immutable, KnownLayout)]
pub(crate) struct Extent {
    /// First file block number that this extent covers
    pub block: U32,
    /// Number of blocks covered by extent. If the value of this field
    /// is <= 32768, the extent is initialized. If the value of the field
    /// is > 32768, the extent is uninitialized and the actual extent length
    /// is ee_len - 32768. Therefore, the maximum length of a initialized
    /// extent is 32768 blocks, and the maximum length of an uninitialized
    /// extent is 32767.
    pub len: U16,

    /// Upper 16-bits of the block number to which this extent points.
    start_hi: U16,
    /// Lower 32-bits of the block number to which this extent points.
    start_lo: U32,
}

impl Extent {
    pub fn start(&self) -> u64 {
        u64_from_hi_lo(self.start_hi.get() as u32, self.start_lo.get())
    }
}

impl Debug for Extent {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Extent")
            .field("block", &self.block.get())
            .field("len", &self.len.get())
            .field("start", &self.start())
            .finish()
    }
}
