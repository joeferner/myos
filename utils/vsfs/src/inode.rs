use file_io::{FilePos, Mode, TimeSeconds};
use myos_api::Uid;

use crate::{
    DataBlockIndex,
    physical::{IMMEDIATE_BLOCK_COUNT, PhysicalINode},
};

#[derive(Debug, Clone)]
pub(crate) struct INode {
    pub uid: Uid,
    pub gid: Uid,
    pub mode: Mode,
    /// size of the file
    pub size: FilePos,
    /// what time was this file last accessed?
    pub time: TimeSeconds,
    /// what time was this file created?
    pub ctime: TimeSeconds,
    /// what time was this file last modified?
    pub mtime: TimeSeconds,
    /// index into the blocks where the first x blocks of data can be found, 0 indicates unused block
    pub blocks: [Option<DataBlockIndex>; IMMEDIATE_BLOCK_COUNT],
    /// if not 0, indicates an index into the block table where you will find more block addresses
    pub indirect_block_idx: Option<DataBlockIndex>,
}

impl INode {
    pub(crate) fn new(mode: Mode, time: TimeSeconds) -> Self {
        const BLOCK_NONE: Option<DataBlockIndex> = None;
        Self {
            uid: Uid::root(),
            gid: Uid::root(),
            mode,
            size: FilePos(0),
            time,
            ctime: time,
            mtime: time,
            blocks: [BLOCK_NONE; IMMEDIATE_BLOCK_COUNT],
            indirect_block_idx: None,
        }
    }
}

impl From<PhysicalINode> for INode {
    fn from(value: PhysicalINode) -> Self {
        let mut blocks = [None; IMMEDIATE_BLOCK_COUNT];
        for i in 0..IMMEDIATE_BLOCK_COUNT {
            blocks[i] = DataBlockIndex::from_u32(value.blocks[i]);
        }

        Self {
            uid: Uid(value.uid),
            gid: Uid(value.gid),
            mode: Mode(value.mode),
            size: FilePos(value.size),
            time: TimeSeconds(value.time),
            ctime: TimeSeconds(value.ctime),
            mtime: TimeSeconds(value.mtime),
            blocks,
            indirect_block_idx: DataBlockIndex::from_u32(value.indirect_block_idx),
        }
    }
}
