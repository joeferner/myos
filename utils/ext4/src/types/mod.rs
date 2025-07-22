pub(crate) mod super_block;
pub(crate) mod block_group_descriptor;
pub(crate) mod inode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct BlockIndex(pub u64);

impl BlockIndex {
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct INodeIndex(pub u32);

impl INodeIndex {
    pub(crate) fn root() -> Self {
        INodeIndex(2)
    }
}
