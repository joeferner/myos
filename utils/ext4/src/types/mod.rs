use file_io::FilePos;

pub(crate) mod bitmap;
pub(crate) mod block_group_descriptor;
pub(crate) mod inode;
pub(crate) mod super_block;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct BlockIndex(pub u64);

impl BlockIndex {
    pub(crate) fn to_file_pos(&self, block_size: u32) -> FilePos {
        FilePos(self.0 * block_size as u64)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct INodeIndex(u32);

impl INodeIndex {
    pub(crate) fn new(v: u32) -> Self {
        Self(v)
    }

    pub(crate) fn root() -> Self {
        INodeIndex(2)
    }

    /// inodes start at 1. 0 is used as a sentinel value to indicate null or no inode.
    pub(crate) fn real_index(&self) -> u32 {
        self.0 - 1
    }
}
