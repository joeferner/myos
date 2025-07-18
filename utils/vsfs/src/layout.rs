use crate::{utils::div_ceil, Addr, BlockIndex, Error, INodeIndex, Result, BLOCK_SIZE, INODES_PER_BLOCK, INODE_SIZE};

pub(crate) struct Layout {
    pub inode_count: u32,
    pub data_block_count: u32,
    pub inode_bitmap_block_count: u32,
    pub data_bitmap_block_count: u32,
    pub inode_block_count: u32,
    pub inode_bitmap_offset: Addr,
    pub data_bitmap_offset: Addr,
    pub inode_offset: Addr,
    pub data_offset: Addr,
}

impl Layout {
    pub(crate) fn new(inode_count: u32, data_block_count: u32) -> Self {
        let inode_bitmap_block_count = div_ceil(div_ceil(inode_count, 8), BLOCK_SIZE as u32);
        let data_bitmap_block_count = div_ceil(div_ceil(data_block_count, 8), BLOCK_SIZE as u32);
        let inode_block_count = div_ceil(inode_count, INODES_PER_BLOCK as u32);

        let inode_bitmap_offset = BLOCK_SIZE as Addr;
        let data_bitmap_offset =
            inode_bitmap_offset + (inode_bitmap_block_count as Addr * BLOCK_SIZE as Addr);
        let inode_offset =
            data_bitmap_offset + (data_bitmap_block_count as Addr * BLOCK_SIZE as Addr);
        let data_offset = inode_offset + (inode_block_count as Addr * BLOCK_SIZE as Addr);

        Self {
            inode_count,
            data_block_count,
            inode_bitmap_block_count,
            data_bitmap_block_count,
            inode_block_count,
            inode_bitmap_offset,
            data_bitmap_offset,
            inode_offset,
            data_offset,
        }
    }

    /// returns the address of the block containing the inode bitmap along with the offset
    /// within the block where to find the inode bitmap data along with the bit number of
    /// inode
    pub(crate) fn calc_inode_bitmap_addr(&self, inode_idx: INodeIndex) -> Result<(Addr, usize, u8)> {
        if inode_idx >= self.inode_count {
            return Err(Error::INodeIndexOutOfRange);
        }

        let bit = (inode_idx % 8) as u8;
        let idx = inode_idx / 8;

        let offset = idx as usize % BLOCK_SIZE;

        let count = idx as Addr / BLOCK_SIZE as Addr;
        let addr = self.inode_bitmap_offset + (count * BLOCK_SIZE as Addr);

        Ok((addr, offset, bit))
    }

    /// returns the address of the block containing the inode along with the offset
    /// within the block where to find the inode data
    pub(crate) fn calc_inode_block_addr(&self, inode_idx: INodeIndex) -> Result<(Addr, usize)> {
        if inode_idx >= self.inode_count {
            return Err(Error::INodeIndexOutOfRange);
        }

        let block_offset = (inode_idx % INODES_PER_BLOCK) as usize * INODE_SIZE;

        let count = inode_idx / INODES_PER_BLOCK;
        let block_addr = self.inode_offset + (count as Addr * BLOCK_SIZE as Addr);

        Ok((block_addr, block_offset))
    }

    /// returns the address of the block containing the data bitmap along with the offset
    /// within the block where to find the data bitmap data along with the bit number of
    /// data
    pub(crate) fn calc_data_bitmap_addr(&self, data_block_idx: BlockIndex) -> Result<(Addr, usize, u8)> {
        if data_block_idx >= self.data_block_count {
            return Err(Error::INodeIndexOutOfRange);
        }

        let bit = (data_block_idx % 8) as u8;
        let idx = data_block_idx / 8;

        let offset = idx as usize % BLOCK_SIZE;

        let count = idx as Addr / BLOCK_SIZE as Addr;
        let addr = self.data_bitmap_offset + (count * BLOCK_SIZE as Addr);

        Ok((addr, offset, bit))
    }

    /// returns the address of the data block
    pub(crate) fn calc_data_addr(&self, data_block_idx: BlockIndex) -> Result<Addr> {
        if data_block_idx >= self.data_block_count {
            return Err(Error::INodeIndexOutOfRange);
        }

        let addr = self.data_offset + (data_block_idx as Addr * BLOCK_SIZE as Addr);

        Ok(addr)
    }
}

#[cfg(test)]
mod tests {
    use crate::Error;

    use super::*;

