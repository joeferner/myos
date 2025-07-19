use file_io::{FilePos, Mode, TimeSeconds};
use myos_api::{ROOT_UID, Uid};

use crate::{DataBlockIndex, physical::IMMEDIATE_BLOCK_COUNT};

pub(crate) struct INode {
    uid: Uid,
    gid: Uid,
    mode: Mode,
    /// size of the file
    size: FilePos,
    /// what time was this file last accessed?
    time: TimeSeconds,
    /// what time was this file created?
    ctime: TimeSeconds,
    /// what time was this file last modified?
    mtime: TimeSeconds,
    /// index into the blocks where the first x blocks of data can be found, 0 indicates unused block
    blocks: [Option<DataBlockIndex>; IMMEDIATE_BLOCK_COUNT],
    /// if not 0, indicates an index into the block table where you will find more block addresses
    indirect_block_idx: Option<DataBlockIndex>,
}

impl INode {
    pub(crate) fn new(mode: Mode, time: TimeSeconds) -> Self {
        const BLOCK_NONE: Option<DataBlockIndex> = None;
        Self {
            uid: ROOT_UID,
            gid: ROOT_UID,
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
