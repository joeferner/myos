use file_io::Result;

use crate::{
    MAX_BLOCK_SIZE,
    source::Ext4Source,
    types::{BlockIndex, INodeIndex},
};

#[repr(C, packed)]
#[derive(Clone, Debug)]
pub(crate) struct Bitmap {
    block_size: u32,
    block: [u8; MAX_BLOCK_SIZE],
}

impl Bitmap {
    pub(crate) fn read<T: Ext4Source>(
        source: &T,
        block_bitmap_block_idx: &BlockIndex,
        block_size: u32,
    ) -> Result<Bitmap> {
        let mut block: [u8; MAX_BLOCK_SIZE] = [0; MAX_BLOCK_SIZE];
        let file_pos = block_bitmap_block_idx.to_file_pos(block_size);
        source.read(&file_pos, &mut block)?;
        Ok(Bitmap { block_size, block })
    }

    pub(crate) fn is_readable(&self, relative_inode_idx: INodeIndex) -> bool {
        let idx = relative_inode_idx.0 / 8;
        if idx >= self.block_size {
            return false;
        }
        let bit = relative_inode_idx.0 % 8;
        let b = self.block[idx as usize];
        (b >> bit) & 1 == 1
    }
}