    #[test]
    pub fn test_calc_inode_bitmap_addr() {
        let inodes_count = BLOCK_SIZE as u32 * 8 + 100;
        let layout = Layout::new(inodes_count, 1);
        let inode_bits_per_block = BLOCK_SIZE as u32 * 8;

        assert_eq!(
            (layout.inode_bitmap_offset, 0, 0),
            layout.calc_inode_bitmap_addr(0).unwrap()
        );

        assert_eq!(
            (layout.inode_bitmap_offset, 1, 0),
            layout.calc_inode_bitmap_addr(8).unwrap()
        );

        assert_eq!(
            (layout.inode_bitmap_offset, 1, 1),
            layout.calc_inode_bitmap_addr(9).unwrap()
        );

        assert_eq!(
            (layout.inode_bitmap_offset, BLOCK_SIZE - 1, 7),
            layout
                .calc_inode_bitmap_addr(inode_bits_per_block - 1)
                .unwrap()
        );

        assert_eq!(
            (layout.inode_bitmap_offset + BLOCK_SIZE as Addr, 0, 0),
            layout.calc_inode_bitmap_addr(inode_bits_per_block).unwrap()
        );

        assert_eq!(
            (layout.inode_bitmap_offset + BLOCK_SIZE as Addr, 12, 3),
            layout.calc_inode_bitmap_addr(inodes_count - 1).unwrap()
        );

        let err = layout.calc_inode_bitmap_addr(inodes_count).err().unwrap();
        match err {
            Error::INodeIndexOutOfRange => (),
            _ => panic!("expected size error"),
        }
    }

    #[test]
    pub fn test_calc_inode_block_addr() {
        let inodes_count = BLOCK_SIZE as u32 * 8 + 100;
        let layout = Layout::new(inodes_count, 1);

        assert_eq!(
            (layout.inode_offset, 0),
            layout.calc_inode_block_addr(0).unwrap()
        );

        assert_eq!(
            (layout.inode_offset, INODE_SIZE),
            layout.calc_inode_block_addr(1).unwrap()
        );

        assert_eq!(
            (
                layout.inode_offset,
                INODE_SIZE * (INODES_PER_BLOCK - 1) as usize
            ),
            layout.calc_inode_block_addr(INODES_PER_BLOCK - 1).unwrap()
        );

        assert_eq!(
            (layout.inode_offset + BLOCK_SIZE as Addr, 0),
            layout.calc_inode_block_addr(INODES_PER_BLOCK).unwrap()
        );

        assert_eq!(
            (2760704, 3034),
            layout.calc_inode_block_addr(inodes_count - 1).unwrap()
        );

        let err = layout.calc_inode_block_addr(inodes_count).err().unwrap();
        match err {
            Error::INodeIndexOutOfRange => (),
            _ => panic!("expected size error"),
        }
    }

    #[test]
    pub fn test_calc_data_bitmap_addr() {
        let data_block_count = BLOCK_SIZE as u32 * 8 + 100;
        let layout = Layout::new(1, data_block_count);
        let data_bits_per_block = BLOCK_SIZE as u32 * 8;

        assert_eq!(
            (layout.data_bitmap_offset, 0, 0),
            layout.calc_data_bitmap_addr(0).unwrap()
        );

        assert_eq!(
            (layout.data_bitmap_offset, 1, 0),
            layout.calc_data_bitmap_addr(8).unwrap()
        );

        assert_eq!(
            (layout.data_bitmap_offset, 1, 1),
            layout.calc_data_bitmap_addr(9).unwrap()
        );

        assert_eq!(
            (layout.data_bitmap_offset, BLOCK_SIZE - 1, 7),
            layout
                .calc_data_bitmap_addr(data_bits_per_block - 1)
                .unwrap()
        );

        assert_eq!(
            (layout.data_bitmap_offset + BLOCK_SIZE as Addr, 0, 0),
            layout.calc_data_bitmap_addr(data_bits_per_block).unwrap()
        );

        assert_eq!(
            (layout.data_bitmap_offset + BLOCK_SIZE as Addr, 12, 3),
            layout.calc_data_bitmap_addr(data_block_count - 1).unwrap()
        );

        let err = layout.calc_data_bitmap_addr(data_block_count).err().unwrap();
        match err {
            Error::INodeIndexOutOfRange => (),
            _ => panic!("expected size error"),
        }
    }

    #[test]
    pub fn test_calc_data_addr() {
        let data_block_count = BLOCK_SIZE as u32 * 8 + 100;
        let layout = Layout::new(1, data_block_count);

        assert_eq!(layout.data_offset, layout.calc_data_addr(0).unwrap());

        assert_eq!(
            layout.data_offset + BLOCK_SIZE as Addr,
            layout.calc_data_addr(1).unwrap()
        );

        assert_eq!(
            layout.data_offset + ((data_block_count - 1) as Addr * BLOCK_SIZE as Addr),
            layout.calc_data_addr(data_block_count - 1).unwrap()
        );

        let err = layout.calc_data_addr(data_block_count).err().unwrap();
        match err {
            Error::INodeIndexOutOfRange => (),
            _ => panic!("expected size error"),
        }
    }
}
