use file_io::FilePos;

pub(crate) mod super_block;
pub(crate) mod block_group_descriptor;
pub(crate) mod inode;
pub(crate) mod bitmap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct BlockIndex(pub u64);

impl BlockIndex {
    pub(crate) fn to_file_pos(&self, block_size: u32) -> FilePos {
        FilePos(self.0 * block_size as u64)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct INodeIndex(pub u32);

impl INodeIndex {
    pub(crate) fn root() -> Self {
        INodeIndex(2)
    }
}
