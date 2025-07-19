use file_io::{FileIoError, FilePos, Result};

use crate::{
    DataBlockIndex, INodeBlockIndex,
    physical::{BLOCK_SIZE, PHYSICAL_INODE_SIZE, PHYSICAL_INODES_PER_BLOCK},
};

pub(crate) struct Layout {
    pub inode_count: u32,
    pub data_block_count: u32,
    pub inode_bitmap_block_count: u32,
    pub data_bitmap_block_count: u32,
    pub inode_block_count: u32,
    pub inode_bitmap_offset: FilePos,
    pub data_bitmap_offset: FilePos,
    pub inode_offset: FilePos,
    pub data_offset: FilePos,
    pub size: FilePos,
}

impl Layout {
    pub(crate) fn new(inode_count: u32, data_block_count: u32) -> Self {
        let inode_bitmap_block_count = inode_count.div_ceil(8).div_ceil(BLOCK_SIZE as u32);
        let data_bitmap_block_count = data_block_count.div_ceil(8).div_ceil(BLOCK_SIZE as u32);
        let inode_block_count = inode_count.div_ceil(PHYSICAL_INODES_PER_BLOCK);

        let inode_bitmap_offset = FilePos(BLOCK_SIZE as u64);
        let data_bitmap_offset =
            FilePos(inode_bitmap_offset.0 + (inode_bitmap_block_count as u64 * BLOCK_SIZE as u64));
        let inode_offset =
            FilePos(data_bitmap_offset.0 + (data_bitmap_block_count as u64 * BLOCK_SIZE as u64));
        let data_offset = FilePos(inode_offset.0 + (inode_block_count as u64 * BLOCK_SIZE as u64));
        let size = FilePos(data_offset.0 + (data_block_count as u64 * BLOCK_SIZE as u64));

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
            size,
        }
    }

    pub fn size(&self) -> FilePos {
        self.size
    }

    /// returns the address of the block containing the inode bitmap along with the offset
    /// within the block where to find the inode bitmap data along with the bit number of
    /// inode
    pub(crate) fn calc_inode_bitmap_addr(
        &self,
        inode_idx: INodeBlockIndex,
    ) -> Result<(FilePos, usize, u8)> {
        if inode_idx.0 >= self.inode_count {
            return Err(FileIoError::Other("INode block index out of range"));
        }

        let bit = (inode_idx.0 % 8) as u8;
        let idx = inode_idx.0 / 8;

        let offset = idx as usize % BLOCK_SIZE;

        let count = idx as u64 / BLOCK_SIZE as u64;
        let addr = self.inode_bitmap_offset.0 + (count * BLOCK_SIZE as u64);

        Ok((FilePos(addr), offset, bit))
    }

    /// returns the address of the block containing the inode along with the offset
    /// within the block where to find the inode data
    pub(crate) fn calc_inode_block_addr(
        &self,
        inode_idx: INodeBlockIndex,
    ) -> Result<(FilePos, usize)> {
        if inode_idx.0 >= self.inode_count {
            return Err(FileIoError::Other("INode block index out of range"));
        }

        let block_offset = (inode_idx.0 % PHYSICAL_INODES_PER_BLOCK) as usize * PHYSICAL_INODE_SIZE;

        let count = inode_idx.0 / PHYSICAL_INODES_PER_BLOCK;
        let block_addr = self.inode_offset.0 + (count as u64 * BLOCK_SIZE as u64);

        Ok((FilePos(block_addr), block_offset))
    }

    /// returns the address of the block containing the data bitmap along with the offset
    /// within the block where to find the data bitmap data along with the bit number of
    /// data
    pub(crate) fn calc_data_bitmap_addr(
        &self,
        data_block_idx: DataBlockIndex,
    ) -> Result<(FilePos, usize, u8)> {
        if data_block_idx.0 >= self.data_block_count {
            return Err(FileIoError::Other("Data block index out of range"));
        }

        let bit = (data_block_idx.0 % 8) as u8;
        let idx = data_block_idx.0 / 8;

        let offset = idx as usize % BLOCK_SIZE;

        let count = idx as u64 / BLOCK_SIZE as u64;
        let addr = self.data_bitmap_offset.0 + (count * BLOCK_SIZE as u64);

        Ok((FilePos(addr), offset, bit))
    }

    /// returns the address of the data block
    pub(crate) fn calc_data_addr(&self, data_block_idx: DataBlockIndex) -> Result<FilePos> {
        if data_block_idx.0 >= self.data_block_count {
            return Err(FileIoError::Other("Data block index out of range"));
        }

        let addr = self.data_offset.0 + (data_block_idx.0 as u64 * BLOCK_SIZE as u64);

        Ok(FilePos(addr))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_calc_inode_bitmap_addr() {
        let inodes_count = BLOCK_SIZE as u32 * 8 + 100;
        let layout = Layout::new(inodes_count, 1);
        let inode_bits_per_block = BLOCK_SIZE as u32 * 8;

        assert_eq!(
            (layout.inode_bitmap_offset, 0, 0),
            layout.calc_inode_bitmap_addr(INodeBlockIndex(0)).unwrap()
        );

        assert_eq!(
            (layout.inode_bitmap_offset, 1, 0),
            layout.calc_inode_bitmap_addr(INodeBlockIndex(8)).unwrap()
        );

        assert_eq!(
            (layout.inode_bitmap_offset, 1, 1),
            layout.calc_inode_bitmap_addr(INodeBlockIndex(9)).unwrap()
        );

        assert_eq!(
            (layout.inode_bitmap_offset, BLOCK_SIZE - 1, 7),
            layout
                .calc_inode_bitmap_addr(INodeBlockIndex(inode_bits_per_block - 1))
                .unwrap()
        );

        assert_eq!(
            (
                FilePos(layout.inode_bitmap_offset.0 + BLOCK_SIZE as u64),
                0,
                0
            ),
            layout
                .calc_inode_bitmap_addr(INodeBlockIndex(inode_bits_per_block))
                .unwrap()
        );

        assert_eq!(
            (
                FilePos(layout.inode_bitmap_offset.0 + BLOCK_SIZE as u64),
                12,
                3
            ),
            layout
                .calc_inode_bitmap_addr(INodeBlockIndex(inodes_count - 1))
                .unwrap()
        );

        let err = layout
            .calc_inode_bitmap_addr(INodeBlockIndex(inodes_count))
            .err()
            .unwrap();
        match err {
            FileIoError::Other(err) => assert_eq!("", err),
            _ => panic!("expected size error"),
        }
    }

    #[test]
    pub fn test_calc_inode_block_addr() {
        let inodes_count = BLOCK_SIZE as u32 * 8 + 100;
        let layout = Layout::new(inodes_count, 1);

        assert_eq!(
            (layout.inode_offset, 0),
            layout.calc_inode_block_addr(INodeBlockIndex(0)).unwrap()
        );

        assert_eq!(
            (layout.inode_offset, PHYSICAL_INODE_SIZE),
            layout.calc_inode_block_addr(INodeBlockIndex(1)).unwrap()
        );

        assert_eq!(
            (
                layout.inode_offset,
                PHYSICAL_INODE_SIZE * (PHYSICAL_INODES_PER_BLOCK - 1) as usize
            ),
            layout
                .calc_inode_block_addr(INodeBlockIndex(PHYSICAL_INODES_PER_BLOCK - 1))
                .unwrap()
        );

        assert_eq!(
            (FilePos(layout.inode_offset.0 + BLOCK_SIZE as u64), 0),
            layout
                .calc_inode_block_addr(INodeBlockIndex(PHYSICAL_INODES_PER_BLOCK))
                .unwrap()
        );

        assert_eq!(
            (FilePos(3145728), 1410),
            layout
                .calc_inode_block_addr(INodeBlockIndex(inodes_count - 1))
                .unwrap()
        );

        let err = layout
            .calc_inode_block_addr(INodeBlockIndex(inodes_count))
            .err()
            .unwrap();
        match err {
            FileIoError::Other(err) => assert_eq!("", err),
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
            layout.calc_data_bitmap_addr(DataBlockIndex(0)).unwrap()
        );

        assert_eq!(
            (layout.data_bitmap_offset, 1, 0),
            layout.calc_data_bitmap_addr(DataBlockIndex(8)).unwrap()
        );

        assert_eq!(
            (layout.data_bitmap_offset, 1, 1),
            layout.calc_data_bitmap_addr(DataBlockIndex(9)).unwrap()
        );

        assert_eq!(
            (layout.data_bitmap_offset, BLOCK_SIZE - 1, 7),
            layout
                .calc_data_bitmap_addr(DataBlockIndex(data_bits_per_block - 1))
                .unwrap()
        );

        assert_eq!(
            (
                FilePos(layout.data_bitmap_offset.0 + BLOCK_SIZE as u64),
                0,
                0
            ),
            layout
                .calc_data_bitmap_addr(DataBlockIndex(data_bits_per_block))
                .unwrap()
        );

        assert_eq!(
            (
                FilePos(layout.data_bitmap_offset.0 + BLOCK_SIZE as u64),
                12,
                3
            ),
            layout
                .calc_data_bitmap_addr(DataBlockIndex(data_block_count - 1))
                .unwrap()
        );

        let err = layout
            .calc_data_bitmap_addr(DataBlockIndex(data_block_count))
            .err()
            .unwrap();
        match err {
            FileIoError::Other(err) => assert_eq!("", err),
            _ => panic!("expected size error"),
        }
    }

    #[test]
    pub fn test_calc_data_addr() {
        let data_block_count = BLOCK_SIZE as u32 * 8 + 100;
        let layout = Layout::new(1, data_block_count);

        assert_eq!(layout.data_offset, layout.calc_data_addr(0).unwrap());

        assert_eq!(
            FilePos(layout.data_offset.0 + BLOCK_SIZE as u64),
            layout.calc_data_addr(DataBlockIndex(1)).unwrap()
        );

        assert_eq!(
            FilePos(layout.data_offset.0 + ((data_block_count - 1) as u64 * BLOCK_SIZE as u64)),
            layout
                .calc_data_addr(DataBlockIndex(data_block_count - 1))
                .unwrap()
        );

        let err = layout
            .calc_data_addr(DataBlockIndex(data_block_count))
            .err()
            .unwrap();
        match err {
            FileIoError::Other(err) => assert_eq!("", err),
            _ => panic!("expected size error"),
        }
    }
}
